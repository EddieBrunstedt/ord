mod paths;

use crate::error::{BoxedTomlError, Error, Result};
use paths::UserPaths;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub struct UserConfig {
    pub api_key: String,
    pub model: String,
    pub templates_dir_path: PathBuf,
    pub config_file_path: PathBuf,
}

pub trait ValidateConfig {
    fn validate_config(&self) -> Result<()>;
}

impl ValidateConfig for UserConfig {
    fn validate_config(&self) -> Result<()> {
        self.validate_api_key()?;
        self.validate_model()
    }
}

impl UserConfig {
    pub fn new() -> Result<Self> {
        let user_paths = UserPaths::new()?;
        Self::from_paths(user_paths)
    }

    pub fn from_paths(user_paths: UserPaths) -> Result<Self> {
        let config_str = Self::read_config_file(&user_paths.config_file)?;
        let toml_config = Self::parse_toml_config(&config_str, &user_paths.config_file)?;

        Ok(Self {
            api_key: toml_config.api_key,
            model: toml_config.model,
            templates_dir_path: user_paths.templates_dir,
            config_file_path: user_paths.config_file,
        })
    }

    fn read_config_file(path: &PathBuf) -> Result<String> {
        fs::read_to_string(path).map_err(|e| Error::FailReadFile {
            path: path.clone(),
            origin_error: e,
        })
    }

    fn parse_toml_config(config_str: &str, config_path: &Path) -> Result<TomlConfig> {
        toml::from_str::<TomlConfig>(config_str).map_err(|origin_error| {
            Error::FailDeserializeToml(Box::new(BoxedTomlError {
                path: config_path.to_path_buf(),
                origin_error: Box::new(origin_error),
            }))
        })
    }

    fn validate_api_key(&self) -> Result<()> {
        if self.api_key.is_empty() {
            Err(Error::FailValidateConfigApiKeyEmpty {
                config_file_path: self.config_file_path.clone(),
            })
        } else {
            Ok(())
        }
    }

    fn validate_model(&self) -> Result<()> {
        if self.model.is_empty() {
            Err(Error::FailValidateConfigModelEmpty {
                config_file_path: self.config_file_path.clone(),
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct TomlConfig {
    pub api_key: String,
    pub model: String,
}

pub fn default_toml_config() -> String {
    format!(
        "{}\n\n{}",
        get_config_file_header(),
        toml::to_string(&TomlConfig::default()).unwrap_or_default()
    )
}

fn get_config_file_header() -> String {
    format!("# {} configuration", env!("CARGO_PKG_NAME"),)
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    #[test]
    fn validate_config_ok() {
        let config = UserConfig {
            api_key: "very_valid_key".to_string(),
            model: "very_valid_model".to_string(),
            templates_dir_path: PathBuf::from("/not-used-in-test"),
            config_file_path: PathBuf::from("/not-used-in-test/config.toml"),
        };

        assert!(config.validate_api_key().is_ok());
        assert!(config.validate_model().is_ok());
    }

    #[test]
    fn validate_config_error() {
        let fx_templates_dir_path = "/some-path";
        let fx_config_file_path = "/some-path/config.toml";

        let config = UserConfig {
            api_key: "".to_string(),
            model: "".to_string(),
            templates_dir_path: PathBuf::from(fx_templates_dir_path),
            config_file_path: PathBuf::from(fx_config_file_path),
        };

        assert_matches!(
            config.validate_model(),
            Err(Error::FailValidateConfigModelEmpty { config_file_path }) if config_file_path == PathBuf::from(fx_config_file_path),
            "Expected FailValidateConfigModelEmpty error with correct path"
        );

        assert_matches!(
            config.validate_api_key(),
            Err(Error::FailValidateConfigApiKeyEmpty { config_file_path }) if config_file_path == PathBuf::from(fx_config_file_path),
            "Expected FailValidateConfigApiKeyEmpty error with correct path"
        );
    }

    #[test]
    fn default_toml_config_ok() {
        let default_config = default_toml_config();
        assert!(default_config.contains("# ord configuration"));
        assert!(default_config.contains("api_key"));
        assert!(default_config.contains("model"));
    }
}
