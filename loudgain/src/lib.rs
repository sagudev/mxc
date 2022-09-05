use std::process::Command;

pub fn loudgain() -> Command {
    Command::new(include_str!("exec.txt").trim())
}
