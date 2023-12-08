use crate::{
    models::types::{AppError, Message},
    utils::common::{print_colorful, read_file},
};

use crossterm::style::Color;
use serde_json::Value as JsonValue;
use std::fs::File;
use std::io::Write;

pub struct ConversationManager {
    pub messages: Vec<Message>,
    tools_json: JsonValue,
    tools_schema: String,
    conversation_id: String,
}

impl ConversationManager {
    pub fn new(tools_json: JsonValue, tools_schema: String) -> Self {
        Self {
            messages: vec![],
            tools_json,
            tools_schema,
            conversation_id: "filler".to_string(),
        }
    }

    pub fn add_user_prompt(&mut self, prompt: String) {
        self.add_message(Message::new("user".to_string(), prompt));
    }

    pub fn add_message(&mut self, message: Message) {
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

    pub fn initialize_conversation(
        &mut self,
        initial_prompt: String,
        include_state: bool,
    ) -> Result<(), AppError> {
        let mut system_message = read_file("system.txt")?;

        if include_state {
            if let Ok(state_content) = read_file("state.txt") {
                system_message.push_str("\nHere is the current project source code:\n");
                system_message.push_str(&state_content);
            } else {
                log::warn!("state.txt not found, continuing without state (run `cargo test` to generate it)");
            }
        }

        system_message.push_str("\ntools JSON:\n");
        system_message.push_str(&serde_json::to_string(&self.tools_json)?);
        system_message.push_str("\ntools JSON schemas:\n");
        system_message.push_str(&self.tools_schema);

        self.add_message(Message::new("system".to_string(), system_message));
        self.add_message(Message::new("user".to_string(), initial_prompt));

        Ok(())
    }

    pub fn load_conversation(&mut self, conversation_id: String) -> Result<(), AppError> {
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
}
