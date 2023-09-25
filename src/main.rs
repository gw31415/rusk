use std::{
    io::{self, BufWriter, Write},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
    process::exit,
};

use clap::Parser;
use deno::re_exports::deno_runtime::tokio_util;
use env_logger::Env;
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
/// Parallel processing, modern and fast Task Runner, written in Rust.
#[command(author, version)]
struct Args {
    /// Names of tasks to be executed
    taskname: Vec<String>,
    /// List task names
    #[arg(short, long)]
    list: bool,
    /// Log output in detail
    #[arg(short, long)]
    verbose: bool,
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
    let Args {
        taskname,
        list,
        verbose,
    } = Args::parse();

    // Setup env_logger
    if verbose {
        env_logger::init_from_env(Env::new().default_filter_or("info"));
    } else {
        env_logger::init();
    }

    if let Err(err) = tokio_util::create_basic_runtime().block_on(async {
        let composer = Composer::new(get_root()?).await;

        // List task names
        if list {
            let mut writer = BufWriter::new(io::stdout());
            for name in composer.task_names() {
                writeln!(writer, "{name}")?;
            }
        }

        // Execute tasks
        composer.execute(taskname).await
    }) {
        error!("{err}");
        exit(1);
    }
}
