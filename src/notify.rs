use crate::error::Result;
use notify_rust::Notification;

/// Sends a desktop notification with the given summary and body.
pub fn send(summary: &str, body: &str) -> Result<()> {
    Notification::new().summary(summary).body(body).show()?;
    Ok(())
}
