# Rusty Tool Assistant

A modular and extensible console-based assistant leveraging OpenAI's GPT models, structured to dynamically execute an expanding set of tools based on user requests.

## Features

- Dynamic tool execution allows for a flexible extension of available commands
- Utilizes OpenAI's GPT models for an interactive and intelligent command line experience
- A Tool Registry allows for seamless onboarding of new tools without altering core logic
- Interactive console interface for engaging user interaction

## Prerequisites

Before running the Rusty Tool Assistant, ensure you have:

- Rust programming language installed
- Access to OpenAI API with an API key

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/laundrevity/rusty-tools.git
   ```

2. Navigate to the project directory:
   ```
   cd rusty-tools
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
- `-m`, `--model` to specify the OpenAI model (default: `gpt-4-1106-preview`).
- `-l`, `--log-level` to specify the logging level (INFO, DEBUG, TRACE, WARN, ERROR).

## Configuration

Set your OpenAI API key in your environment variables to authenticate the API calls:

```
export OPENAI_API_KEY='your_api_key_here'
```

## Tool Registry

The assistant is built with a tool registry system that dynamically loads and manages all available tools. Each tool is encapsulated into its own module, following the `Tool` trait to define its unique behavior.

### Current Tools

- **ShellTool**: Executes Linux shell commands.
- **SnapTool**: Generates a snapshot of the current project state.

## Development

All tool implementations are modular and can be found within the `src/tools` directory. Developers can contribute by creating new tools or enhancing existing ones in a plug-and-play fashion.

### Key Modules:
- `main.rs`: Entry point for the assistant application.
- `assistant.rs`: Manages the interaction loop with the user and tool execution.
- `tool_registry.rs`: Central management of tool instances.
- `tools/`: Directory containing all the individual tools.
- `types.rs`: Custom types and error handling for application-wide use.
- `utils.rs`: Contains utility functions to aid operations like HTTP requests and terminal I/O.

## Extending the Tool Registry

To add a new tool, create a new module within the `src/tools/` directory implementing the `Tool` trait and register it within the `ToolRegistry` in the Assistant.

## Troubleshooting

Common issues can typically be resolved by verifying your OpenAI API key and internet connection. Consult the error messages provided in the terminal for troubleshooting guidance.

## License

Rusty Tool Assistant is open source, released under the [MIT License](LICENSE).

## Acknowledgments

This project is dependent on OpenAI's services and salutes the broader open-source community for ongoing support and contributions.

## Contributions

We welcome contributions from the community. To contribute, please fork the repository, create a feature branch, and submit a pull request for review.