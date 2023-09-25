use std::{collections::HashSet, net::SocketAddr, path::PathBuf};

use deno::re_exports::deno_core::url::Url;
use serde::{Deserialize, Serialize};

use super::TaskName;

#[derive(Serialize, Deserialize, Default)]
/// Task behavior settings.
pub struct TaskSettings {
    /// Dependent Task Name
    #[serde(default)]
    pub depends: HashSet<TaskName>,
    /// Deno behavior settings.
    #[serde(default)]
    pub deno: DenoFlags,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct DenoFlags {
    /// Vector of CLI arguments - these are user script arguments, all Deno
    /// specific flags are removed.
    pub argv: Vec<String>,
    // pub subcommand: DenoSubcommand,
    pub allow_all: bool,
    pub allow_env: Option<Vec<String>>,
    pub deny_env: Option<Vec<String>>,
    pub allow_hrtime: bool,
    pub deny_hrtime: bool,
    pub allow_net: Option<Vec<String>>,
    pub deny_net: Option<Vec<String>>,
    pub allow_ffi: Option<Vec<PathBuf>>,
    pub deny_ffi: Option<Vec<PathBuf>>,
    pub allow_read: Option<Vec<PathBuf>>,
    pub deny_read: Option<Vec<PathBuf>>,
    pub allow_run: Option<Vec<String>>,
    pub deny_run: Option<Vec<String>>,
    pub allow_sys: Option<Vec<String>>,
    pub deny_sys: Option<Vec<String>>,
    pub allow_write: Option<Vec<PathBuf>>,
    pub deny_write: Option<Vec<PathBuf>>,
    pub ca_stores: Option<Vec<String>>,
    // pub ca_data: Option<CaData>,
    pub cache_blocklist: Vec<String>,
    /// This is not exposed as an option in the CLI, it is used internally when
    /// the language server is configured with an explicit cache option.
    pub cache_path: Option<PathBuf>,
    pub cached_only: bool,
    // pub type_check_mode: TypeCheckMode,
    // pub config_flag: ConfigFlag,
    pub node_modules_dir: Option<bool>,
    pub vendor: Option<bool>,
    pub enable_testing_features: bool,
    pub ext: Option<String>,
    pub ignore: Vec<PathBuf>,
    pub import_map_path: Option<String>,
    pub inspect_brk: Option<SocketAddr>,
    pub inspect_wait: Option<SocketAddr>,
    pub inspect: Option<SocketAddr>,
    pub location: Option<Url>,
    pub lock_write: bool,
    pub lock: Option<PathBuf>,
    // pub log_level: Option<Level>,
    pub no_remote: bool,
    pub no_lock: bool,
    pub no_npm: bool,
    pub no_prompt: bool,
    pub reload: bool,
    pub seed: Option<u64>,
    pub unstable: bool,
    pub unsafely_ignore_certificate_errors: Option<Vec<String>>,
    pub v8_flags: Vec<String>,
}
