mod generate_markup;

use crate::{
    cli::BrowserType,
    error::{Error, Result},
};
use generate_markup::generate_markup;
use maud::{html, PreEscaped};
use pulldown_cmark::{html::push_html, Options, Parser};
use std::{env, fs::File, io::Write};
use uuid::Uuid;

pub trait ResultDisplay: Send + Sync {
    fn display_to_stdout(&self, result: &str, no_new_line: &bool) -> Result<()>;
    fn display_in_browser(
        &self,
        result: &str,
        template_title: &str,
        browser_type: &BrowserType,
    ) -> Result<()>;
}

pub struct DefaultResultDisplay;

impl ResultDisplay for DefaultResultDisplay {
    fn display_to_stdout(&self, result: &str, no_new_line: &bool) -> Result<()> {
        if *no_new_line {
            print!("{result}");
        } else {
            println!("{result}");
        }
        Ok(())
    }

    fn display_in_browser(
        &self,
        result: &str,
        template_title: &str,
        browser_type: &BrowserType,
    ) -> Result<()> {
        let content = match browser_type {
            BrowserType::Markdown => {
                let mut options = Options::empty();
                options.insert(Options::ENABLE_STRIKETHROUGH);
                options.insert(Options::ENABLE_TABLES);
                let parser = Parser::new_ext(result, options);
                let mut html_output = String::new();
                push_html(&mut html_output, parser);
                html_output
            }
            BrowserType::Plain => result.replace('\n', "<br>"),
        };

        let generate_markup = generate_markup(&content, template_title);
        let markup = generate_markup;

        let os_temp_dir = env::temp_dir();
        let file_name = format!("ord-result-{}.html", Uuid::new_v4());
        let file_path = os_temp_dir.join(file_name);

        let mut file = File::create(&file_path)?;
        file.write_all(markup.into_string().as_bytes())?;

        webbrowser::open(file_path.to_str().ok_or(Error::FailParsePathUnicode {
            path: file_path.clone(),
        })?)?;

        Ok(())
    }
}

pub fn display_result(
    display: &dyn ResultDisplay,
    result: &str,
    template_title: &str,
    output_to_browser: &Option<BrowserType>,
    no_new_line: &bool,
) -> Result<()> {
    display.display_to_stdout(result, no_new_line)?;

    if let Some(browser_type) = output_to_browser {
        display.display_in_browser(result, template_title, browser_type)?;
    }

    Ok(())
}
