use std::path::PathBuf;

use dirs::config_dir;

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_dict_path: config_dir().unwrap().join("harper-ls/dictionary.txt"),
        }
    }
}
