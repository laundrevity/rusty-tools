use async_trait::async_trait;
use serde_json::{json, Result as JsonResult, Value as JsonValue};
use serde::de::Error as DeError;
use serde_derive::{Deserialize, Serialize};
use tokio::time::{sleep, Duration};
use std::process::{Command, Output};
use std::io;


#[async_trait]
pub trait ToolFunctionExecutor {
    async fn execute(&self, args: &str) -> JsonResult<String>;
}

pub enum ToolFunction {
    GetCurrentWeather,
    ExecuteLinuxCommands
}

pub fn get_tool_function_from_name(name: &str) -> Option<ToolFunction> {
    match name {
        "get_current_weather" => Some(ToolFunction::GetCurrentWeather),
        "execute_linux_commands" => Some(ToolFunction::ExecuteLinuxCommands),
        _ => None,
    }
}

#[async_trait]
impl ToolFunctionExecutor for ToolFunction {
    async fn execute(&self, args: &str) -> JsonResult<String> {
        match self {
            ToolFunction::GetCurrentWeather => {
                // Parse arguments string into JSON
                let args: JsonValue = serde_json::from_str(args)?;

                // Extract location argument
                if let Some(location) = args["location"].as_str() {
                    get_current_weather(location.to_string()).await
                } else {
                    Err(serde_json::Error::custom("Location argument is missing"))
                }
            },
            ToolFunction::ExecuteLinuxCommands => {
                // Parse arguments string into JSON
                let args: JsonValue = serde_json::from_str(args)?;
                if let JsonValue::String(commands_json_string) = args["commands"].clone() {
                    let commands: Vec<LinuxCommand> = serde_json::from_str(&commands_json_string)?;
                    execute_linux_commands(commands).await
                } else {
                    Err(serde_json::Error::custom(
                        "Commands argument must be a string",
                    ))
                }
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct LinuxCommand {
    command: String,
    args: Option<Vec<String>>,
}

async fn get_current_weather(location: String) -> JsonResult<String> {
    // Simulate API call by sleeping for 1 sec
    sleep(Duration::from_secs(1)).await;

    Ok(format!("Decent weather in {}, innit?", location))
}

async fn execute_linux_commands(commands: Vec<LinuxCommand>) -> JsonResult<String> {
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

pub fn get_tools_json() -> JsonValue {
    json!([
        {
            "type": "function",
            "function": {
                "name": "get_current_weather",
                "description": "Get the current weather in a given location",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city to get the weather for"
                        }
                    },
                    "required": ["location"]
                }
            }
        },
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
        }])
}