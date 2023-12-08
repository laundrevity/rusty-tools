use crate::models::{traits::Tool, types::AppError};

use async_trait::async_trait;
use schemars::schema::RootSchema;
use schemars::{schema_for, JsonSchema};
use serde::de::Error as DeError;
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Result as JsonResult, Value as JsonValue};
use std::io;
use std::process::{Command, Output};

// Import this in main file where needed
pub struct ShellTool;

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct ShellCommand {
    command: String,
    args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
struct ShellToolInput {
    commands: Vec<ShellCommand>,
}

#[async_trait]
impl Tool for ShellTool {
    fn name(&self) -> &'static str {
        "shell_tool"
    }

    fn description(&self) -> &'static str {
        "Executes a list of Linux shell commands and returns their concatenated output."
    }

    fn parameters(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "commands": {
                    "type": "string",
                    "description": "The JSON-encoded list of Linux commands to execute. Each command must have a key `command` with a String value, and an optional key `args` with an array of Strings for value. So, for example, we would represent `ls -ltrah` with {'command': 'ls', 'args': ['-ltrah']} (in proper JSON of course)"
                }
            }
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, AppError> {
        let input: ShellToolInput = serde_json::from_value(args)?;
        execute_linux_commands(input.commands).await
    }

    fn input_schema(&self) -> RootSchema {
        schema_for!(ShellToolInput)
    }
}

async fn execute_linux_commands(commands: Vec<ShellCommand>) -> Result<String, AppError> {
    let mut results = Vec::new();
    for linux_command in commands {
        // Spawn a command using the provided command and args
        let mut command = Command::new(&linux_command.command);
        if let Some(args) = linux_command.args {
            command.args(args);
        }

        // Use tokio's spawn_blocking to run the command in a blocking fashion off of the async runtime
        let output = tokio::task::spawn_blocking(move || command.output()).await?;

        // Check and handle command execution results
        results.push(handle_command_output(output, &linux_command.command)?);
    }

    // Join all results into one string, separated by newlines
    Ok(results.join("\n"))
}

fn handle_command_output(output: io::Result<Output>, command: &str) -> JsonResult<String> {
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(stdout)
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            Err(serde_json::Error::custom(format!(
                "Command `{}` failed with error: {}",
                command, stderr
            )))
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Err(serde_json::Error::custom(format!(
            "Command `{}` not found. Please ensure the command exists and is in the PATH.",
            command
        ))),
        Err(e) => Err(serde_json::Error::custom(format!(
            "Failed to execute command `{}` due to error: {}",
            command, e
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn text_execute_shell_commands() {
        let commands = vec![ShellCommand {
            command: "echo".to_string(),
            args: Some(vec!["Hello, world!".to_string()]),
        }];
        let result = execute_linux_commands(commands).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");
    }

    #[tokio::test]
    async fn test_command_not_found() {
        let commands = vec![ShellCommand {
            command: "nonexistent".to_string(),
            args: Some(vec!["arg1".to_string(), "arg2".to_string()]),
        }];
        let result = execute_linux_commands(commands).await;
        assert!(result.is_err());
        if let Err(err) = result {
            assert!(matches!(err, AppError::SerdeJsonError(_)));
            assert!(err.to_string().contains("Command `nonexistent` not found"));
        }
    }

    #[test]
    fn test_shell_input_schema() {
        let shell_tool = ShellTool;
        let _schema = shell_tool.input_schema();
    }
}
