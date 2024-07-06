use crate::error::{Error, Result};
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::process;

const USER_INTERRUPT_MSG: &str = "\nOperation cancelled by user. Exiting...";

pub fn get_user_text(prompt: &str) -> Result<String> {
    // Because dialoguer hides the cursor while in process, we need to handle SIGINT to show the
    // cursor again if the user hits Ctrl+C.
    handle_ctrl_c()?;

    match Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
    {
        Ok(input) => Ok(input),
        Err(dialoguer::Error::IO(err)) if err.kind() == std::io::ErrorKind::Interrupted => {
            println!("{USER_INTERRUPT_MSG}");
            process::exit(0);
        }
        Err(e) => Err(Error::FailProcessUserInput(e)),
    }
}

pub fn get_user_selection<T: ToString>(prompt: &str, items: &[T]) -> Result<usize> {
    // Because dialoguer hides the cursor while in process, we need to handle SIGINT to show the
    // cursor again if the user hits Ctrl+C.
    handle_ctrl_c()?;

    match Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(items)
        .default(0)
        .interact()
    {
        Ok(index) => Ok(index),
        Err(dialoguer::Error::IO(err)) if err.kind() == std::io::ErrorKind::Interrupted => {
            println!("{USER_INTERRUPT_MSG}");
            process::exit(0);
        }
        Err(e) => Err(Error::FailProcessUserInput(e)),
    }
}

fn handle_ctrl_c() -> Result<()> {
    let result = ctrlc::set_handler(move || {
        let term = console::Term::stdout();
        let _ = term.show_cursor();
        std::process::exit(0);
    });

    match result {
        Ok(()) => Ok(()),
        // Since we are running this handler everytime we ask for user input, we simply swallow the
        // error if the handler is already set.
        Err(ctrlc::Error::MultipleHandlers) => Ok(()),
        Err(e) => Err(Error::CtrlC(e)),
    }
}
