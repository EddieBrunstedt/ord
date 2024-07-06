use crate::{
    error::{Error, Result},
    templates::Template,
};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Prompt {
    pub body: String,
}

impl Prompt {
    pub fn from(template: &Template, entries: &[String]) -> Result<Prompt> {
        let entries_map = Self::parse_entries(entries)?;
        Self::validate_placeholders(template, &entries_map)?;
        let prompt_body = Self::generate_prompt_body(template, &entries_map);

        Ok(Prompt { body: prompt_body })
    }

    fn parse_entries(entries: &[String]) -> Result<HashMap<String, String>> {
        let mut entries_map = HashMap::new();

        for entry in entries {
            let (placeholder, replacement) = Self::parse_entry(entry)?;

            if entries_map
                .insert(placeholder.to_string(), replacement.to_string())
                .is_some()
            {
                return Err(Error::DuplicatePlaceholderProvided {
                    placeholder: placeholder.to_string(),
                });
            }
        }

        Ok(entries_map)
    }

    fn parse_entry(entry: &str) -> Result<(&str, &str)> {
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

    fn validate_placeholders(
        template: &Template,
        entries_map: &HashMap<String, String>,
    ) -> Result<()> {
        let unfulfilled_placeholders: Vec<_> = template
            .placeholders
            .iter()
            .filter(|p| !entries_map.contains_key(*p))
            .cloned()
            .collect();

        if !unfulfilled_placeholders.is_empty() {
            return Err(Error::UnfulfilledPlaceholders {
                placeholders: unfulfilled_placeholders,
            });
        }

        let excess_entries: Vec<_> = entries_map
            .keys()
            .filter(|k| !template.placeholders.contains(k))
            .cloned()
            .collect();

        if !excess_entries.is_empty() {
            return Err(Error::ExcessEntries { excess_entries });
        }

        Ok(())
    }

    fn generate_prompt_body(template: &Template, entries_map: &HashMap<String, String>) -> String {
        template
            .placeholders
            .iter()
            .fold(template.content.clone(), |body, placeholder| {
                body.replace(
                    &format!("{{{{:{}}}}}", placeholder),
                    &entries_map[placeholder],
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::templates::{Template, TemplateMetadata};
    use assert_matches::assert_matches;

    fn generate_valid_test_template() -> Template {
        Template {
            title: "Test title".to_string(),
            content: "This is a template with a {{:placeholder}} in it".to_string(),
            placeholders: vec!["placeholder".to_string()],
            metadata: TemplateMetadata {
                description: "This is a template for testing".to_string(),
            },
        }
    }

    #[test]
    fn create_prompt_ok() -> Result<()> {
        let template = generate_valid_test_template();
        let entries = &["placeholder=replacement".to_string()];
        let create_prompt_result = Prompt::from(&template, entries);

        assert!(
            create_prompt_result.is_ok(),
            "Expected 'Ok' but got {:?}",
            create_prompt_result
        );

        Ok(())
    }

    #[test]
    fn create_prompt_excess_placeholders_error() -> Result<()> {
        let template = generate_valid_test_template();
        let entries = &[
            "placeholder=replacement".to_string(),
            "elephant=charles".to_string(),
        ];
        let create_prompt_result = Prompt::from(&template, entries);

        assert_matches!(
            create_prompt_result,
            Err(Error::ExcessEntries { excess_entries }) if excess_entries.contains(&"elephant".to_string()),
            "Expected Error::ExcessEntries with correct entries"
        );

        Ok(())
    }

    #[test]
    fn create_prompt_unfulfilled_placeholders_error() -> Result<()> {
        let template = generate_valid_test_template();
        let entries = &["elephant=charles".to_string()];
        let create_prompt_result = Prompt::from(&template, entries);

        assert_matches!(
            create_prompt_result,
            Err(Error::UnfulfilledPlaceholders { placeholders }) if placeholders.contains(&"placeholder".to_string()),
            "Expected Error::UnfulfilledPlaceholders with correct placeholders"
        );

        Ok(())
    }

    #[test]
    fn create_prompt_invalid_entry_format_1_error() -> Result<()> {
        let template = generate_valid_test_template();
        let invalid_entry = "=".to_string();
        let entries = &[invalid_entry.clone()];
        let create_prompt_result = Prompt::from(&template, entries);

        assert_matches!(
            create_prompt_result,
            Err(Error::InvalidEntryFormat { entry }) if entry == invalid_entry,
            "Expected Error::InvalidEntryFormat with correct entry"
        );

        Ok(())
    }

    #[test]
    fn create_prompt_invalid_entry_format_2_error() -> Result<()> {
        let template = generate_valid_test_template();
        let invalid_entry = "placeholder=".to_string();
        let entries = &[invalid_entry.clone()];
        let create_prompt_result = Prompt::from(&template, entries);

        assert_matches!(
            create_prompt_result,
            Err(Error::InvalidEntryFormat { entry }) if entry == invalid_entry,
            "Expected Error::InvalidEntryFormat with correct entry"
        );

        Ok(())
    }

    #[test]
    fn create_prompt_invalid_entry_format_3_error() -> Result<()> {
        let template = generate_valid_test_template();
        let invalid_entry = "=value".to_string();
        let entries = &[invalid_entry.clone()];
        let create_prompt_result = Prompt::from(&template, entries);

        assert_matches!(
            create_prompt_result,
            Err(Error::InvalidEntryFormat { entry }) if entry == invalid_entry,
            "Expected Error::InvalidEntryFormat with correct entry"
        );

        Ok(())
    }

    #[test]
    fn create_prompt_duplicate_placeholders_error() -> Result<()> {
        let template = generate_valid_test_template();
        let entries = &[
            "placeholder=some text".to_string(),
            "placeholder=some more text".to_string(),
        ];
        let create_prompt_result = Prompt::from(&template, entries);

        assert_matches!(
            create_prompt_result,
            Err(Error::DuplicatePlaceholderProvided { placeholder }) if placeholder == "placeholder",
            "Expected Error::DuplicatePlaceholderProvided with correct placeholder"
        );

        Ok(())
    }
}
