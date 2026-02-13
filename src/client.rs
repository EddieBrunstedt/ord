use crate::error::{Error, Result};
use rig::{
    client::CompletionClient,
    completion::Prompt,
    providers::{anthropic, openai, openrouter},
};
use std::{env, str::FromStr};

/// Supported LLM providers.
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Anthropic,
    OpenAI,
    OpenRouter,
}

impl Provider {
    /// List of supported provider names for error messages.
    pub const SUPPORTED: &[&str] = &["anthropic", "openai", "openrouter"];

    /// Returns the environment variable name for the provider's API key.
    pub fn api_key_env_var(self) -> &'static str {
        match self {
            Self::Anthropic => "ANTHROPIC_API_KEY",
            Self::OpenAI => "OPENAI_API_KEY",
            Self::OpenRouter => "OPENROUTER_API_KEY",
        }
    }
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "anthropic" => Ok(Self::Anthropic),
            "openai" => Ok(Self::OpenAI),
            "openrouter" => Ok(Self::OpenRouter),
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
        let env_var = self.provider.api_key_env_var();
        let api_key =
            env::var(env_var).map_err(|e| Error::FailReadEnvVariable {
                env_variable: env_var.to_string(),
                origin_error: e,
            })?;

        let response = match self.provider {
            Provider::Anthropic => {
                let client: anthropic::Client = anthropic::Client::new(&api_key)?;
                client
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
            Provider::OpenAI => {
                let client: openai::Client = openai::Client::new(&api_key)?;
                client
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
            Provider::OpenRouter => {
                let client: openrouter::Client = openrouter::Client::new(&api_key)?;
                client
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
