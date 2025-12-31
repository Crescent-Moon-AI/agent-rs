# agent-tools

Tool framework for agent-rs - define and execute tools for LLM agents.

[![Crates.io](https://img.shields.io/crates/v/agent-tools.svg)](https://crates.io/crates/agent-tools)
[![Documentation](https://docs.rs/agent-tools/badge.svg)](https://docs.rs/agent-tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-tools = "0.0.1-alpha.1"
```

## Features

- `Tool` trait - define custom tools for agents
- `ToolRegistry` - manage and execute tools
- JSON schema generation for tool definitions

## Usage

```rust
use agent_tools::{Tool, ToolRegistry};
use async_trait::async_trait;
use serde_json::Value;

struct CalculatorTool;

#[async_trait]
impl Tool for CalculatorTool {
    fn name(&self) -> &str {
        "calculator"
    }

    fn description(&self) -> &str {
        "Performs basic arithmetic operations"
    }

    async fn execute(&self, input: Value) -> anyhow::Result<Value> {
        // Tool implementation
        Ok(Value::Null)
    }
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
