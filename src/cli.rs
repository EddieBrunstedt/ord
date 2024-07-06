use clap::{Parser, Subcommand, ValueEnum};

#[derive(Subcommand, Debug)]
pub enum Command {
    #[command(
        about = "Generate and process a prompt",
        visible_alias = "gen",
        long_about = "Generate and process a prompt based on a template. If title or entries are omitted, interactive mode will be activated."
    )]
    Generate(GenerateArgs),
    #[command(
        about = "Display template details",
        long_about = "Display details for a specific template. If title is omitted, interactive mode will be activated."
    )]
    Info {
        #[arg(help = "Title of the template")]
        title: Option<String>,
    },
    #[command(about = "List all available templates")]
    List,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Parser, Debug)]
pub struct GenerateArgs {
    #[arg(value_name = "TEMPLATE_TITLE", help = "Title of the template to use")]
    pub title: Option<String>,
    #[arg(
        help = "Placeholder entries in 'key=value' format",
        long_help = "Specify placeholder entries in 'key=value' format. For example: 'protagonist=Frodo' 'antagonist=Sauron' 'setting=Middle-earth'"
    )]
    pub entries: Vec<String>,
    #[arg(
        long,
        short,
        value_name = "PLACEHOLDER",
        help = "Use clipboard content for a specific placeholder",
        long_help = "Use clipboard content as the value for a specific placeholder. For example: --from-clipboard='text' replaces {{:text}} with clipboard content"
    )]
    pub from_clipboard: Option<String>,
    #[arg(long, short, help = "Copy the processed result to clipboard")]
    pub copy_to_clipboard: bool,
    #[arg(
        short,
        long,
        default_missing_value = "status",
        num_args = 0..=1,
        value_name = "TYPE",
        require_equals = true,
        help = "Display a notification after processing"
    )]
    pub notify: Option<NotificationType>,
    #[arg(
        short,
        long,
        num_args = 0..=1,
        default_missing_value = "markdown",
        value_name = "PARSE_METHOD",
        require_equals = true,
        help = "Display the result in the default browser"
    )]
    pub browser: Option<BrowserType>,
    #[arg(
        long,
        help = "Omit newline after the result",
        long_help = "Omit newline after the result. Useful when piping output to other applications"
    )]
    pub no_new_line: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum NotificationType {
    #[clap(help = "Show a simple completion notification (default)")]
    Status,
    #[clap(help = "Display the full result in the notification")]
    Content,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum BrowserType {
    #[clap(help = "Parse the result as Markdown before displaying (default)")]
    Markdown,
    #[clap(help = "Display the result without parsing")]
    Plain,
}
