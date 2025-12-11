//! Context extensions for MCP integration
//!
//! This module provides extension traits and utilities for integrating MCP
//! resources with agent-rs Context types.

use crate::resource::{MCPResource, ResourceCache};
use crate::MCPClientManager;
use crate::error::MCPError;
use std::sync::Arc;

type Result<T> = std::result::Result<T, MCPError>;

/// Extension trait for adding MCP resource access to agent contexts
///
/// This trait can be implemented for any context type that needs MCP support.
/// It provides methods for accessing resources from MCP servers.
pub trait MCPContextExt {
    /// Get the MCP resource cache
    fn mcp_cache(&self) -> Option<&ResourceCache>;

    /// Load a resource by URI
    ///
    /// This method fetches the resource from the appropriate MCP server
    /// and caches it for future use.
    async fn load_resource(&self, uri: &str) -> Result<MCPResource> {
        let cache = self
            .mcp_cache()
            .ok_or_else(|| MCPError::NotInitialized("MCP cache not available".to_string()))?;

        cache.get_resource(uri).await
    }

    /// Load multiple resources in parallel
    async fn load_resources(&self, uris: &[String]) -> Result<Vec<MCPResource>> {
        let cache = self
            .mcp_cache()
            .ok_or_else(|| MCPError::NotInitialized("MCP cache not available".to_string()))?;

        cache.prefetch(uris).await
    }

    /// Discover available resources
    async fn discover_resources(&self) -> Result<Vec<crate::client::MCPResourceInfo>> {
        let cache = self
            .mcp_cache()
            .ok_or_else(|| MCPError::NotInitialized("MCP cache not available".to_string()))?;

        cache.discover_resources().await
    }

    /// Clear the resource cache
    async fn clear_resources(&self) -> Result<()> {
        let cache = self
            .mcp_cache()
            .ok_or_else(|| MCPError::NotInitialized("MCP cache not available".to_string()))?;

        cache.clear().await;
        Ok(())
    }
}

/// MCP-enabled context wrapper
///
/// This wrapper adds MCP resource access to any context type.
/// It can be used standalone or composed with existing contexts.
pub struct MCPContext {
    /// Resource cache for this context
    cache: ResourceCache,
}

impl MCPContext {
    /// Create a new MCP context
    pub fn new(manager: Arc<MCPClientManager>) -> Self {
        Self {
            cache: ResourceCache::new(manager),
        }
    }

    /// Get the resource cache
    pub fn cache(&self) -> &ResourceCache {
        &self.cache
    }

    /// Load a resource by URI with error handling
    ///
    /// This is a convenience method that provides additional logging
    /// and error handling compared to direct cache access.
    pub async fn get_resource(&self, uri: &str) -> Result<MCPResource> {
        tracing::info!("Loading MCP resource: {}", uri);

        match self.cache.get_resource(uri).await {
            Ok(resource) => {
                tracing::debug!(
                    "Successfully loaded resource '{}' from server '{}'",
                    uri,
                    resource.server_name
                );
                Ok(resource)
            }
            Err(e) => {
                tracing::warn!("Failed to load resource '{}': {}", uri, e);
                Err(e)
            }
        }
    }

    /// Search for resources matching a pattern
    pub async fn search_resources(&self, pattern: &str) -> Result<Vec<crate::client::MCPResourceInfo>> {
        let all_resources = self.cache.discover_resources().await?;

        // Simple pattern matching
        let matching = all_resources
            .into_iter()
            .filter(|r| {
                r.uri.contains(pattern)
                    || r.name.contains(pattern)
                    || r.description
                        .as_ref()
                        .map(|d| d.contains(pattern))
                        .unwrap_or(false)
            })
            .collect();

        Ok(matching)
    }
}

impl MCPContextExt for MCPContext {
    fn mcp_cache(&self) -> Option<&ResourceCache> {
        Some(&self.cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AgentMCPConfig, MCPConfig};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_mcp_context_creation() {
        let config = Arc::new(MCPConfig::default());
        let manager = Arc::new(MCPClientManager::new(config, "test".to_string()));
        let context = MCPContext::new(manager);

        assert!(context.mcp_cache().is_some());
    }

    #[tokio::test]
    async fn test_mcp_context_ext_trait() {
        let config = Arc::new(MCPConfig::default());
        let manager = Arc::new(MCPClientManager::new(config, "test".to_string()));
        let context = MCPContext::new(manager);

        // Test that trait methods work (will fail with NotInitialized since no servers)
        let result = context.discover_resources().await;
        assert!(result.is_ok()); // Should return empty list
    }

    #[test]
    fn test_context_implements_trait() {
        // Compile-time check that MCPContext implements MCPContextExt
        fn assert_impl<T: MCPContextExt>() {}
        assert_impl::<MCPContext>();
    }
}
