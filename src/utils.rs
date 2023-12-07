use crate::types::{AppError, ToolCall};

use serde::de::DeserializeOwned;
use reqwest::Client;
use serde_json::Value;
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand
};
use std::io::{self, Write};


// Utility function to send a post request and wait for a JSON response
pub async fn post_json<T: DeserializeOwned>(
    client: &Client,
    url: &str,
    api_key: &str,
    payload: &Value
) -> Result<T, AppError> {
    client
        .post(url)
        .bearer_auth(api_key)
        .json(payload)
        .send()
        .await
        .map_err(AppError::from)?
        .json()
        .await
        .map_err(AppError::from)
}

// Utility function to print to console with the specified color
pub fn print_colorful(message: &str, color: Color) -> Result<(), AppError> {
    let mut stdout = io::stdout();

    stdout.execute(SetForegroundColor(color)).unwrap();
    print!("{}", message);
    stdout.execute(ResetColor).unwrap();

    io::stdout().flush().map_err(AppError::from)
}

// Utility function for printing the assistant's replies
pub fn print_assistant_reply(reply: &str) -> Result<(), AppError> {
    print_colorful(&format!("Assistant: {}\n", reply), Color::Cyan)
}

// Utility function for printing the user input prefix
pub fn print_user_prompt() -> Result<(), AppError> {
    print_colorful("User: ", Color::Yellow)
}

// Utility function to pretty-print the tool's function call arguments and ask for user approval
pub async fn request_tool_call_approval(tool_call: &ToolCall) -> Result<bool, AppError> {
    // Deserialize JSON arguments string into serde_json::Value
    let args: serde_json::Value = serde_json::from_str(&tool_call.function.arguments)?;

    // Pretty-print the arguments
    let pretty_args = serde_json::to_string_pretty(&args)?;

    // Printing information with formatted pretty-printed arguments
    // Let's use a gentle Blue color for the prompt
    print_colorful(
        &format!("\n{}({}) ? (y/n) ", tool_call.function.name, pretty_args), 
        Color::Blue
    )?;

    // Request user input with a printed prompt
    print!("_ ");
    io::stdout().flush().map_err(AppError::from)?;

    // Read user input
    let mut approval = String::new();
    io::stdin().read_line(&mut approval).map_err(AppError::from)?;
    let approval = approval.trim();

    // Return true if approved ('y' or 'Y'), false otherwise
    Ok(approval.eq_ignore_ascii_case("y"))
}