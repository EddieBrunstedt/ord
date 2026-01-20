//! Binary entry point for the Ord CLI.

use console::Style;

#[tokio::main]
async fn main() {
    if let Err(err) = ord::run().await {
        let error_style = Style::new().red().bold();
        eprintln!("{} {err}", error_style.apply_to("error:"));
        std::process::exit(1);
    }
}
