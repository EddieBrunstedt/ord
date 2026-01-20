use crate::error::{Error, Result};
use crate::template::Template;
use std::collections::HashMap;

/// A prompt ready to be sent to the LLM.
#[derive(Debug)]
pub struct Prompt {
    pub body: String,
}

impl Prompt {
    /// Creates a prompt by substituting placeholder values into a template.
    pub fn from_template(template: &Template, entries: &[String]) -> Result<Self> {
        let entries_map = parse_entries(entries)?;
        validate_placeholders(template, &entries_map)?;
        let body = generate_body(template, &entries_map);
        Ok(Self { body })
    }
}

pub(crate) fn parse_entries(entries: &[String]) -> Result<HashMap<String, String>> {
    let mut map = HashMap::new();

    for entry in entries {
        let (placeholder, replacement) = parse_entry(entry)?;

        if map
            .insert(placeholder.to_string(), replacement.to_string())
            .is_some()
        {
            return Err(Error::DuplicatePlaceholderProvided {
                placeholder: placeholder.to_string(),
            });
        }
    }

    Ok(map)
}

pub(crate) fn parse_entry(entry: &str) -> Result<(&str, &str)> {
    let (placeholder, replacement) =
        entry
            .split_once('=')
            .ok_or_else(|| Error::InvalidEntryFormat {
                entry: entry.to_string(),
            })?;

    let placeholder = placeholder.trim();
    let replacement = replacement.trim();

    if placeholder.is_empty() || replacement.is_empty() {
        return Err(Error::InvalidEntryFormat {
            entry: entry.to_string(),
        });
    }

    Ok((placeholder, replacement))
}

pub(crate) fn validate_placeholders(
    template: &Template,
    entries_map: &HashMap<String, String>,
) -> Result<()> {
    let unfulfilled: Vec<_> = template
        .placeholders
        .iter()
        .filter(|p| !entries_map.contains_key(*p))
        .cloned()
        .collect();

    if !unfulfilled.is_empty() {
        return Err(Error::UnfulfilledPlaceholders {
            placeholders: unfulfilled,
        });
    }

    let excess: Vec<_> = entries_map
        .keys()
        .filter(|k| !template.placeholders.contains(k))
        .cloned()
        .collect();

    if !excess.is_empty() {
        return Err(Error::ExcessEntries {
            excess_entries: excess,
        });
    }

    Ok(())
}

