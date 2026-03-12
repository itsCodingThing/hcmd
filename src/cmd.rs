use clap::{Parser, Subcommand};

use crate::pomo::PomoCommands;

#[derive(Parser, Debug)]
#[command(name = "hcmd", version = "2.0.0", about = "enhanced cmds", long_about = None)]
pub struct Cmd {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    Pomo {
        #[command(subcommand)]
        command: PomoCommands,
    },
    Fd,
}

pub fn create() -> Cmd {
    Cmd::parse()
}

impl Cmd {
    pub fn commands(&self) -> Commands {
        self.command.clone()
    }
}
