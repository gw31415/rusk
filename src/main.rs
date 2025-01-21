use std::io::{BufWriter, Write};

use args::Args;
use colored::Colorize;
use itertools::Itertools;
use path::get_current_dir;
use rusk::{Rusk, RuskError};
use ruskfile::RuskfileComposer;

mod args;
mod digraph;
mod path;
mod rusk;
mod ruskfile;

#[tokio::main]
async fn main() {
    let args = Args::new();

    let mut composer = RuskfileComposer::new();
    composer
        .walkdir(get_current_dir()) // TODO: Project root
        .await;

    if args.is_empty() {
        let stdout = std::io::stdout();
        let mut stdout = BufWriter::new(stdout);

        for task in composer.tasks_list().sorted() {
            writeln!(stdout, "{}", task).unwrap();
        }
        stdout.flush().unwrap();

        let stderr = std::io::stderr();
        let mut stderr = BufWriter::new(stderr);
        for err in composer.errors_list().sorted() {
            writeln!(stderr, "\n{}", err.verbose()).unwrap();
        }
        stderr.flush().unwrap();
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
            eprint!("{}: ", "abort".bold().red());
            eprintln!("{e}");
            std::process::exit(e.exit_code);
        }
        Err(e) => {
            eprint!("{}: ", "error".bold().red());
            eprintln!("{e}");
            std::process::exit(1);
        }
        _ => (),
    }
}
