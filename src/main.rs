#![allow(unused_imports)]

mod api;
mod cli;
mod clipboard;
mod context;
mod display;
mod error;
mod handler;
mod interactive_input;
mod notify;
mod prompt;
mod templates;
mod user_config;

use self::error::Result;
use clap::{CommandFactory, Parser};
use cli::{Args, Command};
use context::Context;
use handler::{DefaultHandler, Handler};

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("ERROR: {err}");
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    let args = Args::parse();
    let context = Context::create()?;
    let handler = DefaultHandler::new(context);

    match &args.command {
        None => Args::command().print_help()?,
        Some(Command::Generate(generate_args)) => handler.generate(generate_args).await?,
        Some(Command::Info { title }) => handler.info(title)?,
        Some(Command::List) => handler.list()?,
    };

    Ok(())
}
