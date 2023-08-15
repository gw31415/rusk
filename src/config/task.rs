use super::script::Script;
use super::task_settings::TaskSettings;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

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
