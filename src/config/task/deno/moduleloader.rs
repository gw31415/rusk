// The MIT License (MIT)
// Copyright © 2023 @skanehira
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the “Software”), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED “AS IS”, WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use deno_ast::{parse_module, EmitOptions, ParseParams, SourceTextInfo};
use deno_runtime::{
    deno_core::{
        anyhow::Error, error::generic_error, futures::future::FutureExt, resolve_import,
        FastString, ModuleLoader, ModuleSource, ModuleSourceFuture, ModuleSpecifier, ModuleType,
        ResolutionKind,
    },
    deno_fetch::create_http_client,
};
use std::future::Future;
use std::pin::Pin;

pub struct XTaskModuleLoader;

impl ModuleLoader for XTaskModuleLoader {
    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            let code = if module_specifier.scheme().starts_with("http") {
                println!("Downloading: {}", module_specifier);
                let client = create_http_client("deno", Default::default())?;
                let resp = client.get(module_specifier.to_string()).send().await?;
                resp.bytes().await?.to_vec()
            } else {
                let path = module_specifier.to_file_path().map_err(|_| {
                    generic_error(format!(
                        "Provided module specifier \"{}\" is not a file URL.",
                        module_specifier
                    ))
                })?;
                std::fs::read(path)?
            };

            let module_type = if module_specifier.to_string().ends_with(".json") {
                ModuleType::Json
            } else {
                ModuleType::JavaScript
            };

            let code = if module_type == ModuleType::JavaScript {
                let source = String::from_utf8(code).unwrap();
                let parsed_source = parse_module(ParseParams {
                    specifier: module_specifier.clone().into(),
                    media_type: deno_ast::MediaType::TypeScript,
                    text_info: SourceTextInfo::new(source.into()),
                    capture_tokens: false,
                    maybe_syntax: None,
                    scope_analysis: false,
                })
                .unwrap();
                let options = EmitOptions::default();
                let source = parsed_source.transpile(&options).unwrap();
                source.text.as_bytes().to_vec()
            } else {
                code
            };

            let module = ModuleSource::new_with_redirect(
                module_type,
                FastString::Owned(String::from_utf8(code)?.into_boxed_str()),
                &module_specifier,
                &module_specifier,
            );
            Ok(module)
        }
        .boxed_local()
    }

    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _kind: ResolutionKind,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn prepare_load(
        &self,
        _module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<String>,
        _is_dyn_import: bool,
    ) -> Pin<Box<dyn Future<Output = Result<(), Error>>>> {
        async { Ok(()) }.boxed_local()
    }
}
