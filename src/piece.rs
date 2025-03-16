use std::path::PathBuf;

trait Piece {
    fn execute(&self);
}

/// Sym/hardlink a file
struct File {
    location: PathBuf,
    /// If the file should be a hardlink or symlink
    hardlink: bool,
    /// If the file should be created as root
    root: bool,
}

impl Piece for File {
    fn execute(&self) {
        // TODO:
        //  - Find the file in the repo
        //  - Link the file (using hardlink & root)
    }
}

/// Run an arbitrary command
struct Command {
    command: String,
    /// If the command should be run as root
    root: bool,
}

impl Piece for File {
    fn execute(&self) {
        // TODO: run bash -c command (using root)
    }
}

/// Request the user to perform an action manually *sad robot face*
struct Manual {
    message: String,
}

impl Piece for Manual {
    fn execute(&self) {
        println!("Manual action required");
        println!("{}", self.message);
        println!("Continue when the action is performed.");
        press_any_button();
    }
}
