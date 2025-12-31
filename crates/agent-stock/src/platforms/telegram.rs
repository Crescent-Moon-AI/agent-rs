//! Telegram bot implementation
//!
//! Simple Telegram bot using the BotInterface

use crate::bot::Command;
use crate::engine::{AnalysisContext, StockAnalysisEngine};
use crate::error::{Result, StockError};
use crate::interface::{
    BotInterface, BotPlatform, BotResponse, Formatter, FormatterFactory, SessionManager,
};
use async_trait::async_trait;

/// Telegram bot configuration
#[derive(Debug, Clone)]
pub struct TelegramConfig {
    /// Bot token from BotFather
    pub token: String,
    
    /// Webhook URL (optional, for webhook mode)
    pub webhook_url: Option<String>,
}

impl TelegramConfig {
    /// Create config from environment variable
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("TELEGRAM_BOT_TOKEN")
            .map_err(|_| StockError::ConfigError("TELEGRAM_BOT_TOKEN not set".to_string()))?;
        
        let webhook_url = std::env::var("TELEGRAM_WEBHOOK_URL").ok();
        
        Ok(Self { token, webhook_url })
    }
}

/// Telegram bot
pub struct TelegramBot {
    config: TelegramConfig,
    engine: StockAnalysisEngine,
    session_manager: SessionManager,
    formatter: Box<dyn Formatter>,
}

impl TelegramBot {
    /// Create a new Telegram bot
    pub fn new(config: TelegramConfig, engine: StockAnalysisEngine) -> Self {
        Self {
            config,
            engine,
            session_manager: SessionManager::new(BotPlatform::Telegram),
            formatter: FormatterFactory::create(BotPlatform::Telegram),
        }
    }
    
    /// Process a command from a user
    pub async fn process_command(&mut self, user_id: &str, input: &str) -> Result<String> {
        let mut session = self.session_manager.get_or_create(user_id)?;
        let mut context = session.context.clone();
        
        let command = Command::parse(input)?;
        
        let response = match command {
            Command::Analyze { symbol } => {
                let result = self.engine.analyze_stock(&symbol, &mut context).await?;
                self.formatter.format_analysis(&result, &context)
            }
            Command::Technical { symbol } => {
                let result = self.engine.analyze_technical(&symbol, &mut context).await?;
                self.formatter.format_analysis(&result, &context)
            }
            Command::Fundamental { symbol } => {
                let result = self.engine.analyze_fundamental(&symbol, &mut context).await?;
                self.formatter.format_analysis(&result, &context)
            }
            Command::News { symbol } => {
                let result = self.engine.analyze_news(&symbol, &mut context).await?;
                self.formatter.format_analysis(&result, &context)
            }
            Command::Earnings { symbol } => {
                let result = self.engine.analyze_earnings(&symbol, &mut context).await?;
                self.formatter.format_analysis(&result, &context)
            }
            Command::Macro => {
                let result = self.engine.analyze_macro(&mut context).await?;
                self.formatter.format_analysis(&result, &context)
            }
            Command::Compare { symbols } => {
                let result = self.engine.compare_stocks(&symbols, &mut context).await?;
                result.summary
            }
            Command::Watch { symbol } => {
                session.watch(symbol.clone());
                format!("✅ Added {symbol} to watchlist")
            }
            Command::Unwatch { symbol } => {
                if session.unwatch(&symbol) {
                    format!("✅ Removed {symbol} from watchlist")
                } else {
                    format!("❌ {symbol} not in watchlist")
                }
            }
            Command::Watchlist => {
                if session.watchlist.is_empty() {
                    "📋 Watchlist is empty".to_string()
                } else {
                    format!("📋 Watchlist:\n{}", session.watchlist.join("\n"))
                }
            }
            Command::Help => self.formatter.format_help(),
            Command::Clear => {
                session.context = AnalysisContext::with_user(user_id);
                "✅ Conversation cleared".to_string()
            }
            _ => "Command not yet implemented".to_string(),
        };
        
        session.context = context;
        self.session_manager.update(user_id, session)?;
        
        Ok(response)
    }
    
    /// Get bot token
    pub fn token(&self) -> &str {
        &self.config.token
    }
}

#[async_trait]
impl BotInterface for TelegramBot {
    fn platform(&self) -> BotPlatform {
        BotPlatform::Telegram
    }
    
    async fn on_message(
        &mut self,
        user_id: &str,
        message: &str,
        _context: &mut AnalysisContext,
    ) -> Result<BotResponse> {
        let response = self.process_command(user_id, message).await?;
        Ok(BotResponse::formatted(response))
    }
    
    async fn on_command(
        &mut self,
        user_id: &str,
        command: &str,
        args: &[String],
        context: &mut AnalysisContext,
    ) -> Result<BotResponse> {
        let full_command = if args.is_empty() {
            format!("/{command}")
        } else {
            format!("/{} {}", command, args.join(" "))
        };
        
        self.on_message(user_id, &full_command, context).await
    }
    
    fn format_response(&self, content: &str, _context: &AnalysisContext) -> BotResponse {
        BotResponse::formatted(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_telegram_config_from_env() {
        std::env::set_var("TELEGRAM_BOT_TOKEN", "test_token");
        let config = TelegramConfig::from_env().unwrap();
        assert_eq!(config.token, "test_token");
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
    }
}
