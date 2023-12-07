use crate::{traits::Tool, types::AppError};
use async_trait::async_trait;
use serde_json::{json, Value as JsonValue, Result as JsonResult};
use serde::de::Error as DeError;
use std::path::Path;
use std::fs;

pub struct SnapTool;


#[async_trait]
impl Tool for SnapTool {
    fn name(&self) ->  &'static str {
        "shell_tool"
    }

    fn description(&self) ->  &'static str {
        "Return the formatted source code of the current project, including Cargo.toml and all .rs files"
    }

    fn parameters(&self) -> JsonValue {
        json!(())
    }

    async fn execute(&self, _args: JsonValue) -> Result<String, AppError> {
        create_project_snapshot().await
    }
}

// Function to create the snapshot
async fn create_project_snapshot() -> Result<String, AppError> {
    let mut snapshot = String::new();

    // Read the Cargo.toml file
    snapshot.push_str("File: Cargo.toml\n");
    snapshot.push_str(&read_file_contents("Cargo.toml").await?);
    snapshot.push_str("\n\n");

    // Read all .rs files in src
    for entry in fs::read_dir("src")? {
        let path = entry?.path();
        if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs") {
            snapshot.push_str(&format!("File: {}\n", path.display()));
            snapshot.push_str(&read_file_contents(&path).await?);
            snapshot.push_str("\n\n");
        }
    }

    // Write snapshot to state.txt
    fs::write("state.txt", &snapshot)
        .map_err(|e| serde_json::Error::custom(format!("Error writing to file: {}", e)))?;

    // Return the created snapshot
    Ok(snapshot)
}

// Helper to read file contents
async fn read_file_contents<P: AsRef<Path>>(path: P) -> JsonResult<String> {
    fs::read_to_string(path)
        .map_err(|e| serde_json::Error::custom(format!("Error reading file: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_get_snapshot() {
        // This test assumes that there is a Cargo.toml file and at least one .rs file in the src directory.
        let result = create_project_snapshot().await;
        assert!(result.is_ok());
        let snapshot = result.unwrap();
        assert!(snapshot.contains("[package]"));
        assert!(snapshot.contains("name = \"rtool\""));
        assert!(snapshot.contains("File: src/types.rs")); // Assuming types.rs will exist
                                                          // Add further assertions based on the specific contents we expect to find in the snapshot
    }
}