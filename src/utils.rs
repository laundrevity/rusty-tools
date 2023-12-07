use crate::types::AppError;

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