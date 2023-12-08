use crate::assistant::conversation_manager::ConversationManager;
use crate::assistant::GLOBAL_TOOL_REGISTRY;
use crate::models::types::Message;
use crate::models::{traits::Tool, types::AppError};
use crate::api::openai_service::OpenAIService;

use async_trait::async_trait;
use reqwest::Client;
use schemars::{schema_for, JsonSchema, schema::RootSchema};
use serde_json::{json, Value as JsonValue};
use serde_derive::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
pub struct GptTool;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GptToolInput {
    include_tools: bool,
    messages: Vec<GptMessage>,
    model: String,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct GptMessage {
    role: String,
    content: String,
}

#[async_trait]
impl Tool for GptTool {
    fn name(&self) -> &'static str {
        "gpt_tool"
    }

    fn description(&self) -> &'static str {
        "Get a response from a new ChatCompletion via the OpenAI API"
    }

    fn parameters(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "include_tools": {
                    "type": "boolean",
                    "description": "Whether or not to allow for tool calls, either `true` or `false`"
                },
                "model": {
                    "type": "string",
                    "description": "GPT model to use, options are: `gpt-4-1106-preview`, `gpt-4`, `gpt-3.5-turbo`"
                },
                "messages": {
                    "type": "string",
                    "description": "The array of messages in our ChatCompletion. This value MUST be JSON. Each message has a key `role` with value in (`user`, `assistant`, `system`) and a key `content` with string value."
                }
            }
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, AppError> {
        let input: GptToolInput = serde_json::from_value(args)?;

        // Get data needed for creating OpenAIService
        let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| AppError::MissingEnvironmentVariable("OPENAI_API_KEY".to_string()))?;

        let tools_json = GLOBAL_TOOL_REGISTRY.generate_tools_json();

        let mut openai_service = OpenAIService::new(
            api_key, 
            input.model, 
            Client::new(), 
            tools_json.clone()
        );

        let mut conversation_manager = ConversationManager::new(
            tools_json,
            GLOBAL_TOOL_REGISTRY.generate_tools_schemas()
        );

        // add the messages to conversation manager
        for gpt_message in input.messages {
            conversation_manager.add_message(Message::new(gpt_message.role, gpt_message.content))?;
        }

        let response = openai_service.call_openai_api(&conversation_manager, input.include_tools).await?;

        dbg!(&response);

        match serde_json::to_string(&response) {
            Ok(response_str) => Ok(response_str),
            Err(e) => Err(AppError::SerdeJsonError(e))
        }

    }

    fn input_schema(&self) -> RootSchema {
        schema_for!(GptToolInput)
    }

}