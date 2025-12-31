//! Stock Analysis Bot
//!
//! This module provides a conversational bot interface for stock analysis.
//!
//! # Features
//!
//! - **Command-based interface**: Use commands like `/analyze AAPL`
//! - **Natural language**: Ask questions in natural language
//! - **Conversation context**: Follow-up questions are handled intelligently
//! - **Watchlist**: Track stocks of interest
//!
//! # Example
//!
//! ```rust,ignore
//! use agent_stock::bot::{StockBot, BotConfig};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = BotConfig::from_env()?;
//!     let bot = StockBot::new(config).await?;
//!     bot.run_repl().await
//! }
//! ```

pub mod commands;
pub mod conversation;

use crate::agents::StockAnalysisAgent;
use crate::config::StockConfig;
use crate::error::{Result, StockError};
use agent_core::Context;
use agent_llm::LLMProvider;
use agent_runtime::AgentRuntime;
use std::sync::Arc;

pub use commands::Command;
pub use conversation::{ConversationContext, ConversationManager, ConversationTurn};

/// Configuration for the stock bot
#[derive(Debug, Clone)]
pub struct BotConfig {
    /// Stock analysis configuration
    pub stock_config: StockConfig,
    /// Welcome message
    pub welcome_message: String,
    /// Prompt prefix
    pub prompt: String,
    /// Whether to show timestamps
    pub show_timestamps: bool,
    /// Maximum history size
    pub max_history: usize,
}

impl Default for BotConfig {
    fn default() -> Self {
        Self {
            stock_config: StockConfig::default(),
            welcome_message: "Stock Analysis Bot - 输入 /help 查看帮助".to_string(),
            prompt: ">>> ".to_string(),
            show_timestamps: false,
            max_history: 50,
        }
    }
}

impl BotConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        let stock_config = StockConfig::builder()
            .with_env_api_key()
            .with_env_finnhub_key()
            .with_env_fred_key()
            .with_env_news_provider()
            .from_env_model()
            .build()?;

        Ok(Self {
            stock_config,
            ..Default::default()
        })
    }

    /// Create a builder
    pub fn builder() -> BotConfigBuilder {
        BotConfigBuilder::default()
    }
}

/// Builder for BotConfig
#[derive(Debug, Default)]
pub struct BotConfigBuilder {
    stock_config: Option<StockConfig>,
    welcome_message: Option<String>,
    prompt: Option<String>,
    show_timestamps: Option<bool>,
    max_history: Option<usize>,
}

impl BotConfigBuilder {
    /// Set stock config
    pub fn stock_config(mut self, config: StockConfig) -> Self {
        self.stock_config = Some(config);
        self
    }

    /// Set welcome message
    pub fn welcome_message(mut self, msg: impl Into<String>) -> Self {
        self.welcome_message = Some(msg.into());
        self
    }

    /// Set prompt
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Set show timestamps
    pub fn show_timestamps(mut self, show: bool) -> Self {
        self.show_timestamps = Some(show);
        self
    }

    /// Set max history
    pub fn max_history(mut self, max: usize) -> Self {
        self.max_history = Some(max);
        self
    }

    /// Build the config
    pub fn build(self) -> BotConfig {
        let defaults = BotConfig::default();
        BotConfig {
            stock_config: self.stock_config.unwrap_or(defaults.stock_config),
            welcome_message: self.welcome_message.unwrap_or(defaults.welcome_message),
            prompt: self.prompt.unwrap_or(defaults.prompt),
            show_timestamps: self.show_timestamps.unwrap_or(defaults.show_timestamps),
            max_history: self.max_history.unwrap_or(defaults.max_history),
        }
    }
}

/// Stock Analysis Bot
pub struct StockBot {
    /// The underlying stock analysis agent
    agent: StockAnalysisAgent,
    /// Conversation manager
    conversation: ConversationManager,
    /// Watchlist
    watchlist: Vec<String>,
    /// Bot configuration
    config: BotConfig,
}

impl StockBot {
    /// Create a new stock bot with the given provider
    pub async fn with_provider(
        provider: Arc<dyn LLMProvider>,
        config: BotConfig,
    ) -> Result<Self> {
        let runtime = AgentRuntime::builder().provider(provider).build()?;
        let runtime = Arc::new(runtime);

        let agent =
            StockAnalysisAgent::new(runtime, Arc::new(config.stock_config.clone())).await?;

        let conversation = ConversationManager::with_max_history(config.max_history);

        Ok(Self {
            agent,
            conversation,
            watchlist: Vec::new(),
            config,
        })
    }

    /// Get the welcome message
    pub fn welcome(&self) -> &str {
        &self.config.welcome_message
    }

    /// Get the prompt
    pub fn prompt(&self) -> &str {
        &self.config.prompt
    }

    /// Process user input and return a response
    pub async fn process_input(&mut self, input: &str) -> Result<String> {
        let command = Command::parse(input)?;
        self.execute_command(command).await
    }