pub(crate) fn generate_body(template: &Template, entries_map: &HashMap<String, String>) -> String {
    template
        .placeholders
        .iter()
        .fold(template.content.clone(), |body, placeholder| {
            body.replace(
                &format!("{{{{:{placeholder}}}}}"),
                &entries_map[placeholder],
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::TemplateMetadata;
    use assert_matches::assert_matches;

    fn create_test_template(placeholders: Vec<&str>, content: &str) -> Template {
        Template {
            title: "test".to_string(),
            content: content.to_string(),
            placeholders: placeholders.into_iter().map(String::from).collect(),
            metadata: TemplateMetadata {
                description: "Test template".to_string(),
            },
        }
    }

    #[test]
    fn parse_entry_parses_valid_entry() {
        let (key, value) = parse_entry("name=John").unwrap();
        assert_eq!(key, "name");
        assert_eq!(value, "John");
    }

    #[test]
    fn parse_entry_trims_whitespace() {
        let (key, value) = parse_entry("  name  =  John Doe  ").unwrap();
        assert_eq!(key, "name");
        assert_eq!(value, "John Doe");
    }

    #[test]
    fn parse_entry_handles_equals_in_value() {
        let (key, value) = parse_entry("equation=a=b+c").unwrap();
        assert_eq!(key, "equation");
        assert_eq!(value, "a=b+c");
    }

    #[test]
    fn parse_entry_fails_without_equals() {
        let result = parse_entry("invalid");
        assert_matches!(result, Err(Error::InvalidEntryFormat { .. }));
    }

    #[test]
    fn parse_entry_fails_with_empty_key() {
        let result = parse_entry("=value");
        assert_matches!(result, Err(Error::InvalidEntryFormat { .. }));
    }

    #[test]
    fn parse_entry_fails_with_empty_value() {
        let result = parse_entry("key=");
        assert_matches!(result, Err(Error::InvalidEntryFormat { .. }));
    }

    #[test]
    fn parse_entry_fails_with_whitespace_only_key() {
        let result = parse_entry("   =value");
        assert_matches!(result, Err(Error::InvalidEntryFormat { .. }));
    }

    #[test]
    fn parse_entries_parses_multiple_entries() {
        let entries = vec!["name=John".to_string(), "place=Paris".to_string()];
        let map = parse_entries(&entries).unwrap();

        assert_eq!(map.get("name"), Some(&"John".to_string()));
        assert_eq!(map.get("place"), Some(&"Paris".to_string()));
    }

    #[test]
    fn parse_entries_fails_on_duplicate_key() {
        let entries = vec!["name=John".to_string(), "name=Jane".to_string()];
        let result = parse_entries(&entries);

        assert_matches!(result, Err(Error::DuplicatePlaceholderProvided { .. }));
    }

    #[test]
    fn parse_entries_handles_empty_list() {
        let entries: Vec<String> = vec![];
        let map = parse_entries(&entries).unwrap();

        assert!(map.is_empty());
    }

    #[test]
    fn validate_placeholders_succeeds_with_exact_match() {
        let template = create_test_template(vec!["name", "place"], "{{:name}} {{:place}}");
        let mut entries = HashMap::new();
        entries.insert("name".to_string(), "John".to_string());
        entries.insert("place".to_string(), "Paris".to_string());

        assert!(validate_placeholders(&template, &entries).is_ok());
    }

    #[test]
    fn validate_placeholders_fails_with_missing_placeholder() {
        let template = create_test_template(vec!["name", "place"], "{{:name}} {{:place}}");
        let mut entries = HashMap::new();
        entries.insert("name".to_string(), "John".to_string());

        let result = validate_placeholders(&template, &entries);
        assert_matches!(result, Err(Error::UnfulfilledPlaceholders { .. }));
    }

    #[test]
    fn validate_placeholders_fails_with_excess_entry() {
        let template = create_test_template(vec!["name"], "{{:name}}");
        let mut entries = HashMap::new();
        entries.insert("name".to_string(), "John".to_string());
        entries.insert("extra".to_string(), "value".to_string());

        let result = validate_placeholders(&template, &entries);
        assert_matches!(result, Err(Error::ExcessEntries { .. }));
    }

    #[test]
    fn generate_body_replaces_single_placeholder() {
        let template = create_test_template(vec!["name"], "Hello {{:name}}!");
        let mut entries = HashMap::new();
        entries.insert("name".to_string(), "World".to_string());

        let body = generate_body(&template, &entries);
        assert_eq!(body, "Hello World!");
    }

    #[test]
    fn generate_body_replaces_multiple_placeholders() {
        let template = create_test_template(vec!["greeting", "name"], "{{:greeting}}, {{:name}}!");
        let mut entries = HashMap::new();
        entries.insert("greeting".to_string(), "Hello".to_string());
        entries.insert("name".to_string(), "World".to_string());

        let body = generate_body(&template, &entries);
        assert_eq!(body, "Hello, World!");
    }

    #[test]
    fn generate_body_replaces_repeated_placeholder() {
        let template = create_test_template(vec!["name"], "{{:name}} meets {{:name}}");
        let mut entries = HashMap::new();
        entries.insert("name".to_string(), "Alice".to_string());

        let body = generate_body(&template, &entries);
        assert_eq!(body, "Alice meets Alice");
    }

    #[test]
    fn prompt_from_template_creates_valid_prompt() {
        let template =
            create_test_template(vec!["name", "place"], "Hello {{:name}} from {{:place}}!");
        let entries = vec!["name=Alice".to_string(), "place=Wonderland".to_string()];

        let prompt = Prompt::from_template(&template, &entries).unwrap();
        assert_eq!(prompt.body, "Hello Alice from Wonderland!");
    }

    #[test]
    fn prompt_from_template_fails_with_missing_entry() {
        let template = create_test_template(vec!["name", "place"], "{{:name}} {{:place}}");
        let entries = vec!["name=Alice".to_string()];

        let result = Prompt::from_template(&template, &entries);
        assert_matches!(result, Err(Error::UnfulfilledPlaceholders { .. }));
    }

    #[test]
    fn prompt_from_template_fails_with_invalid_entry_format() {
        let template = create_test_template(vec!["name"], "{{:name}}");
        let entries = vec!["invalid_entry".to_string()];

        let result = Prompt::from_template(&template, &entries);
        assert_matches!(result, Err(Error::InvalidEntryFormat { .. }));
    }

    #[test]
    fn prompt_from_template_fails_with_excess_entry() {
        let template = create_test_template(vec!["name"], "{{:name}}");
        let entries = vec!["name=Alice".to_string(), "extra=value".to_string()];

        let result = Prompt::from_template(&template, &entries);
        assert_matches!(result, Err(Error::ExcessEntries { .. }));
    }
}
