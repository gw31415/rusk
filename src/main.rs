use rusk::{Rusk, RuskError};

mod files;
mod rusk;

#[tokio::main]
async fn main() {
    let mut config_files = files::ConfigFiles::new(std::env::vars().collect());
    config_files.collect(std::env::current_dir().unwrap()).await;

    if let Err(e) = Into::<Rusk>::into(config_files)
        .execute(Default::default())
        .await
    {
        match e {
            RuskError::TaskError(e) => {
                eprintln!("{e}");
                std::process::exit(e.exit_code);
            }
            RuskError::ConfigParseError(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
