use crate::tool_function::{ToolFunctionExecutor, get_tool_function_from_name, get_tools_json};
use crate::utils::{post_json, print_assistant_reply, print_user_prompt};
use crate::types::{Message, OpenAIResponse, AppError};

use reqwest::Client;
use serde_json::json;
use std::io::{self};

pub struct Assistant {
    client: Client,
    api_key: String,
    model: String,
    messages: Vec<Message>,
}

impl Assistant {
    pub fn new(client: Client, api_key: String, model: &str, initial_prompt: &str) -> Self {
        Assistant {
            client,
            api_key,
            model: model.to_string(),
            messages: vec!(
                Message::new("system".to_string(), "You are a versatile assistant. You have the ability to call various tools to help the user".to_string()),
                Message::new("user".to_string(), initial_prompt.to_string())
            )
        }
    }

    pub async fn run(&mut self) -> Result<(), AppError> {
        // Initial setup
        let url = "https://api.openai.com/v1/chat/completions";

        loop {
            // // JSON payload for the request
            let payload = json!({
                "model": self.model,
                "messages": self.messages,
                "tools": get_tools_json()
            });

            let response: OpenAIResponse = post_json(&self.client, url, &self.api_key, &payload).await?;

            self.messages.push(response.choices[0].message.clone());

            // Now check for tool calls, and if so add them to self.messages with results
            if let Some(tool_calls) = response
                .choices
                .first()
                .and_then(|c| c.message.tool_calls.as_ref()) {
                    for tool_call in tool_calls {
                        let function_name = &tool_call.function.name;
                        let arguments = &tool_call.function.arguments;

                        if let Some(tool_function) = get_tool_function_from_name(function_name) {
                            match tool_function.execute(arguments).await {
                                Ok(result) => {
                                    println!("{}\n=>\n{}", function_name, result);
                                    self.messages.push(Message {
                                        role: "tool".to_string(),
                                        content: Some(result),
                                        tool_calls: None,
                                        tool_call_id: Some(tool_call.id.clone()),
                                        name: Some(function_name.to_string()),
                                    })
                                },
                                Err(e) => {
                                    println!("Error executing function `{}`: {}", function_name, e);
                                }
                            }
                        }
                    }
                    let payload = json!({
                        "model": self.model,
                        "messages": self.messages
                    });
                    let response: OpenAIResponse= post_json(&self.client, url, &self.api_key, &payload).await?;
                    
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

                self.messages.push(Message::new(
                    "user".to_string(),
                    user_input.to_string()
                ));
        }

        Ok(())
    }
}