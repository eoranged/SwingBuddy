//! State management module
//! 
//! This module handles conversation state and user context

pub mod context;
pub mod scenarios;
pub mod storage;

// Re-export commonly used state components
pub use context::ConversationContext;
pub use scenarios::{Scenario, ScenarioManager, ScenarioStep, StepValidation, InputType};
pub use storage::{StateStorage, StateStorageManager, StorageStats, ConnectionInfo};