use proc_macro_crate::auto_register_tools;

use crate::tool_registry::ToolRegistry;
use crate::types::{AppError, Message, OpenAIResponse, ToolCall};
use crate::utils::{
    post_json, print_assistant_reply, print_colorful, print_user_prompt, read_file,
    request_tool_call_approval,
};

use crossterm::style::Color;
use lazy_static::lazy_static;
use reqwest::Client;
use serde_json::{json, Value as JsonValue};
use std::fs::File;
use std::io::{self, Write};
use uuid::Uuid;

// Generate imports and register_tools function
auto_register_tools!();

lazy_static! {
    pub static ref GLOBAL_TOOL_REGISTRY: ToolRegistry = {
        let mut registry = ToolRegistry::new();
        register_tools(&mut registry); // Use the generated function
        registry
    };
}

enum UserCommand {
    Exit,
    ListTools,
    LoadConversation(String),
    Other(String),
}

pub struct Assistant {
    client: Client,
    api_key: String,
    model: String,
    messages: Vec<Message>,
    initial_prompt: String,
    tool_registry: &'static ToolRegistry,
    include_state: bool,
    show_usage: bool,
    tokens: u32,
    conversation_id: String,
    found_next_prompt: bool,
    endpoint_url: &'static str,
}

impl Assistant {
    pub fn new(
        client: Client,
        api_key: String,
        model: &str,
        initial_prompt: &str,
        include_state: bool,
        show_usage: bool,
    ) -> Self {
        let tool_registry = &*GLOBAL_TOOL_REGISTRY;

        Assistant {
            client,
            api_key,
            model: model.to_string(),
            messages: Vec::new(),
            initial_prompt: initial_prompt.to_string(),
            tool_registry,
            include_state,
            show_usage,
            tokens: 0,
            conversation_id: Uuid::new_v4().to_string(),
            found_next_prompt: false,
            endpoint_url: "https://api.openai.com/v1/chat/completions",
        }
    }

    async fn handle_command(&mut self, command: UserCommand) -> Result<(), AppError> {
        match command {
            UserCommand::Exit => {
                self.exit();
            }
            UserCommand::ListTools => {
                self.list_available_tools().await?;
            }
            UserCommand::LoadConversation(conv_id) => {
                self.load_conversation(&conv_id).await?;
            }
            UserCommand::Other(input) => {
                self.process_input(&input).await?;
            }
        }
        Ok(())
    }

    async fn list_available_tools(&self) -> Result<(), AppError> {
        let tools_listing = self.tool_registry.list_tools();
        print_colorful(&tools_listing, Color::Green)?;
        Ok(())
    }

    async fn load_conversation(&mut self, conversation_id: &str) -> Result<(), AppError> {
        let file_path = format!("conversations/{}.json", conversation_id);
        let content = std::fs::read_to_string(file_path)?;
        let new_messages: Vec<Message> = serde_json::from_str(&content)?;
        print_colorful(
            &format!("Successfully loaded conversation {}\n", conversation_id),
            Color::DarkMagenta,
        )?;

        self.messages = new_messages;
        self.conversation_id = conversation_id.to_string();

        Ok(())
    }

    fn exit(&self) {
        print_colorful("Received `exit` command, shutting down...", Color::Grey).unwrap();
        std::process::exit(0);
    }

    async fn process_input(&mut self, input: &String) -> Result<(), AppError> {
        self.add_message(Message::new("user".to_string(), input.to_string()));
        self.found_next_prompt = true;
        Ok(())
    }

    fn get_prompt_tokens_option(&self) -> Option<u32> {
        if self.show_usage {
            Some(self.tokens)
        } else {
            None
        }
    }

