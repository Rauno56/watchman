// #[macro_use]
extern crate dialoguer;
extern crate structopt;

use dialoguer::{theme::ColorfulTheme, Checkboxes};
use std::error;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

use crate::state::ProcessConfig;
use crate::state::State;
use crate::state::StateTrait;

mod state;
mod system;

#[derive(Debug, StructOpt)]
enum SubCommand {
    #[structopt(name = "add")]
    Add {
        command: String,
        #[structopt(long = "name")]
        name: Option<String>,
    },
    #[structopt(name = "show")]
    Show,
    #[structopt(name = "config")]
    Config,
}

#[derive(Debug, StructOpt)]
#[structopt()]
struct Cli {
    #[structopt(subcommand)]
    cmd: Option<SubCommand>,
}

fn update_from_user(mut state: State) -> State {
    let mut all_processes: Vec<&mut ProcessConfig> = state.iter_mut().collect();
    let defs: Vec<bool> = all_processes.iter().map(|proc| proc.is_enabled()).collect();

    let selections = Checkboxes::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick processes you want to be running")
        .items(&all_processes[..])
        // .clear(false)
        .defaults(&defs[..])
        .interact()
        .unwrap();

    defs.iter().enumerate().for_each(|(i, old)| {
        let new = selections.contains(&i);
        match (*old, new) {
            (true, false) => {
                println!("Disabling {}", all_processes[i]);
                all_processes[i].kill();
            }
            (false, true) => {
                println!("Enabling {}", all_processes[i]);
                all_processes[i].run();
            }
            _ => {}
        };
    });

    state
}

fn interactive(mut state: State) -> State {
    state.fix_all();

    state = update_from_user(state);

    state
}

fn main() -> std::result::Result<(), Box<error::Error>> {
    let args = Cli::from_args();

    let file_input = "example.watchman.state.json";
    let state_path = fs::canonicalize(Path::new(file_input))?;
    let mut state: State = State::from_file(&state_path)?;

    // println!("{:?}", args);
    match args.cmd {
        Some(subcommand) => match subcommand {
            SubCommand::Config => println!("{}", state_path.to_str().unwrap()),
            _ => unimplemented!(),
        },
        None => state = interactive(state),
    }

    state.to_file(&state_path)?;

    Ok(())
}
