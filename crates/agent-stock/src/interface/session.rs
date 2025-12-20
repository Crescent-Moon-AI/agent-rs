//! Session management for bot users

use crate::engine::AnalysisContext;
use crate::error::{Result, StockError};
use crate::interface::BotPlatform;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub user_id: String,
    pub platform: BotPlatform,
    pub context: AnalysisContext,
    pub watchlist: Vec<String>,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub language: String,
    pub notifications: bool,
    pub analysis_depth: String,
    pub custom: HashMap<String, String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            notifications: true,
            analysis_depth: "standard".to_string(),
            custom: HashMap::new(),
        }
    }
}

impl UserSession {
    pub fn new(user_id: impl Into<String>, platform: BotPlatform) -> Self {
        let user_id = user_id.into();
        let mut context = AnalysisContext::new();
        context.user_id = Some(user_id.clone());
        
        let now = Utc::now();
        Self {
            user_id,
            platform,
            context,
            watchlist: Vec::new(),
            preferences: UserPreferences::default(),
            created_at: now,
            last_active: now,
        }
    }
    
    pub fn update_activity(&mut self) {
        self.last_active = Utc::now();
        self.context.update_activity();
    }
    
    pub fn is_expired(&self, max_age_seconds: i64) -> bool {
        let max_age = chrono::Duration::seconds(max_age_seconds);
        Utc::now() - self.last_active > max_age
    }
    
    pub fn watch(&mut self, symbol: impl Into<String>) {
        let symbol = symbol.into();
        if !self.watchlist.contains(&symbol) {
            self.watchlist.push(symbol);
        }
        self.update_activity();
    }
    
    pub fn unwatch(&mut self, symbol: &str) -> bool {
        if let Some(pos) = self.watchlist.iter().position(|s| s == symbol) {
            self.watchlist.remove(pos);
            self.update_activity();
            true
        } else {
            false
        }
    }
    
    pub fn current_symbol(&self) -> Option<&str> {
        self.context.current_symbol()
    }
}

pub trait SessionStorage: Send + Sync {
    fn get(&self, user_id: &str) -> Option<UserSession>;
    fn set(&mut self, user_id: &str, session: UserSession) -> Result<()>;
    fn delete(&mut self, user_id: &str) -> bool;
    fn cleanup_expired(&mut self, max_age_seconds: i64) -> usize;
    fn active_sessions(&self) -> Vec<UserSession>;
}

pub struct InMemoryStorage {
    sessions: Arc<RwLock<HashMap<String, UserSession>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStorage for InMemoryStorage {
    fn get(&self, user_id: &str) -> Option<UserSession> {
        self.sessions.read().ok()?.get(user_id).cloned()
    }
    
    fn set(&mut self, user_id: &str, session: UserSession) -> Result<()> {
        self.sessions
            .write()
            .map_err(|e| StockError::Other(format!("Lock error: {}", e)))?
            .insert(user_id.to_string(), session);
        Ok(())
    }
    
    fn delete(&mut self, user_id: &str) -> bool {
        self.sessions
            .write()
            .ok()
            .and_then(|mut sessions| sessions.remove(user_id))
            .is_some()
    }
    
    fn cleanup_expired(&mut self, max_age_seconds: i64) -> usize {
        let mut sessions = match self.sessions.write() {
            Ok(s) => s,
            Err(_) => return 0,
        };
        
        let initial_count = sessions.len();
        sessions.retain(|_, session| !session.is_expired(max_age_seconds));
        initial_count - sessions.len()
    }
    
    fn active_sessions(&self) -> Vec<UserSession> {
        self.sessions
            .read()
            .ok()
            .map(|sessions| sessions.values().cloned().collect())
            .unwrap_or_default()
    }
}

pub struct SessionManager {
    storage: Box<dyn SessionStorage>,
    default_platform: BotPlatform,
    session_ttl: i64,
}

impl SessionManager {
    pub fn new(platform: BotPlatform) -> Self {
        Self {
            storage: Box::new(InMemoryStorage::new()),
            default_platform: platform,
            session_ttl: 3600,
        }
    }
    
    pub fn with_storage(storage: Box<dyn SessionStorage>, platform: BotPlatform) -> Self {
        Self {
            storage,
            default_platform: platform,
            session_ttl: 3600,
        }
    }
    
    pub fn with_ttl(mut self, ttl_seconds: i64) -> Self {
        self.session_ttl = ttl_seconds;
        self
    }
    
    pub fn get_or_create(&mut self, user_id: &str) -> Result<UserSession> {
        if let Some(mut session) = self.storage.get(user_id) {
            if !session.is_expired(self.session_ttl) {
                session.update_activity();
                self.storage.set(user_id, session.clone())?;
                return Ok(session);
            }
        }
        
        let session = UserSession::new(user_id, self.default_platform);
        self.storage.set(user_id, session.clone())?;
        Ok(session)
    }
    
    pub fn get(&self, user_id: &str) -> Option<UserSession> {
        self.storage.get(user_id)
    }
    
    pub fn update(&mut self, user_id: &str, mut session: UserSession) -> Result<()> {
        session.update_activity();
        self.storage.set(user_id, session)
    }
    
    pub fn delete(&mut self, user_id: &str) -> bool {
        self.storage.delete(user_id)
    }
    
    pub fn cleanup_expired(&mut self) -> usize {
        self.storage.cleanup_expired(self.session_ttl)
    }
    
    pub fn active_count(&self) -> usize {
        self.storage.active_sessions().len()
    }
}
