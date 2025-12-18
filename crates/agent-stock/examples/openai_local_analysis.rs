//! Stock analysis example using local LLM via OpenAI-compatible API
//!
//! This example demonstrates how to use the stock analysis agent with a local
//! LLM deployment like LM Studio, llama.cpp, or vLLM through the OpenAI-compatible API.
//!
//! # Features
//!
//! - **Technical Analysis**: RSI, MACD, Bollinger Bands, moving averages
//! - **Fundamental Analysis**: P/E, market cap, revenue, profit margins
//! - **News & Sentiment**: Recent news analysis and market sentiment
//! - **Earnings Analysis**: SEC 10-K/10-Q filings, quarterly earnings (NEW)
//! - **Macro Economic Analysis**: Fed policy, inflation, GDP, employment (NEW)
//! - **Geopolitical Analysis**: Trade tensions, sanctions, global risks (NEW)
//!
//! # Configuration
//!
//! Set environment variables:
//! ```bash
//! # Windows PowerShell
//! $env:OPENAI_API_BASE="http://localhost:1234/v1"
//! $env:OPENAI_MODEL="your-model-name"
//! $env:STOCK_RESPONSE_LANGUAGE="chinese"  # or "english", default is chinese
//! # Optional API Keys:
//! $env:ALPHA_VANTAGE_API_KEY="your-key-here"  # For fundamental data
//! $env:FRED_API_KEY="your-key-here"           # For macro economic data (NEW)
//!
//! # Linux/Mac
//! export OPENAI_API_BASE="http://localhost:1234/v1"
//! export OPENAI_MODEL="your-model-name"
//! export STOCK_RESPONSE_LANGUAGE="chinese"  # or "english", default is chinese
//! # Optional API Keys:
//! export ALPHA_VANTAGE_API_KEY="your-key-here"  # For fundamental data
//! export FRED_API_KEY="your-key-here"           # For macro economic data (NEW)
//! ```
//!
//! # Usage
//!
//! ```bash
//! # Basic analysis
//! cargo run --example openai_local_analysis -p agent-stock AAPL
//!
//! # With all features (comprehensive)
//! cargo run --example openai_local_analysis -p agent-stock AAPL --full
//!
//! # Macro economic analysis only
//! cargo run --example openai_local_analysis -p agent-stock -- --macro
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
    println!("  - Cache TTL (earnings): {:?}", stock_config.cache_ttl_earnings);
    println!("  - Cache TTL (macro): {:?}", stock_config.cache_ttl_macro);
    println!("  - Max retries: {}\n", stock_config.max_retries);

    // Check for optional API keys
    if env::var("FRED_API_KEY").is_ok() {
        println!("  - FRED API Key: Configured (macro data available)");
    }
    if env::var("ALPHA_VANTAGE_API_KEY").is_ok() {
        println!("  - Alpha Vantage API Key: Configured");
    }
    println!();

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

    // Example 5: Earnings Analysis (NEW)
    println!("=== 5. Earnings & Financial Reports ===");
    println!("Analyzing SEC filings and earnings data for {}...", symbol);
    match agent.analyze_earnings(symbol).await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 6: Macro Economic Analysis (NEW)
    println!("=== 6. Macro Economic Environment ===");
    println!("Analyzing Fed policy, inflation, and economic indicators...");
    match agent.analyze_macro().await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 7: Geopolitical Analysis (NEW)
    println!("=== 7. Geopolitical Risks ===");
    println!("Analyzing trade tensions, sanctions, and global risks...");
    match agent.analyze_geopolitical().await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 8: Comprehensive Analysis (NEW)
    println!("=== 8. Comprehensive Investment Analysis ===");
    println!("Synthesizing all analysis for {}...", symbol);
    match agent.analyze_comprehensive(symbol).await {
        Ok(result) => println!("{}\n", result),
        Err(e) => println!("Error: {}\n", e),
    }

    // Example 9: Custom Query
    println!("=== 9. Custom Query ===");
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
    println!("\nAnalysis Categories:");
    println!("  1. Current Price - Real-time quote data");
    println!("  2. Technical - RSI, MACD, moving averages");
    println!("  3. Fundamental - P/E, market cap, financials");
    println!("  4. News - Recent news and sentiment");
    println!("  5. Earnings - SEC filings, quarterly reports");
    println!("  6. Macro - Fed policy, inflation, GDP");
    println!("  7. Geopolitical - Trade wars, sanctions, risks");
    println!("  8. Comprehensive - Full synthesis");
    println!("\nNote: This example uses a local LLM via OpenAI-compatible API.");
    println!("Configure via OPENAI_API_BASE and OPENAI_MODEL environment variables.");

    Ok(())
}
