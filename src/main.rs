use std::{
    fmt::Display,
    io::{BufWriter, Write},
    time::Duration,
};

use args::Args;
use colored::Colorize;
use fs::RuskfileComposer;
use itertools::Itertools;
use path::get_current_dir;
use rusk::{Rusk, RuskError};

mod args;
mod digraph;
mod fs;
mod path;
mod rusk;
mod taskkey;

#[cold]
fn abort(title: &'static str, msg: impl Display, code: i32) -> ! {
    eprintln!("{}: {}", title.bold().red(), msg);
    std::process::exit(code);
}

const SCAN_TIMEOUT: Duration = Duration::from_millis(500);

#[tokio::main]
async fn main() {
    let args = Args::new();

    let mut composer = RuskfileComposer::new();
    // TODO: Config to select either Project root or Current dir as root
    if tokio::time::timeout(SCAN_TIMEOUT, composer.walkdir(get_current_dir()))
        .await
        .is_err()
    {
        abort(
            "abort",
            format_args!("Scan took over {SCAN_TIMEOUT:?}. Try in deeper directory."),
            1,
        );
    }

    if args.no_pargs() {
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
                writeln!(stderr, "\n{}", err.into_verbose()).unwrap();
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
        RuskfileConvertError(#[from] fs::RuskfileConvertError),
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
        abort(title, err, code);
    }
}
