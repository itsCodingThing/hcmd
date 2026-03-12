use clap::Subcommand;

use indicatif::{ProgressBar, ProgressStyle};
use std::thread;
use std::time::Duration;

use crate::storage;

#[derive(Debug, Subcommand, Clone)]
pub enum PomoCommands {
    #[command(arg_required_else_help = true)]
    #[command(name = "add", about = "add a timer to storage")]
    Add {
        /// Give timer a name
        #[arg(short, long)]
        name: String,

        /// Crate a timer for specified value in mins
        /// minimum value should be 5 min
        #[arg(short, long, default_value_t = 10)]
        duration: u64,
    },

    #[command(arg_required_else_help = true)]
    #[command(name = "create", about = "create a quick timer")]
    Create {
        /// Give timer a name
        #[arg(short, long)]
        name: String,

        /// Crate a timer for specified value in mins
        /// minimum value should be 5 min
        #[arg(short, long)]
        duration: u64,
    },

    #[command(arg_required_else_help = true)]
    #[command(name = "run", about = "run a stored timer by name")]
    Run {
        /// timer a name
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(arg_required_else_help = true)]
    #[command(name = "remove", about = "remove a stored timer by name or id")]
    Remove {
        /// timer a name
        #[arg(short, long)]
        name: Option<String>,
    },

    #[command(name = "nuke", about = "remove all stored timers")]
    Nuke,

    #[command(name = "list", about = "list all the availalbe timers")]
    List,
}

pub fn init(cmd: PomoCommands) {
    let welcome_banner = include_str!("../ascii.txt");
    println!("{}", welcome_banner);

    let mut store = storage::create().expect("unable to create storage");

    match cmd {
        PomoCommands::Create { duration, name } => {
            let secs = duration * 60;

            println!("name: {name}");
            setup_timer(secs);
            println!("Timer is complete")
        }

        PomoCommands::Add { name, duration } => {
            println!("name: {name}");
            println!("duration: {duration}");

            store.add(name, duration);
        }

        PomoCommands::Run { name } => {
            let mut duration = 0;
            let mut tname = String::new();

            if let Some(n) = name {
                println!("search by name: {n}");

                for timer in store.timers() {
                    if timer.name() == n.as_str() {
                        tname = timer.name();
                        duration = timer.duration();
                    }
                }
            }

            let secs = duration * 60;
            if secs > 0 {
                println!("name: {tname}");
                println!("mins: {duration}");

                setup_timer(secs);
            }
        }

        PomoCommands::Remove { name } => {
            if let Some(n) = name {
                println!("search by name: {n:?}");
                store.remove_by_name(n);
            }
        }

        PomoCommands::List => {
            if store.timers().is_empty() {
                println!("there are 0 timers");
            }

            for value in store.timers() {
                println!("{value:?}");
            }
        }

        PomoCommands::Nuke => {
            store.remove_all();
            println!("removed all the timers");
        }
    }
}

fn setup_timer(duration: u64) {
    let bar = ProgressBar::new(duration);
    let default_style = ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/green}")
        .unwrap()
        .progress_chars("#>*");
    bar.set_style(default_style);

    let mut seconds = Duration::from_secs(duration);
    let wait = Duration::from_secs(1);

    loop {
        thread::sleep(wait);

        seconds = seconds.saturating_sub(Duration::from_secs(1));
        bar.inc(1);

        if seconds.is_zero() {
            break;
        }
    }

    bar.finish();
}
