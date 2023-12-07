# Rusty Tool Assistant

A versatile console-based assistant powered by an OpenAI model, capable of executing various tool functions to assist users with their queries.

## Features

- Execute a series of Linux commands with concatenated output
- Generate a snapshot of the current project's source code
- Utilize OpenAI's models to understand and process user queries
- Interactive console interface for user inputs and tool executions

## Prerequisites

Before running the Rusty Tool Assistant, ensure you have:

- Rust programming language installed
- Access to OpenAI API with an API key

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/laundrevity/rtool.git
   ```
2. Navigate to the project directory:
   ```
   cd rtool
   ```
3. Build the project using Cargo (Rust's package manager):
   ```
   cargo build --release
   ```

## Usage

To start the assistant, run the following command with the desired initial prompt:

```
cargo run -- "Your initial prompt here"
```

Optional flags:
- `-m`, `--model` to specify the OpenAI model. Default is `gpt-4-1106-preview`.

## Configuration

To use the assistant, you need to set your OpenAI API key in your environment variables:

```
export OPENAI_API_KEY='your_api_key_here'
```

## Tool Functions

### execute_linux_commands

Executes a given list of Linux commands and returns their output. 

#### Example:
```
{
    "commands": "[{\"command\": \"echo\", \"args\": [\"Hello, World!\"]}]"
}
```
### get_snapshot

Generates a snapshot of the current project's source code, including the `Cargo.toml` and `.rs` files in the `src` directory.

## Development

For developers looking to contribute to or modify the assistant, source files are located in the `src` directory.

### Key Modules:
- `main.rs`: Entry point for the assistant application.
- `assistant.rs`: Core logic for interacting with the user and processing requests.
- `tool_function.rs`: Definitions and execution details for tool functions.
- `types.rs`: Defines the data types used throughout the application.
- `utils.rs`: Helper functions for various tasks such as making HTTP requests and printing colored console outputs.

## Troubleshooting

If you encounter any issues, refer to the error messages provided by the assistant. Ensure your OpenAI API key is correctly set and you have a stable internet connection.

## License

The Rusty Tool Assistant is open source and available under the [MIT License](LICENSE).

## Acknowledgments

This project utilizes services provided by [OpenAI](https://openai.com/). We thank the open-source community for their contributions and support.