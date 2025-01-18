use std::{env::args, io::BufWriter, io::Write};

use colored::Colorize;
use rusk::{Rusk, RuskError};
use ruskfile::RuskfileComposer;

mod digraph;
mod rusk;
mod ruskfile;

#[tokio::main]
async fn main() {
    let mut composer = RuskfileComposer::new();
    composer
        .walkdir(
            std::env::current_dir().unwrap(), // TODO: Project root
        )
        .await;
    let args: Vec<String> = args().skip(1).collect();

    if args.is_empty() {
        let mut stdout = BufWriter::new(std::io::stdout());
        for task in composer.tasks_list() {
            writeln!(stdout, "{}", task).unwrap();
        }
        return;
    }

    if let Err(e) = Into::<Rusk>::into(composer)
        .exec(args, Default::default())
        .await
    {
        match e {
            RuskError::TaskFailed(e) => {
                eprint!("{} ", "Abort:".bold().red());
                eprintln!("{e}");
                std::process::exit(e.exit_code);
            }
            e => {
                eprint!("{} ", "Error:".bold().red());
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
