mod open_ai;

use crate::error::Result;
use async_trait::async_trait;

pub use open_ai::{OpenAIClient, OpenAIConfig};

#[async_trait]
pub trait AIClient: Send + Sync {
    async fn send_request(&self, body: &str) -> Result<String>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use mockito::Server;

    #[tokio::test]
    async fn openai_client_send_request_ok() {
        let mut mock_server = Server::new_async().await;

        let mock = mock_server
            .mock("POST", "/v1/chat/completions")
            .match_header("Authorization", "Bearer an_api_key")
            .match_body(r#"{"model":"gpt-4o","messages":[{"content":"Hello, AI!","role":"user"}]}"#)
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body(
                r#"
            {
                "id": "chatcmpl-123",
                "object": "chat.completion",
                "created": 1677652288,
                "choices": [{
                    "index": 0,
                    "message": {
                        "role": "assistant",
                        "content": "Hello, talking meat! How can I assist you today?"
                    },
                    "finish_reason": "stop"
                }],
                "usage": {
                    "prompt_tokens": 9,
                    "completion_tokens": 12,
                    "total_tokens": 21
                },
                "system_fingerprint": null,
                "model": "gpt-4o"
            }
        "#,
            )
            .create();

        let config = OpenAIConfig {
            api_key: "an_api_key".to_string(),
            model: "gpt-4o".to_string(),
        };

        let mut client = OpenAIClient::new(config);
        client.host = mock_server.url();
        let result = client.send_request("Hello, AI!").await;

        mock.assert();

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            "Hello, talking meat! How can I assist you today?"
        );
    }

    #[tokio::test]
    async fn openai_client_send_request_error() {
        let mut mock_server = Server::new_async().await;

        let mock = mock_server
            .mock("POST", "/v1/chat/completions")
            .with_status(400)
            .with_header("content-type", "application/json")
            .with_body(
                r#"{"error": {"message": "Invalid request", "type": "invalid_request_error"}}"#,
            )
            .create();

        let config = OpenAIConfig {
            api_key: "an_api_key".to_string(),
            model: "gpt-4o".to_string(),
        };

        let mut client = OpenAIClient::new(config);
        client.host = mock_server.url();

        let result = client.send_request("Hello, AI!").await;

        mock.assert();
        assert!(result.is_err());

        match result.unwrap_err() {
            Error::Api { status, message } => {
                assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
                assert_eq!(
                    message,
                    r#"{"error": {"message": "Invalid request", "type": "invalid_request_error"}}"#
                );
            }
            _ => panic!("Expected Error::Api"),
        }
    }
}
