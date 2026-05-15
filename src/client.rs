use crate::error::{Error, Result};
use rig::{
    client::{CompletionClient, Nothing},
    completion::Prompt,
    providers::{anthropic, ollama, openai, openrouter},
};
use std::{env, str::FromStr};

const ANTHROPIC_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";
const OLLAMA_HOST_ENV: &str = "OLLAMA_HOST";
const OPENAI_API_KEY_ENV: &str = "OPENAI_API_KEY";
const OPENROUTER_API_KEY_ENV: &str = "OPENROUTER_API_KEY";

/// Supported LLM providers.
#[derive(Debug, Clone, Copy)]
pub enum Provider {
    Anthropic,
    Ollama,
    OpenAI,
    OpenRouter,
}

impl Provider {
    /// All supported providers, in canonical order.
    pub const ALL: &[Self] = &[
        Self::Anthropic,
        Self::Ollama,
        Self::OpenAI,
        Self::OpenRouter,
    ];

    /// The canonical lowercase name used for parsing and error messages.
    pub const fn name(self) -> &'static str {
        match self {
            Self::Anthropic => "anthropic",
            Self::Ollama => "ollama",
            Self::OpenAI => "openai",
            Self::OpenRouter => "openrouter",
        }
    }

    /// Comma-separated list of supported provider names, for error messages.
    pub fn supported_list() -> String {
        Self::ALL
            .iter()
            .map(|p| p.name())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl FromStr for Provider {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        Self::ALL
            .iter()
            .find(|p| p.name() == s)
            .copied()
            .ok_or_else(|| Error::UnsupportedProvider {
                provided: s.to_owned(),
            })
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
                let api_key = read_env_var(ANTHROPIC_API_KEY_ENV)?;
                let client: anthropic::Client = anthropic::Client::new(&api_key)?;
                client
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
            Provider::Ollama => {
                let client: ollama::Client = match env::var(OLLAMA_HOST_ENV) {
                    Ok(host) => ollama::Client::builder()
                        .api_key(Nothing)
                        .base_url(&host)
                        .build()?,
                    Err(_) => ollama::Client::new(Nothing)?,
                };

                client
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
            Provider::OpenAI => {
                let api_key = read_env_var(OPENAI_API_KEY_ENV)?;
                let client: openai::Client = openai::Client::new(&api_key)?;
                client
                    .agent(&self.model)
                    .max_tokens(self.max_tokens)
                    .build()
                    .prompt(prompt)
                    .await?
            }
            Provider::OpenRouter => {
                let api_key = read_env_var(OPENROUTER_API_KEY_ENV)?;
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

/// Reads a required environment variable, mapping its absence to a domain error.
fn read_env_var(name: &str) -> Result<String> {
    env::var(name).map_err(|e| Error::FailReadEnvVariable {
        env_variable: name.to_string(),
        origin_error: e,
    })
}
