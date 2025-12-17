//! MCP resource management and caching
//!
//! This module provides functionality for discovering, reading, and caching
//! resources from MCP servers. Resources are data sources that agents can
//! access, such as files, documents, or database entries.

use crate::MCPClientManager;
use crate::client::{MCPContent, MCPResourceInfo};
use crate::error::MCPError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type Result<T> = std::result::Result<T, MCPError>;

/// A cached resource from an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPResource {
    /// Resource URI (e.g., "file:///path/to/file.txt")
    pub uri: String,

    /// MIME type of the resource
    pub mime_type: Option<String>,

    /// Human-readable description
    pub description: Option<String>,

    /// Resource content
    pub content: Vec<MCPContent>,

    /// Server that provided this resource
    pub server_name: String,
}

impl MCPResource {
    /// Get the text content of the resource, if any
    pub fn get_text(&self) -> Option<String> {
        let mut texts = Vec::new();
        for content in &self.content {
            if let MCPContent::Text { text } = content {
                texts.push(text.clone());
            }
        }

        if texts.is_empty() {
            None
        } else {
            Some(texts.join("\n"))
        }
    }

    /// Convert resource content to JSON
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }

    /// Check if this resource has text content
    pub fn has_text(&self) -> bool {
        self.content
            .iter()
            .any(|c| matches!(c, MCPContent::Text { .. }))
    }

    /// Check if this resource has image content
    pub fn has_image(&self) -> bool {
        self.content
            .iter()
            .any(|c| matches!(c, MCPContent::Image { .. }))
    }
}

/// Resource cache with automatic expiration
pub struct ResourceCache {
    /// Cached resources by URI
    resources: Arc<RwLock<HashMap<String, MCPResource>>>,

    /// MCP client manager for fetching resources
    manager: Arc<MCPClientManager>,
}

impl ResourceCache {
    /// Create a new resource cache
    pub fn new(manager: Arc<MCPClientManager>) -> Self {
        Self {
            resources: Arc::new(RwLock::new(HashMap::new())),
            manager,
        }
    }

    /// Discover all available resources across all configured servers
    pub async fn discover_resources(&self) -> Result<Vec<MCPResourceInfo>> {
        self.manager.discover_resources().await
    }

    /// Get a resource by URI, using cache if available
    ///
    /// If the resource is not in the cache, it will be fetched from the
    /// appropriate MCP server and cached for future use.
    pub async fn get_resource(&self, uri: &str) -> Result<MCPResource> {
        // Check cache first
        {
            let cache = self.resources.read().await;
            if let Some(resource) = cache.get(uri) {
                return Ok(resource.clone());
            }
        }

        // Not in cache, fetch from server
        let resource = self.fetch_and_cache(uri).await?;
        Ok(resource)
    }

    /// Fetch a resource from the appropriate server and cache it
    async fn fetch_and_cache(&self, uri: &str) -> Result<MCPResource> {
        // Discover which server has this resource
        let resources = self.manager.discover_resources().await?;

        let resource_info = resources
            .iter()
            .find(|r| r.uri == uri)
            .ok_or_else(|| MCPError::ResourceNotFound(uri.to_string()))?;

        // Read the resource content
        let content = self
            .manager
            .read_resource(&resource_info.server_name, uri)
            .await?;

        // Create cached resource
        let resource = MCPResource {
            uri: uri.to_string(),
            mime_type: resource_info.mime_type.clone(),
            description: resource_info.description.clone(),
            content,
            server_name: resource_info.server_name.clone(),
        };

        // Cache it
        {
            let mut cache = self.resources.write().await;
            cache.insert(uri.to_string(), resource.clone());
        }

        Ok(resource)
    }

    /// Prefetch multiple resources into the cache
    pub async fn prefetch(&self, uris: &[String]) -> Result<Vec<MCPResource>> {
        let mut resources = Vec::new();

        for uri in uris {
            match self.get_resource(uri).await {
                Ok(resource) => resources.push(resource),
                Err(e) => {
                    tracing::warn!("Failed to prefetch resource '{}': {}", uri, e);
                }
            }
        }

        Ok(resources)
    }

