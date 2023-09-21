use deno_runtime::tokio_util;
use rusk::compose::Composer;

fn main() {
    tokio_util::create_basic_runtime().block_on(async {
        println!("================= Deps tree =================");
        let composer = Composer::new(".").await;
        for (a, deps) in composer.get_deptree("help").unwrap() {
            println!("- {a}");
            for d in deps {
                println!("  └ {d} ");
            }
        }
        println!("================== Started ==================");
        composer.execute("help").await
    }).unwrap();
}
