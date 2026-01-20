use clap::{
    Parser, Subcommand, ValueEnum,
    builder::styling::{AnsiColor, Color, Style, Styles},
};

const ABOUT: &str = "Process text through customizable LLM templates";

const AFTER_HELP: &str = concat!(
    "\x1b[1;33mExamples:\x1b[0m\n",
    "  \x1b[36mord run grammar -f text -c -n\x1b[0m\n",
    "      Read clipboard, fix grammar, copy result back\n",
    "\n",
    "  \x1b[36mord run summarize -f text -b\x1b[0m\n",
    "      Read clipboard, open result as rendered markdown\n",
    "\n",
    "  \x1b[36mord run commit_message text=\"$(git diff --staged)\"\x1b[0m\n",
    "      Generate a commit message from staged changes\n",
    "\n",
    "\x1b[1;33mConfiguration:\x1b[0m\n",
    "  Stored in \x1b[36mord/\x1b[0m under your OS config directory:\n",
    "      Linux:   \x1b[2m~/.config  (or $XDG_CONFIG_HOME)\x1b[0m\n",
    "      macOS:   \x1b[2m~/Library/Application Support\x1b[0m\n",
    "      Windows: \x1b[2m%APPDATA%\x1b[0m\n",
    "\n",
    "  Set \x1b[36mANTHROPIC_API_KEY\x1b[0m or \x1b[36mOPENAI_API_KEY\x1b[0m environment variable.\n",
    "\n",
    "\x1b[1;33mMore info:\x1b[0m\n",
    "  \x1b[36m",
    env!("CARGO_PKG_HOMEPAGE"),
    "\x1b[0m"
);

/// Available CLI subcommands.
#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(
        about = "Run a template to process text",
        visible_alias = "r",
        long_about = "Run a template to process text through the configured LLM. \
                      If template or entries are omitted, interactive mode will guide you."
    )]
    Run(RunArgs),
    #[command(
        about = "Show template details",
        long_about = "Display the full details of a specific template including its \
                      description, placeholders, and prompt content. If no template is \
                      specified, interactive selection will be available."
    )]
    Info {
        #[arg(help = "Template name")]
        title: Option<String>,
    },
    #[command(about = "List all available templates")]
    List,
}

/// Root CLI argument parser.
#[derive(Parser, Debug)]
#[command(
    name = "ord",
    version,
    about = ABOUT,
    after_help = AFTER_HELP,
    arg_required_else_help = false,
    styles = get_styles(),
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

const fn get_styles() -> Styles {
    Styles::styled()
        .usage(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .header(
            Style::new()
                .bold()
                .fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
        )
        .literal(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))))
        .placeholder(Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))))
}

/// Arguments for the `run` subcommand.
#[derive(Parser, Debug)]
pub struct RunArgs {
    #[arg(value_name = "TEMPLATE", help = "Template name to run")]
    pub title: Option<String>,

    #[arg(
        value_name = "KEY=VALUE",
        help = "Placeholder values (e.g., name=Alice)"
    )]
    pub entries: Vec<String>,

    #[arg(
        long,
        short,
        value_name = "KEY",
        help = "Fill placeholder from clipboard content"
    )]
    pub from_clipboard: Option<String>,

    #[arg(long, short, help = "Copy result to clipboard")]
    pub copy_to_clipboard: bool,

    #[arg(
        short,
        long,
        default_missing_value = "status",
        num_args = 0..=1,
        value_name = "TYPE",
        require_equals = true,
        help = "Send notification on completion [status|content]"
    )]
    pub notify: Option<NotificationType>,

    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = "markdown",
        value_name = "FORMAT",
        require_equals = true,
        help = "Open result in browser [markdown|plain]"
    )]
    pub browser: Option<BrowserType>,

    #[arg(long, help = "Omit trailing newline from output")]
    pub no_newline: bool,
}

/// Type of notification to send on completion.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum NotificationType {
    #[clap(help = "Show a simple completion notification (default)")]
    Status,
    #[clap(help = "Display the full result in the notification")]
    Content,
}

