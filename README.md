# Rusty Tool Assistant

This is an advanced toolkit powered by Rust and integrated with AI capabilities using OpenAI's GPT models. It provides a set of tools that can be executed through an AI-powered conversational interface and performs operations ranging from file management to executing shell commands.

## Features

- Asynchronous Rust design for efficient performance.
- Integration with OpenAI's GPT models for AI-assisted operations.
- A growing collection of tools for various automated tasks.
- A dynamic tool registry system for easy addition of new tools.
- Simple conversation-based interface to interact with the assistant.
- Ability to load and continue previous conversations.

## Quick Start

Run the following command to start the assistant with an initial prompt:

```shell
$ cargo run -- 'your initial prompt here'
```

The initial prompt is used to initialize the conversation with the assistant. You can continue to interact with the assistant via the command line.

## Tools

- `file_tool`: Manages file operations like creating, deleting, and updating files.
- `pipeline_tool`: Executes a series of tool calls in a pipeline, passing the output from one as the input to another.
- `snap_tool`: Captures the current state of the project into a formatted snapshot.
- `shell_tool`: Executes Linux shell commands.

Each tool may have its own parameters and expected input format.

## Development

To add a new tool, implement the `Tool` trait for your tool structure and register the tool using the provided macro.

```rust
#[macro_use]
extern crate auto_register_tools;

mod tools {
    // Your tool modules here
}

fn main() {
   // Register tools and start your application logic
}
```

## License

RTool is open-source software licensed under the MIT license.
