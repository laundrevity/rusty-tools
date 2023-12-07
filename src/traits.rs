use crate::types::AppError;

use serde_json::Value as JsonValue;
use async_trait::async_trait;

#[async_trait]
pub trait Tool {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn parameters(&self) -> JsonValue; // JSON object representing parameters
    async fn execute(&self, args: JsonValue) -> Result<String, AppError>; 
}