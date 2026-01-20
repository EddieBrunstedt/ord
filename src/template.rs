use crate::error::{Error, Result};
use console::Style;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    path::Path,
    sync::LazyLock,
};

/// A parsed template with its content, placeholders, and metadata.
#[derive(Debug, Clone)]
pub struct Template {
    pub content: String,
    pub placeholders: Vec<String>,
    pub title: String,
    pub metadata: TemplateMetadata,
}

/// YAML front matter metadata from a template file.
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TemplateMetadata {
    pub description: String,
}

/// Collection of templates loaded from a directory.
#[derive(Debug)]
pub struct TemplateCollection {
    templates: Vec<Template>,
}

/// Criteria for finding a template in a collection.
pub enum FindBy<'a> {
    Title(&'a str),
    Index(usize),
}

impl TemplateCollection {
    /// Loads all templates from the specified directory.
    pub fn from_directory(templates_dir: &Path) -> Result<Self> {
        let templates = load_templates_from_directory(templates_dir)?;
        Ok(Self { templates })
    }

    /// Finds a template by title or index.
    pub fn find(&self, criteria: FindBy) -> Result<&Template> {
        match criteria {
            FindBy::Title(title) => self.templates.iter().find(|t| t.title == title),
            FindBy::Index(index) => self.templates.get(index),
        }
        .ok_or_else(|| match criteria {
            FindBy::Title(title) => Error::TemplateNotFound {
                template_title: title.to_string(),
            },
            FindBy::Index(index) => Error::TemplateIndexNotFound { index: index + 1 },
        })
    }

    /// Returns a list of all template titles.
    pub fn titles(&self) -> Vec<String> {
        self.templates.iter().map(|t| t.title.clone()).collect()
    }

    /// Prints all templates with their descriptions.
    pub fn list(&self) {
        let title_style = Style::new().cyan().bold();
        let desc_style = Style::new().dim();

        let max_title_len = self
            .templates
            .iter()
            .map(|t| t.title.len())
            .max()
            .unwrap_or(0);

        println!();
        for template in &self.templates {
            let padded_title = format!("{:width$}", template.title, width = max_title_len);
            println!(
                "  {}  {}",
                title_style.apply_to(&padded_title),
                desc_style.apply_to(&template.metadata.description)
            );
        }
        println!();
    }
}

impl Template {
    fn from_file(path: &Path) -> Result<Template> {
        let filename = sanitize_filename(path)?;
        let content = fs::read_to_string(path).map_err(|e| Error::FailReadFileToString {
            path: path.to_path_buf(),
            origin_error: e,
        })?;

        Template::parse(&filename, &content, path)
    }

    pub(crate) fn parse(title: &str, raw_content: &str, path: &Path) -> Result<Template> {
        let parts: Vec<&str> = raw_content.splitn(3, "---").collect();

        if parts.len() < 3 {
            return Err(Error::InvalidTemplateFormat {
                path: path.to_path_buf(),
            });
        }

        let content = parts[2].trim().to_string();
        let metadata: TemplateMetadata = parse_metadata(parts[1])?;
        let placeholders = extract_placeholders(&content);

        if placeholders.is_empty() {
            return Err(Error::TemplateContainsNoPlaceholders {
                path: path.to_path_buf(),
            });
        }

        Ok(Template {
            content,
            metadata,
            placeholders,
            title: title.to_string(),
        })
    }

    /// Prints the template details to stdout.
    pub fn display(&self) {
        let label_style = Style::new().dim();
        let value_style = Style::new().white();
        let title_style = Style::new().cyan().bold();
        let placeholder_style = Style::new().green();
        let border_style = Style::new().dim();

        let placeholders_formatted = self
            .placeholders
            .iter()
            .map(|p| format!("{{{{{}}}}}", p))
            .collect::<Vec<_>>()
            .join("  ");

        let border = "─".repeat(60);

        println!();
        println!(
            "  {}  {}",
            label_style.apply_to("Template:"),
            title_style.apply_to(&self.title)
        );
        println!(
            "  {}  {}",
            label_style.apply_to("Description:"),
            value_style.apply_to(&self.metadata.description)
        );
        println!(
            "  {}  {}",
            label_style.apply_to("Placeholders:"),
            placeholder_style.apply_to(&placeholders_formatted)
        );
        println!();
        println!("  {}", border_style.apply_to(&border));
        for line in self.content.lines() {
            println!("  {}", line);
        }
        println!("  {}", border_style.apply_to(&border));
        println!();
    }
}

