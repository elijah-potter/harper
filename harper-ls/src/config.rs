use std::path::PathBuf;

use dirs::{config_dir, data_local_dir};
use resolve_path::PathResolveExt;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
    pub file_dict_path: PathBuf,
}

impl Config {
    pub fn from_json_value(value: Value) -> anyhow::Result<Self> {
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
        }
    }
}
