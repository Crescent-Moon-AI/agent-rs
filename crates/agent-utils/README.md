# agent-utils

Shared utilities for agent-rs - logging, configuration, and common helpers.

[![Crates.io](https://img.shields.io/crates/v/agent-utils.svg)](https://crates.io/crates/agent-utils)
[![Documentation](https://docs.rs/agent-utils/badge.svg)](https://docs.rs/agent-utils)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-utils = "0.0.1-alpha.1"
```

## Features

- Logging initialization with tracing
- Configuration management
- Common utility functions

## Usage

```rust
use agent_utils::init_tracing;

fn main() {
    init_tracing();
    // Your application code
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
