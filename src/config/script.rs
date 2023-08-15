use std::fmt::Display;
use std::str::FromStr;
use strum::{Display, EnumString};
use thiserror::Error;

/// Error parsing script.
#[derive(Error, Debug)]
#[error("An error occurred while parsing the script.")]
pub struct ScriptParseError(String);

/// The script that describes the operation of the task.
pub struct Script {
    /// The body of the source code of the task.
    pub code: String,
    /// The type of the script.
    pub r#type: ScriptType,
}

const SHEBANG_PREFIX: &str = "#!@";

impl Display for Script {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{SHEBANG_PREFIX}{}", self.r#type)?;
        write!(f, "{}", self.code)
    }
}

impl FromStr for Script {
    type Err = ScriptParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((shebang, code)) = s.split_once('\n') {
            if let Some(script_type) = shebang.strip_prefix(SHEBANG_PREFIX) {
                if let Ok(r#type) = ScriptType::from_str(script_type) {
                    return Ok(Script {
                        code: code.to_string(),
                        r#type,
                    });
                }
            }
        }
        Err(ScriptParseError(s.to_string()))
    }
}

#[derive(Display, EnumString)]
#[strum(serialize_all = "snake_case")]
/// The type of the script.
pub enum ScriptType {
    /// deno_runtime script.
    Deno,
}
