use crate::models::{traits::Tool, types::AppError};

use async_trait::async_trait;
use schemars::schema::RootSchema;
use schemars::{schema_for, JsonSchema};
use serde_derive::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use tokio::fs::{self, File};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Serialize, Deserialize, JsonSchema)]
struct FileOperation {
    op: FileOpType,
    file_path: String,
    content: Option<String>, // For create, insert, and update operations
    line: Option<usize>,     // For line-specific operations
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum FileOpType {
    Create,
    Delete,
    InsertLine,
    DeleteLine,
    UpdateLine,
}

#[derive(JsonSchema, Deserialize)]
struct FileToolInput {
    operations: Vec<FileOperation>,
}

pub struct FileTool;

#[async_trait]
impl Tool for FileTool {
    fn name(&self) -> &'static str {
        "file_tool"
    }

    fn description(&self) -> &'static str {
        "Performs file operations such as create, delete, and update on files."
    }

    fn parameters(&self) -> JsonValue {
        // Define and return the JSON parameters schema for file operations
        json!({
            "type": "object",
            "properties": {
                "operations": {
                    "type": "array",
                    "description": "The JSON-encoded list of file operations to execute",
                    "items": {
                        "type": "object",
                        "properties": {
                            "op": {
                                "type": "string",
                                "enum": ["create", "delete", "insertline", "deleteline", "updateline"],
                                "description": "type of file operation"
                            },
                            "file_path": {
                                "type": "string",
                                "description": "path of file to operate upon"
                            },
                            "content": {
                                "type": "string",
                                "description": "new file contents"
                            },
                            "line": {
                                "type": "integer",
                                "description": "line number for insertline, updateline, and deleteline operations"
                            }
                        },
                        "required": ["op", "file_path"]
                    }
                }
            }
        })
    }

    async fn execute(&self, args: JsonValue) -> Result<String, AppError> {
        let input: FileToolInput = serde_json::from_value(args)?;

        for operation in input.operations {
            match operation.op {
                FileOpType::Create => {
                    if let Some(content) = operation.content {
                        create_file(&operation.file_path, &content).await?;
                    } else {
                        return Err(AppError::CommandError(
                            "Missing file content for create operation".to_string(),
                        ));
                    }
                }
                FileOpType::Delete => {
                    delete_file(&operation.file_path).await?;
                }
                FileOpType::InsertLine => {
                    if let (Some(content), Some(line)) = (operation.content, operation.line) {
                        insert_line(&operation.file_path, line, &content).await?;
                    } else {
                        return Err(AppError::CommandError(
                            "Missing line content or line number for insert line operation"
                                .to_string(),
                        ));
                    }
                }
                FileOpType::DeleteLine => {
                    if let Some(line) = operation.line {
                        delete_line(&operation.file_path, line).await?;
                    } else {
                        return Err(AppError::CommandError(
                            "Missing line number for delete line operation".to_string(),
                        ));
                    }
                }
                FileOpType::UpdateLine => {
                    if let (Some(content), Some(line)) = (operation.content, operation.line) {
                        update_line(&operation.file_path, line, &content).await?;
                    } else {
                        return Err(AppError::CommandError(
                            "Missing line content or line number for update line operation"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        Ok("File operations completed successfully.".to_string())
    }

    fn input_schema(&self) -> RootSchema {
        schema_for!(FileToolInput)
    }
}

async fn create_file(file_path: &str, content: &str) -> Result<(), AppError> {
    let mut file = File::create(file_path).await.map_err(AppError::from)?;
    file.write_all(content.as_bytes())
        .await
        .map_err(AppError::from)?;
    Ok(())
}

async fn delete_file(file_path: &str) -> Result<(), AppError> {
    fs::remove_file(file_path).await.map_err(AppError::from)?;
    Ok(())
}

async fn insert_line(
    file_path: &str,
    line_number: usize,
    line_content: &str,
) -> Result<(), AppError> {
    let file = File::open(file_path).await.map_err(AppError::from)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut contents = Vec::new();

    // Read lines and insert new line at the specified line number
    let mut current_line = 1;
    while let Some(line) = lines.next_line().await.map_err(AppError::from)? {
        if current_line == line_number {
            contents.push(format!("{}\n", line_content));
        }
        contents.push(format!("{}\n", line));
        current_line += 1;
    }

    // Handle case when line_number is greater than the total number of lines in the file
    if line_number >= current_line {
        contents.push(format!("{}\n", line_content));
    }

    // Write the modified contents back to the file
    let mut file = File::create(file_path).await.map_err(AppError::from)?;
    file.write_all(contents.concat().as_bytes())
        .await
        .map_err(AppError::from)?;
    Ok(())
}

async fn delete_line(file_path: &str, line_number: usize) -> Result<(), AppError> {
    let file = File::open(file_path).await.map_err(AppError::from)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut contents = Vec::new();

    // Read lines and exclude the line to be deleted
    let mut current_line = 1;
    while let Some(line) = lines.next_line().await.map_err(AppError::from)? {
        if current_line != line_number {
            contents.push(format!("{}\n", line));
        }
        current_line += 1;
    }

    // Write the modified contents back to the file
    let mut file = File::create(file_path).await.map_err(AppError::from)?;
    file.write_all(contents.concat().as_bytes())
        .await
        .map_err(AppError::from)?;
    Ok(())
}

async fn update_line(
    file_path: &str,
    line_number: usize,
    new_content: &str,
) -> Result<(), AppError> {
    let file = File::open(file_path).await.map_err(AppError::from)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut contents = Vec::new();

    // Read lines and update the specified line number
    let mut current_line = 1;
    while let Some(line) = lines.next_line().await.map_err(AppError::from)? {
        if current_line == line_number {
            contents.push(format!("{}\n", new_content));
        } else {
            contents.push(format!("{}\n", line));
        }
        current_line += 1;
    }

    // Write the modified contents back to the file
    let mut file = File::create(file_path).await.map_err(AppError::from)?;
    file.write_all(contents.concat().as_bytes())
        .await
        .map_err(AppError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tokio::fs;

    // Sets up the testing environment and ensures any test data is cleaned up.
    async fn setup_test_environment(test_file: &str) {
        // Clean up before tests to ensure a fresh state
        let _ = fs::remove_file(test_file).await;
    }

    #[tokio::test]
    async fn test_create_file() {
        let test_file = "test_create.txt";
        setup_test_environment(test_file).await;

        create_file(test_file, "New file content").await.unwrap();
        let content = fs::read_to_string(test_file).await.unwrap();
        assert_eq!(content, "New file content");

        // Cleanup
        let _ = fs::remove_file(test_file).await;
    }

    #[tokio::test]
    async fn test_delete_file() {
        let test_file = "test_delete.txt";
        create_file(test_file, "To be deleted").await.unwrap();

        delete_file(test_file).await.unwrap();
        let exists = Path::new(test_file).exists();
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_insert_line() {
        let test_file = "test_insert_line.txt";
        create_file(test_file, "First line\nSecond line")
            .await
            .unwrap();

        insert_line(test_file, 2, "Inserted line").await.unwrap();
        let content = fs::read_to_string(test_file).await.unwrap();
        assert_eq!(content, "First line\nInserted line\nSecond line\n");

        // Cleanup
        let _ = fs::remove_file(test_file).await;
    }

    #[tokio::test]
    async fn test_delete_line() {
        let test_file = "test_delete_line.txt";
        create_file(test_file, "First line\nSecond line\nThird line")
            .await
            .unwrap();

        delete_line(test_file, 2).await.unwrap();
        let content = fs::read_to_string(test_file).await.unwrap();
        assert_eq!(content, "First line\nThird line\n");

        // Cleanup
        let _ = fs::remove_file(test_file).await;
    }

    #[tokio::test]
    async fn test_update_line() {
        let test_file = "test_update_line.txt";
        create_file(test_file, "First line\nSecond line")
            .await
            .unwrap();

        update_line(test_file, 2, "Updated line").await.unwrap();
        let content = fs::read_to_string(test_file).await.unwrap();
        assert_eq!(content, "First line\nUpdated line\n");

        // Cleanup
        let _ = fs::remove_file(test_file).await;
    }

    #[test]
    fn test_file_input_schema() {
        let file_tool = FileTool;
        let _schema = file_tool.input_schema();
        dbg!(_schema);
    }
}
