//! CLI Bot - placeholder for now

use crate::engine::StockAnalysisEngine;
use crate::interface::{BotPlatform, SessionManager};

pub struct CliBot {
    _engine: StockAnalysisEngine,
    _session_manager: SessionManager,
}

impl CliBot {
    pub fn new(engine: StockAnalysisEngine) -> Self {
        Self {
            _engine: engine,
            _session_manager: SessionManager::new(BotPlatform::CLI),
        }
    }
}