fn load_templates_from_directory(templates_dir: &Path) -> Result<Vec<Template>> {
    let templates: Vec<Template> = fs::read_dir(templates_dir)
        .map_err(|e| Error::FailReadDirectory {
            path: templates_dir.to_path_buf(),
            origin_error: e,
        })?
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().and_then(OsStr::to_str) == Some("md"))
        .map(|entry| Template::from_file(&entry.path()))
        .collect::<Result<Vec<_>>>()?;

    if templates.is_empty() {
        return Err(Error::NoTemplatesAvailable {
            templates_dir: templates_dir.to_path_buf(),
        });
    }

    let duplicate_titles = find_duplicate_titles(&templates);
    if !duplicate_titles.is_empty() {
        return Err(Error::DuplicateTemplateTitles { duplicate_titles });
    }

    let mut sorted_templates = templates;
    sorted_templates.sort_by_key(|t| t.title.to_lowercase());
    Ok(sorted_templates)
}

static PLACEHOLDER_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{\{:(\w+)\}\}").expect("valid regex"));

pub(crate) fn extract_placeholders(text: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    PLACEHOLDER_REGEX
        .captures_iter(text)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|placeholder| seen.insert(placeholder.clone()))
        .collect()
}

fn find_duplicate_titles(templates: &[Template]) -> Vec<String> {
    let mut counts = HashMap::new();
    for template in templates {
        *counts.entry(&template.title).or_insert(0) += 1;
    }

    counts
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(title, _)| title.clone())
        .collect()
}

pub(crate) fn sanitize_filename(path: &Path) -> Result<String> {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| Error::InvalidPath {
            path: path.to_path_buf(),
        })
        .map(|s| {
            s.to_lowercase()
                .chars()
                .filter_map(|c| match c {
                    'a'..='z' | '0'..='9' | '-' | '_' => Some(c),
                    ' ' => Some('_'),
                    _ => None,
                })
                .collect()
        })
}

