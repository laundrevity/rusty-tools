use crate::{
    assistant::conversation_manager::ConversationManager,
    models::types::{AppError, OpenAIResponse},
    utils::common::post_json,
};

use reqwest::Client;
use serde_json::{json, Value as JsonValue};

pub struct OpenAIService {
    url: String,
    api_key: String,
    model: String,
    client: Client,
    tools_json: JsonValue,
}

impl OpenAIService {
    pub fn new(api_key: String, model: String, client: Client, tools_json: JsonValue) -> Self {
        Self {
            url: "https://api.openai.com/v1/chat/completions".to_string(),
            api_key,
            model,
            client,
            tools_json,
        }
    }

    pub async fn call_openai_api(
        &mut self,
        conversation_manager: &ConversationManager,
        include_tools: bool,
    ) -> Result<OpenAIResponse, AppError> {
        let payload = if include_tools {
            json!({
                "model": self.model,
                "messages": conversation_manager.messages,
                "tools": self.tools_json,
            })
        } else {
            json!({
                "model": self.model,
                "messages": conversation_manager.messages,
            })
        };

        post_json(&self.client, &self.url, &self.api_key, &payload).await
    }
}
