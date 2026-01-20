//! Ord - Process text through customizable LLM templates.
//!
//! This crate provides a CLI tool for running text through LLM prompts defined
//! as reusable markdown templates.

mod app;
mod cli;
mod client;
mod clipboard;
mod config;
mod embedded_templates;
mod error;
mod input;
mod notify;
mod output;
mod prompt;
mod template;

pub use app::App;
pub use cli::{Args, Command, RunArgs};
pub use error::{Error, Result};

use clap::{CommandFactory, Parser};
use cli::NotificationType;

/// Runs the Ord CLI application.
///
/// Parses command-line arguments and dispatches to the appropriate subcommand:
/// - `run`: Execute a template to process text
/// - `info`: Display template details
/// - `list`: List all available templates
pub async fn run() -> Result<()> {
    let args = Args::parse();

    match &args.command {
        None => Args::command().print_help()?,
        Some(Command::Run(run_args)) => {
            let result = async {
                let app = App::new()?;
                app.run(run_args).await
            }
            .await;

            if let Err(ref err) = result {
                send_error_notification(&run_args.notify, err);
            }

            result?;
        }
        Some(Command::Info { title }) => App::new()?.info(title)?,
        Some(Command::List) => App::new()?.list(),
    }

    Ok(())
}

fn send_error_notification(notify_option: &Option<NotificationType>, err: &Error) {
    if notify_option.is_some() {
        let _ = notify::send("Ord Error", &err.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn would_send_notification(notify_option: &Option<NotificationType>) -> bool {
        notify_option.is_some()
    }

    #[test]
    fn error_notification_sent_when_notify_is_status() {
        let notify_option = Some(NotificationType::Status);
        assert!(would_send_notification(&notify_option));
    }

    #[test]
    fn error_notification_sent_when_notify_is_content() {
        let notify_option = Some(NotificationType::Content);
        assert!(would_send_notification(&notify_option));
    }

    #[test]
    fn error_notification_not_sent_when_notify_is_none() {
        let notify_option: Option<NotificationType> = None;
        assert!(!would_send_notification(&notify_option));
    }
}
