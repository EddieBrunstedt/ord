use crate::error::{Error, Result};
use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    ffi::OsStr,
    fs,
    path::Path,
};

pub trait TemplateRepository: Send + Sync {
    fn find(&self, criteria: &TemplateCriteria) -> Result<&Template>;
    fn list(&self);
    fn get_template_titles(&self) -> Vec<String>;
}

pub trait TemplateDisplay: Send + Sync {
    fn display(&self) -> Result<()>;
}

#[derive(Debug)]
pub struct TemplateCollection {
    templates: Vec<Template>,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub content: String,
    pub placeholders: Vec<String>,
    pub title: String,
    pub metadata: TemplateMetadata,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct TemplateMetadata {
    pub description: String,
}

pub enum TemplateCriteria<'a> {
    Title(&'a str),
    Index(usize),
}

impl TemplateRepository for TemplateCollection {
    fn find(&self, criteria: &TemplateCriteria) -> Result<&Template> {
        match criteria {
            TemplateCriteria::Title(title) => self.templates.iter().find(|t| t.title == *title),
            TemplateCriteria::Index(index) => self.templates.get(*index),
        }
        .ok_or_else(|| match criteria {
            TemplateCriteria::Title(title) => Error::TemplateNotFound {
                template_title: title.to_string(),
            },
            TemplateCriteria::Index(index) => Error::TemplateIndexNotFound { index: index + 1 },
        })
    }

    fn list(&self) {
        for (index, template) in self.templates.iter().enumerate() {
            println!(
                "{}: {} ({})",
                index + 1,
                template.title,
                template.metadata.description
            );
        }
    }

    fn get_template_titles(&self) -> Vec<String> {
        self.templates.iter().map(|t| t.title.clone()).collect()
    }
}

impl TemplateCollection {
    pub fn new(templates: Vec<Template>) -> Self {
        Self { templates }
    }