pub(crate) fn parse_metadata(yaml_str: &str) -> Result<TemplateMetadata> {
    serde_yaml::from_str(yaml_str).map_err(|e| {
        let expected_structure = serde_yaml::to_string(&TemplateMetadata::default())
            .unwrap_or_else(|_| "Failed to generate example".to_string());
        Error::FailParseYaml {
            origin_error: e,
            expected_structure,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_matches::assert_matches;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_template_file(dir: &TempDir, name: &str, content: &str) {
        let path = dir.path().join(name);
        let mut file = fs::File::create(path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
    }

    const VALID_TEMPLATE: &str = r#"---
description: Test template
---

Hello {{:name}}, welcome to {{:place}}!"#;

    const VALID_TEMPLATE_SINGLE_PLACEHOLDER: &str = r#"---
description: Single placeholder
---

Hello {{:name}}!"#;

    #[test]
    fn extract_placeholders_finds_single_placeholder() {
        let placeholders = extract_placeholders("Hello {{:name}}!");
        assert_eq!(placeholders, vec!["name"]);
    }

    #[test]
    fn extract_placeholders_finds_multiple_placeholders() {
        let placeholders = extract_placeholders("{{:greeting}} {{:name}}, welcome to {{:place}}!");
        assert_eq!(placeholders, vec!["greeting", "name", "place"]);
    }

    #[test]
    fn extract_placeholders_deduplicates() {
        let placeholders = extract_placeholders("{{:name}} and {{:name}} again");
        assert_eq!(placeholders, vec!["name"]);
    }

    #[test]
    fn extract_placeholders_returns_empty_for_no_placeholders() {
        let placeholders = extract_placeholders("Hello world!");
        assert!(placeholders.is_empty());
    }

    #[test]
    fn extract_placeholders_ignores_invalid_formats() {
        let placeholders = extract_placeholders("{{name}} {{ :name }} {name}");
        assert!(placeholders.is_empty());
    }

    #[test]
    fn sanitize_filename_converts_to_lowercase() {
        let result = sanitize_filename(Path::new("/path/to/MyTemplate.md")).unwrap();
        assert_eq!(result, "mytemplate");
    }

    #[test]
    fn sanitize_filename_replaces_spaces_with_underscores() {
        let result = sanitize_filename(Path::new("/path/to/my template.md")).unwrap();
        assert_eq!(result, "my_template");
    }

    #[test]
    fn sanitize_filename_removes_special_characters() {
        let result = sanitize_filename(Path::new("/path/to/my@template!.md")).unwrap();
        assert_eq!(result, "mytemplate");
    }

    #[test]
    fn sanitize_filename_keeps_valid_characters() {
        let result = sanitize_filename(Path::new("/path/to/my-template_123.md")).unwrap();
        assert_eq!(result, "my-template_123");
    }

    #[test]
    fn parse_metadata_parses_valid_yaml() {
        let metadata = parse_metadata("description: Test description").unwrap();
        assert_eq!(metadata.description, "Test description");
    }

    #[test]
    fn parse_metadata_handles_empty_description() {
        let metadata = parse_metadata("description: \"\"").unwrap();
        assert_eq!(metadata.description, "");
    }

    #[test]
    fn parse_metadata_fails_on_invalid_yaml() {
        let result = parse_metadata("invalid: [yaml: structure");
        assert_matches!(result, Err(Error::FailParseYaml { .. }));
    }

    #[test]
    fn template_parse_creates_valid_template() {
        let template = Template::parse("test", VALID_TEMPLATE, Path::new("test.md")).unwrap();

        assert_eq!(template.title, "test");
        assert_eq!(template.metadata.description, "Test template");
        assert_eq!(template.placeholders, vec!["name", "place"]);
        assert!(template.content.contains("Hello {{:name}}"));
    }

    #[test]
    fn template_parse_fails_without_front_matter() {
        let content = "Hello {{:name}}!";
        let result = Template::parse("test", content, Path::new("test.md"));
        assert_matches!(result, Err(Error::InvalidTemplateFormat { .. }));
    }

    #[test]
    fn template_parse_fails_with_incomplete_front_matter() {
        let content = "---\ndescription: Test\nHello {{:name}}!";
        let result = Template::parse("test", content, Path::new("test.md"));
        assert_matches!(result, Err(Error::InvalidTemplateFormat { .. }));
    }

    #[test]
    fn template_parse_fails_with_no_placeholders() {
        let content = "---\ndescription: Test\n---\nHello world!";
        let result = Template::parse("test", content, Path::new("test.md"));
        assert_matches!(result, Err(Error::TemplateContainsNoPlaceholders { .. }));
    }

    #[test]
    fn collection_from_directory_loads_templates() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "greeting.md", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();

        assert_eq!(collection.titles(), vec!["greeting"]);
    }

    #[test]
    fn collection_from_directory_sorts_alphabetically() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "zebra.md", VALID_TEMPLATE_SINGLE_PLACEHOLDER);
        create_template_file(&dir, "apple.md", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();

        assert_eq!(collection.titles(), vec!["apple", "zebra"]);
    }

    #[test]
    fn collection_from_directory_fails_on_empty_directory() {
        let dir = TempDir::new().unwrap();

        let result = TemplateCollection::from_directory(dir.path());
        assert_matches!(result, Err(Error::NoTemplatesAvailable { .. }));
    }

    #[test]
    fn collection_from_directory_ignores_non_md_files() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "valid.md", VALID_TEMPLATE);
        create_template_file(&dir, "ignored.txt", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();

        assert_eq!(collection.titles().len(), 1);
        assert_eq!(collection.titles()[0], "valid");
    }

    #[test]
    fn collection_find_by_title_succeeds() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "greeting.md", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();
        let template = collection.find(FindBy::Title("greeting")).unwrap();

        assert_eq!(template.title, "greeting");
    }

    #[test]
    fn collection_find_by_title_fails_for_unknown() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "greeting.md", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();
        let result = collection.find(FindBy::Title("unknown"));

        assert_matches!(result, Err(Error::TemplateNotFound { .. }));
    }

    #[test]
    fn collection_find_by_index_succeeds() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "greeting.md", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();
        let template = collection.find(FindBy::Index(0)).unwrap();

        assert_eq!(template.title, "greeting");
    }

    #[test]
    fn collection_find_by_index_fails_for_out_of_bounds() {
        let dir = TempDir::new().unwrap();
        create_template_file(&dir, "greeting.md", VALID_TEMPLATE);

        let collection = TemplateCollection::from_directory(dir.path()).unwrap();
        let result = collection.find(FindBy::Index(99));

        assert_matches!(result, Err(Error::TemplateIndexNotFound { .. }));
    }
}
