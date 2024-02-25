use std::path::PathBuf;

use dirs::{config_dir, data_local_dir};

#[derive(Debug, Clone)]
pub struct Config {
    pub user_dict_path: PathBuf,
    pub file_dict_path: PathBuf
}

impl Default for Config {
    fn default() -> Self {
        Self {
            user_dict_path: config_dir().unwrap().join("harper-ls/dictionary.txt"),
            file_dict_path: data_local_dir()
                .unwrap()
                .join("harper-ls/file_dictionaries/")
        }
    }
}
