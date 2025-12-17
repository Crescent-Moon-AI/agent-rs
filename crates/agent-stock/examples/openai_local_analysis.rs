//! Stock analysis example using local LLM via OpenAI-compatible API
//!
//! This example demonstrates how to use the stock analysis agent with a local
//! LLM deployment like LM Studio, llama.cpp, or vLLM through the OpenAI-compatible API.
//!
//! # Configuration
//!
//! Set environment variables:
//! ```bash
//! # Windows PowerShell
//! $env:OPENAI_API_BASE="http://localhost:1234/v1"
//! $env:OPENAI_MODEL="your-model-name"
//! $env:STOCK_RESPONSE_LANGUAGE="chinese"  # or "english", default is chinese
//! # Optional:
//! $env:ALPHA_VANTAGE_API_KEY="your-key-here"
//!
//! # Linux/Mac
//! export OPENAI_API_BASE="http://localhost:1234/v1"
//! export OPENAI_MODEL="your-model-name"
//! export STOCK_RESPONSE_LANGUAGE="chinese"  # or "english", default is chinese
//! # Optional:
//! export ALPHA_VANTAGE_API_KEY="your-key-here"
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example openai_local_analysis -p agent-stock AAPL
//! cargo run --example openai_local_analysis -p agent-stock TSLA
//! ```

use agent_core::Agent;
use agent_llm::providers::{OpenAIConfig, OpenAIProvider};
use agent_runtime::AgentRuntime;
use agent_stock::{StockAnalysisAgent, StockConfig};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("info,agent_stock=debug")
        .init();

    // Get stock symbol from command line arguments
    let args: Vec<String> = env::args().collect();
    let symbol = if args.len() > 1 {
        &args[1]
    } else {
        "AAPL" // Default to Apple
    };

    println!("=== Stock Analysis with Local LLM ===\n");
    println!("Analyzing: {}\n", symbol);

    // Get configuration from environment variables
    let api_base = env::var("OPENAI_API_BASE").unwrap_or_else(|_| {
        eprintln!("⚠ Warning: OPENAI_API_BASE not set!");
        eprintln!("   Using default: http://localhost:1234/v1");
        eprintln!("   Set it for your LM Studio instance:");
        eprintln!("   export OPENAI_API_BASE=http://localhost:1234/v1\n");
        "http://localhost:1234/v1".to_string()
    });

    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| {
        eprintln!("⚠ Warning: OPENAI_MODEL not set!");
        eprintln!("   Using default: gpt-3.5-turbo");
        eprintln!("   Set it to your loaded model:");
        eprintln!("   export OPENAI_MODEL=qwen/qwen3-vl-8b\n");
        "gpt-3.5-turbo".to_string()
    });

    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "not-needed".to_string());

    // Configure OpenAI provider
    let openai_config = OpenAIConfig::new(api_key)
        .with_api_base(api_base.clone())
        .with_timeout(180); // Longer timeout for local models

    println!("LLM Configuration:");
    println!("  Provider:  OpenAI-compatible");
    println!("  API Base:  {}", openai_config.api_base);
    println!("  Model:     {}", model);
    println!("  Timeout:   {}s\n", openai_config.timeout_secs);

    // Create OpenAI provider for local LLM
    let llm_provider = Arc::new(OpenAIProvider::with_config(openai_config)?);

    // Create stock configuration
    let stock_config = StockConfig::builder()
        .with_env_api_key() // Load Alpha Vantage key from environment if available
        .from_env_model() // Load model settings and language from environment
        .model(model.clone()) // Override with the model we're using
        .build()?;

    println!("Stock Data Configuration:");
    println!("  - Primary provider: {:?}", stock_config.default_provider);
    println!("  - Response language: {:?}", stock_config.response_language);
    println!("  - Cache TTL (realtime): {:?}", stock_config.cache_ttl_realtime);
    println!("  - Max retries: {}\n", stock_config.max_retries);

    // Create runtime with local LLM provider
    let runtime = AgentRuntime::builder().provider(llm_provider).build()?;
    let runtime = Arc::new(runtime);

    // Create stock analysis agent
    println!("Initializing stock analysis agent with local LLM...");
    let agent = StockAnalysisAgent::new(runtime, Arc::new(stock_config)).await?;
    println!("Agent ready!\n");

    // Example 1: Get current price
    println!("=== 1. Current Price ===");
    let result = agent
        .process(
            format!("What is the current price of {}?", symbol),
            &mut agent_core::Context::new(),
        )
        .await?;
    println!("{}\n", result);

    // Example 2: Technical analysis
    println!("=== 2. Technical Analysis ===");
    match agent.analyze_technical(symbol).await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 3: Fundamental analysis (requires Alpha Vantage API key)
    if env::var("ALPHA_VANTAGE_API_KEY").is_ok() {
        println!("=== 3. Fundamental Analysis ===");
        match agent.analyze_fundamental(symbol).await {
            Ok(result) => println!("{}\n", result),
            Err(e) => println!("Error: {}\n", e),
        }
    } else {
        println!("=== 3. Fundamental Analysis ===");
        println!("Skipped (requires ALPHA_VANTAGE_API_KEY)\n");
    }

    // Example 4: News and sentiment
    println!("=== 4. News & Sentiment ===");
    match agent.analyze_news(symbol).await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 5: Simple query using local LLM
    println!("=== 5. Custom Query ===");
    let result = agent
        .process(
            format!(
                "Based on the available data, should I buy, sell, or hold {} stock? \
                 Give me a brief recommendation in 2-3 sentences.",
                symbol
            ),
            &mut agent_core::Context::new(),
        )
        .await?;
    println!("{}\n", result);

    println!("=== Analysis Complete ===");
    println!("\nNote: This example uses a local LLM via LM Studio.");
    println!("Model: qwen/qwen3-vl-8b running at http://198.18.0.1:22222");

    Ok(())
}
