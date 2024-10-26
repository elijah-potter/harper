use std::path::PathBuf;

use dirs::{config_dir, data_local_dir};
use harper_core::linting::LintGroupConfig;
use resolve_path::PathResolveExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
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
            DiagnosticSeverity::Hint => tower_lsp::lsp_types::DiagnosticSeverity::HINT,
        }
    }
}

/// Configuration for how code actions are displayed.
/// Originally motivated by [#89](https://github.com/elijah-potter/harper/issues/89).
#[derive(Debug, Clone, Default)]
pub struct CodeActionConfig {
    /// Instructs `harper-ls` to place unstable code actions last.
    /// In this case, "unstable" refers their existence and action.
    ///
    /// For example, we always want to allow users to add "misspelled" elements
    /// to dictionary, regardless of the spelling suggestions.
    pub force_stable: bool,
}

impl CodeActionConfig {
    pub fn from_lsp_config(value: Value) -> anyhow::Result<Self> {
        let mut base = CodeActionConfig::default();

        let Value::Object(value) = value else {
            return Err(anyhow::format_err!(
                "The code action configuration must be an object."
            ));
        };

        if let Some(force_stable_val) = value.get("forceStable") {
            let Value::Bool(force_stable) = force_stable_val else {
                return Err(anyhow::format_err!("forceStable must be a boolean value."));
            };
            base.force_stable = *force_stable;
        };

        Ok(base)
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
    pub file_dict_path: PathBuf,
    pub lint_config: LintGroupConfig,
    pub diagnostic_severity: DiagnosticSeverity,
    pub code_action_config: CodeActionConfig,
    pub isolate_english: bool,
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

        if let Some(v) = value.get("fileDictPath") {
            if let Value::String(path) = v {
                base.file_dict_path = path.try_resolve()?.to_path_buf();
            } else {
                return Err(anyhow::format_err!("fileDict path must be a string."));
            }
        }

        if let Some(v) = value.get("linters") {
            base.lint_config = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("diagnosticSeverity") {
            base.diagnostic_severity = serde_json::from_value(v.clone())?;
        }

        if let Some(v) = value.get("codeActions") {
            base.code_action_config = CodeActionConfig::from_lsp_config(v.clone())?;
        }

        if let Some(v) = value.get("isolateEnglish") {
            if let Value::Bool(v) = v {
                base.isolate_english = *v;
            } else {
                return Err(anyhow::format_err!(
                    "isolateEnglish path must be a boolean."
                ));
            }
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
            diagnostic_severity: DiagnosticSeverity::Hint,
            code_action_config: CodeActionConfig::default(),
            isolate_english: false,
        }
    }
}