    pub fn from_directory(templates_dir: &Path) -> Result<Self> {
        let templates = load_templates_from_directory(templates_dir)?;
        Ok(Self::new(templates))
    }
}

impl TemplateDisplay for Template {
    fn display(&self) -> Result<()> {
        println!();
        println!(
            "Title: {}\n\
             Description: {}\n\
             Placeholders: {:?}\n\
             ----------------------------------\n\
             {}\n\
             ----------------------------------",
            self.title, self.metadata.description, self.placeholders, self.content
        );
        Ok(())
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

    fn parse(title: &str, raw_content: &str, path: &Path) -> Result<Template> {
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

    let duplicate_titles = find_duplicate_template_titles(&templates);
    if !duplicate_titles.is_empty() {
        return Err(Error::DuplicateTemplateTitles { duplicate_titles });
    }

    let mut sorted_templates = templates;
    sorted_templates.sort_by_key(|t| t.title.to_lowercase());
    Ok(sorted_templates)
}

fn extract_placeholders(text: &str) -> Vec<String> {
    lazy_static! {
        static ref PLACEHOLDER_REGEX: Regex =
            Regex::new(r"\{\{:(\w+)\}\}").expect("Must be a valid regex pattern");
    }

    let mut placeholders_set = HashSet::new();

    PLACEHOLDER_REGEX
        .captures_iter(text)
        .filter_map(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
        .filter(|placeholder| placeholders_set.insert(placeholder.clone()))
        .collect()
}

fn find_duplicate_template_titles(templates: &[Template]) -> Vec<String> {
    let mut counts = HashMap::new();
    let mut duplicates = Vec::new();

    for template in templates {
        *counts.entry(&template.title).or_insert(0) += 1;
    }

    for (title, count) in &counts {
        if *count > 1 {
            duplicates.push((*title).to_string());
        }
    }

    duplicates
}

fn sanitize_filename(path: &Path) -> Result<String> {
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

fn parse_metadata(yaml_str: &str) -> Result<TemplateMetadata> {
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
    use serial_test::serial;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn template_parse_ok() {
        let path = Path::new("test.md");
        let content = r#"---
description: Test template
---
This is a {{:placeholder}} template."#;
        let template = Template::parse("test", content, path).unwrap();

        assert_eq!(template.title, "test");
        assert_eq!(template.metadata.description, "Test template");
        assert_eq!(template.placeholders, vec!["placeholder"]);
        assert_eq!(template.content, "This is a {{:placeholder}} template.");
    }

    #[test]
    fn template_parse_no_placeholders_error() {
        let fx_path = "/this-is-a-path/test.md";
        let content = r#"---
description: Test template
---
This template contains no placeholders"#;
        let parse_result = Template::parse("test", content, Path::new(fx_path));

        assert_matches!(
            parse_result,
            Err(Error::TemplateContainsNoPlaceholders { path }) if path == PathBuf::from(fx_path),
            "Expected TemplateContainsNoPlaceholders error with correct path"
        );
    }

    #[test]
    fn template_parse_invalid_format_error() {
        let fx_path = "/this-is-a-path/test.md";
        let content = r#"This is a {{:template}}, but it is missing YAML frontmatter"#;
        let parse_result = Template::parse("test", content, Path::new(&fx_path));

        assert_matches!(
            parse_result,
            Err(Error::InvalidTemplateFormat { path }) if path == PathBuf::from(fx_path),
            "Expected InvalidTemplateFormat error with correct path"
        );
    }

    #[test]
    fn template_parse_empty_yaml_error() {
        let content = r#"---
---
This is a placeholder {{:template}} without description"#;
        let fx_path = Path::new("/this-is-a-path/test.md");
        let parse_result = Template::parse("test", content, fx_path);

        assert_matches!(
            parse_result,
            Err(Error::FailParseYaml { .. }),
            "Expected FailToParseYaml error"
        );
    }

    #[test]
    #[serial]
    fn template_from_file_ok() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test_template.md");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "description: Test template").unwrap();
        writeln!(file, "---").unwrap();
        writeln!(file, "This is a {{{{:placeholder}}}} template.").unwrap();

        let template = Template::from_file(&file_path).unwrap();

        assert_eq!(template.title, "test_template");
        assert_eq!(template.metadata.description, "Test template");
        assert_eq!(template.placeholders, vec!["placeholder"]);
        assert_eq!(template.content, "This is a {{:placeholder}} template.");
    }

    #[test]
    fn template_find_collection_ok() {
        let templates = vec![
            Template {
                title: "test1".to_string(),
                content: "Test 1".to_string(),
                placeholders: vec!["placeholder1".to_string()],
                metadata: TemplateMetadata {
                    description: "Test 1 description".to_string(),
                },
            },
            Template {
                title: "test2".to_string(),
                content: "Test 2".to_string(),
                placeholders: vec!["placeholder2".to_string()],
                metadata: TemplateMetadata {
                    description: "Test 2 description".to_string(),
                },
            },
        ];

        let collection = TemplateCollection::new(templates);

        let found = collection.find(&TemplateCriteria::Title("test1")).unwrap();
        assert_eq!(found.title, "test1");

        let found = collection.find(&TemplateCriteria::Index(1)).unwrap();
        assert_eq!(found.title, "test2");

        assert!(collection
            .find(&TemplateCriteria::Title("nonexistent"))
            .is_err());
        assert!(collection.find(&TemplateCriteria::Index(2)).is_err());
    }

    #[test]
    fn extract_placeholders_ok() {
        let text = "This is a {{:placeholder1}} and {{:placeholder2}} test.";
        let placeholders = extract_placeholders(text);
        assert_eq!(placeholders, vec!["placeholder1", "placeholder2"]);
    }

    #[test]
    fn sanitize_filename_ok() {
        assert!(sanitize_filename(Path::new("")).is_err());
        assert_eq!(
            sanitize_filename(Path::new("Test File.md")).unwrap(),
            "test_file"
        );
        assert_eq!(
            sanitize_filename(Path::new("test-file_123.md")).unwrap(),
            "test-file_123"
        );
    }

    #[test]
    fn find_duplicate_template_titles_ok() {
        let templates = vec![
            Template {
                title: "test1".to_string(),
                content: "Test 1".to_string(),
                placeholders: vec!["placeholder1".to_string()],
                metadata: TemplateMetadata::default(),
            },
            Template {
                title: "test2".to_string(),
                content: "Test 2".to_string(),
                placeholders: vec!["placeholder2".to_string()],
                metadata: TemplateMetadata::default(),
            },
            Template {
                title: "test1".to_string(),
                content: "Test 1 duplicate".to_string(),
                placeholders: vec!["placeholder3".to_string()],
                metadata: TemplateMetadata::default(),
            },
        ];

        let duplicates = find_duplicate_template_titles(&templates);
        assert_eq!(duplicates, vec!["test1"]);
    }
}
