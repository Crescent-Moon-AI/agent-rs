//! Retry logic with exponential backoff
//!
//! This module provides utilities for retrying operations with configurable
//! retry policies and exponential backoff.

use crate::error::MCPError;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

type Result<T> = std::result::Result<T, MCPError>;

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Initial backoff duration
    pub initial_backoff: Duration,

    /// Maximum backoff duration
    pub max_backoff: Duration,

    /// Backoff multiplier (typically 2.0 for exponential backoff)
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(10),
            backoff_multiplier: 2.0,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(
        max_attempts: u32,
        initial_backoff: Duration,
        max_backoff: Duration,
        backoff_multiplier: f64,
    ) -> Self {
        Self {
            max_attempts,
            initial_backoff,
            max_backoff,
            backoff_multiplier,
        }
    }

    /// Create a policy with no retries
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            initial_backoff: Duration::from_secs(0),
            max_backoff: Duration::from_secs(0),
            backoff_multiplier: 1.0,
        }
    }

    /// Create a policy with fast retries (for testing)
    pub fn fast() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff: Duration::from_millis(10),
            max_backoff: Duration::from_millis(100),
            backoff_multiplier: 2.0,
        }
    }

    /// Calculate backoff duration for a given attempt
    fn backoff_duration(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }

        let backoff_ms = self.initial_backoff.as_millis() as f64
            * self.backoff_multiplier.powi((attempt - 1) as i32);

        let backoff = Duration::from_millis(backoff_ms as u64);

        // Cap at max backoff
        if backoff > self.max_backoff {
            self.max_backoff
        } else {
            backoff
        }
    }

    /// Check if an error is retryable
    fn is_retryable(error: &MCPError) -> bool {
        matches!(
            error,
            MCPError::ConnectionFailed(_)
                | MCPError::RequestFailed(_)
                | MCPError::NotConnected
        )
    }

    /// Execute an async operation with retry logic
    ///
    /// # Arguments
    ///
    /// * `operation_name` - Name of the operation (for logging)
    /// * `operation` - Async operation to execute
    ///
    /// # Returns
    ///
    /// Result of the operation, or the last error if all attempts fail
    pub async fn execute<F, Fut, T>(&self, operation_name: &str, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let mut last_error = None;

        for attempt in 0..self.max_attempts {
            debug!(
                "Attempt {}/{} for operation: {}",
                attempt + 1,
                self.max_attempts,
                operation_name
            );

            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!(
                            "Operation '{}' succeeded after {} retries",
                            operation_name, attempt
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    if !Self::is_retryable(&e) {
                        debug!("Operation '{}' failed with non-retryable error", operation_name);
                        return Err(e);
                    }

                    last_error = Some(e);

                    if attempt + 1 < self.max_attempts {
                        let backoff = self.backoff_duration(attempt + 1);
                        warn!(
                            "Operation '{}' failed (attempt {}/{}): {:?}. Retrying in {:?}",
                            operation_name,
                            attempt + 1,
                            self.max_attempts,
                            last_error,
                            backoff
                        );
                        sleep(backoff).await;
                    }
                }
            }
        }

        // All attempts failed
        let error = last_error.unwrap_or_else(|| {
            MCPError::InternalError("Retry failed with no error".to_string())
        });

        warn!(
            "Operation '{}' failed after {} attempts: {:?}",
            operation_name, self.max_attempts, error
        );

        Err(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[test]
    fn test_default_policy() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.initial_backoff, Duration::from_millis(100));
        assert_eq!(policy.max_backoff, Duration::from_secs(10));
        assert_eq!(policy.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_no_retry_policy() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_attempts, 1);
    }

    #[test]
    fn test_backoff_calculation() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.backoff_duration(0), Duration::from_secs(0));
        assert_eq!(policy.backoff_duration(1), Duration::from_millis(100));
        assert_eq!(policy.backoff_duration(2), Duration::from_millis(200));
        assert_eq!(policy.backoff_duration(3), Duration::from_millis(400));
        assert_eq!(policy.backoff_duration(4), Duration::from_millis(800));
    }

    #[test]
    fn test_backoff_capped_at_max() {
        let policy = RetryPolicy::new(
            10,
            Duration::from_secs(1),
            Duration::from_secs(5),
            2.0,
        );

        // Should be capped at 5 seconds
        assert!(policy.backoff_duration(10) <= Duration::from_secs(5));
    }

    #[test]
    fn test_is_retryable() {
        assert!(RetryPolicy::is_retryable(&MCPError::ConnectionFailed(
            "test".to_string()
        )));
        assert!(RetryPolicy::is_retryable(&MCPError::RequestFailed(
            "test".to_string()
        )));
        assert!(RetryPolicy::is_retryable(&MCPError::NotConnected));

        assert!(!RetryPolicy::is_retryable(&MCPError::ConfigError(
            "test".to_string()
        )));
        assert!(!RetryPolicy::is_retryable(&MCPError::InvalidUri(
            "test".to_string()
        )));
    }

    #[tokio::test]
    async fn test_execute_success_first_try() {
        let policy = RetryPolicy::fast();
        let attempt_count = Arc::new(Mutex::new(0));
        let count = attempt_count.clone();

        let result = policy
            .execute("test_op", || {
                let count = count.clone();
                async move {
                    *count.lock().await += 1;
                    Ok::<i32, MCPError>(42)
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempt_count.lock().await, 1);
    }

    #[tokio::test]
    async fn test_execute_success_after_retry() {
        let policy = RetryPolicy::fast();
        let attempt_count = Arc::new(Mutex::new(0));
        let count = attempt_count.clone();

        let result = policy
            .execute("test_op", || {
                let count = count.clone();
                async move {
                    let mut current = count.lock().await;
                    *current += 1;
                    let val = *current;
                    drop(current);

                    if val < 2 {
                        Err(MCPError::ConnectionFailed("test".to_string()))
                    } else {
                        Ok::<i32, MCPError>(42)
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(*attempt_count.lock().await, 2);
    }

    #[tokio::test]
    async fn test_execute_all_attempts_fail() {
        let policy = RetryPolicy::fast();
        let attempt_count = Arc::new(Mutex::new(0));
        let count = attempt_count.clone();

        let result = policy
            .execute("test_op", || {
                let count = count.clone();
                async move {
                    *count.lock().await += 1;
                    Err::<i32, MCPError>(MCPError::ConnectionFailed("test".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(*attempt_count.lock().await, 3);
    }

    #[tokio::test]
    async fn test_execute_non_retryable_error() {
        let policy = RetryPolicy::fast();
        let attempt_count = Arc::new(Mutex::new(0));
        let count = attempt_count.clone();

        let result = policy
            .execute("test_op", || {
                let count = count.clone();
                async move {
                    *count.lock().await += 1;
                    Err::<i32, MCPError>(MCPError::ConfigError("test".to_string()))
                }
            })
            .await;

        assert!(result.is_err());
        assert_eq!(*attempt_count.lock().await, 1); // Should not retry
    }
}
