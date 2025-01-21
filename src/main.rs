use std::io::{BufWriter, Write};

use colored::Colorize;
use itertools::Itertools;
use path::get_current_dir;
use rusk::{Rusk, RuskError};
use ruskfile::RuskfileComposer;

mod digraph;
mod path;
mod rusk;
mod ruskfile;

mod args {
    use std::env;

    pub struct Args {
        inner: env::Args,
        first: Option<String>,
        first_read: bool,
    }

    impl Args {
        pub fn new() -> Self {
            let mut inner = env::args();
            inner.next(); // Skip the first argument
            let first = inner.next();
            Self {
                inner,
                first,
                first_read: false,
            }
        }
        pub fn was_empty(&self) -> bool {
            !self.first_read && self.first.is_none()
        }
    }

    impl Iterator for Args {
        type Item = String;

        fn next(&mut self) -> Option<Self::Item> {
            if !self.first_read {
                self.first_read = true;
                return self.first.take();
            }
            self.inner.next()
        }
    }
}

#[tokio::main]
async fn main() {
    let args = args::Args::new();

    let mut composer = RuskfileComposer::new();
    composer
        .walkdir(get_current_dir()) // TODO: Project root
        .await;

    if args.was_empty() {
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

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
