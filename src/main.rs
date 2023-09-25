use std::{io, os::unix::prelude::OsStrExt, path::PathBuf, process::exit};

use clap::Parser;
use deno::re_exports::{deno_core::futures::future::try_join_all, deno_runtime::tokio_util};
use log::error;
use rusk::compose::Composer;

const ROOT_PATTERNS: &[&[u8]] = &[
    b".git",
    b".svn",
    b".hg",
    b".bzr",
    // ".gitignore",
    b"Rakefile",
    b"pom.xml",
    b"project.clj",
    b"package.json",
    b"manifest.json",
    // "*.csproj",
    // "*.sln",
];

#[derive(Parser)]
struct Args {
    /// Names of tasks to be executed
    taskname: Vec<String>,
}

fn get_root() -> io::Result<PathBuf> {
    use std::{env::current_dir, fs::read_dir};
    let path = current_dir()?;
    let path_ancestors = path.as_path().ancestors();
    for p in path_ancestors {
        let contains_ruskfile = read_dir(p)?.any(|p| {
            ROOT_PATTERNS.contains(
                &{
                    let Ok(entry) = p else { return false; };
                    entry
                }
                .file_name()
                .as_bytes(),
            )
        });
        if contains_ruskfile {
            return Ok(PathBuf::from(p));
        }
    }
    Ok(path)
}

fn main() {
    env_logger::init();
    let Args { taskname } = Args::parse();

    if let Err(err) = tokio_util::create_basic_runtime().block_on(async {
        let composer = Composer::new(get_root()?).await;
        try_join_all(taskname.into_iter().map(|task| composer.execute(task)))
            .await
            .and(Ok(()))
    }) {
        error!("{err}");
        exit(1);
    }
}
