use std::collections::HashMap;

use config::{Rusk, RuskError, Task};

mod config;

#[tokio::main]
async fn main() {
    let cwd = std::env::current_dir().unwrap();
    let config = Rusk {
        envs: std::env::vars().collect(),
        tasks: [
            (
                "task1".into(),
                Task {
                    envs: HashMap::new(),
                    script: "false && echo 'task1 start' && sleep 2 && echo 'task1 done'".into(),
                    cwd: cwd.clone(),
                    depends: vec![],
                },
            ),
            (
                "task2".into(),
                Task {
                    envs: HashMap::new(),
                    script: "echo 'task2 start' && sleep 1 && echo 'task2 done'".into(),
                    cwd: cwd.clone(),
                    depends: vec![], // vec!["task1".to_string()],
                },
            ),
        ]
        .into(),
    };
    match config.execute(Default::default()).await {
        Ok(()) => println!("All tasks are done"),
        Err(e) => match e {
            RuskError::TaskError(e) => {
                eprintln!("{e}");
                std::process::exit(e.exit_code);
            }
            RuskError::ConfigParseError(e) => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        },
    }
}
