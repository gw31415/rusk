use std::{env::args, io::BufWriter, io::Write};

use colored::Colorize;
use rusk::{Rusk, RuskError};
use ruskfile::RuskFileComposer;

mod digraph;
mod rusk;
mod ruskfile;

#[tokio::main]
async fn main() {
    let mut composer = RuskFileComposer::new();
    composer
        .walkdir(
            std::env::current_dir().unwrap(), // TODO: Project root
        )
        .await;
    let args: Vec<String> = args().collect();

    if args.len() == 1 {
        let mut stdout = BufWriter::new(std::io::stdout());
        for task in composer.tasks_list() {
            writeln!(stdout, "{}", task).unwrap();
        }
        return;
    }

    if let Err(e) = Into::<Rusk>::into(composer)
        .exec(&args[1..], Default::default())
        .await
    {
        match e {
            RuskError::TaskFailed(e) => {
                eprintln!("{e}");
                std::process::exit(e.exit_code);
            }
            e => {
                eprint!("{}: ", "Error".bold().on_red());
                eprintln!("{}", e.to_string().red().bold());
                std::process::exit(1);
            }
        }
    }
}