/// Format for browser output.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum BrowserType {
    #[clap(help = "Parse the result as Markdown before displaying (default)")]
    Markdown,
    #[clap(help = "Display the result without parsing")]
    Plain,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn parse_args(args: &[&str]) -> Args {
        let mut full_args = vec!["ord"];
        full_args.extend(args);
        Args::try_parse_from(full_args).unwrap()
    }

    fn parse_args_err(args: &[&str]) -> clap::Error {
        let mut full_args = vec!["ord"];
        full_args.extend(args);
        Args::try_parse_from(full_args).unwrap_err()
    }

    #[test]
    fn no_command_returns_none() {
        let args = parse_args(&[]);
        assert!(args.command.is_none());
    }

    #[test]
    fn run_with_template_title() {
        let args = parse_args(&["run", "my_template"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.title, Some("my_template".to_string()));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_alias_r_works() {
        let args = parse_args(&["r", "my_template"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.title, Some("my_template".to_string()));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_entries() {
        let args = parse_args(&["run", "template", "name=John", "place=Paris"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.entries, vec!["name=John", "place=Paris"]);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_from_clipboard() {
        let args = parse_args(&["run", "template", "--from-clipboard=text"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.from_clipboard, Some("text".to_string()));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_from_clipboard_short() {
        let args = parse_args(&["run", "template", "-f", "text"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.from_clipboard, Some("text".to_string()));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_copy_to_clipboard() {
        let args = parse_args(&["run", "template", "--copy-to-clipboard"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert!(run_args.copy_to_clipboard);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_copy_to_clipboard_short() {
        let args = parse_args(&["run", "template", "-c"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert!(run_args.copy_to_clipboard);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_notify_default() {
        let args = parse_args(&["run", "template", "--notify"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.notify, Some(NotificationType::Status));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_notify_status() {
        let args = parse_args(&["run", "template", "--notify=status"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.notify, Some(NotificationType::Status));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_notify_content() {
        let args = parse_args(&["run", "template", "--notify=content"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.notify, Some(NotificationType::Content));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_browser_default() {
        let args = parse_args(&["run", "template", "--browser"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.browser, Some(BrowserType::Markdown));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_browser_markdown() {
        let args = parse_args(&["run", "template", "--browser=markdown"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.browser, Some(BrowserType::Markdown));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_browser_plain() {
        let args = parse_args(&["run", "template", "--browser=plain"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.browser, Some(BrowserType::Plain));
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_with_no_newline() {
        let args = parse_args(&["run", "template", "--no-newline"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert!(run_args.no_newline);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn run_without_title_for_interactive_mode() {
        let args = parse_args(&["run"]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert!(run_args.title.is_none());
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn list_command() {
        let args = parse_args(&["list"]);
        assert!(matches!(args.command, Some(Command::List)));
    }

    #[test]
    fn info_with_title() {
        let args = parse_args(&["info", "my_template"]);
        match args.command {
            Some(Command::Info { title }) => {
                assert_eq!(title, Some("my_template".to_string()));
            }
            _ => panic!("Expected Info command"),
        }
    }

    #[test]
    fn info_without_title_for_interactive_mode() {
        let args = parse_args(&["info"]);
        match args.command {
            Some(Command::Info { title }) => {
                assert!(title.is_none());
            }
            _ => panic!("Expected Info command"),
        }
    }

    #[test]
    fn run_with_multiple_flags() {
        let args = parse_args(&[
            "r",
            "template",
            "name=Alice",
            "-f",
            "text",
            "-c",
            "--notify=content",
            "--browser=plain",
            "--no-newline",
        ]);
        match args.command {
            Some(Command::Run(run_args)) => {
                assert_eq!(run_args.title, Some("template".to_string()));
                assert_eq!(run_args.entries, vec!["name=Alice"]);
                assert_eq!(run_args.from_clipboard, Some("text".to_string()));
                assert!(run_args.copy_to_clipboard);
                assert_eq!(run_args.notify, Some(NotificationType::Content));
                assert_eq!(run_args.browser, Some(BrowserType::Plain));
                assert!(run_args.no_newline);
            }
            _ => panic!("Expected Run command"),
        }
    }

    #[test]
    fn invalid_command_fails() {
        let err = parse_args_err(&["invalid_command"]);
        assert!(err.kind() == clap::error::ErrorKind::InvalidSubcommand);
    }

    #[test]
    fn invalid_notify_value_fails() {
        let err = parse_args_err(&["run", "--notify=invalid"]);
        assert!(err.kind() == clap::error::ErrorKind::InvalidValue);
    }

    #[test]
    fn invalid_browser_value_fails() {
        let err = parse_args_err(&["run", "--browser=invalid"]);
        assert!(err.kind() == clap::error::ErrorKind::InvalidValue);
    }
}
