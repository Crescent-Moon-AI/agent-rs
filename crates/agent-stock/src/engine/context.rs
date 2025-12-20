//! Analysis context management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisContext {
    pub user_id: Option<String>,
    pub current_symbols: Vec<String>,
    pub session_id: String,
    pub conversation_turns: Vec<ConversationTurn>,
    pub cached_results: HashMap<String, CachedData>,
    pub preferences: AnalysisPreferences,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationTurn {
    pub input: String,
    pub response: String,
    pub symbols: Vec<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedData {
    pub value: String,
    pub key: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisPreferences {
    pub language: String,
    pub parallel_execution: bool,
    pub include_macro: bool,
    pub depth: AnalysisDepth,
    pub data_sources: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnalysisDepth {
    Quick,
    Standard,
    Deep,
}

impl Default for AnalysisContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AnalysisContext {
    pub fn new() -> Self {
        let now = Utc::now();
        Self {
            user_id: None,
            current_symbols: Vec::new(),
            session_id: uuid::Uuid::new_v4().to_string(),
            conversation_turns: Vec::new(),
            cached_results: HashMap::new(),
            preferences: AnalysisPreferences::default(),
            created_at: now,
            last_active: now,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_user(user_id: impl Into<String>) -> Self {
        let mut ctx = Self::new();
        ctx.user_id = Some(user_id.into());
        ctx
    }
    
    pub fn set_symbols(&mut self, symbols: Vec<String>) {
        self.current_symbols = symbols;
        self.update_activity();
    }
    
    pub fn add_symbol(&mut self, symbol: impl Into<String>) {
        let symbol = symbol.into();
        if !self.current_symbols.contains(&symbol) {
            self.current_symbols.push(symbol);
        }
        self.update_activity();
    }
    
    pub fn current_symbol(&self) -> Option<&str> {
        self.current_symbols.last().map(|s| s.as_str())
    }
    
    pub fn add_turn(&mut self, input: String, response: String, symbols: Vec<String>) {
        self.conversation_turns.push(ConversationTurn {
            input,
            response,
            symbols,
            timestamp: Utc::now(),
        });
        self.update_activity();
    }
    
    pub fn update_activity(&mut self) {
        self.last_active = Utc::now();
    }
    
    pub fn is_expired(&self, max_age_seconds: i64) -> bool {
        let max_age = chrono::Duration::seconds(max_age_seconds);
        Utc::now() - self.last_active > max_age
    }
}

impl Default for AnalysisPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            parallel_execution: true,
            include_macro: false,
            depth: AnalysisDepth::Standard,
            data_sources: vec!["yahoo".to_string()],
        }
    }
}
