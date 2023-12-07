mod assistant;
mod tool_function;
mod types;
mod utils;

use crate::types::AppError;

use std::env;
use assistant::Assistant;
use clap::{Arg, Command};
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    // Parse command-line arguments
    let matches = Command::new("Assistant")
        .version("1.0.0")
        .author("Conor Mahany <conor@mahany.io>")
        .about("Console interface for AI-powered assistant")
        .arg(Arg::new("initial_prompt")
            .help("Sets the initial prompt for the assistant")
            .required(true)
            .index(1))
        .arg(Arg::new("model")
            .short('m')
            .long("model")
            .help("Sets the model to use with the OpenAI API")
            .takes_value(true)
            .default_value("gpt-4-1106-preview"))
        .get_matches();

    // Retrieve the command-line arguments and API key from env
    let initial_prompt = matches.value_of("initial_prompt").unwrap();
    let model = matches.value_of("model").unwrap();
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| AppError::MissingEnvironmentVariable("OPENAI_API_KEY".to_string()))?;

    let mut assistant = Assistant::new(
        Client::new(), 
        api_key,
        model,
        initial_prompt
    );

    assistant.run().await?;

    Ok(())
}
