use serde_derive::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub function: FunctionCall,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Message {
    pub role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Choice {
    index: u32,
    pub message: Message,
    finish_reason: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenAIResponse {
    id: String,
    model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

impl Message {
    pub fn new(role: String, content: String) -> Self {
        Self {
            role,
            content: Some(content),
            tool_calls: None,
            tool_call_id: None,
            name: None,
        }
    }
}

#[derive(Debug)]
pub enum AppError {
    ReqwestError(reqwest::Error),
    IOError(std::io::Error),
    SerdeJsonError(serde_json::Error),
    TaskJoinError(tokio::task::JoinError),
    MissingEnvironmentVariable(String),
    CommandError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ReqwestError(e) => write!(f, "HTTP request failed: {}", e),
            AppError::IOError(e) => write!(f, "IO error: {}", e),
            AppError::SerdeJsonError(e) => write!(f, "Serialization/Deserialization error: {}", e),
            AppError::TaskJoinError(e) => write!(f, "TaskJoinError: {}", e),
            AppError::MissingEnvironmentVariable(e) => {
                write!(f, "Missing environment variable: {}", e)
            }
            AppError::CommandError(e) => write!(f, "Error with command: {}", e),
        }
    }
}

// Implement std::error::Error for our custom type.
impl std::error::Error for AppError {}

// Implement From traits for converting from error types to our custom AppError type.
impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        AppError::ReqwestError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IOError(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::SerdeJsonError(err)
    }
}

impl From<tokio::task::JoinError> for AppError {
    fn from(err: tokio::task::JoinError) -> Self {
        AppError::TaskJoinError(err)
    }
}
