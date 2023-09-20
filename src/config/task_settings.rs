use std::collections::HashSet;

use deno_runtime::permissions::PermissionsOptions as DenoPermissionsOptions;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default)]
/// Task behavior settings.
pub struct TaskSettings {
    /// Dependent Task Name
    #[serde(default)]
    pub depends: HashSet<String>,
    /// Deno behavior settings.
    #[serde(default)]
    pub deno: DenoSettings,
}

#[derive(Serialize, Deserialize, Default)]
/// Deno behavior settings.
pub struct DenoSettings {
    #[serde(flatten)]
    /// Deno Permission.
    pub permissions: DenoPermissionsOptions,
}
