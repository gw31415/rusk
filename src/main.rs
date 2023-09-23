use std::{io, path::PathBuf, os::unix::prelude::OsStrExt};

use deno_runtime::{deno_core::error::AnyError, tokio_util};
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
                .as_os_str()
                .as_bytes(),
            )
        });
        if contains_ruskfile {
            return Ok(PathBuf::from(p));
        }
    }
    Ok(path)
}

fn main() -> Result<(), AnyError> {
    tokio_util::create_basic_runtime().block_on(async {
        println!("================= Deps tree =================");
        let composer = Composer::new(get_root()?).await;
        for (a, deps) in composer.get_deptree("help").unwrap() {
            println!("- {a}");
            for d in deps {
                println!("  └ {d} ");
            }
        }
        println!("================== Started ==================");
        composer.execute("help").await
    })?;
    Ok(())
}
