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
use serde_with::{serde_as, DisplayFromStr};
use std::collections::HashMap;

mod script;
mod task_settings;
pub use script::*;
pub use task_settings::*;

/// Rusk config file.
#[derive(Serialize, Deserialize)]
pub struct Config {
    /// Pairs of Names and Tasks.
    pub tasks: HashMap<String, Task>,
}

#[serde_as]
#[derive(Serialize, Deserialize)]
/// One indivisual Task.
pub struct Task {
    /// The script that describes the operation of the task.
    #[serde_as(as = "DisplayFromStr")]
    pub script: Script,
    /// Task behavior settings.
    #[serde(flatten)]
    pub config: TaskSettings,
}
