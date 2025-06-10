//! Simple test infrastructure for basic testing
//! 
//! This provides a minimal test setup that can work without complex dependencies

use std::sync::Once;

static INIT: Once = Once::new();

/// Initialize test environment
pub fn init_test_env() {
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt::try_init();
    });
}

/// Simple test configuration
#[derive(Debug, Clone)]
pub struct SimpleTestConfig {
    pub mock_telegram: bool,
    pub use_temp_files: bool,
}

impl Default for SimpleTestConfig {
    fn default() -> Self {
        Self {
            mock_telegram: true,
            use_temp_files: true,
        }
    }
}

/// Simple test context for basic testing
pub struct SimpleTestContext {
    pub config: SimpleTestConfig,
    pub temp_dir: Option<tempfile::TempDir>,
}

impl SimpleTestContext {
    /// Create a new simple test context
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        init_test_env();
        
        let config = SimpleTestConfig::default();
        let temp_dir = if config.use_temp_files {
            Some(tempfile::tempdir()?)
        } else {
            None
        };

        Ok(Self {
            config,
            temp_dir,
        })
    }

    /// Create with custom config
    pub fn new_with_config(config: SimpleTestConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        init_test_env();
        
        let temp_dir = if config.use_temp_files {
            Some(tempfile::tempdir()?)
        } else {
            None
        };

        Ok(Self {
            config,
            temp_dir,
        })
    }

    /// Get temp directory path
    pub fn temp_path(&self) -> Option<&std::path::Path> {
        self.temp_dir.as_ref().map(|d| d.path())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_context_creation() {
        let ctx = SimpleTestContext::new().expect("Failed to create simple test context");
        assert!(ctx.temp_dir.is_some());
    }

    #[test]
    fn test_simple_context_with_config() {
        let config = SimpleTestConfig {
            mock_telegram: false,
            use_temp_files: false,
        };
        
        let ctx = SimpleTestContext::new_with_config(config).expect("Failed to create test context");
        assert!(ctx.temp_dir.is_none());
        assert!(!ctx.config.mock_telegram);
    }
}