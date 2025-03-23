use std::process;

pub(crate) fn if_sudo(program: &str, sudo: bool) -> process::Command {
    if sudo {
        let mut cmd = process::Command::new("sudo");
        cmd.arg(program);
        cmd
    } else {
        process::Command::new(program)
    }
}

pub(crate) fn press_enter() {
    todo!();
}
