use crate::client::Provider;
use crate::error::{BoxedTomlError, Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

const CONFIG_DIR_NAME: &str = "ord";
const CONFIG_FILE_NAME: &str = "config.toml";
const TEMPLATES_DIR_NAME: &str = "templates";

const DEFAULT_PROVIDER: &str = "anthropic";
const DEFAULT_MODEL: &str = "claude-sonnet-4-5";
const DEFAULT_MAX_TOKENS: u64 = 4096;

/// Application configuration loaded from the config file.
#[derive(Debug)]
pub struct Config {
    pub config_file_path: PathBuf,
    pub templates_dir_path: PathBuf,
    pub provider: Provider,
    pub model: String,
    pub max_tokens: u64,
}

impl Config {
    /// Loads configuration from the OS-specific config directory.
    ///
    /// Creates the config file with defaults if it doesn't exist.
    pub fn load() -> Result<Self> {
        let base_path = get_os_config_path()?;
        let config_file_path = ensure_file_exists(base_path.join(CONFIG_FILE_NAME))?;
        let templates_dir_path = ensure_dir_exists(base_path.join(TEMPLATES_DIR_NAME))?;

        let config_str =
            fs::read_to_string(&config_file_path).map_err(|e| Error::FailReadFile {
                path: config_file_path.clone(),
                origin_error: e,
            })?;

        let toml_config = parse_toml(&config_str, &config_file_path)?;

        Ok(Self {
            config_file_path,
            templates_dir_path,
            provider: toml_config.provider.parse()?,
            model: toml_config.model,
            max_tokens: toml_config.max_tokens,
        })
    }

    /// Validates that required configuration values are present.
    pub fn validate(&self) -> Result<()> {
        if self.model.is_empty() {
            return Err(Error::FailValidateConfigModelEmpty {
                config_file_path: self.config_file_path.clone(),
            });
        }

        Ok(())
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub(crate) struct TomlConfig {
    pub model: String,
    pub provider: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u64,
}

impl Default for TomlConfig {
    fn default() -> Self {
        Self {
            provider: DEFAULT_PROVIDER.to_string(),
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
        }
    }
}

fn default_max_tokens() -> u64 {
    4096
}

pub(crate) fn parse_toml(config_str: &str, config_path: &Path) -> Result<TomlConfig> {
    toml::from_str::<TomlConfig>(config_str).map_err(|origin_error| {
        Error::FailDeserializeToml(Box::new(BoxedTomlError {
            path: config_path.to_path_buf(),
            origin_error: Box::new(origin_error),
        }))
    })
}

fn get_os_config_path() -> Result<PathBuf> {
    let base_path = match env::consts::OS {
        "windows" => get_path_from_env("APPDATA"),
        "macos" => get_config_path_with_fallback("XDG_CONFIG_HOME", || {
            get_path_from_env("HOME").map(|home| home.join("Library").join("Application Support"))
        }),
        _ => get_config_path_with_fallback("XDG_CONFIG_HOME", || {
            get_path_from_env("HOME").map(|home| home.join(".config"))
        }),
    }?;

    ensure_dir_exists(base_path.join(CONFIG_DIR_NAME))
}

fn get_config_path_with_fallback<F>(primary_env: &str, fallback: F) -> Result<PathBuf>
where
    F: FnOnce() -> Result<PathBuf>,
{
    get_path_from_env(primary_env).or_else(|_| fallback())
}

fn get_path_from_env(env_var: &str) -> Result<PathBuf> {
    env::var(env_var)
        .map(PathBuf::from)
        .map_err(|e| Error::FailReadEnvVariable {
            env_variable: env_var.to_string(),
            origin_error: e,
        })
}

fn ensure_dir_exists(path: PathBuf) -> Result<PathBuf> {
    fs::create_dir_all(&path).map_err(|e| Error::FailCreateDirectory {
        path: path.clone(),
        origin_error: e,
    })?;
    Ok(path)
}

fn ensure_file_exists(path: PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        let file = fs::File::create(&path).map_err(|e| Error::FailCreateFile {
            path: path.clone(),
            origin_error: e,
        })?;

        if path.file_name().is_some_and(|n| n == CONFIG_FILE_NAME) {
            let mut writer = BufWriter::new(file);
            let default_config = default_toml_config();
            writeln!(writer, "{default_config}").map_err(|e| Error::FailWriteFile {
                path: path.clone(),
                origin_error: e,
            })?;
            writer.flush().map_err(|e| Error::FailWriteFile {
                path: path.clone(),
                origin_error: e,
            })?;
        }
    }
    Ok(path)
}

fn default_toml_config() -> String {
    format!(
        "# {} configuration\n\n{}",
        env!("CARGO_PKG_NAME"),
        toml::to_string(&TomlConfig::default()).unwrap_or_default()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn provider_from_str_parses_anthropic() {
        let provider: Provider = "anthropic".parse().unwrap();
        assert!(matches!(provider, Provider::Anthropic));
    }

    #[test]
    fn provider_from_str_parses_ollama() {
        let provider: Provider = "ollama".parse().unwrap();
        assert!(matches!(provider, Provider::Ollama));
    }

    #[test]
    fn provider_from_str_parses_openai() {
        let provider: Provider = "openai".parse().unwrap();
        assert!(matches!(provider, Provider::OpenAI));
    }

    #[test]
    fn provider_from_str_parses_openrouter() {
        let provider: Provider = "openrouter".parse().unwrap();
        assert!(matches!(provider, Provider::OpenRouter));
    }

    #[test]
    fn provider_from_str_fails_for_unknown() {
        let result: Result<Provider> = "unknown".parse();
        assert_matches!(result, Err(Error::UnsupportedProvider { .. }));
    }

    #[test]
    fn provider_from_str_is_case_sensitive() {
        let result: Result<Provider> = "Anthropic".parse();
        assert_matches!(result, Err(Error::UnsupportedProvider { .. }));
    }

    #[test]
    fn parse_toml_parses_valid_config() {
        let config_str = r#"
            model = "gpt-4"
            provider = "openai"
            max_tokens = 1024
        "#;

        let config = parse_toml(config_str, Path::new("test.toml")).unwrap();

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.provider, "openai");
        assert_eq!(config.max_tokens, 1024);
    }

    #[test]
    fn parse_toml_parses_valid_config_with_default_max_tokens() {
        let config_str = r#"
            model = "gpt-4"
            provider = "openai"
        "#;

        let config = parse_toml(config_str, Path::new("test.toml")).unwrap();

        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.provider, "openai");
        assert_eq!(config.max_tokens, 4096);
    }

    #[test]
    fn parse_toml_parses_anthropic_config() {
        let config_str = r#"
            model = "claude-3-opus"
            provider = "anthropic"
            max_tokens = 8192
        "#;

        let config = parse_toml(config_str, Path::new("test.toml")).unwrap();

        assert_eq!(config.model, "claude-3-opus");
        assert_eq!(config.provider, "anthropic");
        assert_eq!(config.max_tokens, 8192);
    }

    #[test]
    fn parse_toml_fails_on_invalid_toml() {
        let config_str = "invalid toml { syntax";
        let result = parse_toml(config_str, Path::new("test.toml"));

        assert_matches!(result, Err(Error::FailDeserializeToml(_)));
    }

    #[test]
    fn parse_toml_fails_on_missing_fields() {
        let config_str = r#"
            model = "gpt-4"
        "#;

        let result = parse_toml(config_str, Path::new("test.toml"));
        assert_matches!(result, Err(Error::FailDeserializeToml(_)));
    }

    #[test]
    fn config_validate_succeeds_with_model() {
        let config = Config {
            config_file_path: PathBuf::from("test.toml"),
            templates_dir_path: PathBuf::from("templates"),
            provider: Provider::OpenAI,
            model: "gpt-4".to_string(),
            max_tokens: 4096,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn config_validate_fails_with_empty_model() {
        let config = Config {
            config_file_path: PathBuf::from("test.toml"),
            templates_dir_path: PathBuf::from("templates"),
            provider: Provider::OpenAI,
            model: "".to_string(),
            max_tokens: 4096,
        };

        let result = config.validate();
        assert_matches!(result, Err(Error::FailValidateConfigModelEmpty { .. }));
    }

    #[test]
    fn ensure_dir_exists_creates_directory() {
        let dir = TempDir::new().unwrap();
        let new_dir = dir.path().join("new_directory");

        let result = ensure_dir_exists(new_dir.clone()).unwrap();

        assert_eq!(result, new_dir);
        assert!(new_dir.exists());
    }

    #[test]
    fn ensure_dir_exists_succeeds_for_existing_directory() {
        let dir = TempDir::new().unwrap();

        let result = ensure_dir_exists(dir.path().to_path_buf()).unwrap();

        assert_eq!(result, dir.path());
    }

    #[test]
    fn ensure_file_exists_creates_config_file_with_defaults() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);

        let result = ensure_file_exists(config_path.clone()).unwrap();

        assert_eq!(result, config_path);
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("# ord configuration"));
    }

    #[test]
    fn ensure_file_exists_does_not_overwrite_existing_file() {
        let dir = TempDir::new().unwrap();
        let config_path = dir.path().join(CONFIG_FILE_NAME);

        let mut file = fs::File::create(&config_path).unwrap();
        writeln!(file, "custom content").unwrap();

        ensure_file_exists(config_path.clone()).unwrap();

        let content = fs::read_to_string(&config_path).unwrap();
        assert_eq!(content.trim(), "custom content");
    }
}
