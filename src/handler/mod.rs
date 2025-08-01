use crate::{
    cli::{GenerateArgs, NotificationType},
    context::Context,
    display::{display_result, DefaultResultDisplay},
    error::Result,
    interactive_input::{get_user_selection, get_user_text},
    notify::send_notification,
    prompt::Prompt,
    templates::{Template, TemplateCriteria, TemplateDisplay},
    user_config::ValidateConfig,
};
use async_trait::async_trait;

#[async_trait]
pub trait Handler {
    async fn generate(&self, args: &GenerateArgs) -> Result<()>;
    fn list(&self) -> Result<()>;
    fn info(&self, template_title: &Option<String>) -> Result<()>;
}

pub struct DefaultHandler {
    context: Context,
}

impl DefaultHandler {
    pub fn new(context: Context) -> Self {
        Self { context }
    }

    fn select_template(&self, title: &Option<String>) -> Result<&Template> {
        match title {
            Some(title) => self
                .context
                .template_collection
                .find(&TemplateCriteria::Title(title)),
            None => {
                let template_titles = self.context.template_collection.get_template_titles();
                let selection = get_user_selection("Template:", &template_titles)?;
                self.context
                    .template_collection
                    .find(&TemplateCriteria::Index(selection))
            }
        }
    }

    fn gather_entries(&self, template: &Template, args: &GenerateArgs) -> Result<Vec<String>> {
        let mut entries = if args.entries.is_empty() && args.from_clipboard.is_none() {
            self.gather_user_entries(template)?
        } else {
            args.entries.to_vec()
        };

        if let Some(from_clipboard) = &args.from_clipboard {
            let clipboard_content = self.context.clipboard.read()?;
            entries.push(format!("{from_clipboard}={clipboard_content}"));
        }

        Ok(entries)
    }

    fn gather_user_entries(&self, template: &Template) -> Result<Vec<String>> {
        template
            .placeholders
            .iter()
            .enumerate()
            .map(|(i, placeholder)| {
                let prompt = format!(
                    "Replacement {}/{}: {}",
                    i + 1,
                    template.placeholders.len(),
                    placeholder
                );
                let user_input = get_user_text(&prompt)?;
                Ok(format!("{placeholder}={user_input}"))
            })
            .collect()
    }

    async fn process_prompt(&self, prompt: &Prompt) -> Result<String> {
        self.context.ai_client.send_request(&prompt.body).await
    }

    fn handle_result(&self, result: &str, args: &GenerateArgs, template_title: &str) -> Result<()> {
        display_result(
            self.context.display.as_ref(),
            result,
            template_title,
            &args.browser,
            &args.no_new_line,
        )?;

        match &args.notify {
            Some(NotificationType::Status) => send_notification("Ord", "Processing is complete")?,
            Some(NotificationType::Content) => send_notification("Ord", result)?,
            _ => {}
        }

        if args.copy_to_clipboard {
            self.context.clipboard.write(result.to_string())?;
        }

        Ok(())
    }
}

#[async_trait]
impl Handler for DefaultHandler {
    async fn generate(&self, args: &GenerateArgs) -> Result<()> {
        self.context.user_config.validate_config()?;
        let template = self.select_template(&args.title)?;
        let entries = self.gather_entries(template, args)?;
        let prompt = Prompt::from(template, &entries)?;
        let result = self.process_prompt(&prompt).await?;
        self.handle_result(&result, args, &template.title)
    }

    fn list(&self) -> Result<()> {
        self.context.template_collection.list();
        Ok(())
    }

    fn info(&self, template_title: &Option<String>) -> Result<()> {
        let template = self.select_template(template_title)?;
        template.display()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::AIClient,
        clipboard::Clipboard,
        display::ResultDisplay,
        templates::{Template, TemplateCollection, TemplateMetadata},
        user_config::UserConfig,
    };
    use async_trait::async_trait;
    use std::path::PathBuf;

    struct MockAIClient;

    #[async_trait]
    impl AIClient for MockAIClient {
        async fn send_request(&self, _body: &str) -> Result<String> {
            Ok("Mocked AI response".to_string())
        }
    }

    struct MockClipboard;

    impl Clipboard for MockClipboard {
        fn read(&self) -> Result<String> {
            Ok("Clipboard content".to_string())
        }

        fn write(&self, _content: String) -> Result<()> {
            Ok(())
        }
    }

    struct MockDisplay;

    impl ResultDisplay for MockDisplay {
        fn display_to_stdout(&self, _result: &str, _no_new_line: &bool) -> Result<()> {
            Ok(())
        }

        fn display_in_browser(
            &self,
            _result: &str,
            _template_title: &str,
            _browser_type: &crate::cli::BrowserType,
        ) -> Result<()> {
            Ok(())
        }
    }

    fn setup_test_handler() -> DefaultHandler {
        let template_1 = Template {
            title: "test_1".to_string(),
            content: "This is a {{:test}} template".to_string(),
            placeholders: vec!["test".to_string()],
            metadata: TemplateMetadata {
                description: "Test description 1".to_string(),
            },
        };

        let template_2 = Template {
            title: "test_2".to_string(),
            content: "This is another {{:test}} template".to_string(),
            placeholders: vec!["test".to_string()],
            metadata: TemplateMetadata {
                description: "Test description 2".to_string(),
            },
        };

        let context = Context {
            ai_client: Box::new(MockAIClient),
            template_collection: Box::new(TemplateCollection::new(vec![template_1, template_2])),
            clipboard: Box::new(MockClipboard),
            display: Box::new(MockDisplay),
            user_config: UserConfig {
                api_key: "test-api-key".to_string(),
                model: "gpt-4o".to_string(),
                templates_dir_path: PathBuf::from("/test-path"),
                config_file_path: PathBuf::from("/test-path/config.toml"),
            },
        };

        DefaultHandler::new(context)
    }

    #[tokio::test]
    async fn generate_ok() -> Result<()> {
        let handler = setup_test_handler();
        let args = GenerateArgs {
            title: Some("test_1".to_string()),
            entries: vec!["test=fake".to_string()],
            from_clipboard: None,
            copy_to_clipboard: false,
            notify: None,
            browser: None,
            no_new_line: false,
        };

        let result = handler.generate(&args).await;

        assert!(result.is_ok(), "Expected 'Ok' but got: {result:?}");
        Ok(())
    }

    #[test]
    fn info_ok() -> Result<()> {
        let handler = setup_test_handler();
        let result = handler.info(&Some("test_2".to_string()));
        assert!(result.is_ok(), "Expected 'Ok' but got: {result:?}");
        Ok(())
    }

    #[test]
    fn list_ok() -> Result<()> {
        let handler = setup_test_handler();
        let result = handler.list();
        assert!(result.is_ok(), "Expected 'Ok' but got: {result:?}");
        Ok(())
    }
}
