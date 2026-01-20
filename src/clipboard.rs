use crate::error::{Error, Result};
use copypasta_ext::ClipboardProviderExt;
use std::sync::Mutex;

/// Thread-safe wrapper around the system clipboard.
pub struct Clipboard {
    provider: Mutex<Box<dyn ClipboardProviderExt>>,
}

impl Clipboard {
    /// Creates a new clipboard instance.
    pub fn new() -> Result<Self> {
        let provider = copypasta_ext::try_context().ok_or(Error::FailInitClipboard)?;
        Ok(Self {
            provider: Mutex::new(provider),
        })
    }

    /// Reads the current clipboard contents.
    pub fn read(&self) -> Result<String> {
        self.provider
            .lock()
            .map_err(|e| Error::MutexLock(e.to_string()))?
            .get_contents()
            .map_err(|e| Error::FailReadClipboard(e.to_string()))
    }

    /// Writes content to the clipboard.
    pub fn write(&self, content: &str) -> Result<()> {
        self.provider
            .lock()
            .map_err(|e| Error::MutexLock(e.to_string()))?
            .set_contents(content.to_string())
            .map_err(|e| Error::FailWriteClipboard(e.to_string()))
    }
}
