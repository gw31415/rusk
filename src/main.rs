use std::{io, path::PathBuf};

use deno_runtime::{deno_core::error::AnyError, tokio_util};
use rusk::compose::Composer;

fn get_root() -> io::Result<PathBuf> {
    use rusk::compose::RUSK_FILE;
    use std::{env::current_dir, fs::read_dir};
    let path = current_dir()?;
    let path_ancestors = path.as_path().ancestors();
    for p in path_ancestors {
        let contains_ruskfile = read_dir(p)?.any(|p| p.unwrap().file_name() == *RUSK_FILE);
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
