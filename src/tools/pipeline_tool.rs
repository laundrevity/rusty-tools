use crate::assistant::GLOBAL_TOOL_REGISTRY;
use crate::traits::Tool;
use crate::types::AppError;

use async_trait::async_trait;
use schemars::{schema_for, JsonSchema};
use serde_derive::{Deserialize, Serialize};
use serde_json::{self, json, Value as JsonValue};
#[derive(Serialize, Deserialize, Debug)]
pub struct PipelineTool;

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
struct PipelineStep {
    id: String,
    tool: String,
    parameters: JsonValue,
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
struct PipelineToolInput {
    steps: Vec<PipelineStep>,
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
        "Executes a series of tool calls, passing the output of one as the input to another using substitutions `${priorStepId}`. DO NOT STRINGIFY THE PARAMETERS IN PIPELINE STEPS -- PASS JSON DIRECTLY. Input value for the `steps` key MUST BE JSON, NOT A STRING. The `parameters` value for each step also MUST BE JSON, NOT A STRING."
    }

    fn parameters(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {
                "steps": {
                    "type": "string",
                    "description": "The list of pipeline steps to execute, with possible substitutions. This value MUST be JSON, NOT a string. Each pipeline step must have a string with key `id`, a string with key `tool`, and a JSON value with key `parameters`. The `id` value can be subsequently referenced by other pipeline steps to use the result of a prior step as input. The `tool` value must be one of the known tools (e.g. shell_tool, snap_tool, etc). The `parameters` value should be JSON of arguments to pass to the tool -- NOT a stringified JSON, but just JSON. Do NOT prepend the tool names with `functions.` - the passed tool names should just be like `shell_tool`, `snap_tool`, etc. You can use `${stepId}` to access the value in a later step from the step with id `stepId`."
                }
            }
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, AppError> {
        let input: PipelineToolInput = serde_json::from_value(args)?;

        let mut context = serde_json::Map::new();

        for step in input.steps {
            let params_json = step.parameters; // Get JSON as a string

            // Deserialize JSON string to JsonValue
            // let params_json = serde_json::from_str(raw_params_json)
            //     .map_err(|_| AppError::CommandError("Invalid JSON parameters".into()))?;

            // Substitute placeholders and resolve to a final JsonValue
            let resolved_params =
                Self::resolve_parameters(&params_json, &JsonValue::Object(context.clone()));

            let output = GLOBAL_TOOL_REGISTRY
                .execute_tool(&step.tool, resolved_params)
                .await?;

            context.insert(step.id.clone(), JsonValue::String(output));
        }

        Ok(serde_json::to_string(&JsonValue::Object(context))?)
    }

    fn input_schema(&self) -> String {
        let schema = schema_for!(PipelineToolInput);
        serde_json::to_string(&schema).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_simple_pipeline() {
        let pipeline_tool = PipelineTool;

        let step = PipelineStep {
            id: "echoResult".to_string(),
            tool: "shell_tool".to_string(),
            parameters: json!({
                "commands": [{
                "command": "echo",
                "args": ["hello world"]}]
            }),
        };

        let input = PipelineToolInput { steps: vec![step] };

        let json_input = serde_json::to_value(input).unwrap();

        // Execute pipeline with given input
        let result = pipeline_tool.execute(json_input).await.unwrap();

        // Deserialize the JSON result to check the output
        let result_value: serde_json::Value = serde_json::from_str(&result).unwrap();
        let echo_output = result_value.get("echoResult").unwrap().as_str().unwrap();

        assert_eq!(echo_output, "hello world");
    }

    #[tokio::test]
    async fn test_substitution_pipeline() {
        let pipeline_tool = PipelineTool;

        let first_step = PipelineStep {
            id: "firstEcho".to_string(),
            tool: "shell_tool".to_string(),
            parameters: json!({
                "commands": [
                    {
                        "command": "echo",
                        "args": ["hello"]
                    }
                ]
            }),
        };

        let second_step = PipelineStep {
            id: "secondEcho".to_string(),
            tool: "shell_tool".to_string(),
            parameters: json!({
                "commands": [
                    {
                        "command": "echo",
                        "args": ["${firstEcho} world"]
                    }
                ]
            }),
        };

        let input = PipelineToolInput {
            steps: vec![first_step, second_step],
        };

        let json_input = serde_json::to_value(input).unwrap();

        // Execute pipeline with given input
        let result = pipeline_tool.execute(json_input).await.unwrap();

        // Deserialize the JSON result to check the output
        let result_value: JsonValue = serde_json::from_str(&result).unwrap();
        let first_echo_output = result_value.get("firstEcho").unwrap().as_str().unwrap();
        let second_echo_output = result_value.get("secondEcho").unwrap().as_str().unwrap();

        // The echo command on shell typically adds a newline character at the end
        assert_eq!(first_echo_output, "hello");
        // The output from the first command will be the one that gets substituted into the second command call.
        assert_eq!(second_echo_output, "hello world");
    }

    #[test]
    fn test_pipeline_input_schema() {
        let pipeline_tool = PipelineTool;
        let schema_str = pipeline_tool.input_schema();
        let _schema: schemars::schema::RootSchema = serde_json::from_str(&schema_str).unwrap();
        dbg!(_schema);
    }
}
