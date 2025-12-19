//! Stock Analysis Bot CLI
//!
//! An interactive command-line interface for stock analysis.
//!
//! # Usage
//!
//! ```bash
//! # Set up environment variables
//! export OPENAI_API_BASE="http://localhost:1234/v1"
//! export OPENAI_MODEL="your-model-name"
//!
//! # Run the bot
//! cargo run --bin stock-bot -p agent-stock
//! ```

use agent_llm::providers::{OpenAIConfig, OpenAIProvider};
use agent_stock::bot::{BotConfig, StockBot};
use std::env;
use std::io::{self, BufRead, Write};
use std::sync::Arc;

fn print_banner() {
    println!(
        r#"
╔══════════════════════════════════════════════════════════════╗
║                    Stock Analysis Bot                        ║
║                                                              ║
║  Commands:                                                   ║
║    /analyze <symbol>  - 综合分析 (Comprehensive analysis)    ║
║    /technical <symbol> - 技术分析 (Technical)                ║
║    /compare <s1> <s2>  - 比较股票 (Compare stocks)           ║
║    /help              - 显示帮助 (Help)                      ║
║    /exit              - 退出 (Exit)                          ║
║                                                              ║
║  Or ask in natural language:                                 ║
║    "苹果股票最近表现怎么样?"                                 ║
║    "What are AAPL's technical indicators?"                   ║
╚══════════════════════════════════════════════════════════════╝
"#
    );
}

fn get_provider_config() -> (OpenAIConfig, String) {
    let api_base = env::var("OPENAI_API_BASE").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_API_BASE not set, using default");
        "http://localhost:1234/v1".to_string()
    });

    let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| {
        eprintln!("Warning: OPENAI_MODEL not set, using default");
        "gpt-3.5-turbo".to_string()
    });

    let api_key = env::var("OPENAI_API_KEY").unwrap_or_else(|_| "not-needed".to_string());

    let config = OpenAIConfig::new(api_key)
        .with_api_base(api_base)
        .with_timeout(180);

    (config, model)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG").unwrap_or_else(|_| "warn,agent_stock=info".to_string()),
        )
        .init();

    print_banner();

    // Get LLM provider configuration
    let (openai_config, model) = get_provider_config();

    println!("Configuration:");
    println!("  API Base: {}", openai_config.api_base);
    println!("  Model: {}", model);
    println!();

    // Create OpenAI provider
    let provider = Arc::new(OpenAIProvider::with_config(openai_config)?);

    // Create bot configuration
    let bot_config = BotConfig::builder()
        .stock_config(
            agent_stock::StockConfig::builder()
                .with_env_all_keys()
                .from_env_model()
                .model(model)
                .build()?,
        )
        .build();

    // Create the bot
    println!("Initializing stock analysis agent...");
    let mut bot = StockBot::with_provider(provider, bot_config).await?;
    println!("Ready!\n");

    // Run REPL
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    loop {
        // Print prompt
        print!("{}", bot.prompt());
        stdout.flush()?;

        // Read input
        let mut input = String::new();
        match stdin.lock().read_line(&mut input) {
            Ok(0) => {
                // EOF
                println!("\nGoodbye!");
                break;
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error reading input: {}", e);
                continue;
            }
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // Process input
        match bot.process_input(input).await {
            Ok(response) => {
                println!("{}\n", response);
            }
            Err(e) => {
                // Check if it's an exit request
                if e.to_string() == "exit" {
                    println!("Goodbye!");
                    break;
                }
                eprintln!("Error: {}\n", e);
            }
        }
    }

    Ok(())
}
