use crate::assistant::GLOBAL_TOOL_REGISTRY;
use crate::traits::Tool;
use crate::types::AppError;

use async_trait::async_trait;
use serde_derive::{Deserialize, Serialize};
use serde_json::{self, json, value::RawValue, Value as JsonValue};

#[derive(Serialize, Deserialize, Debug)]
pub struct PipelineTool;

#[derive(Serialize, Deserialize)]
struct PipelineStep {
    id: String,
    tool: String,
    parameters: Box<RawValue>,
}

impl PipelineTool {
    // Helper function to substitute placeholders with context values
    fn substitute_placeholders(value: &str, context: &JsonValue) -> String {
        // Note: This is a naive implementation. You may need a more robust method to handle placeholders.
        let mut result = value.to_string();
        if let Some(context_map) = context.as_object() {
            for (key, val) in context_map {
                let placeholder = format!("${{{}}}", key);
                if let Some(rep) = val.as_str() {
                    result = result.replace(&placeholder, rep);
                }
            }
        }
        result
    }

    // Helper function to resolve parameters
    fn resolve_parameters(params: &JsonValue, context: &JsonValue) -> JsonValue {
        match params {
            JsonValue::Object(map) => {
                let mut resolved = serde_json::Map::new();
                for (key, val) in map {
                    resolved.insert(key.clone(), Self::resolve_parameters(val, context));
                }
                JsonValue::Object(resolved)
            }
            JsonValue::Array(vec) => JsonValue::Array(
                vec.iter()
                    .map(|v| Self::resolve_parameters(v, context))
                    .collect(),
            ),
            JsonValue::String(s) => JsonValue::String(Self::substitute_placeholders(s, context)),
            _ => params.clone(),
        }
    }
}

#[async_trait]
impl Tool for PipelineTool {
    fn name(&self) -> &'static str {
        "pipeline_tool"
    }

    fn description(&self) -> &'static str {
        "Executes a series of tool calls, passing the output of one as the input to another using substitutions `${priorStepId}`"
    }

    fn parameters(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "steps": {
                    "type": "string",
                    "description": "The JSON-encoded list of pipeline steps to execute, with possible substitutions. Each pipeline step must have a string with key `id`, a string with key `tool`, and a string with key `parameters`. The `id` value can be subsequently referenced by other pipeline steps to use the result of a prior step as input. The `tool` value must be one of the known tools (e.g. shell_tool, snap_tool, etc). The `parameters` value should be a stringified JSON of arguments to pass to the tool. Do NOT prepend the tool names with `functions.` - the passed tool names should just be like `shell_tool`, `snap_tool`, etc. You can use `${stepId}` to access the value in a later step from the step with id `stepId`. "
                }
            }
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, AppError> {
        if let JsonValue::String(steps_json_string) = args["steps"].clone() {
            let steps: Vec<PipelineStep> = serde_json::from_str(&steps_json_string)?;
            let mut context = serde_json::Map::new();

            for step in steps {
                let raw_parameters_json = step.parameters.get(); // Get JSON as a string.

                // Deserialize JSON string to JsonValue (serde_json::Value)
                let parameters_json = serde_json::from_str(raw_parameters_json)
                    .map_err(|_| AppError::CommandError("Invalid JSON parameters".into()))?;

                // Substitute placeholders and resolve to a final JsonValue.
                let resolved_parameters =
                    Self::resolve_parameters(&parameters_json, &JsonValue::Object(context.clone()));

                // Serialize parameters if they need to be a string otherwise keep as JsonValue.
                let serialized_parameters = match &resolved_parameters {
                    JsonValue::Object(_) | JsonValue::Array(_) => {
                        serde_json::to_string(&resolved_parameters)?
                    }
                    _ => resolved_parameters.to_string(),
                };

                // Execute the tool and save the output in the context.
                let output = GLOBAL_TOOL_REGISTRY
                    .execute_tool(&step.tool, serde_json::from_str(&serialized_parameters)?)
                    .await?;

                context.insert(step.id.clone(), JsonValue::String(output));
            }

            Ok(serde_json::to_string(&JsonValue::Object(context))?)
        } else {
            return Err(AppError::CommandError(
                "`steps` argument to PipelineTool must be a string.".into(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PipelineTool;
    use crate::traits::Tool;
    use serde_json::{json, Value as JsonValue};

    #[tokio::test]
    async fn test_simple_pipeline() {
        let pipeline_tool = PipelineTool;

        // The `commands` parameter for `ShellTool` should be a stringified JSON array.
        let commands_json_string = json!(
            [
                {
                    "command": "echo",
                    "args": ["hello world"]
                }
            ]
        )
        .to_string();

        // Step that simulates `echo "hello world"`
        let args = json!({
            "steps": json!([
            {
                "id": "echoResult",
                "tool": "shell_tool",
                "parameters": {
                    "commands": commands_json_string
                }
            }
            ]).to_string()
        });

        // dbg!(&args);

        // Execute pipeline with given args
        let result = pipeline_tool.execute(args).await.unwrap();

        // Deserialize the JSON result to check the output
        let result_value: serde_json::Value = serde_json::from_str(&result).unwrap();
        let echo_output = result_value.get("echoResult").unwrap().as_str().unwrap();

        assert_eq!(echo_output, "hello world");
    }

    #[tokio::test]
    async fn test_substitution_pipeline() {
        let pipeline_tool = PipelineTool;

        // The `commands` parameters for ShellTool must be stringified JSON
        let first_commands_string_json = json!(
            [
                {
                    "command": "echo",
                    "args": ["hello"]
                }
            ]
        )
        .to_string();

        let second_commands_json_string = json!(
            [
                {
                    "command": "echo",
                    // The substitution placeholder is used here,
                    // expecting to be replaced with output from first step
                    "args": ["${firstEcho} world"]
                }
            ]
        )
        .to_string();

        // Steps that simulate first echo "hello" and then echo result
        let args = json!({
            "steps": json!([
                {
                    "id": "firstEcho",
                    "tool": "shell_tool",
                    "parameters": {
                        "commands": first_commands_string_json,
                    }
                },
                {
                    "id": "secondEcho",
                    "tool": "shell_tool",
                    "parameters": {
                        "commands": second_commands_json_string,
                    }
                },
            ]).to_string(),
        });

        let result = pipeline_tool.execute(args).await.unwrap();

        // Deserialize the JSON result to check the output
        let result_value: JsonValue = serde_json::from_str(&result).unwrap();
        let first_echo_output = result_value.get("firstEcho").unwrap().as_str().unwrap();
        let second_echo_output = result_value.get("secondEcho").unwrap().as_str().unwrap();

        // The echo command on shell typically adds a newline character at the end
        assert_eq!(first_echo_output, "hello");
        // The output from the first command will be the one that gets substituted into the second command call.
        assert_eq!(second_echo_output, "hello world");
    }
}
