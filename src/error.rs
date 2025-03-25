use std::{fmt, path::PathBuf};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

// Constants for repeated strings
const EXAMPLE_TEMPLATES_URL: &str = concat!(
    env!("CARGO_PKG_HOMEPAGE"),
    "/blob/main/",
    env!("TEMPLATE_EXAMPLES_FOLDER")
);
const API_KEYS_URL: &str = "https://platform.openai.com/api-keys";
const MODELS_URL: &str = "https://platform.openai.com/docs/models";

#[derive(Error, Debug)]
pub enum Error {
    // File and Directory Operations
    #[error("Failed to read file '{path}':\n{origin_error}")]
    FailReadFile {
        path: PathBuf,
        origin_error: std::io::Error,
    },

    #[error("Failed to write to file '{path}':\n{origin_error}")]
    FailWriteFile {
        path: String,
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

    #[error("Failed to create directory '{path}':\n{origin_error}")]
    FailCreateDirectory {
        path: String,
        origin_error: std::io::Error,
    },

    #[error("Failed to create file '{path}':\n{origin_error}")]
    FailCreateFile {
        path: String,
        origin_error: std::io::Error,
    },

    // Path and Format Errors
    #[error("Invalid file path '{path}'. Ensure it contains valid Unicode characters.")]
    FailParsePathUnicode { path: PathBuf },

    #[error("Invalid path: '{path}'")]
    InvalidPath { path: PathBuf },

    #[error("Invalid entry format: '{entry}'\nEntries should be formatted as '<PLACEHOLDER>=<REPLACEMENT>'.")]
    InvalidEntryFormat { entry: String },

    // Parsing Errors
    #[error("Failed to parse YAML:\n{origin_error}\n\nRequired metadata format:\n{expected_structure}\n\nEnsure this YAML block is present at the beginning of your template.")]
    FailParseYaml {
        origin_error: serde_yaml::Error,
        expected_structure: String,
    },

    #[error(transparent)]
    FailDeserializeToml(#[from] Box<BoxedTomlError>),

    // Template Errors
    #[error("Duplicate placeholder: '{placeholder}'\nOnly one replacement value is allowed for each placeholder.")]
    DuplicatePlaceholderProvided { placeholder: String },

    #[error("Unfulfilled placeholders: {placeholders:?}\nPlease provide values for all placeholders in the template.")]
    UnfulfilledPlaceholders { placeholders: Vec<String> },

    #[error("Excess entries detected. The following placeholders are not part of the template:\n{excess_entries:?}\n\nPlease remove these entries or check for typos.")]
    ExcessEntries { excess_entries: Vec<String> },

    #[error("The template at '{path}' contains no placeholders.\nPlease add placeholders to the template or verify that you're using the correct file.")]
    TemplateContainsNoPlaceholders { path: PathBuf },

    #[error("Duplicate template titles detected:\n{duplicate_titles:?}\n\nPlease ensure all template titles are unique.")]
    DuplicateTemplateTitles { duplicate_titles: Vec<String> },

    #[error("No template found with title '{template_title}'.\nPlease check the template title and try again.")]
    TemplateNotFound { template_title: String },

    #[error("No template available for index {index}.\nPlease ensure the index is within the range of available templates.")]
    TemplateIndexNotFound { index: usize },

    #[error("The template at '{path}' is not in the correct format.\nPlease check the template structure and try again.")]
    InvalidTemplateFormat { path: PathBuf },

    #[error(
        "No valid templates found in '{templates_dir}'.\n\nFor example templates, visit:\n{EXAMPLE_TEMPLATES_URL}"
    )]
    NoTemplatesAvailable { templates_dir: PathBuf },

    // Clipboard Errors
    #[error("Failed to read from clipboard:\n{0}")]
    FailReadClipboard(String),

    #[error("Failed to write to clipboard:\n{0}")]
    FailWriteClipboard(String),

    #[error("Failed to initialize clipboard.")]
    FailInitClipboard,

    // Configuration Errors
    #[error("Failed to read environment variable '{env_variable}':\n{origin_error}")]
    FailReadEnvVariable {
        env_variable: String,
        origin_error: std::env::VarError,
    },

    #[error("Failed to get API key. Please either set the 'OPENAI_API_KEY' environment variable or add your API key to: {config_file_path}\n\nYou can find or generate your API key at: {API_KEYS_URL}")]
    FailNoApiKey {
        env_variable_name: String,
        config_file_path: PathBuf,
    },

    #[error("Failed to validate config: Model is not specified.\n\nYou can find a list of available models at: {MODELS_URL}\nPlease specify the model in: {config_file_path}")]
    FailValidateConfigModelEmpty { config_file_path: PathBuf },

    // API and Network Errors
    #[error("Failed to create request header '{header}':\n{origin_error}")]
    FailCreateHeader {
        header: String,
        origin_error: reqwest::header::InvalidHeaderValue,
    },

    #[error("Request error (Status {status}):\n{message}")]
    Api {
        status: reqwest::StatusCode,
        message: String,
    },

    #[error("HTTP request failed:\n{0}")]
    RequestFailed(#[from] reqwest::Error),

    // Concurrency Errors
    #[error("Failed to acquire mutex lock:\n{0}")]
    MutexLock(String),

    // User Input Errors
    #[error("Failed to process user input:\n{0}")]
    FailProcessUserInput(#[from] dialoguer::Error),

    // Notification Errors
    #[error("Failed to send notification:\n{0}")]
    FailSendNotification(#[from] notify_rust::error::Error),

    // General I/O Errors
    #[error("I/O operation failed:\n{0}")]
    Io(#[from] std::io::Error),

    // Control Flow Errors
    #[error("{0}")]
    CtrlC(#[from] ctrlc::Error),
}

// Box the large toml::de::Error to keep the overall Error enum small
// This stores only a pointer in the enum, with the actual error on the heap
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
