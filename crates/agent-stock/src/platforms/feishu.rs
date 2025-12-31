//! Feishu (Lark) bot implementation

use crate::bot::Command;
use crate::engine::{AnalysisContext, StockAnalysisEngine};
use crate::error::{Result, StockError};
use crate::interface::{
    BotInterface, BotPlatform, BotResponse, Formatter, FormatterFactory, SessionManager,
};
use async_trait::async_trait;

/// Feishu bot configuration
#[derive(Debug, Clone)]
pub struct FeishuConfig {
    /// App ID
    pub app_id: String,
    
    /// App secret
    pub app_secret: String,
    
    /// Verification token (optional)
    pub verification_token: Option<String>,
}

impl FeishuConfig {
    /// Create config from environment variables
    pub fn from_env() -> Result<Self> {
        let app_id = std::env::var("FEISHU_APP_ID")
            .map_err(|_| StockError::ConfigError("FEISHU_APP_ID not set".to_string()))?;
        
        let app_secret = std::env::var("FEISHU_APP_SECRET")
            .map_err(|_| StockError::ConfigError("FEISHU_APP_SECRET not set".to_string()))?;
        
        let verification_token = std::env::var("FEISHU_VERIFICATION_TOKEN").ok();
        
        Ok(Self {
            app_id,
            app_secret,
            verification_token,
        })
    }
}

/// Feishu bot
pub struct FeishuBot {
    _config: FeishuConfig,
    engine: StockAnalysisEngine,
    session_manager: SessionManager,
    formatter: Box<dyn Formatter>,
}

impl FeishuBot {
    /// Create a new Feishu bot
    pub fn new(config: FeishuConfig, engine: StockAnalysisEngine) -> Self {
        Self {
            _config: config,
            engine,
            session_manager: SessionManager::new(BotPlatform::Feishu),
            formatter: FormatterFactory::create(BotPlatform::Feishu),
        }
    }
    
    /// Process a command
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
            Command::Help => self.formatter.format_help(),
            Command::Watchlist => {
                if session.watchlist.is_empty() {
                    "📋 Watchlist is empty".to_string()
                } else {
                    format!("📋 Watchlist:\n{}", session.watchlist.join("\n"))
                }
            }
            _ => "Command not yet implemented".to_string(),
        };
        
        session.context = context;
        self.session_manager.update(user_id, session)?;
        
        Ok(response)
    }
}

#[async_trait]
impl BotInterface for FeishuBot {
    fn platform(&self) -> BotPlatform {
        BotPlatform::Feishu
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