    fn read_user_command(&self) -> Result<Option<UserCommand>, AppError> {
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            Ok(Some(UserCommand::Exit))
        } else if user_input.eq_ignore_ascii_case("list tools") {
            Ok(Some(UserCommand::ListTools))
        } else if user_input.to_lowercase().starts_with("load ") {
            let parts: Vec<&str> = user_input.splitn(2, ' ').collect();
            if parts.len() == 2 {
                Ok(Some(UserCommand::LoadConversation(parts[1].to_string())))
            } else {
                Err(AppError::CommandError("Invalid load command".to_string()))
            }
        } else {
            Ok(Some(UserCommand::Other(user_input.to_string())))
        }
    }

    pub fn initialize_conversation(&mut self) -> Result<(), AppError> {
        let mut system_message = read_file("system.txt")?;

        if self.include_state {
            if let Ok(state_contents) = read_file("state.txt") {
                system_message.push_str("\nHere is the current project source code:\n");
                system_message.push_str(&state_contents);
            } else {
                log::warn!("state.txt not found, continuing without state (run `cargo test` to generate it)");
            }
        }

        system_message.push_str("\ntools JSON:\n");
        system_message.push_str(&self.tool_registry.generate_tools_json().to_string());
        system_message.push_str("\ntools JSON schemas:\n");
        system_message.push_str(&self.tool_registry.generate_tools_schemas());

        self.add_message(Message::new("system".to_string(), system_message));
        self.add_message(Message::new(
            "user".to_string(),
            self.initial_prompt.to_string(),
        ));

        Ok(())
    }

    async fn get_response(&mut self, use_tools: bool) -> Result<OpenAIResponse, AppError> {
        let payload = if use_tools {
            json!({
                "model": self.model,
                "messages": self.messages,
                "tools": self.tool_registry.generate_tools_json()
            })
        } else {
            json!({
                "model": self.model,
                "messages": self.messages
            })
        };

        let response: OpenAIResponse =
            post_json(&self.client, self.endpoint_url, &self.api_key, &payload).await?;
        self.tokens = response.usage.total_tokens;
        self.add_message(response.choices[0].message.clone());

        Ok(response)
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        self.initialize_conversation()?;

        loop {
            let mut response: OpenAIResponse = self.get_response(true).await?;

            if let Some(tool_calls) = &response.choices[0].message.tool_calls {
                for tool_call in tool_calls {
                    self.handle_tool_call(tool_call).await?;
                }

                response = self.get_response(false).await?;
            }

            match &response.choices[0].message.content {
                Some(content) => {
                    print_assistant_reply(&content)?;
                }
                None => {}
            }

            // Handle user input
            let tokens = self.get_prompt_tokens_option();
            self.found_next_prompt = false;

            // Keep looping until we get next user prompt
            while !self.found_next_prompt {
                print_user_prompt(tokens)?;
                if let Some(command) = self.read_user_command()? {
                    self.handle_command(command).await?;
                } else {
                    print_colorful("Failed to parse user command", Color::Red)?;
                    log::error!("Failed to parse user command");
                    break;
                }
            }
        }
    }

    async fn handle_tool_call(&mut self, tool_call: &ToolCall) -> Result<(), AppError> {
        let function_name = &tool_call.function.name;
        let arguments: JsonValue = serde_json::from_str(&tool_call.function.arguments)?;
        let tool_result;

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

        self.add_message(Message {
            role: "tool".to_string(),
            content: Some(tool_result),
            tool_calls: None,
            tool_call_id: Some(tool_call.id.clone()),
            name: Some(function_name.to_string()),
        });

        Ok(())
    }

    fn add_message(&mut self, message: Message) {
        log::info!("[+] Message: {:?}", message);
        self.messages.push(message);

        // Try to serialize the entire conversation to JSON and write to file
        match serde_json::to_string(&self.messages) {
            Ok(messages_str) => {
                let conversation_file_path = format!("conversations/{}.json", self.conversation_id);
                let mut file = File::create(&conversation_file_path)
                    .map_err(AppError::from)
                    .unwrap();

                file.write_all(messages_str.as_bytes()).unwrap();
            }
            Err(_) => {
                log::warn!("Failed to write conversation")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serde_messages() {
        let mut messages = vec![Message::new(
            "system".to_string(),
            "You're a lumberjack and you're okay.".to_string(),
        )];
        messages.push(Message::new(
            "user".to_string(),
            "What do you seek?".to_string(),
        ));

        let messages_str = serde_json::to_string(&messages).unwrap();
        let decoded_messages: Vec<Message> = serde_json::from_str(&messages_str).unwrap();

        assert_eq!(messages, decoded_messages);
    }
}
