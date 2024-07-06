use copypasta_ext::display;

use crate::{
    api::{AIClient, OpenAIClient, OpenAIConfig},
    clipboard::{Clipboard, ClipboardManager},
    display::{DefaultResultDisplay, ResultDisplay},
    error::{Error, Result},
    templates::{TemplateCollection, TemplateRepository},
    user_config::UserConfig,
};

pub struct Context {
    pub ai_client: Box<dyn AIClient>,
    pub template_collection: Box<dyn TemplateRepository>,
    pub clipboard: Box<dyn Clipboard>,
    pub display: Box<dyn ResultDisplay>,
    pub user_config: UserConfig,
}

impl Context {
    pub fn create() -> Result<Self> {
        let user_config = UserConfig::new()?;

        let ai_client = Box::new(OpenAIClient::new(OpenAIConfig {
            api_key: user_config.api_key.clone(),
            model: user_config.model.clone(),
        }));

        let template_collection = Box::new(TemplateCollection::from_directory(
            &user_config.templates_dir_path,
        )?);

        let clipboard = Box::new(ClipboardManager::new(
            copypasta_ext::try_context().ok_or(Error::FailInitClipboard)?,
        ));

        let display = Box::new(DefaultResultDisplay);

        Ok(Self::new(
            ai_client,
            template_collection,
            clipboard,
            display,
            user_config,
        ))
    }

    fn new(
        ai_client: Box<dyn AIClient>,
        template_collection: Box<dyn TemplateRepository>,
        clipboard: Box<dyn Clipboard>,
        display: Box<dyn ResultDisplay>,
        user_config: UserConfig,
    ) -> Self {
        Self {
            ai_client,
            template_collection,
            clipboard,
            display,
            user_config,
        }
    }
}
