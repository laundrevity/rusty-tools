use crossterm::style::Color;
use proc_macro_crate::auto_register_tools;

pub mod command_handler;
pub mod conversation_manager;

use crate::{
    api::openai_service::OpenAIService,
    models::types::{AppError, Message, OpenAIResponse, ToolCall},
    registry::tool_registry::ToolRegistry,
    utils::common::{print_assistant_reply, print_colorful, request_tool_call_approval},
};
use conversation_manager::ConversationManager;

use lazy_static::lazy_static;
use reqwest::Client;
use serde_json::Value as JsonValue;

use self::command_handler::CommandHandler;

auto_register_tools!();

lazy_static! {
    pub static ref GLOBAL_TOOL_REGISTRY: ToolRegistry = {
        let mut registry = ToolRegistry::new();
        register_tools(&mut registry); // Use the generated function
        registry
    };
}

pub struct Assistant {
    conversation_manager: ConversationManager,
    openai_service: OpenAIService,
    tool_registry: &'static ToolRegistry,
    tokens: u32,
}

impl Assistant {
    pub fn new(client: Client, api_key: String, model: String) -> Self {
        let tool_registry = &*GLOBAL_TOOL_REGISTRY;
        let tools_json: JsonValue = tool_registry.generate_tools_json();
        let tools_schema: String = tool_registry.generate_tools_schemas();

        Assistant {
            conversation_manager: ConversationManager::new(tools_json.clone(), tools_schema),
            openai_service: OpenAIService::new(api_key, model, client, tools_json),
            tool_registry,
            tokens: 0,
        }
    }

    pub async fn get_response(&mut self, include_tools: bool) -> Result<OpenAIResponse, AppError> {
        let response: OpenAIResponse = self
            .openai_service
            .call_openai_api(&self.conversation_manager, include_tools)
            .await?;
        self.tokens = response.usage.total_tokens;
        self.conversation_manager
            .add_message(response.choices[0].message.clone())?;

        Ok(response)
    }

    pub async fn handle_tool_call(&mut self, tool_call: &ToolCall) -> Result<(), AppError> {
        let function_name = &tool_call.function.name;
        // Could we actually have the arguments directly be a JsonValue, instead of a String?
        let arguments: JsonValue = serde_json::from_str(&tool_call.function.arguments)?;
        let tool_result: String;

        if request_tool_call_approval(&tool_call).await? {
            match self
                .tool_registry
                .execute_tool(&function_name, arguments)
                .await
            {
                Ok(result) => {
                    let tool_call_str = format!("{:?}\n=>\n{}\n", tool_call, result);
                    log::info!("Succesfully executed tool call: {}", tool_call_str);
                    print_colorful(&tool_call_str, Color::DarkMagenta)?;
                    tool_result = result;
                }
                Err(e) => {
                    let error_str =
                        format!("Error executing tool call: {:?}\n=>\n{}", tool_call, e);
                    log::warn!("{}", error_str);
                    print_colorful(&error_str, Color::Red)?;
                    tool_result = e.to_string();
                }
            }
        } else {
            let error_str = format!("User rejected tool call: {:?}", tool_call);
            log::warn!("{}", error_str);
            print_colorful(&error_str, Color::DarkRed)?;
            tool_result = error_str;
        }

        self.conversation_manager.add_message(Message {
            role: "tool".to_string(),
            content: Some(tool_result),
            tool_calls: None,
            tool_call_id: Some(tool_call.id.clone()),
            name: Some(function_name.to_string()),
        })?;

        Ok(())
    }

    pub async fn run(
        &mut self,
        initial_prompt: String,
        include_state: bool,
    ) -> Result<(), AppError> {
        // Initialize conversations with system message (optionally including state) and first user message
        self.conversation_manager
            .initialize_conversation(initial_prompt, include_state)?;

        // Assistant main loop
        loop {
            // Get a response including possible tool calls
            let mut response: OpenAIResponse = self.get_response(true).await?;

            if let Some(tool_calls) = &response.choices[0].message.tool_calls {
                for tool_call in tool_calls {
                    self.handle_tool_call(tool_call).await?;
                }

                // Now let Assistant generate response using the tool call results
                response = self.get_response(false).await?;
            }

            match &response.choices[0].message.content {
                Some(content) => {
                    print_assistant_reply(&content)?;
                }
                None => {}
            }

            CommandHandler::get_user_prompt(&mut self.conversation_manager, self.tokens)?;
        }
    }
}
