//! # Examples
//! ```
//! use std::collections::HashMap;
//! use rusk::config::{Config, Script, ScriptType, Task};
//!
//! let config_file = Config {
//!     tasks: HashMap::from([(
//!         "sample".to_string(),
//!         Task {
//!             config: Default::default(),
//!             script: Script {
//!                 code: "console.log(\"Hello, world!\");\n".to_string(),
//!                 r#type: ScriptType::Deno,
//!             },
//!         },
//!     )]),
//! };
//! print!("{}", toml::to_string(&config_file).unwrap());
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod script;
mod task;
mod task_name;
mod task_settings;
pub use script::*;
pub use task::*;
pub use task_name::*;
pub use task_settings::*;

/// Rusk config file.
#[derive(Serialize, Deserialize)]
pub struct RuskFileContent {
    /// Pairs of Names and Tasks.
    pub tasks: HashMap<TaskName, Atom>,
}
