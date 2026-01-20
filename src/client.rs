use crate::error::{Error, Result};
use rig::{
    client::{CompletionClient, ProviderClient},
    completion::Prompt,
    providers::{anthropic, openai},
};
use std::str::FromStr;

/// Supported LLM providers.
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Anthropic,
    OpenAI,
}

impl Provider {
    /// List of supported provider names for error messages.
    pub const SUPPORTED: &[&str] = &["anthropic", "openai"];
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "anthropic" => Ok(Self::Anthropic),
            "openai" => Ok(Self::OpenAI),
            _ => Err(Error::UnsupportedProvider {
                provided: s.to_owned(),
            }),
        }
    }
}

/// LLM client for sending prompts and receiving completions.
pub struct Client {
    provider: Provider,
    model: String,
    max_tokens: u64,
}

impl Client {
    /// Creates a new client with the specified provider, model, and token limit.
    pub fn new(provider: Provider, model: String, max_tokens: u64) -> Self {
        Self {
            provider,
            model,
            max_tokens,
        }
    }

    /// Sends a prompt to the configured LLM and returns the response.
    pub async fn send(&self, prompt: &str) -> Result<String> {
        let response = match self.provider {
            Provider::Anthropic => {
                anthropic::Client::from_env()
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
            Provider::OpenAI => {
                openai::Client::from_env()
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
        };
        Ok(response)
    }
}
