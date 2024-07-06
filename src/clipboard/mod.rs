use crate::error::{Error, Result};
use copypasta_ext::ClipboardProviderExt;
use std::sync::Mutex;

pub trait Clipboard: Send + Sync {
    fn read(&self) -> Result<String>;
    fn write(&self, content: String) -> Result<()>;
}

pub struct ClipboardManager {
    provider: Mutex<Box<dyn ClipboardProviderExt>>,
}

impl ClipboardManager {
    pub fn new(provider: Box<dyn ClipboardProviderExt>) -> Self {
        Self {
            provider: Mutex::new(provider),
        }
    }
}

impl Clipboard for ClipboardManager {
    fn read(&self) -> Result<String> {
        self.provider
            .lock()
            .map_err(|e| Error::MutexLock(e.to_string()))?
            .get_contents()
            .map_err(|e| Error::FailReadClipboard(e.to_string()))
    }

    fn write(&self, content: String) -> Result<()> {
        self.provider
            .lock()
            .map_err(|e| Error::MutexLock(e.to_string()))?
            .set_contents(content)
            .map_err(|e| Error::FailWriteClipboard(e.to_string()))
    }
}
