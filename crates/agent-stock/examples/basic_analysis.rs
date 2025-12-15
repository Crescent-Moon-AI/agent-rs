//! Basic stock analysis example
//!
//! This example demonstrates how to use the stock analysis agent system
//! to analyze stocks.
//!
//! To run this example:
//! ```bash
//! # Set your API keys (Anthropic is required, Alpha Vantage is optional)
//! export ANTHROPIC_API_KEY=your_key_here
//! export ALPHA_VANTAGE_API_KEY=your_key_here  # Optional
//!
//! # Run the example
//! cargo run --example basic_analysis AAPL
//! ```

use agent_llm::providers::AnthropicProvider;
use agent_runtime::AgentRuntime;
use agent_stock::{StockAnalysisAgent, StockConfig};
use std::env;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    // Get stock symbol from command line arguments
    let args: Vec<String> = env::args().collect();
    let symbol = if args.len() > 1 {
        &args[1]
    } else {
        "AAPL" // Default to Apple
    };

    println!("=== Stock Analysis System ===\n");
    println!("Analyzing: {}\n", symbol);

    // Create configuration
    let config = StockConfig::builder()
        .with_env_api_key() // Load Alpha Vantage key from environment
        .build()?;

    println!("Configuration:");
    println!("  - Primary data provider: {:?}", config.default_provider);
    println!("  - Cache TTL (realtime): {:?}", config.cache_ttl_realtime);
    println!("  - Max retries: {}\n", config.max_retries);

    // Create LLM provider (Anthropic)
    let anthropic_key = env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable must be set");

    let llm_provider = Arc::new(AnthropicProvider::new(anthropic_key)?);

    // Create runtime
    let runtime = AgentRuntime::builder()
        .provider(llm_provider)
        .build()?;

    let runtime = Arc::new(runtime);

    // Create stock analysis agent
    println!("Initializing stock analysis agent...");
    let agent = StockAnalysisAgent::new(runtime, Arc::new(config)).await?;
    println!("Agent ready!\n");

    // Example 1: Get current price
    println!("=== 1. Current Price ===");
    let result = agent.process(
        format!("What is the current price of {}?", symbol),
        &mut agent_core::Context::new(),
    ).await?;
    println!("{}\n", result);

    // Example 2: Technical analysis
    println!("=== 2. Technical Analysis ===");
    let result = agent.analyze_technical(symbol).await?;
    println!("{}\n", result);

    // Example 3: Fundamental analysis (requires Alpha Vantage API key)
    if env::var("ALPHA_VANTAGE_API_KEY").is_ok() {
        println!("=== 3. Fundamental Analysis ===");
        let result = agent.analyze_fundamental(symbol).await?;
        println!("{}\n", result);
    } else {
        println!("=== 3. Fundamental Analysis ===");
        println!("Skipped (requires ALPHA_VANTAGE_API_KEY)\n");
    }

    // Example 4: News and sentiment
    println!("=== 4. News & Sentiment ===");
    let result = agent.analyze_news(symbol).await?;
    println!("{}\n", result);

    // Example 5: Comprehensive analysis
    println!("=== 5. Comprehensive Analysis ===");
    let result = agent.analyze(symbol).await?;
    println!("{}\n", result);

    println!("=== Analysis Complete ===");

    Ok(())
}
