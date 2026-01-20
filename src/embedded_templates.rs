use crate::error::{Error, Result};
use std::{fs, path::Path};

struct EmbeddedTemplate {
    filename: &'static str,
    content: &'static str,
}

const EMBEDDED_TEMPLATES: &[EmbeddedTemplate] = &[
    EmbeddedTemplate {
        filename: "commit_message.md",
        content: include_str!("../example_templates/commit_message.md"),
    },
    EmbeddedTemplate {
        filename: "definition.md",
        content: include_str!("../example_templates/definition.md"),
    },
    EmbeddedTemplate {
        filename: "eli5.md",
        content: include_str!("../example_templates/eli5.md"),
    },
    EmbeddedTemplate {
        filename: "grammar.md",
        content: include_str!("../example_templates/grammar.md"),
    },
    EmbeddedTemplate {
        filename: "shorten.md",
        content: include_str!("../example_templates/shorten.md"),
    },
    EmbeddedTemplate {
        filename: "summarize.md",
        content: include_str!("../example_templates/summarize.md"),
    },
    EmbeddedTemplate {
        filename: "summarize_logs.md",
        content: include_str!("../example_templates/summarize_logs.md"),
    },
];

/// Writes all embedded example templates to the specified directory.
pub fn install_example_templates(templates_dir: &Path) -> Result<()> {
    for template in EMBEDDED_TEMPLATES {
        let path = templates_dir.join(template.filename);
        fs::write(&path, template.content).map_err(|e| Error::FailWriteFile {
            path,
            origin_error: e,
        })?;
    }
    Ok(())
}

/// Returns the number of embedded example templates available.
pub fn example_template_count() -> usize {
    EMBEDDED_TEMPLATES.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn install_example_templates_creates_all_files() {
        let dir = TempDir::new().unwrap();

        install_example_templates(dir.path()).unwrap();

        let files: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();

        assert_eq!(files.len(), EMBEDDED_TEMPLATES.len());
    }

    #[test]
    fn install_example_templates_writes_correct_content() {
        let dir = TempDir::new().unwrap();

        install_example_templates(dir.path()).unwrap();

        let grammar_content = fs::read_to_string(dir.path().join("grammar.md")).unwrap();
        assert!(grammar_content.contains("Correct grammar and spelling"));
    }

    #[test]
    fn example_template_count_returns_correct_value() {
        assert_eq!(example_template_count(), 7);
    }
}
