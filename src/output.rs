use crate::cli::BrowserType;
use crate::error::{Error, Result};
use maud::{PreEscaped, html};
use pulldown_cmark::{Options, Parser, html::push_html};
use regex::Regex;
use std::{env, fs::File, io::Write, sync::LazyLock};
use uuid::Uuid;

/// Prints the result to stdout, optionally omitting the trailing newline.
pub fn print_result(result: &str, no_newline: bool) {
    if no_newline {
        print!("{result}");
    } else {
        println!("{result}");
    }
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// Opens the result in a browser as a styled HTML page.
pub fn open_in_browser(
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

    let markup = generate_markup(&content, template_title);

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

static H1_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<h1.*?>(.*?)</h1>").expect("valid regex"));

pub(crate) fn generate_markup(content: &str, template_title: &str) -> PreEscaped<String> {
    let page_title = H1_REGEX
        .captures(content)
        .and_then(|cap| cap.get(1))
        .map(|m| format!("{} - Ord", m.as_str().trim()))
        .unwrap_or_else(|| format!("{} - Ord", capitalize(template_title)));

    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (page_title) }
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=Fira+Code&display=swap";
                link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/github-dark.min.css";
                style { (STYLES) }
            }
            body {
                div class="container" {
                    div class="content" {
                        (maud::PreEscaped(content))
                    }
                }
                script src="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/highlight.min.js" {}
                script { "hljs.configure({languages:[]});hljs.highlightAll();" }
            }
        }
    }
}

const STYLES: &str = r#"
:root {
    --bg: #08080a;
    --text: #a0a0a0;
    --text-hi: #e8e8e8;
    --accent: #d4956a;
    --surface: #111113;
    --border: #222225;
}
* { box-sizing: border-box; }
::selection { background: var(--accent); color: var(--bg); }
html { font-size: 17px; -webkit-font-smoothing: antialiased; }
body {
    background: var(--bg);
    color: var(--text);
    font-family: 'Inter', system-ui, sans-serif;
    line-height: 1.75;
    margin: 0;
    min-height: 100vh;
    display: flex;
    flex-direction: column;
}
.container { max-width: 840px; margin: 0 auto; padding: 4rem 3rem; flex-grow: 1; }
.content { overflow-wrap: break-word; }
.content > *:first-child { margin-top: 0; }
.content > *:last-child { margin-bottom: 0; }
h1, h2, h3, h4, h5, h6 { color: var(--text-hi); font-weight: 600; line-height: 1.35; margin: 2.5rem 0 1rem; }
h1 { font-size: 2rem; font-weight: 700; margin: 0 0 2.5rem; padding-bottom: 1.25rem; border-bottom: 1px solid var(--border); }
h2 { font-size: 1.5rem; margin-top: 2.5rem; padding-bottom: 0.5rem; border-bottom: 1px solid var(--border); }
h3 { font-size: 1.25rem; color: var(--text); }
h4 { font-size: 1.125rem; }
h5, h6 { font-size: 0.875rem; text-transform: uppercase; letter-spacing: 0.08em; color: #5a5a5a; }
p { margin: 0 0 1rem; }
a { color: var(--accent); text-decoration: none; }
a:hover { text-decoration: underline; }
strong, b { font-weight: 600; color: var(--text-hi); }
em, i { color: var(--text-hi); }
ul, ol { margin: 0 0 1.5rem; padding-left: 1.5rem; }
li { margin-bottom: 0.5rem; }
li::marker { color: var(--accent); }
pre {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 1.25rem 1.5rem;
    margin: 0 -1.5rem 1.5rem;
    overflow-x: auto;
    font-size: 0.9rem;
    line-height: 1.6;
}
code {
    font-family: 'Fira Code', 'SF Mono', monospace;
    font-size: 0.875em;
    background: #18181b;
    padding: 0.2em 0.4em;
    border-radius: 4px;
    border: 1px solid var(--border);
}
pre code, pre code.hljs { background: transparent; padding: 0; border: none; border-radius: 0; font-size: inherit; }
blockquote { margin: 1.5rem 0; padding-left: 1.5rem; border-left: 3px solid var(--accent); color: #5a5a5a; font-style: italic; }
blockquote p:last-child { margin-bottom: 0; }
table { width: 100%; border-collapse: collapse; margin: 1.5rem 0; font-size: 0.95rem; }
th, td { padding: 0.75rem 1rem; text-align: left; border-bottom: 1px solid var(--border); }
th { font-weight: 600; color: var(--text-hi); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; }
tr:last-child td { border-bottom: none; }
hr { border: none; height: 1px; background: var(--border); margin: 3rem 0; }
img { max-width: 100%; height: auto; border-radius: 8px; }
@media (max-width: 640px) {
    html { font-size: 16px; }
    .container { padding: 2rem 1.25rem; }
    h1 { font-size: 1.625rem; }
    h2 { font-size: 1.25rem; margin-top: 2rem; }
    pre { margin: 0 -1.25rem 1.5rem; padding: 1rem 1.25rem; border-radius: 0; border-left: none; border-right: none; }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_markup_includes_content() {
        let markup = generate_markup("<p>Test content</p>", "test");
        let html = markup.into_string();

        assert!(html.contains("<p>Test content</p>"));
    }

    #[test]
    fn generate_markup_uses_h1_for_title() {
        let markup = generate_markup("<h1>My Title</h1><p>Content</p>", "template");
        let html = markup.into_string();

        assert!(html.contains("<title>My Title - Ord</title>"));
    }

    #[test]
    fn generate_markup_capitalizes_template_title_when_no_h1() {
        let markup = generate_markup("<p>Content without h1</p>", "test_template");
        let html = markup.into_string();

        assert!(html.contains("<title>Test_template - Ord</title>"));
    }

    #[test]
    fn generate_markup_includes_styles() {
        let markup = generate_markup("<p>Content</p>", "test");
        let html = markup.into_string();

        assert!(html.contains("--bg: #08080a"));
        assert!(html.contains("font-family: 'Inter', system-ui"));
    }

    #[test]
    fn generate_markup_is_valid_html() {
        let markup = generate_markup("<p>Content</p>", "test");
        let html = markup.into_string();

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<html lang=\"en\">"));
        assert!(html.contains("<head>"));
        assert!(html.contains("<body>"));
        assert!(html.contains("</html>"));
    }

    #[test]
    fn generate_markup_includes_meta_tags() {
        let markup = generate_markup("<p>Content</p>", "test");
        let html = markup.into_string();

        assert!(html.contains("charset=\"UTF-8\""));
        assert!(html.contains("viewport"));
    }

    #[test]
    fn generate_markup_handles_empty_content() {
        let markup = generate_markup("", "test");
        let html = markup.into_string();

        assert!(html.contains("<div class=\"content\">"));
        assert!(html.contains("</div>"));
    }

    #[test]
    fn generate_markup_handles_special_characters_in_title() {
        let markup = generate_markup("<h1>Test &amp; Title</h1>", "test");
        let html = markup.into_string();

        assert!(html.contains("<h1>Test &amp; Title</h1>"));
    }
}
