use crate::cli::TopLevelArgs;
use crate::cli::add::add_internal;
use crate::execution_data::ExecutionData;
use crate::full_piece::FullPiece;
use crate::machine::{Machine, MachineData};
use crate::pieces::falconf_init::FalconfInit;
use crate::pieces::{NonBulkPieceEnum, PieceEnum};
use crate::repo::Repo;
use color_eyre::Result;
use color_eyre::eyre::{WrapErr as _, eyre};
use log::{debug, info};
use std::fs;
use std::fs::remove_dir_all;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Installation {
    machine: Machine,
    repo: Repo,
}

impl Installation {
    pub const fn machine(&self) -> Machine {
        self.machine
    }

    pub const fn repo(&self) -> &Repo {
        &self.repo
    }

    pub const fn repo_mut(&mut self) -> &mut Repo {
        &mut self.repo
    }

    pub fn init(top_level_args: &TopLevelArgs, remote: &str, new: bool) -> Result<Self> {
        let root = &top_level_args.path;
        debug!("Looking at {}", root.display());

        if root.try_exists()? {
            return Err(eyre!("Installation already exists"));
        }
        fs::create_dir(root)?;

        match Self::_init(top_level_args, remote, new) {
            Ok(installation) => Ok(installation),
            Err(e) => {
                info!(
                    "Found error during init; removing newly created .falconf directory to avoid half-initialized state"
                );
                remove_dir_all(&top_level_args.path)?;
                Err(e)
            }
        }
    }

    fn _init(top_level_args: &TopLevelArgs, remote: &str, new: bool) -> Result<Self> {
        let root = &top_level_args.path;

        let machine_path = root.join("machine");
        let repository_path = Self::get_repository_path(root);

        let machine = Machine::new();
        fs::write(&machine_path, machine.0.to_string())?;
        let machine_data = MachineData::new_this()?;

        let mut repo = Repo::init(remote, &repository_path, machine, machine_data, new)?;

        if new {
            add_internal(
                top_level_args,
                vec![(
                    FullPiece::new(
                        PieceEnum::NonBulk(NonBulkPieceEnum::FalconfInit(FalconfInit::new(
                            remote.to_string(),
                        ))),
                        None,
                    ),
                    true,
                )],
            )?;
        } else {
            let pieces = repo.data_mut().pieces_mut();

            let mut falconf_inits: Vec<_> = pieces
                .iter_mut()
                .filter(|(_id, piece)| {
                    matches!(
                        piece.piece,
                        PieceEnum::NonBulk(NonBulkPieceEnum::FalconfInit(_))
                    )
                })
                .collect();
            if falconf_inits.is_empty() {
                return Err(eyre!("Expected a falconf init piece to be present"));
            } else if falconf_inits.len() > 1 {
                return Err(eyre!("Expected only one falconf init piece to be present"));
            }
            let (_id, ref mut falconf_init) = falconf_inits[0];
            falconf_init.done(machine);

            repo.write_and_push(vec![], None)?;
        }

        Self::from_repo(top_level_args, repo)
    }

    pub fn get(top_level_args: &TopLevelArgs) -> Result<Self> {
        let root = &top_level_args.path;
        debug!("Looking at {}", root.display());

        if !root.is_dir() {
            return Err(eyre!(
                "No installation found at {root:?}. Run `falconf init` first!"
            ));
        }

        let repo = Repo::get_from_path(&Self::get_repository_path(root))?;

        Self::from_repo(top_level_args, repo)
    }

    fn from_repo(top_level_args: &TopLevelArgs, repo: Repo) -> Result<Self> {
        let root = &top_level_args.path;

        let machine = Machine(
            fs::read_to_string(root.join("machine"))?
                .parse()
                .wrap_err("`machine` file does not contain a valid UUID".to_owned())?,
        );

        Ok(Self { machine, repo })
    }

    fn get_repository_path(root: &Path) -> PathBuf {
        root.join("repository")
    }

    fn check_synced(&mut self, execution_data: &ExecutionData) {
        let data = self.repo.data_mut();
        let (pieces, machines) = data.as_mut_parts();

        let (to_execute, to_undo) = FullPiece::get_todo(pieces, execution_data);

        if !to_execute.is_empty() || !to_undo.is_empty() {
            info!(
                "You have changes on the remote that are not executed locally! Use `falconf sync` to execute them. Unsynced changes:"
            );
            for (id, piece) in to_execute {
                info!("- Execute: {}", piece.print(id, machines));
            }
            for (id, piece) in to_undo {
                info!("- Undo: {}", piece.print(id, machines));
            }
        }
    }

    pub fn pull_and_read(
        &mut self,
        check_synced: bool,
        execution_data: &ExecutionData,
    ) -> Result<()> {
        self.repo.pull_and_read()?;
        if check_synced {
            self.check_synced(execution_data);
        }
        Ok(())
    }
}
