use std::{env::args, io::BufWriter, io::Write};

use colored::Colorize;
use rusk::{Rusk, RuskError};

mod files;
mod rusk;

#[tokio::main]
async fn main() {
    let mut config_files = files::RuskConfigFiles::new(std::env::vars().collect());
    config_files.walkdir(std::env::current_dir().unwrap()).await;
    let args: Vec<String> = args().collect();

    if args.len() == 1 {
        let mut stdout = BufWriter::new(std::io::stdout());
        for task in config_files.tasks_list() {
            writeln!(stdout, "{}", task).unwrap();
        }
        return;
    }

    if let Err(e) = Into::<Rusk>::into(config_files)
        .execute(&args[1..], Default::default())
        .await
    {
        match e {
            RuskError::TaskError(e) => {
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
