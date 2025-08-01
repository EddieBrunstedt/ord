use super::default_toml_config;
use crate::user_config::{Error, Result};
use std::{
    env, fs,
    io::{BufWriter, Write},
    path::PathBuf,
};

const CONFIG_DIR_NAME: &str = "ord";
const CONFIG_FILE_NAME: &str = "config.toml";
const TEMPLATES_DIR_NAME: &str = "templates";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserPaths {
    pub config_file: PathBuf,
    pub templates_dir: PathBuf,
}

impl UserPaths {
    pub fn new() -> Result<Self> {
        let base_path = get_os_config_path()?;
        Ok(Self {
            config_file: ensure_file_exists(base_path.join(CONFIG_FILE_NAME))?,
            templates_dir: ensure_dir_exists(base_path.join(TEMPLATES_DIR_NAME))?,
        })
    }
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
    std::fs::create_dir_all(&path).map_err(|e| Error::FailCreateDirectory {
        path: path.to_string_lossy().to_string(),
        origin_error: e,
    })?;
    Ok(path)
}

fn ensure_file_exists(path: PathBuf) -> Result<PathBuf> {
    if !path.exists() {
        let file = fs::File::create(&path).map_err(|e| Error::FailCreateFile {
            path: path.to_string_lossy().to_string(),
            origin_error: e,
        })?;

        if path.file_name().unwrap_or_default() == CONFIG_FILE_NAME {
            let mut writer = BufWriter::new(file);
            let default_config = default_toml_config();
            writeln!(writer, "{default_config}").map_err(|e| Error::FailWriteFile {
                path: path.to_string_lossy().to_string(),
                origin_error: e,
            })?;
            writer.flush().map_err(|e| Error::FailWriteFile {
                path: path.to_string_lossy().to_string(),
                origin_error: e,
            })?;
        }
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_env() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        env::set_var("HOME", temp_dir.path());
        env::set_var("APPDATA", temp_dir.path());
        env::set_var("XDG_CONFIG_HOME", temp_dir.path().join(".config"));
        temp_dir
    }

    #[test]
    #[serial]
    fn new_user_paths_ok() {
        let _temp_dir = setup_test_env();
        let user_paths = UserPaths::new().unwrap();
        assert!(user_paths.config_file.exists());
        assert!(user_paths.templates_dir.exists());
    }

    #[test]
    #[serial]
    fn get_os_config_path_ok() {
        let temp_dir = setup_test_env();
        let config_path = get_os_config_path().unwrap();
        assert!(config_path.starts_with(temp_dir.path()));
        assert!(config_path.ends_with(CONFIG_DIR_NAME));
    }

    #[test]
    #[serial]
    fn get_path_from_env_ok() {
        let temp_dir = setup_test_env();
        let home_path = get_path_from_env("HOME").unwrap();
        assert_eq!(home_path, temp_dir.path());
    }

    #[test]
    #[serial]
    fn ensure_dir_exists_ok() {
        let temp_dir = setup_test_env();
        let test_dir = temp_dir.path().join("test_dir");
        let result = ensure_dir_exists(test_dir.clone()).unwrap();
        assert_eq!(result, test_dir);
        assert!(test_dir.exists());
    }

    #[test]
    #[serial]
    fn ensure_file_exists_ok() {
        let temp_dir = setup_test_env();
        let test_file = temp_dir.path().join("test_file.txt");
        let result = ensure_file_exists(test_file.clone()).unwrap();
        assert_eq!(result, test_file);
        assert!(test_file.exists());
    }

    #[test]
    #[serial]
    fn ensure_config_file_exists_ok() {
        let temp_dir = setup_test_env();
        let config_file = temp_dir.path().join(CONFIG_FILE_NAME);
        let result = ensure_file_exists(config_file.clone()).unwrap();
        assert_eq!(result, config_file);
        assert!(config_file.exists());
        let content = fs::read_to_string(config_file).unwrap();
        assert_eq!(content.trim(), default_toml_config().trim());
    }

    #[test]
    #[serial]
    fn get_config_path_with_fallback_ok() {
        let temp_dir = setup_test_env();
        env::remove_var("XDG_CONFIG_HOME");
        let result = get_config_path_with_fallback("XDG_CONFIG_HOME", || {
            Ok(temp_dir.path().join(".config"))
        })
        .unwrap();
        assert_eq!(result, temp_dir.path().join(".config"));
    }

    #[test]
    #[serial]
    #[cfg(target_os = "windows")]
    fn windows_config_path_ok() {
        let temp_dir = setup_test_env();
        let config_path = get_os_config_path().unwrap();
        assert_eq!(config_path, temp_dir.path().join(CONFIG_DIR_NAME));
    }

    #[test]
    #[serial]
    #[cfg(target_os = "macos")]
    fn macos_config_path_ok() {
        let temp_dir = setup_test_env();
        let config_path = get_os_config_path().unwrap();
        assert_eq!(
            config_path,
            temp_dir.path().join(".config").join(CONFIG_DIR_NAME)
        );
    }

    #[test]
    #[serial]
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    fn linux_config_path_ok() {
        let temp_dir = setup_test_env();
        let config_path = get_os_config_path().unwrap();
        assert_eq!(
            config_path,
            temp_dir.path().join(".config").join(CONFIG_DIR_NAME)
        );
    }
}
