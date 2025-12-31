//! Conversation management for the stock analysis bot
//!
//! This module provides conversation history tracking and context management
//! for multi-turn interactions with the stock analysis agent.

use chrono::{DateTime, Utc};
use std::collections::VecDeque;

/// Maximum number of conversation turns to keep in history
const MAX_HISTORY_SIZE: usize = 50;

/// A single turn in the conversation
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    /// User's input
    pub user_input: String,
    /// Assistant's response
    pub assistant_response: String,
    /// Stock symbols mentioned in this turn
    pub symbols: Vec<String>,
    /// Timestamp of the turn
    pub timestamp: DateTime<Utc>,
}

impl ConversationTurn {
    /// Create a new conversation turn
    pub fn new(user_input: String, assistant_response: String, symbols: Vec<String>) -> Self {
        Self {
            user_input,
            assistant_response,
            symbols,
            timestamp: Utc::now(),
        }
    }
}

/// Context for the current conversation
#[derive(Debug, Clone, Default)]
pub struct ConversationContext {
    /// Current stock symbol being discussed
    pub current_symbol: Option<String>,
    /// Last analysis type performed
    pub last_analysis_type: Option<String>,
    /// Symbols mentioned in recent conversation
    pub recent_symbols: Vec<String>,
}

/// Manager for conversation history and context
#[derive(Debug)]
pub struct ConversationManager {
    /// Conversation history
    history: VecDeque<ConversationTurn>,
    /// Current conversation context
    context: ConversationContext,
    /// Maximum history size
    max_history: usize,
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ConversationManager {
    /// Create a new conversation manager
    pub fn new() -> Self {
        Self {
            history: VecDeque::with_capacity(MAX_HISTORY_SIZE),
            context: ConversationContext::default(),
            max_history: MAX_HISTORY_SIZE,
        }
    }

    /// Create with custom history size
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            history: VecDeque::with_capacity(max_history),
            context: ConversationContext::default(),
            max_history,
        }
    }

    /// Add a new turn to the conversation
    pub fn add_turn(&mut self, user_input: String, response: String, symbols: Vec<String>) {
        // Update context with any mentioned symbols
        if let Some(symbol) = symbols.first() {
            self.context.current_symbol = Some(symbol.clone());
        }

        for symbol in &symbols {
            if !self.context.recent_symbols.contains(symbol) {
                self.context.recent_symbols.push(symbol.clone());
                // Keep only last 10 symbols
                if self.context.recent_symbols.len() > 10 {
                    self.context.recent_symbols.remove(0);
                }
            }
        }

        // Add to history
        let turn = ConversationTurn::new(user_input, response, symbols);
        self.history.push_back(turn);

        // Trim history if needed
        while self.history.len() > self.max_history {
            self.history.pop_front();
        }
    }

    /// Get the current context
    pub fn context(&self) -> &ConversationContext {
        &self.context
    }

    /// Get mutable context
    pub fn context_mut(&mut self) -> &mut ConversationContext {
        &mut self.context
    }

    /// Get the current symbol being discussed
    pub fn current_symbol(&self) -> Option<&str> {
        self.context.current_symbol.as_deref()
    }

    /// Set the current symbol
    pub fn set_current_symbol(&mut self, symbol: impl Into<String>) {
        self.context.current_symbol = Some(symbol.into());
    }

    /// Get the conversation history
    pub fn history(&self) -> &VecDeque<ConversationTurn> {
        &self.history
    }

    /// Get the last N turns
    pub fn last_turns(&self, n: usize) -> Vec<&ConversationTurn> {
        self.history.iter().rev().take(n).collect()
    }

    /// Check if this is a follow-up question
    ///
    /// Returns true if the query appears to reference previous context
    pub fn is_follow_up(&self, query: &str) -> bool {
        let query_lower = query.to_lowercase();

        // Check for pronouns and references
        let follow_up_indicators = [
            "它",
            "这只",
            "那只",
            "刚才",
            "继续",
            "再",
            "还有",
            "另外",
            "it",
            "this",
            "that",
            "the stock",
            "same",
            "also",
            "continue",
            "more",
            "what about",
            "how about",
        ];

        follow_up_indicators
            .iter()
            .any(|indicator| query_lower.contains(indicator))
    }

    /// Resolve references in a query using conversation context
    ///
    /// If the query references a previous stock (e.g., "它" or "this stock"),
    /// this will return a modified query with the actual symbol.
    pub fn resolve_references(&self, query: &str) -> String {
        if let Some(symbol) = &self.context.current_symbol {
            // Replace common reference patterns with the actual symbol
            let patterns = [
                ("它", symbol.as_str()),
                ("这只股票", symbol.as_str()),
                ("这支股票", symbol.as_str()),
                ("那只股票", symbol.as_str()),
                ("该股", symbol.as_str()),
                ("this stock", symbol.as_str()),
                ("that stock", symbol.as_str()),
                ("the stock", symbol.as_str()),
            ];

            let mut resolved = query.to_string();
            for (pattern, replacement) in patterns {
                resolved = resolved.replace(pattern, replacement);
            }

            // If query doesn't contain any symbol, prepend the current one
            let has_symbol = resolved
                .split_whitespace()
                .any(|word| word.chars().all(|c| c.is_ascii_uppercase()) && word.len() <= 5);

            if !has_symbol && self.is_follow_up(query) {
                resolved = format!("{symbol}: {resolved}");
            }

            resolved
        } else {
            query.to_string()
        }
    }

    /// Clear conversation history
    pub fn clear(&mut self) {
        self.history.clear();
        self.context = ConversationContext::default();
    }

    /// Get number of turns in history
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.history.is_empty()
    }

    /// Format recent history as a string for context
    pub fn format_recent_context(&self, n: usize) -> String {
        let turns: Vec<_> = self.history.iter().rev().take(n).collect();

        if turns.is_empty() {
            return String::new();
        }

        let mut context = String::new();
        context.push_str("Recent conversation:\n");

        for (i, turn) in turns.iter().rev().enumerate() {
            context.push_str(&format!("User {}: {}\n", i + 1, turn.user_input));
            // Truncate long responses
            let response_excerpt: String = turn.assistant_response.chars().take(200).collect();
            context.push_str(&format!("Assistant {}: {}...\n", i + 1, response_excerpt));
        }

        context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_manager() {
        let mut manager = ConversationManager::new();

        manager.add_turn(
            "Analyze AAPL".to_string(),
            "Apple analysis...".to_string(),
            vec!["AAPL".to_string()],
        );

        assert_eq!(manager.len(), 1);
        assert_eq!(manager.current_symbol(), Some("AAPL"));
    }

    #[test]
    fn test_follow_up_detection() {
        let mut manager = ConversationManager::new();
        manager.add_turn(
            "Analyze AAPL".to_string(),
            "Analysis...".to_string(),
            vec!["AAPL".to_string()],
        );

        assert!(manager.is_follow_up("它的技术指标如何?"));
        assert!(manager.is_follow_up("What about this stock's fundamentals?"));
        assert!(!manager.is_follow_up("Analyze GOOGL"));
    }

    #[test]
    fn test_reference_resolution() {
        let mut manager = ConversationManager::new();
        manager.add_turn(
            "Analyze AAPL".to_string(),
            "Analysis...".to_string(),
            vec!["AAPL".to_string()],
        );

        let resolved = manager.resolve_references("这只股票的PE是多少?");
        assert!(resolved.contains("AAPL"));
    }

    #[test]
    fn test_clear() {
        let mut manager = ConversationManager::new();
        manager.add_turn(
            "Test".to_string(),
            "Response".to_string(),
            vec!["TEST".to_string()],
        );

        manager.clear();
        assert!(manager.is_empty());
        assert!(manager.current_symbol().is_none());
    }

    #[test]
    fn test_history_limit() {
        let mut manager = ConversationManager::with_max_history(3);

        for i in 0..5 {
            manager.add_turn(
                format!("Query {}", i),
                format!("Response {}", i),
                vec![],
            );
        }

        assert_eq!(manager.len(), 3);
    }
}
