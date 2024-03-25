use std::path::PathBuf;

use dirs::{config_dir, data_local_dir};
use harper_core::LintGroupConfig;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint
}

impl DiagnosticSeverity {
    /// Converts `self` to the equivalent LSP type.
    pub fn to_lsp(self) -> tower_lsp::lsp_types::DiagnosticSeverity {
        match self {
            DiagnosticSeverity::Error => tower_lsp::lsp_types::DiagnosticSeverity::ERROR,
            DiagnosticSeverity::Warning => tower_lsp::lsp_types::DiagnosticSeverity::WARNING,
            DiagnosticSeverity::Information => {
                tower_lsp::lsp_types::DiagnosticSeverity::INFORMATION
            }
            DiagnosticSeverity::Hint => tower_lsp::lsp_types::DiagnosticSeverity::HINT
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
    pub file_dict_path: PathBuf,
    pub lint_config: LintGroupConfig,
    pub diagnostic_severity: DiagnosticSeverity
}

impl Config {
    pub fn from_lsp_config(value: Value) -> anyhow::Result<Self> {
        let mut base = Config::default();

        let Value::Object(value) = value else {
            return Err(anyhow::format_err!("Settings must be an object."));
        };

        let Some(Value::Object(value)) = value.get("harper-ls") else {
            return Err(anyhow::format_err!(
                "Settings must contain a \"harper-ls\" key."
            ));
        };

        if let Some(v) = value.get("userDictPath") {
            if let Value::String(path) = v {
                base.user_dict_path = path.try_resolve()?.to_path_buf();
            } else {
                return Err(anyhow::format_err!("userDict path must be a string."));
            }
        }

        if let Some(v) = value.get("linters") {
            base.lint_config = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("diagnosticSeverity") {
            base.diagnostic_severity = serde_json::from_value(v.clone())?;
        }

        Ok(base)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_dict_path: config_dir().unwrap().join("harper-ls/dictionary.txt"),
            file_dict_path: data_local_dir()
                .unwrap()
                .join("harper-ls/file_dictionaries/"),
            lint_config: LintGroupConfig::default(),
            diagnostic_severity: DiagnosticSeverity::Hint
        }
    }
}
