# agent-rs

> Exploratory Rust workspace for building LLM-powered agents

[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Overview

`agent-rs` is a modular, type-safe framework for building intelligent agents powered by large language models. The project emphasizes flexibility, performance, and clean architectural patterns for exploring agentic AI systems.

## Features

- 🦀 **Pure Rust** - Leveraging Rust's safety guarantees and performance
- 🔌 **Provider Agnostic** - Support for multiple LLM providers (OpenAI, Anthropic, Ollama)
- 🛠️ **Extensible Tools** - Build custom tools with simple trait implementations
- 🔄 **Workflow Orchestration** - Multi-agent coordination and complex workflows
- ⚡ **Async First** - Built on Tokio for high-performance async I/O
- 🎯 **Type Safe** - Compile-time guarantees for correctness
- 📦 **Modular Design** - Well-separated crates for different concerns

## Quick Start

### Prerequisites

- **Rust**: 1.85.0 or later (for Edition 2024 support)
- **Cargo**: Comes with Rust

Install Rust via [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

### Installation

Clone the repository:

```bash
git clone https://github.com/yourusername/agent-rs.git
cd agent-rs
```

Build the workspace:

```bash
cargo build --workspace --all-features
```

### Running Examples

```bash
# Simple agent example (coming soon)
cargo run --example simple_agent

# Multi-agent workflow (coming soon)
cargo run --example multi_agent_workflow

# Custom tool example (coming soon)
cargo run --example custom_tool_example
```

## Architecture

The project is organized as a Cargo workspace with the following crates:

### Core Crates

- **[agent-core](crates/agent-core/)** - Core agent abstractions and runtime
  - `Agent` trait - Main interface for agent implementations
  - `Message` - Communication between agents and users
  - `Context` - Execution context for agents
  - Error handling

- **[agent-llm](crates/agent-llm/)** - LLM provider abstraction layer
  - `LLMProvider` trait - Provider-agnostic interface
  - Request/response types
  - Streaming support (planned)

- **[agent-providers](crates/agent-providers/)** - Concrete provider implementations
  - OpenAI (GPT-4, GPT-3.5) - Planned
  - Anthropic (Claude 3.5) - Planned
  - Ollama (local models) - Planned
  - Feature-gated for flexibility

- **[agent-tools](crates/agent-tools/)** - Tool management framework
  - `Tool` trait - Interface for agent capabilities
  - `ToolRegistry` - Tool discovery and management
  - Built-in tools (planned)

- **[agent-workflow](crates/agent-workflow/)** - Multi-agent orchestration
  - Workflow patterns (sequential, parallel, graph-based)
  - Agent coordination
  - State management

- **[agent-utils](crates/agent-utils/)** - Shared utilities
  - Logging setup (tracing)
  - Configuration management
  - Common utilities

### Development Crates

- **[agent-derive](crates/agent-derive/)** - Procedural macros
  - `#[derive(Agent)]` - Planned
  - Reduce boilerplate

- **[agent-cli](crates/agent-cli/)** - Command-line interface
  - Interactive agent sessions
  - Example implementations

- **[xtask](crates/xtask/)** - Project automation
  - Dependency management
  - Testing and linting
  - Coverage reporting

See [CLAUDE.md](CLAUDE.md) for detailed architecture documentation.

## Usage

### Basic Agent (Conceptual)

```rust
use agent_core::{Agent, Message, Context};
use agent_providers::AnthropicProvider;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create an LLM provider
    let provider = AnthropicProvider::new("your-api-key");

    // Create an agent
    let agent = MyAgent::new(provider);

    // Process a message
    let mut context = Context::new();
    let response = agent.process(
        Message::user("Hello, how can you help me?"),
        &mut context
    ).await?;

    println!("Agent: {}", response.content);

    Ok(())
}
```

### Creating Custom Tools (Conceptual)

```rust
use agent_tools::{Tool, ToolRegistry};
use async_trait::async_trait;
use serde_json::Value;

struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    async fn execute(&self, params: Value) -> agent_core::Result<Value> {
        // Implementation
        Ok(Value::Null)
    }

    fn name(&self) -> &str {
        "web_search"
    }

    fn description(&self) -> &str {
        "Search the web for information"
    }
}
```

## Development

### Building

```bash
# Build all crates
cargo build --workspace --all-features

# Build specific crate
cargo build -p agent-core

# Build release version
cargo build --workspace --all-features --release
```

### Testing

```bash
# Run all tests
cargo test --workspace --all-features

# Run tests for specific crate
cargo test -p agent-core

# Run tests with output
cargo test --workspace --all-features -- --nocapture
```

### Linting

```bash
# Run clippy
cargo clippy --workspace --all-features -- -D warnings

# Check formatting
cargo fmt --all -- --check

# Apply formatting
cargo fmt --all
```

### Documentation

```bash
# Generate and open documentation
cargo doc --workspace --all-features --open

# Generate without opening
cargo doc --workspace --all-features --no-deps
```

## Project Automation

This project uses `cargo-xtask` for automation tasks:

```bash
# Check for outdated dependencies (with web search)
cargo xtask check-deps

# Update dependencies interactively (with version verification)
cargo xtask update-deps

# Run all tests
cargo xtask test

# Run clippy lints
cargo xtask lint

# Generate test coverage report
cargo xtask coverage
```

## Dependency Management

**IMPORTANT**: This project follows a strict dependency management protocol:

Before adding or updating ANY dependency, you MUST:

1. Search for the latest stable version on [crates.io](https://crates.io/)
2. Verify compatibility with Rust 2024 edition
3. Update the version in `workspace.dependencies` in the root `Cargo.toml`
4. Run tests to ensure nothing breaks

Use the `/update-deps` slash command (Claude Code) or `cargo xtask update-deps` to automate this process with built-in version checking.

### Current Dependencies

All dependencies are managed centrally in the workspace `Cargo.toml`:

- **Runtime**: tokio 1.48, async-trait 0.1.89
- **Serialization**: serde 1.0.228, serde_json 1.0
- **Error Handling**: anyhow 1.0, thiserror 2.0
- **HTTP**: reqwest 0.12.24
- **Logging**: tracing 0.1.41, tracing-subscriber 0.3
- **CLI**: clap 4.5.51
- **Testing**: mockall 0.14, tokio-test 0.4

See [Cargo.toml](Cargo.toml) for the complete list.

## Contributing

Contributions are welcome! This is an exploratory project, so feel free to experiment and propose new ideas.

### Development Workflow

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run tests (`cargo test --workspace --all-features`)
5. Run lints (`cargo clippy --workspace --all-features`)
6. Format code (`cargo fmt --all`)
7. Commit changes (use [conventional commits](https://www.conventionalcommits.org/))
8. Push to branch (`git push origin feature/amazing-feature`)
9. Open a Pull Request

### Code Style

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Write rustdoc comments for all public APIs
- Include examples in documentation
- Maintain test coverage above 70%

## Roadmap

- [x] Workspace architecture and initial setup
- [x] Core trait definitions
- [x] Basic project structure
- [ ] Implement concrete LLM providers
  - [ ] Anthropic (Claude 3.5 Sonnet)
  - [ ] OpenAI (GPT-4)
  - [ ] Ollama (local models)
- [ ] Build tool framework
  - [ ] File operations
  - [ ] Web search
  - [ ] Code execution
- [ ] Workflow orchestration
  - [ ] Sequential workflows
  - [ ] Parallel execution
  - [ ] Graph-based workflows
- [ ] CLI interface
  - [ ] Interactive REPL
  - [ ] Configuration management
- [ ] Examples and documentation
  - [ ] Simple agent example
  - [ ] Multi-agent workflow
  - [ ] Custom tool example
  - [ ] Architecture guide
- [ ] Advanced features
  - [ ] Streaming responses
  - [ ] Agent memory/persistence
  - [ ] RAG (Retrieval-Augmented Generation)
  - [ ] Observability and telemetry

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

Inspired by excellent projects in the Rust AI/Agent ecosystem:

- [AutoAgents](https://github.com/liquidos-ai/AutoAgents) - Rust LLM agent framework
- [Kowalski](https://dev.to/yarenty/kowalski-the-rust-native-agentic-ai-framework-53k4) - Rust-native agentic AI
- [Rig](https://rig.rs/) - Rust library for LLM-powered applications

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust Edition 2024](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0.html)
- [Tokio Documentation](https://tokio.rs/)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)

---

**Status**: Early development / Experimental
**Rust Version**: 1.85+ (Edition 2024)
**Last Updated**: 2025-12-09
