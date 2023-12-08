use std::io;

use crossterm::style::Color;

use crate::{models::types::AppError, utils::common::print_colorful};

use super::{conversation_manager::ConversationManager, GLOBAL_TOOL_REGISTRY};

pub enum Command {
    Exit,
    ListTools,
    LoadConversation(String),
    Prompt(String),
}

pub enum CommandHandler {}

impl CommandHandler {
    pub fn read_user_command(tokens: u32) -> Result<Command, AppError> {
        print_colorful(&format!("[{}] User: ", tokens), Color::Yellow)?;

        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
            Ok(Command::Exit)
        } else if user_input.eq_ignore_ascii_case("list tools") {
            Ok(Command::ListTools)
        } else if user_input.to_lowercase().starts_with("load") {
            let parts: Vec<&str> = user_input.splitn(2, ' ').collect();
            if parts.len() == 2 {
                Ok(Command::LoadConversation(parts[1].to_string()))
            } else {
                Err(AppError::CommandError("Invalid load command".to_string()))
            }
        } else {
            Ok(Command::Prompt(user_input.to_string()))
        }
    }

    pub fn get_user_prompt(
        conversation_manager: &mut ConversationManager,
        tokens: u32,
    ) -> Result<(), AppError> {
        let mut got_prompt = false;
        while !got_prompt {
            match CommandHandler::read_user_command(tokens) {
                Ok(command) => match command {
                    Command::Exit => {
                        std::process::exit(0);
                    }
                    Command::ListTools => {
                        print_colorful(&GLOBAL_TOOL_REGISTRY.list_tools(), Color::DarkGreen)?
                    }
                    Command::LoadConversation(conversation_id) => {
                        conversation_manager.load_conversation(conversation_id)?;
                    }
                    Command::Prompt(prompt) => {
                        got_prompt = true;
                        conversation_manager.add_user_prompt(prompt)?;
                    }
                },
                Err(e) => {
                    log::warn!("Failed to parse user command: {}", e);
                }
            }
        }

        Ok(())
    }
}
