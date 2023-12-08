use crate::models::{traits::Tool, types::AppError};
use async_recursion::async_recursion;
use async_trait::async_trait;
use schemars::schema::RootSchema;
use schemars::{schema_for, JsonSchema};
use serde::de::Error as DeError;
use serde_json::{json, Result as JsonResult, Value as JsonValue};
use std::fs;
use std::path::Path;

pub struct SnapTool;

#[derive(JsonSchema)]
struct SnapToolInput {}

#[async_trait]
impl Tool for SnapTool {
    fn name(&self) -> &'static str {
        "snap_tool"
    }

    fn description(&self) -> &'static str {
        "Return the formatted source code of the current project, including Cargo.toml and all .rs files"
    }

    fn parameters(&self) -> JsonValue {
        json!({
            "type": "object",
            "properties": {}
        })
    }

    async fn execute(&self, _args: JsonValue) -> Result<String, AppError> {
        create_project_snapshot().await
    }

    fn input_schema(&self) -> RootSchema {
        schema_for!(SnapToolInput)
    }
}

// Function to recursively create the snapshot
async fn create_project_snapshot() -> Result<String, AppError> {
    let root_path = Path::new("src");
    let proc_macro_path = Path::new("proc_macro_crate/src");
    let mut snapshot = String::new();

    // Include Cargo.toml at the root of the snapshot
    snapshot.push_str("File: Cargo.toml\n");
    snapshot.push_str(&read_file_contents("Cargo.toml").await?);
    snapshot.push_str("\n\n");

    // Include Cargo.toml in proc_macro_crate
    snapshot.push_str("File: proc_macro_crate/Cargo.toml\n");
    snapshot.push_str(&read_file_contents("proc_macro_crate/Cargo.toml").await?);
    snapshot.push_str("\n\n");

    // Start the recursion from the root_path, which is "src"
    read_directory_contents(&root_path, &mut snapshot).await?;

    read_directory_contents(&proc_macro_path, &mut snapshot).await?;

    // Write snapshot to state.txt
    fs::write("state.txt", &snapshot)
        .map_err(|e| serde_json::Error::custom(format!("Error writing to file: {}", e)))?;

    // Return the created snapshot
    Ok(snapshot)
}

// Helper to read the contents of a directory recursively
#[async_recursion]
async fn read_directory_contents<P: AsRef<Path> + Send>(
    dir_path: P,
    snapshot: &mut String,
) -> Result<(), AppError> {
    let dir_path = dir_path.as_ref();

    // Iterate over the directory
    for entry in fs::read_dir(dir_path).map_err(AppError::IOError)? {
        let entry = entry.map_err(AppError::IOError)?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively call this function for nested directories
            read_directory_contents(&path, snapshot).await?;
        } else if path.is_file() && path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs")
        {
            // Add the contents of Rust source files to the snapshot
            snapshot.push_str(&format!(
                "File: {}\n",
                path.strip_prefix("src/").unwrap_or(&path).display()
            ));
            snapshot.push_str(&read_file_contents(&path).await?);
            snapshot.push_str("\n\n");
        }
    }

    Ok(())
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
