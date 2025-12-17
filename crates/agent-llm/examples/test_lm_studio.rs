//! Simple test to verify OpenAI-compatible API connection
//!
//! This example tests the connection to OpenAI-compatible APIs including:
//! - LM Studio
//! - llama.cpp server
//! - vLLM
//! - Ollama
//! - Standard OpenAI API
//!
//! # Usage
//!
//! Using environment variables (recommended):
//! ```bash
//! # Windows PowerShell
//! $env:OPENAI_API_BASE="http://localhost:1234/v1"
//! $env:OPENAI_MODEL="your-model-name"
//! cargo run --example test_lm_studio --features openai -p agent-llm
//!
//! # Linux/Mac
//! export OPENAI_API_BASE="http://localhost:1234/v1"
//! export OPENAI_MODEL="your-model-name"
//! cargo run --example test_lm_studio --features openai -p agent-llm
//! ```
//!
//! Using command line arguments:
//! ```bash
//! cargo run --example test_lm_studio --features openai -p agent-llm -- <api_base> <model>
//! # Example:
//! cargo run --example test_lm_studio --features openai -p agent-llm -- http://localhost:1234/v1 llama-3-8b
//! ```

use agent_llm::{CompletionRequest, LLMProvider, Message};
use agent_llm::providers::{OpenAIConfig, OpenAIProvider};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Testing OpenAI-Compatible API Connection ===\n");

    // Get configuration from environment variables or command line arguments
    let args: Vec<String> = env::args().collect();

    // Get API base URL
    let api_base = if args.len() > 1 {
        // From command line argument
        args[1].clone()
    } else if let Ok(base) = env::var("OPENAI_API_BASE") {
        // From environment variable
        base
    } else {
        // No configuration provided - show help
        println!("Error: API base URL not specified!\n");
        println!("Usage:");
        println!("  1. Set environment variables:");
        println!("     export OPENAI_API_BASE=http://localhost:1234/v1");
        println!("     export OPENAI_MODEL=your-model-name");
        println!("     cargo run --example test_lm_studio --features openai -p agent-llm\n");
        println!("  2. Pass as command line arguments:");
        println!("     cargo run --example test_lm_studio --features openai -p agent-llm -- <api_base> <model>\n");
        println!("Examples:");
        println!("  LM Studio:     http://localhost:1234/v1");
        println!("  llama.cpp:     http://localhost:8080/v1");
        println!("  vLLM:          http://localhost:8000/v1");
        println!("  Ollama:        http://localhost:11434/v1");
        return Err("Configuration required".into());
    };

    // Get model name
    let model = if args.len() > 2 {
        // From command line argument
        args[2].clone()
    } else if let Ok(m) = env::var("OPENAI_MODEL") {
        // From environment variable
        m
    } else {
        // No model specified - show warning and use default
        println!("⚠ Warning: No model specified. Using default: gpt-3.5-turbo");
        println!("  Set OPENAI_MODEL environment variable or pass as second argument.\n");
        "gpt-3.5-turbo".to_string()
    };

    // Get API key (can be anything for local deployments)
    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| {
        println!("ℹ Using default API key (not-needed) for local deployment\n");
        "not-needed".to_string()
    });

    // Configure OpenAI provider
    let config = OpenAIConfig::new(api_key)
        .with_api_base(api_base.clone())
        .with_timeout(180);

    println!("Configuration:");
    println!("  API Base: {}", config.api_base);
    println!("  Model:    {}", model);
    println!("  Timeout:  {}s\n", config.timeout_secs);

    // Create provider
    let provider = OpenAIProvider::with_config(config)?;
    println!("Provider: {}\n", provider.name());

    // Simple test request
    println!("Sending test request...");
    let request = CompletionRequest::builder(model.clone())
        .add_message(Message::user("Hello! Please respond with a brief greeting in 1-2 sentences."))
        .max_tokens(100)
        .temperature(0.7)
        .build();

    match provider.complete(request).await {
        Ok(response) => {
            println!("\n✓ Success!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Response:\n{}", response.message.text().unwrap_or("No text"));
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("\nToken usage:");
            println!("  Input:  {}", response.usage.input_tokens);
            println!("  Output: {}", response.usage.output_tokens);
            println!("  Total:  {}", response.usage.total());
            println!("\nStop reason: {:?}", response.stop_reason);
        }
        Err(e) => {
            println!("\n✗ Error!");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("Failed to connect: {}", e);
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("\nPlease verify:");
            println!("  1. Server is running at: {}", api_base);
            println!("  2. Model is loaded: {}", model);
            println!("  3. API endpoint is accessible");
            println!("  4. Model name is correct");
            println!("\nTroubleshooting:");
            println!("  - Check server logs for errors");
            println!("  - Verify the API base URL is correct");
            println!("  - Test with curl: curl {}/models", api_base.trim_end_matches("/v1"));
            return Err(e.into());
        }
    }

    println!("\n=== Test Complete ===");
    println!("Connection to {} verified successfully!", api_base);
    Ok(())
}
