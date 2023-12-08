use crate::types::AppError;

use async_trait::async_trait;
use serde_json::Value as JsonValue;

#[async_trait]
pub trait Tool: Sync + Send {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> JsonValue; // JSON object representing parameters
    async fn execute(&self, args: JsonValue) -> Result<String, AppError>;
    fn input_schema(&self) -> String;
}
