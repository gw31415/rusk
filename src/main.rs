use std::{
    io::{self, BufWriter, Write},
    os::unix::prelude::OsStrExt,
    path::PathBuf,
    process::exit,
};

use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::{generate, Shell};
use deno::re_exports::deno_runtime::tokio_util;
use env_logger::Env;
use log::{error, info};
use rusk::compose::Composer;
use serde_json::json;

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
    /// Print information
    #[arg(long)]
    info: Option<InfoTarget>,
    /// Log output in detail
    #[arg(short, long)]
    verbose: bool,
    /// Generate shell completion
    #[arg(long, value_name = "SHELL")]
    completion: Option<Shell>,
}

#[derive(ValueEnum, Clone)]
enum InfoTarget {
    /// Print the task list
    Tasks,
    /// Print information on current projects
    Project,
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
            let path = PathBuf::from(p);
            info!("Initialize in {:?}", path);
            return Ok(path);
        }
    }
    info!("Project root not found. Initialize in current directory.");
    Ok(path)
}

fn main() {
    let Args {
        taskname,
        info,
        verbose,
        completion: shell,
    } = Args::parse();

    // Setup env_logger
    if verbose {
        env_logger::init_from_env(Env::new().default_filter_or("info"));
    } else {
        env_logger::init();

        // Shell completion
        if let Some(shell) = shell {
            shell_completion(shell);
        }
    }

    if let Err(err) = tokio_util::create_basic_runtime().block_on(async {
        let project_root = get_root()?;
        let composer = Composer::new(&project_root).await;

        // Print infomation
        if let Some(info) = info {
            let mut writer = BufWriter::new(io::stdout());
            match info {
                InfoTarget::Tasks => {
                    for name in composer.task_names() {
                        writeln!(writer, "{name}")?;
                    }
                }
                InfoTarget::Project => {
                    let data = json!({
                        "project_root": project_root,
                        "project": composer,
                    });
                    writeln!(writer, "{data}")?;
                    // writer.write_all(&serde_json::to_vec(&composer)?)?;
                    // writeln!(writer)?;
                }
            }
            writer.flush()?;
            exit(0);
        }

        // Execute tasks
        composer.execute(taskname).await
    }) {
        error!("{err}");
        exit(1);
    }
}

#[cold]
fn shell_completion(shell: Shell) {
    let mut stdout = BufWriter::new(io::stdout());
    let mut cmd = Args::command();
    let name = cmd.get_name().to_string();
    if Shell::Fish == shell {
        writeln!(stdout, "complete -c {name} -xa '({name} --info tasks)'").unwrap();
    }
    generate(shell, &mut cmd, name, &mut stdout);
}
