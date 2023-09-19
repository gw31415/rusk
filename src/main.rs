use deno_runtime::{tokio_util, deno_core::error::AnyError};
use rusk::compose::Composer;

fn main() -> Result<(), AnyError> {
    tokio_util::create_basic_runtime().block_on(async {
        let composer = Composer::new(".").await;
        for name in composer.task_names() {
            println!("{name}");
        }
        composer.execute("help").await
    })?;
    Ok(())
}
