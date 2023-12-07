use serde::de::Error as DeError;
use serde_json::{json, Value as JsonValue, Result as JsonResult};
use async_trait::async_trait;
use crate::traits::Tool;
use crate::types::AppError;
use serde_derive::{Serialize, Deserialize};
use std::process::{Command, Output};
use std::io;

// Import this in main file where needed
pub struct ShellTool;

#[derive(Debug, Deserialize, Serialize)]
struct ShellCommand {
    command: String,
    args: Option<Vec<String>>,
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
        if let JsonValue::String(commands_json_string) = args["commands"].clone() {
            let commands: Vec<ShellCommand> = serde_json::from_str(&commands_json_string)?;
            execute_linux_commands(commands).await
        } else {
            Err(AppError::CommandError(
                "commands argument to execute_linux_commands must be a string".to_string(),
            )) 
        }
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
        let output = tokio::task::spawn_blocking(move || command.output())
            .await
            .unwrap();

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
        Err(e) => Err(serde_json::Error::custom(format!(
            "Failed to execute command `{}`: {}",
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
            args: Some(vec!["Hello, world!".to_string()])
        }];
        let result = execute_linux_commands(commands).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, world!");
    }
}