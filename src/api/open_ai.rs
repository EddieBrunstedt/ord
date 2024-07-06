use super::AIClient;
use crate::error::{Error, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

pub struct OpenAIConfig {
    pub api_key: String,
    pub model: String,
}

pub struct OpenAIClient {
    config: OpenAIConfig,
    client: reqwest::Client,
    endpoint: String,
    pub host: String,
}

impl OpenAIClient {
    pub fn new(config: OpenAIConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
            host: "https://api.openai.com".to_string(),
            endpoint: "/v1/chat/completions".to_string(),
        }
    }

    fn create_authorization_header(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        let auth_value = format!("Bearer {}", self.config.api_key);
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&auth_value).map_err(|e| Error::FailCreateHeader {
                header: "Authorization".to_string(),
                origin_error: e,
            })?,
        );
        Ok(headers)
    }

    fn create_request_body(&self, content: &str) -> RequestBody {
        RequestBody {
            model: &self.config.model,
            messages: vec![Message::new(content)],
        }
    }

    fn get_full_url(&self) -> String {
        format!("{}{}", self.host, self.endpoint)
    }
}

#[derive(Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    content: String,
    role: String,
}

impl Message {
    fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
            role: "user".to_string(),
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct OpenAIResponse {
    choices: Vec<Choice>,
    created: i64,
    id: String,
    model: String,
    object: String,
    system_fingerprint: Option<String>,
    usage: Usage,
}

impl OpenAIResponse {
    fn get_result_text(&self) -> &str {
        &self.choices[0].message.content
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Choice {
    message: Message,
    finish_reason: String,
    index: i32,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Usage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[async_trait]
impl AIClient for OpenAIClient {
    async fn send_request(&self, body: &str) -> Result<String> {
        let headers = self.create_authorization_header()?;
        let request_body = self.create_request_body(body);

        let response = self
            .client
            .post(&self.get_full_url())
            .headers(headers)
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(Error::Api {
                status: response.status(),
                message: response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Unable to read error message".to_string()),
            });
        }

        let ai_response: OpenAIResponse = response.json().await.map_err(Error::RequestFailed)?;

        Ok(ai_response.get_result_text().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_authorization_header_ok() {
        let config = OpenAIConfig {
            api_key: "test_api_key".to_string(),
            model: "gpt-4o".to_string(),
        };

        let client = OpenAIClient::new(config);
        let headers = client.create_authorization_header().unwrap();

        assert_eq!(headers.get("Authorization").unwrap(), "Bearer test_api_key",);
    }

    #[test]
    fn create_request_body_ok() {
        let config = OpenAIConfig {
            api_key: "test_api_key".to_string(),
            model: "gpt-4o".to_string(),
        };

        let client = OpenAIClient::new(config);
        let body = client.create_request_body("Test message");

        assert_eq!(body.model, "gpt-4o");
        assert_eq!(body.messages.len(), 1);
        assert_eq!(body.messages[0].content, "Test message");
        assert_eq!(body.messages[0].role, "user");
    }
}
