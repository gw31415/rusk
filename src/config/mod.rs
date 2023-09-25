//! This module is responsible for parsing the contents of RUSKFILE.
//!
//! # Examples
//! ```
//! use std::collections::HashMap;
//! use rusk::config::{Atom, RuskFileContent, Script, ScriptType};
//!
//! let config_file: RuskFileContent = toml::from_str(r#"
//! [tasks.hello]
//! deno = {allow_read = ["."]}
//! script = '''
//! #!@deno
//! console.log("Bye, world!");
//! '''
//!
//! [tasks.bye]
//! depends = ["hello"]
//! script = '''
//! #!@deno
//! console.log("Bye, world!");
//! '''
//! "#).unwrap();
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

/// Settings written in one RUSKFILE
#[derive(Serialize, Deserialize)]
pub struct RuskFileContent {
    /// Pairs of Names and Tasks.
    pub tasks: HashMap<TaskName, Atom>,
}
