# agent-runtime

Agent runtime for agent-rs - executor, dependency injection, and agent lifecycle management.

[![Crates.io](https://img.shields.io/crates/v/agent-runtime.svg)](https://crates.io/crates/agent-runtime)
[![Documentation](https://docs.rs/agent-runtime/badge.svg)](https://docs.rs/agent-runtime)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-runtime = "0.0.1-alpha.1"
```

## Features

- `AgentRuntime` - main runtime for executing agents
- `AgentExecutor` - execute agents with tool support
- `SimpleAgent` - basic agent implementation
- `DelegatingAgent` - agent that delegates to sub-agents
- Event handlers for monitoring agent execution

## Usage

```rust
use agent_runtime::{AgentRuntime, AgentRuntimeBuilder, SimpleAgent};

async fn example() -> anyhow::Result<()> {
    let runtime = AgentRuntimeBuilder::new()
        .build()
        .await?;

    let agent = SimpleAgent::new(/* ... */);
    let result = runtime.execute(&agent, "Hello!").await?;

    Ok(())
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
