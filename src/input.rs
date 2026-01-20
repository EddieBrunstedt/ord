use crate::error::{Error, Result};
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::{process, sync::Once};

const USER_INTERRUPT_MSG: &str = "\nOperation cancelled by user. Exiting...";

static CTRL_C_HANDLER: Once = Once::new();

/// Prompts the user for text input and returns the entered string.
pub fn get_text(prompt: &str) -> Result<String> {
    setup_ctrl_c_handler();

    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .map_err(|e| match e {
            dialoguer::Error::IO(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                println!("{USER_INTERRUPT_MSG}");
                process::exit(0);
            }
            _ => Error::FailProcessUserInput(e),
        })
}

/// Prompts the user to select from a list of items and returns the selected index.
pub fn get_selection<T: ToString + std::fmt::Display>(prompt: &str, items: &[T]) -> Result<usize> {
    setup_ctrl_c_handler();

    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
        .map_err(|e| match e {
            dialoguer::Error::IO(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                println!("{USER_INTERRUPT_MSG}");
                process::exit(0);
            }
            _ => Error::FailProcessUserInput(e),
        })
}

/// Prompts the user for a yes/no confirmation and returns the result.
pub fn get_confirmation(prompt: &str) -> Result<bool> {
    setup_ctrl_c_handler();

    Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(true)
        .interact()
        .map_err(|e| match e {
            dialoguer::Error::IO(ref err) if err.kind() == std::io::ErrorKind::Interrupted => {
                println!("{USER_INTERRUPT_MSG}");
                process::exit(0);
            }
            _ => Error::FailProcessUserInput(e),
        })
}

fn setup_ctrl_c_handler() {
    CTRL_C_HANDLER.call_once(|| {
        let _ = ctrlc::set_handler(|| {
            let _ = console::Term::stdout().show_cursor();
            process::exit(0);
        });
    });
}
