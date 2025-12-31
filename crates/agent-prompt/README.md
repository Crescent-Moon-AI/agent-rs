# agent-prompt

Prompt template management for agent-rs - Jinja2 templates with multi-language support.

[![Crates.io](https://img.shields.io/crates/v/agent-prompt.svg)](https://crates.io/crates/agent-prompt)
[![Documentation](https://docs.rs/agent-prompt/badge.svg)](https://docs.rs/agent-prompt)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-prompt = "0.0.1-alpha.1"
```

## Features

- Jinja2-style template rendering with minijinja
- Multi-language prompt support (English, Chinese, etc.)
- Template registry for managing multiple prompts
- Optional file-based template loading

### Optional Features

- `core-integration` - Integration with agent-core types
- `file-loader` - Load templates from filesystem

## Usage

```rust
use agent_prompt::{JinjaTemplate, PromptBuilder};

let template = JinjaTemplate::new("Hello, {{ name }}!")?;
let result = template.render(&serde_json::json!({"name": "World"}))?;
assert_eq!(result, "Hello, World!");
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