    /// Clear the resource cache
    pub async fn clear(&self) {
        let mut cache = self.resources.write().await;
        cache.clear();
    }

    /// Remove a specific resource from the cache
    pub async fn invalidate(&self, uri: &str) {
        let mut cache = self.resources.write().await;
        cache.remove(uri);
    }

    /// Get the number of cached resources
    pub async fn size(&self) -> usize {
        let cache = self.resources.read().await;
        cache.len()
    }

    /// List all URIs currently in the cache
    pub async fn cached_uris(&self) -> Vec<String> {
        let cache = self.resources.read().await;
        cache.keys().cloned().collect()
    }
}

/// Helper for filtering resources based on patterns
pub struct ResourceFilter {
    /// URI patterns to include (e.g., "file:///*.txt")
    allowed_patterns: Vec<String>,

    /// URI patterns to exclude
    denied_patterns: Vec<String>,
}

impl ResourceFilter {
    /// Create a new resource filter
    pub fn new(allowed: Vec<String>, denied: Vec<String>) -> Self {
        Self {
            allowed_patterns: allowed,
            denied_patterns: denied,
        }
    }

    /// Check if a resource URI should be included
    pub fn should_include(&self, uri: &str) -> bool {
        // Check deny list first
        for pattern in &self.denied_patterns {
            if Self::matches_pattern(uri, pattern) {
                return false;
            }
        }

        // If no allow patterns, allow everything (not in deny list)
        if self.allowed_patterns.is_empty() {
            return true;
        }

        // Check allow list
        for pattern in &self.allowed_patterns {
            if Self::matches_pattern(uri, pattern) {
                return true;
            }
        }

        false
    }

    /// Simple pattern matching with wildcards
    fn matches_pattern(uri: &str, pattern: &str) -> bool {
        if pattern == "*" {
            return true;
        }

        // Convert glob pattern to regex
        let regex_pattern = pattern
            .replace(".", "\\.")
            .replace("*", ".*")
            .replace("?", ".");

        if let Ok(regex) = regex::Regex::new(&format!("^{}$", regex_pattern)) {
            regex.is_match(uri)
        } else {
            uri.contains(pattern)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_filter_allow_all() {
        let filter = ResourceFilter::new(vec!["*".to_string()], vec![]);
        assert!(filter.should_include("file:///test.txt"));
        assert!(filter.should_include("https://example.com/doc"));
    }

    #[test]
    fn test_resource_filter_specific_pattern() {
        let filter = ResourceFilter::new(vec!["file:///*.txt".to_string()], vec![]);
        assert!(filter.should_include("file:///test.txt"));
        assert!(!filter.should_include("file:///test.md"));
    }

    #[test]
    fn test_resource_filter_deny_overrides() {
        let filter =
            ResourceFilter::new(vec!["*".to_string()], vec!["file:///*.secret".to_string()]);
        assert!(filter.should_include("file:///test.txt"));
        assert!(!filter.should_include("file:///password.secret"));
    }

    #[test]
    fn test_resource_get_text() {
        let resource = MCPResource {
            uri: "test://resource".to_string(),
            mime_type: Some("text/plain".to_string()),
            description: None,
            content: vec![
                MCPContent::Text {
                    text: "Hello".to_string(),
                },
                MCPContent::Text {
                    text: "World".to_string(),
                },
            ],
            server_name: "test".to_string(),
        };

        assert_eq!(resource.get_text(), Some("Hello\nWorld".to_string()));
    }

    #[test]
    fn test_resource_has_content_types() {
        let text_resource = MCPResource {
            uri: "test://text".to_string(),
            mime_type: None,
            description: None,
            content: vec![MCPContent::Text {
                text: "test".to_string(),
            }],
            server_name: "test".to_string(),
        };

        assert!(text_resource.has_text());
        assert!(!text_resource.has_image());
    }
}
