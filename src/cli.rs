use crate::{
    cmd::{self, Commands},
    fd, pomo,
};

pub fn init() {
    let cmd = cmd::create();

    match cmd.commands() {
        Commands::Pomo { command } => pomo::init(command),

        Commands::Fd => {
            if fd::init().is_err() {
                println!("ratatui error");
            }
        }
    }
}
