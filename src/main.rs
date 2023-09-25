use std::{io, os::unix::prelude::OsStrExt, path::PathBuf, process::exit};

use deno::re_exports::deno_runtime::tokio_util;
use log::{debug, error};
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

    if let Err(err) = tokio_util::create_basic_runtime().block_on(async {
        debug!("================= Deps tree =================");
        let composer = Composer::new(get_root()?).await;
        for (a, deps) in composer.get_deptree("help".to_string())? {
            debug!("- {a}");
            for d in deps {
                debug!("  └ {d} ");
            }
        }
        debug!("================== Started ==================");
        composer.execute("help".to_string()).await
    }) {
        error!("{err}");
        exit(1);
    }
}
