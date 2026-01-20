use crate::{
    cli::{NotificationType, RunArgs},
    client::Client,
    clipboard::Clipboard,
    config::Config,
    embedded_templates,
    error::{Error, Result},
    input, notify, output,
    prompt::Prompt,
    template::{FindBy, Template, TemplateCollection},
};

/// Core application state and orchestration.
///
/// Manages the LLM client, clipboard access, configuration, and template collection.
pub struct App {
    client: Client,
    clipboard: Clipboard,
    config: Config,
    templates: TemplateCollection,
}

impl App {
    /// Creates a new application instance.
    ///
    /// Loads configuration, initializes the LLM client, loads templates from the
    /// configured directory, and sets up clipboard access.
    pub fn new() -> Result<Self> {
        let config = Config::load()?;
        let client = Client::new(config.provider, config.model.clone(), config.max_tokens);
        let templates = Self::load_templates(&config)?;
        let clipboard = Clipboard::new()?;

        Ok(Self {
            client,
            clipboard,
            config,
            templates,
        })
    }

    fn load_templates(config: &Config) -> Result<TemplateCollection> {
        match TemplateCollection::from_directory(&config.templates_dir_path) {
            Ok(templates) => Ok(templates),
            Err(Error::NoTemplatesAvailable { ref templates_dir }) => {
                let count = embedded_templates::example_template_count();
                let prompt = format!(
                    "No templates found. Install {count} example templates to get started?"
                );

                if input::get_confirmation(&prompt)? {
                    embedded_templates::install_example_templates(templates_dir)?;
                    println!("Example templates installed to {}", templates_dir.display());
                    TemplateCollection::from_directory(&config.templates_dir_path)
                } else {
                    Err(Error::NoTemplatesAvailable {
                        templates_dir: templates_dir.clone(),
                    })
                }
            }
            Err(e) => Err(e),
        }
    }

    /// Executes a template with the provided arguments.
    ///
    /// Validates configuration, selects the template, gathers placeholder values,
    /// sends the prompt to the LLM, and handles the result according to the options.
    pub async fn run(&self, args: &RunArgs) -> Result<()> {
        self.config.validate()?;

        let template = self.select_template(&args.title)?;
        let entries = self.gather_entries(template, args)?;
        let prompt = Prompt::from_template(template, &entries)?;
        let result = self.client.send(&prompt.body).await?;

        self.handle_result(&result, args, &template.title)
    }

    /// Lists all available templates with their descriptions.
    pub fn list(&self) {
        self.templates.list();
    }

    /// Displays detailed information about a specific template.
    pub fn info(&self, title: &Option<String>) -> Result<()> {
        let template = self.select_template(title)?;
        template.display();
        Ok(())
    }

    fn select_template(&self, title: &Option<String>) -> Result<&Template> {
        match title {
            Some(title) => self.templates.find(FindBy::Title(title)),
            None => {
                let titles = self.templates.titles();
                let selection = input::get_selection("Template:", &titles)?;
                self.templates.find(FindBy::Index(selection))
            }
        }
    }

    fn gather_entries(&self, template: &Template, args: &RunArgs) -> Result<Vec<String>> {
        let mut entries = if args.entries.is_empty() && args.from_clipboard.is_none() {
            self.gather_user_entries(template)?
        } else {
            args.entries.clone()
        };

        if let Some(placeholder) = &args.from_clipboard {
            let clipboard_content = self.clipboard.read()?;
            entries.push(format!("{placeholder}={clipboard_content}"));
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
                let user_input = input::get_text(&prompt)?;
                Ok(format!("{placeholder}={user_input}"))
            })
            .collect()
    }

    fn handle_result(&self, result: &str, args: &RunArgs, template_title: &str) -> Result<()> {
        output::print_result(result, args.no_newline);

        if let Some(browser_type) = &args.browser {
            output::open_in_browser(result, template_title, browser_type)?;
        }

        match &args.notify {
            Some(NotificationType::Status) => notify::send("Ord", "Processing is complete")?,
            Some(NotificationType::Content) => notify::send("Ord", result)?,
            None => {}
        }

        if args.copy_to_clipboard {
            self.clipboard.write(result)?;
        }

        Ok(())
    }
}
