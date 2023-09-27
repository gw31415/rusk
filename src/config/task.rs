use super::task_settings::TaskSettings;
use super::DenoFlags;
use super::{script::Script, ScriptType};
use deno::args::Flags;
use deno::factory::CliFactory;
use deno::file_fetcher::File;
use deno::re_exports::deno_core::url::Url;
use deno::re_exports::deno_runtime::deno_core::error::AnyError;
use deno::re_exports::deno_runtime::permissions::{Permissions, PermissionsContainer};
use deno_media_type::MediaType;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::path::PathBuf;
use std::rc::Rc;

/// A Task that is the smallest unit of the sequential relationship, and is assigned a name in the RUSKFILE.
pub type Task = Vec<(Atom, Rc<PathBuf>)>;

#[serde_as]
#[derive(Serialize, Deserialize)]
/// Task elements corresponding to a single script.
pub struct Atom {
    /// The script that describes the operation of the Atom.
    #[serde_as(as = "DisplayFromStr")]
    pub script: Script,
    /// Task behavior settings.
    #[serde(flatten)]
    pub config: TaskSettings,
}

impl Atom {
    pub async fn execute(&self, url: Url) -> Result<(), AnyError> {
        let Atom { script, config } = self;
        let Script { code, r#type } = script;
        match r#type {
            ScriptType::Deno => {
                let TaskSettings { deno, .. } = config;
                let DenoFlags {
                    argv,
                    allow_all,
                    allow_env,
                    deny_env,
                    allow_hrtime,
                    deny_hrtime,
                    allow_net,
                    deny_net,
                    allow_ffi,
                    deny_ffi,
                    allow_read,
                    deny_read,
                    allow_run,
                    deny_run,
                    allow_sys,
                    deny_sys,
                    allow_write,
                    deny_write,
                    ca_stores,
                    cache_blocklist,
                    cache_path,
                    cached_only,
                    node_modules_dir,
                    vendor,
                    enable_testing_features,
                    ext,
                    ignore,
                    import_map_path,
                    inspect_brk,
                    inspect_wait,
                    inspect,
                    location,
                    lock_write,
                    lock,
                    no_remote,
                    no_lock,
                    no_npm,
                    no_prompt,
                    reload,
                    seed,
                    unstable,
                    unsafely_ignore_certificate_errors,
                    v8_flags,
                } = deno.clone();
                let flags = Flags {
                    argv,
                    // subcommand,
                    allow_all,
                    allow_env,
                    deny_env,
                    allow_hrtime,
                    deny_hrtime,
                    allow_net,
                    deny_net,
                    allow_ffi,
                    deny_ffi,
                    allow_read,
                    deny_read,
                    allow_run,
                    deny_run,
                    allow_sys,
                    deny_sys,
                    allow_write,
                    deny_write,
                    ca_stores,
                    // ca_data,
                    cache_blocklist,
                    cache_path,
                    cached_only,
                    // type_check_mode,
                    // config_flag,
                    node_modules_dir,
                    vendor,
                    enable_testing_features,
                    ext,
                    ignore,
                    import_map_path,
                    inspect_brk,
                    inspect_wait,
                    inspect,
                    location,
                    lock_write,
                    lock,
                    // log_level,
                    no_remote,
                    no_lock,
                    no_npm,
                    no_prompt,
                    reload,
                    seed,
                    unstable,
                    unsafely_ignore_certificate_errors,
                    v8_flags,
                    ..Default::default()
                };
                run_deno(flags, code.clone().into_bytes(), url).await?;
            }
        }
        Ok(())
    }
}

async fn run_deno(flags: Flags, source: Vec<u8>, url: Url) -> Result<i32, AnyError> {
    let factory = CliFactory::from_flags(flags).await?;
    let cli_options = factory.cli_options();

    maybe_npm_install(&factory).await?;

    let file_fetcher = factory.file_fetcher()?;
    let worker_factory = factory.create_cli_main_worker_factory().await?;
    let permissions = PermissionsContainer::new(Permissions::from_options(
        &cli_options.permissions_options(),
    )?);
    // Create a dummy source file.
    let source_file = File {
        maybe_types: None,
        media_type: MediaType::TypeScript,
        source: String::from_utf8(source)?.into(),
        specifier: url.clone(),
        maybe_headers: None,
    };
    // Save our fake file into file fetcher cache
    // to allow module access by TS compiler
    file_fetcher.insert_cached(source_file);

    let mut worker = worker_factory
        .create_main_worker(url, permissions)
        .await?;
    let exit_code = worker.run().await?;
    Ok(exit_code)
}

async fn maybe_npm_install(factory: &CliFactory) -> Result<(), AnyError> {
    // ensure an "npm install" is done if the user has explicitly
    // opted into using a node_modules directory
    if factory.cli_options().node_modules_dir_enablement() == Some(true) {
        factory
            .package_json_deps_installer()
            .await?
            .ensure_top_level_install()
            .await?;
    }
    Ok(())
}
