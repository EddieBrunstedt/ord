mod paths;

use crate::error::{BoxedTomlError, Error, Result};
use console::user_attended;
use paths::UserPaths;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const OPENAI_API_KEY_ENV_VAR: &str = "OPENAI_API_KEY";

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
        self.validate_model()
    }
}

impl UserConfig {
    pub fn new_with_env<E: EnvService>(env_service: &E) -> Result<Self> {
        let user_paths = UserPaths::new()?;
        let config_str = Self::read_config_file(&user_paths.config_file)?;
        let toml_config = Self::parse_toml_config(&config_str, &user_paths.config_file)?;
        let api_key = Self::get_api_key_from_env_or_config(
            env_service,
            &toml_config,
            &user_paths.config_file,
        )?;

        Ok(Self {
            api_key,
            model: toml_config.model,
            templates_dir_path: user_paths.templates_dir,
            config_file_path: user_paths.config_file,
        })
    }

    pub fn new() -> Result<Self> {
        Self::new_with_env(&SystemEnvService)
    }

    fn read_config_file(path: &PathBuf) -> Result<String> {
        fs::read_to_string(path).map_err(|e| Error::FailReadFile {
            path: path.clone(),
            origin_error: e,
        })
    }

    fn get_api_key_from_env_or_config<E: EnvService>(
        env_service: &E,
        toml_config: &TomlConfig,
        config_path: &Path,
    ) -> Result<String> {
        let api_key =
            Self::get_api_key_from_env(env_service).or_else(|| toml_config.api_key.clone());

        api_key.ok_or(Error::FailNoApiKey {
            config_file_path: config_path.to_path_buf(),
            env_variable_name: OPENAI_API_KEY_ENV_VAR.to_string(),
        })
    }

    fn get_api_key_from_env<E: EnvService>(env_service: &E) -> Option<String> {
        env_service.get_var(OPENAI_API_KEY_ENV_VAR)
    }

    fn parse_toml_config(config_str: &str, config_path: &Path) -> Result<TomlConfig> {
        toml::from_str::<TomlConfig>(config_str).map_err(|origin_error| {
            Error::FailDeserializeToml(Box::new(BoxedTomlError {
                path: config_path.to_path_buf(),
                origin_error: Box::new(origin_error),
            }))
        })
    }

    fn validate_model(&self) -> Result<()> {
        if self.model.is_empty() {
            return Err(Error::FailValidateConfigModelEmpty {
                config_file_path: self.config_file_path.clone(),
            });
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug, Default)]
struct TomlConfig {
    pub api_key: Option<String>,
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
    format!("# {} configuration", env!("CARGO_PKG_NAME"))
}

pub trait EnvService {
    fn get_var(&self, key: &str) -> Option<String>;
}

pub struct SystemEnvService;

impl EnvService for SystemEnvService {
    fn get_var(&self, key: &str) -> Option<String> {
        env::var(key).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;

    pub struct MockEnvService {
        variables: std::collections::HashMap<String, String>,
    }

    impl MockEnvService {
        fn new() -> Self {
            Self {
                variables: std::collections::HashMap::new(),
            }
        }

        pub fn with_var(mut self, key: &str, value: &str) -> Self {
            self.variables.insert(key.to_string(), value.to_string());
            self
        }
    }

    impl EnvService for MockEnvService {
        fn get_var(&self, key: &str) -> Option<String> {
            self.variables.get(key).cloned()
        }
    }

    #[test]
    fn validate_config_ok() {
        let config = UserConfig {
            api_key: "very_valid_key".to_string(),
            model: "very_valid_model".to_string(),
            templates_dir_path: PathBuf::from("/not-used-in-test"),
            config_file_path: PathBuf::from("/not-used-in-test/config.toml"),
        };

        assert!(config.validate_config().is_ok());
    }

    #[test]
    fn validate_config_error_empty_model() {
        let fx_templates_dir_path = "/some-path";
        let fx_config_file_path = "/some-path/config.toml";

        let config = UserConfig {
            api_key: "valid_key".to_string(),
            model: "".to_string(),
            templates_dir_path: PathBuf::from(fx_templates_dir_path),
            config_file_path: PathBuf::from(fx_config_file_path),
        };

        assert_matches!(
            config.validate_config(),
            Err(Error::FailValidateConfigModelEmpty { config_file_path }) if config_file_path == PathBuf::from(fx_config_file_path),
            "Expected FailValidateConfigModelEmpty error with correct path"
        );
    }

    #[test]
    fn default_toml_config_ok() {
        let default_config = default_toml_config();
        assert!(default_config.contains("# ord configuration"));
        assert!(default_config.contains("model"));
    }

    #[test]
    fn validate_env_takes_precedence() {
        let fx_env_api_key = "env_api_key_test";
        let mock_env = MockEnvService::new().with_var(OPENAI_API_KEY_ENV_VAR, fx_env_api_key);

        let fx_toml_api_key = "toml_api_key_test";
        let toml_config = TomlConfig {
            api_key: Some(fx_toml_api_key.to_string()),
            model: "some_model".to_string(),
        };

        let result = UserConfig::get_api_key_from_env_or_config(
            &mock_env,
            &toml_config,
            Path::new("/dummy/path"),
        );

        assert_matches!(result, Ok(key) if key == fx_env_api_key);
    }

    #[test]
    fn validate_toml_fallback() {
        let mock_env = MockEnvService::new();

        let fx_toml_api_key = "toml_api_key_test";
        let toml_config = TomlConfig {
            api_key: Some(fx_toml_api_key.to_string()),
            model: "some_model".to_string(),
        };

        let result = UserConfig::get_api_key_from_env_or_config(
            &mock_env,
            &toml_config,
            Path::new("/dummy/path"),
        );

        assert_matches!(result, Ok(key) if key == fx_toml_api_key);
    }

    #[test]
    fn validate_no_api_key_error() {
        let mock_env = MockEnvService::new();

        let toml_config = TomlConfig {
            api_key: None,
            model: "some_model".to_string(),
        };

        let fx_config_path = Path::new("/dummy/path");

        let result =
            UserConfig::get_api_key_from_env_or_config(&mock_env, &toml_config, fx_config_path);

        assert_matches!(
            result,
            Err(Error::FailNoApiKey { config_file_path, env_variable_name })
            if config_file_path == fx_config_path && env_variable_name == OPENAI_API_KEY_ENV_VAR
        );
    }
}
