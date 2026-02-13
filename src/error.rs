use std::{fmt, path::PathBuf};
use thiserror::Error;

/// A specialized Result type for Ord operations.
pub type Result<T> = std::result::Result<T, Error>;

/// All possible errors that can occur in Ord.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Failed to read file '{path}':\n{origin_error}")]
    FailReadFile {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Failed to write to file '{}':\n{origin_error}", path.display())]
    FailWriteFile {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Failed to read directory '{path}':\n{origin_error}")]
    FailReadDirectory {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Failed to read content of file '{path}':\n{origin_error}")]
    FailReadFileToString {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Failed to create directory '{}':\n{origin_error}", path.display())]
    FailCreateDirectory {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Failed to create file '{}':\n{origin_error}", path.display())]
    FailCreateFile {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Invalid file path '{path}'. Ensure it contains valid Unicode characters.")]
    FailParsePathUnicode { path: PathBuf },

    #[error("Invalid path: '{path}'")]
    InvalidPath { path: PathBuf },

    #[error(
        "Invalid entry format: '{entry}'\nEntries should be formatted as '<PLACEHOLDER>=<REPLACEMENT>'."
    )]
    InvalidEntryFormat { entry: String },

    #[error(
        "Failed to parse YAML:\n{origin_error}\n\nRequired metadata format:\n{expected_structure}"
    )]
    FailParseYaml {
        origin_error: serde_yaml::Error,
        expected_structure: String,
    },

    #[error(transparent)]
    FailDeserializeToml(#[from] Box<BoxedTomlError>),

    #[error(
        "Duplicate placeholder: '{placeholder}'\nOnly one replacement value is allowed for each placeholder."
    )]
    DuplicatePlaceholderProvided { placeholder: String },

    #[error(
        "Unfulfilled placeholders: {placeholders:?}\nPlease provide values for all placeholders in the template."
    )]
    UnfulfilledPlaceholders { placeholders: Vec<String> },

    #[error(
        "Excess entries detected. The following placeholders are not part of the template:\n{excess_entries:?}"
    )]
    ExcessEntries { excess_entries: Vec<String> },

    #[error("The template at '{path}' contains no placeholders.")]
    TemplateContainsNoPlaceholders { path: PathBuf },

    #[error(
        "Duplicate template titles detected:\n{duplicate_titles:?}\n\nPlease ensure all template titles are unique."
    )]
    DuplicateTemplateTitles { duplicate_titles: Vec<String> },

    #[error("No template found with title '{template_title}'.")]
    TemplateNotFound { template_title: String },

    #[error("No template available for index {index}.")]
    TemplateIndexNotFound { index: usize },

    #[error("The template at '{path}' is not in the correct format.")]
    InvalidTemplateFormat { path: PathBuf },

    #[error("No valid templates found in '{templates_dir}'.")]
    NoTemplatesAvailable { templates_dir: PathBuf },

    #[error("Failed to read from clipboard:\n{0}")]
    FailReadClipboard(String),

    #[error("Failed to write to clipboard:\n{0}")]
    FailWriteClipboard(String),

    #[error("Failed to initialize clipboard.")]
    FailInitClipboard,

    #[error("Unsupported provider '{provided}'.\n\nSupported providers: {}", crate::client::Provider::SUPPORTED.join(", "))]
    UnsupportedProvider { provided: String },

    #[error("Failed to read environment variable '{env_variable}':\n{origin_error}")]
    FailReadEnvVariable {
        env_variable: String,
        origin_error: std::env::VarError,
    },

    #[error("Model is not specified in config file: {config_file_path}")]
    FailValidateConfigModelEmpty { config_file_path: PathBuf },

    #[error("LLM client initialization failed:\n{0}")]
    LlmClientInit(#[from] rig::http_client::Error),

    #[error("LLM request failed:\n{0}")]
    LlmRequestFailed(#[from] rig::completion::PromptError),

    #[error("Failed to acquire mutex lock:\n{0}")]
    MutexLock(String),

    #[error("Failed to process user input:\n{0}")]
    FailProcessUserInput(#[from] dialoguer::Error),

    #[error("Failed to send notification:\n{0}")]
    FailSendNotification(#[from] notify_rust::error::Error),

    #[error("I/O operation failed:\n{0}")]
    Io(#[from] std::io::Error),
}

/// Wrapper for TOML deserialization errors with path context.
#[derive(Error, Debug)]
pub struct BoxedTomlError {
    pub path: PathBuf,
    pub origin_error: Box<toml::de::Error>,
}

impl fmt::Display for BoxedTomlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Failed to parse TOML from '{}': {}",
            self.path.display(),
            self.origin_error
        )
    }
}
