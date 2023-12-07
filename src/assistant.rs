use crate::tool_function::{get_tool_function_from_name, get_tools_json, ToolFunctionExecutor};
use crate::types::{AppError, Message, OpenAIResponse};
use crate::utils::{
    post_json, print_assistant_reply, print_colorful, print_user_prompt, read_file,
    request_tool_call_approval,
};

use crossterm::style::Color;
use reqwest::Client;
use serde_json::json;
use std::io::{self};

pub struct Assistant {
    client: Client,
    api_key: String,
    model: String,
    messages: Vec<Message>,
    initial_prompt: String,
}

impl Assistant {
    pub fn new(client: Client, api_key: String, model: &str, initial_prompt: &str) -> Self {
        Assistant {
            client,
            api_key,
            model: model.to_string(),
            messages: Vec::new(),
            initial_prompt: initial_prompt.to_string(),
        }
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        // Initial setup
        let url = "https://api.openai.com/v1/chat/completions";
        let system_message = read_file("system.txt").unwrap();

        self.add_message(Message::new("system".to_string(), system_message));
        self.add_message(Message::new(
            "user".to_string(),
            self.initial_prompt.to_string(),
        ));

        loop {
            // // JSON payload for the request
            let payload = json!({
                "model": self.model,
                "messages": self.messages,
                "tools": get_tools_json()
            });

            let response: OpenAIResponse =
                post_json(&self.client, url, &self.api_key, &payload).await?;

            self.add_message(response.choices[0].message.clone());

            // Now check for tool calls, and if so add them to self.messages with results
            if let Some(tool_calls) = response
                .choices
                .first()
                .and_then(|c| c.message.tool_calls.as_ref())
            {
                for tool_call in tool_calls {
                    let function_name = &tool_call.function.name;
                    let arguments = &tool_call.function.arguments;

                    // Ask user for approval before executing tool call
                    if request_tool_call_approval(tool_call).await? {
                        if let Some(tool_function) = get_tool_function_from_name(function_name) {
                            match tool_function.execute(arguments).await {
                                Ok(result) => {
                                    log::info!(
                                        "Successfully executed tool call: {:?}\n=>\n{}",
                                        tool_call,
                                        result
                                    );
                                    print_colorful(
                                        &format!("{}\n=>\n{}\n", tool_call.function.name, result),
                                        Color::Magenta,
                                    )?;
                                    self.add_message(Message {
                                        role: "tool".to_string(),
                                        content: Some(result),
                                        tool_calls: None,
                                        tool_call_id: Some(tool_call.id.clone()),
                                        name: Some(function_name.to_string()),
                                    })
                                }
                                Err(e) => {
                                    log::warn!("Failed to execute tool call: {:?}", tool_call);
                                    println!("Error executing function `{}`: {}", function_name, e);
                                }
                            }
                        }
                    } else {
                        // User rejected the tool call, add appropriate message to conversation
                        log::warn!("User rejected tool call: {:?}", tool_call);
                        print_colorful(
                            &format!("User rejected tool call: {:?}", tool_call),
                            Color::DarkRed,
                        )?;

                        self.add_message(Message {
                            role: "tool".to_string(),
                            content: Some("User rejected function call".to_string()),
                            tool_calls: None,
                            tool_call_id: Some(tool_call.id.clone()),
                            name: Some(function_name.to_string()),
                        });
                    }
                }

                let payload = json!({
                    "model": self.model,
                    "messages": self.messages
                });
                let response: OpenAIResponse =
                    post_json(&self.client, url, &self.api_key, &payload).await?;

                print_assistant_reply(response.choices[0].message.content.as_ref().unwrap())?;
            } else {
                print_assistant_reply(response.choices[0].message.content.as_ref().unwrap())?;
            }

            // Handle user input
            print_user_prompt()?;
            let mut user_input = String::new();
            io::stdin().read_line(&mut user_input).unwrap();
            let user_input = user_input.trim();

            if user_input.eq_ignore_ascii_case("exit") || user_input.eq_ignore_ascii_case("quit") {
                break;
            }

            self.add_message(Message::new("user".to_string(), user_input.to_string()));
        }

        Ok(())
    }

    fn add_message(&mut self, message: Message) {
        log::info!("[+] Message: {:?}", message);
        self.messages.push(message);
    }
}
