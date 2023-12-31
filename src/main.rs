mod api;
mod assistant;
mod models;
mod registry;
mod tools;
mod utils;

use crate::models::types::AppError;

use crate::assistant::Assistant;
use clap::{Arg, ArgMatches, Command};
use reqwest::Client;
use simplelog::*;
use std::env;
use std::fs::OpenOptions;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    let matches = parse_command_line_arguments()?;
    setup_logging(matches.value_of("log-level"))?;

    log::info!("Logger initialized");

    // Create conversation archive
    let convs_dir = "conversations";
    std::fs::create_dir_all(convs_dir).map_err(AppError::IOError)?;

    // Retrieve the command-line arguments and API key from env
    let initial_prompt = matches.value_of("initial_prompt").unwrap();
    let model = matches.value_of("model").unwrap();
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| AppError::MissingEnvironmentVariable("OPENAI_API_KEY".to_string()))?;

    let mut assistant = Assistant::new(Client::new(), api_key, model.to_string());

    assistant
        .run(initial_prompt.to_string(), matches.is_present("state"))
        .await?;

    Ok(())
}

fn parse_command_line_arguments() -> Result<ArgMatches, AppError> {
    let matches = Command::new("Assistant")
        .version("1.0.0")
        .author("Conor Mahany <conor@mahany.io>")
        .about("Console interface for AI-powered assistant")
        .arg(
            Arg::new("initial_prompt")
                .help("Sets the initial prompt for the assistant")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .help("Sets the model to use with the OpenAI API")
                .takes_value(true)
                .default_value("gpt-4-1106-preview"),
        )
        .arg(
            Arg::new("log-level")
                .short('l')
                .takes_value(true)
                .possible_values(&["INFO", "DEBUG", "TRACE", "WARN", "ERROR"])
                .default_value("INFO")
                .help("Sets the log level"),
        )
        .arg(
            Arg::new("state")
                .short('s')
                .long("state")
                .help("Appends the contents of state.txt to the initial system prompt")
                .takes_value(false),
        )
        .get_matches();

    Ok(matches)
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

    CombinedLogger::init(vec![WriteLogger::new(
        log_level,
        Config::default(),
        log_file,
    )])
    .map_err(|e| AppError::CommandError(e.to_string()))
}
