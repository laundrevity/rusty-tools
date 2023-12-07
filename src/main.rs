mod assistant;
mod tool_function;
mod types;
mod utils;

use std::env;
use assistant::Assistant;
use reqwest::{Client, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let api_key = env::var("OPENAI_API_KEY").expect("API key is not set in the environment");
    let mut assistant = Assistant::new(Client::new(), api_key);

    assistant.run().await?;

    Ok(())
}
