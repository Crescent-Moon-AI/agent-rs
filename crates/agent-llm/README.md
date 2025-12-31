# agent-llm

LLM provider abstraction layer for agent-rs - supports Anthropic, OpenAI, and Ollama.

[![Crates.io](https://img.shields.io/crates/v/agent-llm.svg)](https://crates.io/crates/agent-llm)
[![Documentation](https://docs.rs/agent-llm/badge.svg)](https://docs.rs/agent-llm)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Part of the [agent-rs](https://github.com/Crescent-Moon-AI/agent-rs) framework.

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
agent-llm = { version = "0.0.1-alpha.1", features = ["anthropic"] }
```

## Features

- `anthropic` - Anthropic Claude API support
- `openai` - OpenAI API support
- `ollama` - Ollama local LLM support

## Usage

```rust
use agent_llm::{LLMProvider, CompletionRequest, Message, Role};

#[cfg(feature = "anthropic")]
use agent_llm::providers::AnthropicProvider;

async fn example() -> anyhow::Result<()> {
    let provider = AnthropicProvider::new("your-api-key")?;

    let request = CompletionRequest::builder()
        .model("claude-3-sonnet-20240229")
        .messages(vec![
            Message::new(Role::User, "Hello!")
        ])
        .build();

    let response = provider.complete(request).await?;
    println!("{}", response.content);
    Ok(())
}
```

## License

MIT License - see [LICENSE](../../LICENSE) for details.
