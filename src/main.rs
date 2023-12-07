mod assistant;
mod tool_function;
mod types;
mod utils;

use crate::types::AppError;

use std::env;
use assistant::Assistant;
use clap::{Arg, Command};
use reqwest::Client;
use simplelog::*;
use std::fs::OpenOptions;

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
        .arg(Arg::new("log-level")
            .short('l')
            .takes_value(true)
            .possible_values(&["INFO", "DEBUG", "TRACE", "WARN", "ERROR"])
            .default_value("INFO")
            .help("Sets the log level"))
        .get_matches();

    
    setup_logging(matches.value_of("log-level"))?;

    log::info!("Logger initialized");

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

// Logging setup function
fn setup_logging(log_level_arg: Option<&str>) -> Result<(), AppError> {
    let log_level = match log_level_arg {
        Some("DEBUG") => LevelFilter::Debug,
        Some("ERROR") => LevelFilter::Error,
        Some("WARN") => LevelFilter::Warn,
        _ => LevelFilter::Info,
    };

    let logs_dir = "logs";
    std::fs::create_dir_all(logs_dir).map_err(AppError::IOError)?;

    let datetime: String = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
    let log_file_path = format!("{}/{}.log", logs_dir, datetime);

    let log_file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(log_file_path)
        .map_err(AppError::IOError)?;

    CombinedLogger::init(vec![
        WriteLogger::new(log_level, Config::default(), log_file),
    ]).map_err(|e| AppError::CommandError(e.to_string()))
}