    /// Execute a parsed command
    pub async fn execute_command(&mut self, command: Command) -> Result<String> {
        match command {
            Command::Analyze { symbol } => {
                self.conversation.set_current_symbol(&symbol);
                let result = self.agent.analyze_comprehensive(&symbol).await?;
                self.conversation
                    .add_turn(format!("/analyze {symbol}"), result.clone(), vec![symbol]);
                Ok(result)
            }
            Command::Technical { symbol } => {
                self.conversation.set_current_symbol(&symbol);
                let result = self.agent.analyze_technical(&symbol).await?;
                self.conversation.add_turn(
                    format!("/technical {symbol}"),
                    result.clone(),
                    vec![symbol],
                );
                Ok(result)
            }
            Command::Fundamental { symbol } => {
                self.conversation.set_current_symbol(&symbol);
                let result = self.agent.analyze_fundamental(&symbol).await?;
                self.conversation.add_turn(
                    format!("/fundamental {symbol}"),
                    result.clone(),
                    vec![symbol],
                );
                Ok(result)
            }
            Command::News { symbol } => {
                self.conversation.set_current_symbol(&symbol);
                let result = self.agent.analyze_news(&symbol).await?;
                self.conversation
                    .add_turn(format!("/news {symbol}"), result.clone(), vec![symbol]);
                Ok(result)
            }
            Command::Earnings { symbol } => {
                self.conversation.set_current_symbol(&symbol);
                let result = self.agent.analyze_earnings(&symbol).await?;
                self.conversation.add_turn(
                    format!("/earnings {symbol}"),
                    result.clone(),
                    vec![symbol],
                );
                Ok(result)
            }
            Command::Macro => {
                let result = self.agent.analyze_macro().await?;
                self.conversation
                    .add_turn("/macro".to_string(), result.clone(), vec![]);
                Ok(result)
            }
            Command::Geopolitical => {
                let result = self.agent.analyze_geopolitical().await?;
                self.conversation
                    .add_turn("/geopolitical".to_string(), result.clone(), vec![]);
                Ok(result)
            }
            Command::Compare { symbols } => {
                let result = self.agent.compare_stocks(&symbols).await?;
                self.conversation.add_turn(
                    format!("/compare {}", symbols.join(" ")),
                    result.clone(),
                    symbols,
                );
                Ok(result)
            }
            Command::Watch { symbol } => {
                if self.watchlist.contains(&symbol) {
                    Ok(format!("{symbol} is already in watchlist"))
                } else {
                    self.watchlist.push(symbol.clone());
                    Ok(format!("Added {symbol} to watchlist"))
                }
            }
            Command::Unwatch { symbol } => {
                if let Some(pos) = self.watchlist.iter().position(|s| s == &symbol) {
                    self.watchlist.remove(pos);
                    Ok(format!("Removed {symbol} from watchlist"))
                } else {
                    Ok(format!("{symbol} is not in watchlist"))
                }
            }
            Command::Watchlist => {
                if self.watchlist.is_empty() {
                    Ok("Watchlist is empty. Use /watch <symbol> to add stocks.".to_string())
                } else {
                    Ok(format!("Watchlist:\n  {}", self.watchlist.join("\n  ")))
                }
            }
            Command::Clear => {
                self.conversation.clear();
                Ok("Conversation history cleared.".to_string())
            }
            Command::Help => Ok(Command::help_text().to_string()),
            Command::Exit => Err(StockError::Other("exit".to_string())),
            Command::Query { text } => {
                // Process natural language query
                let resolved = self.conversation.resolve_references(&text);
                let symbols = self.agent.router().extract_symbols(&resolved);

                if let Some(symbol) = symbols.first() {
                    self.conversation.set_current_symbol(symbol);
                }

                let mut context = Context::new();
                let result = self.agent.smart_process(&resolved, &mut context).await?;

                self.conversation.add_turn(text, result.clone(), symbols);
                Ok(result)
            }
        }
    }

    /// Get the watchlist
    pub fn watchlist(&self) -> &[String] {
        &self.watchlist
    }

    /// Get the conversation manager
    pub fn conversation(&self) -> &ConversationManager {
        &self.conversation
    }

    /// Get mutable conversation manager
    pub fn conversation_mut(&mut self) -> &mut ConversationManager {
        &mut self.conversation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bot_config_default() {
        let config = BotConfig::default();
        assert!(!config.welcome_message.is_empty());
        assert_eq!(config.prompt, ">>> ");
    }

    #[test]
    fn test_bot_config_builder() {
        let config = BotConfig::builder()
            .prompt("$ ")
            .show_timestamps(true)
            .max_history(100)
            .build();

        assert_eq!(config.prompt, "$ ");
        assert!(config.show_timestamps);
        assert_eq!(config.max_history, 100);
    }
}
