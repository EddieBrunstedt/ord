use maud::{html, PreEscaped};
use regex::Regex;

pub fn generate_markup(content: &str, template_title: &str) -> PreEscaped<String> {
    let homepage = env!("CARGO_PKG_HOMEPAGE");

    fn capitalize(s: &str) -> String {
        s.chars()
            .next()
            .map(|c| c.to_uppercase().collect::<String>() + &s[1..])
            .unwrap_or_else(|| s.to_string())
    }

    let h1_texts: Vec<String> = {
        let re = Regex::new(r"<h1.*?>(.*?)</h1>").unwrap();
        re.captures_iter(content)
            .filter_map(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
            .collect()
    };

    let page_title = match h1_texts.first() {
        Some(text) => format!("{} - Ord", text),
        None => format!("{} - Ord", capitalize(template_title)),
    };

    html! {
        (maud::DOCTYPE)
        html lang="en" {
            head {
                meta charset="UTF-8";
                meta name="viewport" content="width=device-width, initial-scale=1.0";
                title { (page_title) }
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=Fira+Code&display=swap";
                style { r#"
                    :root {
                        --bg-color: #121212;
                        --text-color: #e0e0e0;
                        --accent-color: #FFA726;
                        --secondary-color: #9E9E9E;
                        --container-bg: #1E1E1E;
                        --code-bg: #2C2C2C;
                        --border-color: #333333;
                        --link-color: #FFB74D;
                        --link-hover-color: #FFC107;
                        --template-title-color: #FFFFFF;
                    }
                    html {
                        font-size: 16px;
                    }
                    @media screen and (min-width: 320px) {
                        html {
                            font-size: calc(16px + 2 * ((100vw - 320px) / 680));
                        }
                    }
                    @media screen and (min-width: 1000px) {
                        html {
                            font-size: 18px;
                        }
                    }
                    body {
                        background-color: var(--bg-color);
                        color: var(--text-color);
                        font-family: 'Inter', sans-serif;
                        margin: 0;
                        padding: 0;
                        line-height: 1.6;
                        min-height: 100vh;
                        display: flex;
                        flex-direction: column;
                    }
                    .container {
                        max-width: 800px;
                        margin: 2rem auto;
                        padding: 2rem;
                        background-color: var(--container-bg);
                        border-radius: 8px;
                        box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
                        flex-grow: 1;
                    }
                    h1, h2, h3, h4, h5, h6 {
                        color: var(--accent-color);
                        margin-top: 1.5rem;
                        margin-bottom: 1rem;
                        line-height: 1.2;
                    }
                    h1 { 
                        font-size: 2.25rem; 
                        padding-bottom: 1rem;
                        border-bottom: 2px dashed var(--border-color);
                        margin-bottom: 1.5rem;
                    }
                    h2 { font-size: 1.75rem; }
                    h3 { font-size: 1.5rem; }
                    h4 { font-size: 1.25rem; }
                    h5 { font-size: 1.1rem; }
                    h6 { font-size: 1rem; }
                    .content {
                        word-wrap: break-word;
                    }
                    .content p {
                        margin-bottom: 1rem;
                    }
                    .content a {
                        color: var(--link-color);
                        text-decoration: none;
                        border-bottom: 1px solid var(--link-color);
                        transition: all 0.3s ease;
                    }
                    .content a:hover, .content a:focus {
                        color: var(--link-hover-color);
                        border-bottom-color: var(--link-hover-color);
                    }
                    .content a:focus {
                        outline: 2px solid var(--link-hover-color);
                        outline-offset: 2px;
                    }
                    .content pre {
                        background-color: var(--code-bg);
                        padding: 1rem;
                        border-radius: 4px;
                        overflow-x: auto;
                        border: 1px solid var(--border-color);
                        margin-bottom: 1rem;
                    }
                    .content code {
                        font-family: 'Fira Code', 'Consolas', 'Monaco', 'Andale Mono', 'Ubuntu Mono', monospace;
                        font-size: 0.9em;
                        background-color: var(--code-bg);
                        padding: 0.2em 0.4em;
                        border-radius: 3px;
                        color: #d4d4d4;
                    }
                    .content table {
                        border-collapse: separate;
                        border-spacing: 0;
                        width: 100%;
                        margin-bottom: 1rem;
                        border: 1px solid var(--border-color);
                        border-radius: 4px;
                        overflow: hidden;
                    }
                    .content th, .content td {
                        border: 1px solid var(--border-color);
                        padding: 0.6rem;
                        text-align: left;
                    }
                    .content th {
                        background-color: var(--code-bg);
                        font-weight: 600;
                    }
                    .content tr:nth-child(even) {
                        background-color: rgba(255, 255, 255, 0.03);
                    }
                    .footer {
                        text-align: center;
                        font-size: 0.9rem;
                        color: var(--secondary-color);
                        padding: 1rem 0 2rem;
                    }
                    .footer a {
                        color: var(--link-color);
                        text-decoration: none;
                        transition: color 0.3s ease;
                    }
                    .footer a:hover, .footer a:focus {
                        color: var(--link-hover-color);
                    }
                    .footer a:focus {
                        outline: 2px solid var(--link-hover-color);
                        outline-offset: 2px;
                    }
                    .template-title {
                        font-weight: 600;
                        color: var(--template-title-color);
                    }
                    @media (max-width: 768px) {
                        .container {
                            margin: 1rem;
                            padding: 1rem;
                        }
                    }
                "# }
            }
            body {
                div class="container" {
                    div class="content" {
                        (maud::PreEscaped(content))
                    }
                }
                div class="footer" {
                    span class="template-title" { (capitalize(template_title)) }
                    " by "
                    a href=(homepage) { "Ord" }
                }
            }
        }
    }
}
