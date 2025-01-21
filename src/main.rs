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
        {
            let mut stdout = BufWriter::new(std::io::stdout().lock());
            for task in composer.tasks_list().sorted() {
                writeln!(stdout, "{}", task).unwrap();
            }
            stdout.flush().unwrap();
        }
        {
            let mut stderr = BufWriter::new(std::io::stderr().lock());
            for err in composer.errors_list().sorted() {
                writeln!(stderr, "\n{}", err.verbose()).unwrap();
            }
            stderr.flush().unwrap();
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

    if let Err(err) = res {
        let (title, code) = match &err {
            MainError::RuskError(RuskError::TaskFailed(e)) => ("abort", e.exit_code),
            _ => ("error", 1),
        };
        eprintln!("{}: {}", title.bold().red(), err);
        std::process::exit(code);
    }
}
