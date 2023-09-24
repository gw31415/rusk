mod deno;

use super::task_settings::TaskSettings;
use super::DenoSettings;
use super::{script::Script, ScriptType};
use deno::moduleloader::*;
use deno_runtime::{
    deno_core::{error::AnyError, url::Url},
    permissions::{Permissions, PermissionsContainer},
    worker::{MainWorker, WorkerOptions},
};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::fs::canonicalize;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub type Task = Vec<(Atom, Rc<PathBuf>)>;

#[serde_as]
#[derive(Serialize, Deserialize)]
/// One indivisual Task.
pub struct Atom {
    /// The script that describes the operation of the task.
    #[serde_as(as = "DisplayFromStr")]
    pub script: Script,
    /// Task behavior settings.
    #[serde(flatten)]
    pub config: TaskSettings,
}

impl Atom {
    pub async fn execute(&self, path: &Path) -> Result<(), AnyError> {
        let Atom { script, config } = self;
        let Script { code, r#type } = script;
        match r#type {
            ScriptType::Deno => {
                let TaskSettings { deno, .. } = config;
                let DenoSettings { permissions } = deno;
                let url = Url::from_file_path(canonicalize(path)?).unwrap();
                let mut worker = MainWorker::bootstrap_from_options(
                    Url::from_directory_path(canonicalize(std::env::current_dir()?)?).unwrap(),
                    PermissionsContainer::new(Permissions::from_options(permissions)?),
                    WorkerOptions {
                        module_loader: Rc::new(XTaskModuleLoader),
                        ..Default::default()
                    },
                );
                let id = worker
                    .js_runtime
                    .load_main_module(
                        &url,
                        Some(deno_runtime::deno_core::FastString::Owned(
                            code.to_owned().into_boxed_str(),
                        )),
                    )
                    .await?;
                worker.evaluate_module(id).await?;
            }
        }
        Ok(())
    }
}