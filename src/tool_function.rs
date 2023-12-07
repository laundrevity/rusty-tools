use async_trait::async_trait;
use serde_json::{json, Result as JsonResult, Value as JsonValue};
use serde::de::Error as DeError;
use serde_derive::{Deserialize, Serialize};
use std::process::{Command, Output};
use std::io;
use std::fs;
use std::path::Path;

use crate::types::AppError;


#[async_trait]
pub trait ToolFunctionExecutor {
    async fn execute(&self, args: &str) -> Result<String, AppError>;
}

pub enum ToolFunction {
    ExecuteLinuxCommands,
    GetSnapshot,
}

pub fn get_tool_function_from_name(name: &str) -> Option<ToolFunction> {
    match name {
        "execute_linux_commands" => Some(ToolFunction::ExecuteLinuxCommands),
        "get_snapshot" => Some(ToolFunction::GetSnapshot),
        _ => None,
    }
}

#[async_trait]
impl ToolFunctionExecutor for ToolFunction {
    async fn execute(&self, args: &str) -> Result<String, AppError> {
        match self {
            ToolFunction::ExecuteLinuxCommands => {
                // Parse arguments string into JSON
                let args: JsonValue = serde_json::from_str(args)?;
                if let JsonValue::String(commands_json_string) = args["commands"].clone() {
                    let commands: Vec<LinuxCommand> = serde_json::from_str(&commands_json_string)?;
                    execute_linux_commands(commands).await
                } else {
                    Err(AppError::CommandError("commands argument to execute_linux_commands must be a string".to_string()))
                }
            },
            ToolFunction::GetSnapshot => {
                create_project_snapshot().await
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LinuxCommand {
    command: String,
    args: Option<Vec<String>>,
}

async fn execute_linux_commands(commands: Vec<LinuxCommand>) -> Result<String, AppError> {
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

// Function to create the snapshot
async fn create_project_snapshot() -> Result<String, AppError> {
    let mut snapshot = String::new();

    // Read the Cargo.toml file
    snapshot.push_str("File: Cargo.toml\n");
    snapshot.push_str(
        &read_file_contents("Cargo.toml").await?
    );
    snapshot.push_str("\n\n");

    // Read all .rs files in src
    for entry in fs::read_dir("src")? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
            snapshot.push_str(&format!("File: {}\n", path.display()));
            snapshot.push_str(
                &read_file_contents(&path).await?
            );
            snapshot.push_str("\n\n");
        }
    }

    // Write snapshot to state.txt
    fs::write("state.txt", &snapshot).map_err(|e| serde_json::Error::custom(format!("Error writing to file: {}", e)))?;

    // Return the created snapshot
    Ok(snapshot)
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

// Helper to read file contents
async fn read_file_contents<P: AsRef<Path>>(path: P) -> JsonResult<String> {
    fs::read_to_string(path).map_err(|e| serde_json::Error::custom(format!("Error reading file: {}", e)))
}

pub fn get_tools_json() -> JsonValue {
    json!([
        {
            "type": "function",
            "function": {
                "name": "execute_linux_commands",
                "description": "Execute a list of Linux commands in the shell and return their concatenated output",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "commands": {
                            "type": "string",
                            "description": "The JSON-encoded list of Linux commands to execute. Each command must have a key `command` with a String value, and an optional key `args` with an array of Strings for value. So, for example, we would represent `ls -ltrah` with {'command': 'ls', 'args': ['-ltrah']} (in proper JSON of course)"
                        }
                    },
                    "required": ["commands"]
                }
            }
        },
        {
            "type": "function",
            "function": {
                "name": "get_snapshot",
                "description": "Return the formatted source code of the current project",
                "parameters": {
                    "type": "object",
                    "properties": {},
                }
            }
        }])
}