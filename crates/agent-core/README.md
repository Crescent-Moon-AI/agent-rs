# agent-core

Core abstractions for building LLM-powered agents - Agent trait, Context, and Error types.

[![Crates.io](https://img.shields.io/crates/v/agent-core.svg)](https://crates.io/crates/agent-core)
[![Documentation](https://docs.rs/agent-core/badge.svg)](https://docs.rs/agent-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-core = "0.0.1-alpha.1"
```

## Features

- `Agent` trait - core abstraction for AI agents
- `Context` - execution context with dependency injection
- `Error` types - comprehensive error handling

## Usage

```rust
use agent_core::{Agent, Context, Result};
use async_trait::async_trait;

struct MyAgent;

#[async_trait]
impl Agent for MyAgent {
    async fn run(&self, ctx: &Context, input: &str) -> Result<String> {
        Ok(format!("Processed: {}", input))
    }
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
