use crate::models::traits::Tool;
use crate::models::types::AppError;
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool + Sync + Send>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) {
        self.tools.insert(tool.name().to_string(), Box::new(tool));
    }

    pub fn generate_tools_json(&self) -> JsonValue {
        let tools_json: Vec<_> = self
            .tools
            .values()
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name(),
                        "description": tool.description(),
                        "parameters": tool.parameters(),
                    }
                })
            })
            .collect();
        JsonValue::Array(tools_json)
    }

    pub fn generate_tools_schemas(&self) -> String {
        let mut schemas_str = String::new();
        for tool in self.tools.values() {
            schemas_str.push_str(&format!("{} schena:\n", tool.name()));
            schemas_str.push_str(&serde_json::to_string(&tool.input_schema()).unwrap());
            schemas_str.push_str("\n\n");
        }

        schemas_str
    }

    pub async fn execute_tool(&self, tool_name: &str, args: JsonValue) -> Result<String, AppError> {
        if let Some(tool) = self.tools.get(tool_name) {
            tool.execute(args).await
        } else {
            Err(AppError::CommandError(format!(
                "Tool `{}` not found",
                tool_name
            )))
        }
    }

    pub fn list_tools(&self) -> String {
        let mut tools_listing = String::from("Available Tools:\n\n");
        for tool in self.tools.values() {
            tools_listing.push_str(&format!("{} - {}\n", tool.name(), tool.description()));
        }
        tools_listing
    }
}
