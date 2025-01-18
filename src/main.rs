use std::{env::args, io::BufWriter, io::Write};

use colored::Colorize;
use rusk::{Rusk, RuskError};
use ruskfile::RuskfileComposer;

mod digraph;
mod rusk;
mod ruskfile;

#[tokio::main]
async fn main() {
    let args: Vec<String> = args().skip(1).collect();

    let mut composer = RuskfileComposer::new();
    composer
        .walkdir(
            std::env::current_dir().unwrap(), // TODO: Project root
        )
        .await;

    if args.is_empty() {
        let mut stdout = BufWriter::new(std::io::stdout());
        for task in composer.tasks_list() {
            writeln!(stdout, "{}", task).unwrap();
        }
        return;
    }

    #[derive(Debug, thiserror::Error)]
    enum MainError {
        #[error(transparent)]
        RuskError(#[from] RuskError),
        #[error(transparent)]
        RuskfileConvertError(#[from] ruskfile::RuskfileConvertError),
    }

    let res: Result<(), MainError> = async move {
        let composer = Rusk::try_from(composer)?;
        composer.exec(args, Default::default()).await?;
        Ok(())
    }
    .await;

    match res {
        Err(MainError::RuskError(RuskError::TaskFailed(e))) => {
            eprint!("{} ", "Abort:".bold().red());
            eprintln!("{e}");
            std::process::exit(e.exit_code);
        }
        Err(e) => {
            eprint!("{} ", "Error:".bold().red());
            eprintln!("{e}");
            std::process::exit(1);
        }
        _ => (),
    }
}
