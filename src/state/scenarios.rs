//! Conversation scenarios implementation
//! 
//! This module defines the various conversation scenarios that users can go through,
//! including onboarding, group setup, event management, and admin operations.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::utils::errors::{SwingBuddyError, Result};
use super::context::ConversationContext;

/// Represents a conversation scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of the scenario
    pub description: String,
    /// Initial step when starting this scenario
    pub initial_step: String,
    /// All possible steps in this scenario
    pub steps: HashMap<String, ScenarioStep>,
    /// Maximum duration for this scenario (in seconds)
    pub max_duration: Option<u64>,
    /// Whether this scenario can be interrupted
    pub interruptible: bool,
}

/// Represents a step within a scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioStep {
    /// Step identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description of what happens in this step
    pub description: String,
    /// Possible next steps from this step
    pub next_steps: Vec<String>,
    /// Whether this step requires user input
    pub requires_input: bool,
    /// Validation rules for user input
    pub validation: Option<StepValidation>,
    /// Whether this step can be skipped
    pub skippable: bool,
}

/// Validation rules for a scenario step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepValidation {
    /// Input type expected
    pub input_type: InputType,
    /// Minimum length (for text inputs)
    pub min_length: Option<usize>,
    /// Maximum length (for text inputs)
    pub max_length: Option<usize>,
    /// Pattern to match (regex)
    pub pattern: Option<String>,
    /// Custom validation message
    pub error_message: Option<String>,
}

/// Types of input expected in a step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InputType {
    Text,
    Number,
    Date,
    Time,
    Email,
    Phone,
    Location,
    Choice(Vec<String>),
}

/// Scenario manager for handling all conversation scenarios
#[derive(Debug, Clone)]
pub struct ScenarioManager {
    scenarios: HashMap<String, Scenario>,
}

impl ScenarioManager {
    /// Create a new scenario manager with default scenarios
    pub fn new() -> Self {
        let mut manager = Self {
            scenarios: HashMap::new(),
        };
        
        manager.register_default_scenarios();
        manager
    }

    /// Register all default scenarios
    fn register_default_scenarios(&mut self) {
        self.register_scenario(create_onboarding_scenario());
        self.register_scenario(create_group_setup_scenario());
        self.register_scenario(create_event_creation_scenario());
        self.register_scenario(create_admin_panel_scenario());
    }

    /// Register a new scenario
    pub fn register_scenario(&mut self, scenario: Scenario) {
        self.scenarios.insert(scenario.id.clone(), scenario);
    }

    /// Get a scenario by ID
    pub fn get_scenario(&self, id: &str) -> Option<&Scenario> {
        self.scenarios.get(id)
    }

    /// Get all available scenarios
    pub fn get_all_scenarios(&self) -> Vec<&Scenario> {
        self.scenarios.values().collect()
    }

    /// Start a scenario for a user
    pub fn start_scenario(&self, context: &mut ConversationContext, scenario_id: &str) -> Result<()> {
        let scenario = self.get_scenario(scenario_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown scenario: {}", scenario_id)))?;

        context.start_scenario(scenario_id, &scenario.initial_step)?;
        
        // Set scenario-specific expiry if defined
        if let Some(max_duration) = scenario.max_duration {
            let expiry = chrono::Utc::now() + chrono::Duration::seconds(max_duration as i64);
            context.set_expiry(expiry);
        }

        Ok(())
    }

    /// Move to the next step in a scenario
    pub fn next_step(&self, context: &mut ConversationContext, next_step: &str) -> Result<()> {
        let scenario_id = context.scenario.as_ref()
            .ok_or_else(|| SwingBuddyError::InvalidStateTransition {
                from: "no_scenario".to_string(),
                to: next_step.to_string(),
            })?;

        let scenario = self.get_scenario(scenario_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown scenario: {}", scenario_id)))?;

        let current_step_id = context.step.as_ref()
            .ok_or_else(|| SwingBuddyError::InvalidStateTransition {
                from: "no_step".to_string(),
                to: next_step.to_string(),
            })?;

        let current_step = scenario.steps.get(current_step_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown step: {}", current_step_id)))?;

        // Validate that the next step is allowed
        if !current_step.next_steps.contains(&next_step.to_string()) {
            return Err(SwingBuddyError::InvalidStateTransition {
                from: current_step_id.clone(),
                to: next_step.to_string(),
            });
        }

        // Validate that the next step exists
        if !scenario.steps.contains_key(next_step) {
            return Err(SwingBuddyError::InvalidInput(format!("Unknown step: {}", next_step)));
        }

        context.next_step(next_step)?;
        Ok(())
    }

    /// Validate user input for the current step
    pub fn validate_input(&self, context: &ConversationContext, input: &str) -> Result<()> {
        let scenario_id = context.scenario.as_ref()
            .ok_or_else(|| SwingBuddyError::InvalidInput("No active scenario".to_string()))?;

        let step_id = context.step.as_ref()
            .ok_or_else(|| SwingBuddyError::InvalidInput("No active step".to_string()))?;

        let scenario = self.get_scenario(scenario_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown scenario: {}", scenario_id)))?;

        let step = scenario.steps.get(step_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown step: {}", step_id)))?;

        if let Some(validation) = &step.validation {
            self.validate_input_against_rules(input, validation)?;
        }

        Ok(())
    }

    /// Validate input against validation rules
    fn validate_input_against_rules(&self, input: &str, validation: &StepValidation) -> Result<()> {
        // Check length constraints
        if let Some(min_length) = validation.min_length {
            if input.len() < min_length {
                return Err(SwingBuddyError::InvalidInput(
                    validation.error_message.clone()
                        .unwrap_or_else(|| format!("Input too short (minimum {} characters)", min_length))
                ));
            }
        }

        if let Some(max_length) = validation.max_length {
            if input.len() > max_length {
                return Err(SwingBuddyError::InvalidInput(
                    validation.error_message.clone()
                        .unwrap_or_else(|| format!("Input too long (maximum {} characters)", max_length))
                ));
            }
        }

        // Check pattern matching
        if let Some(pattern) = &validation.pattern {
            let regex = regex::Regex::new(pattern)
                .map_err(|_| SwingBuddyError::Config("Invalid regex pattern".to_string()))?;
            
            if !regex.is_match(input) {
                return Err(SwingBuddyError::InvalidInput(
                    validation.error_message.clone()
                        .unwrap_or_else(|| "Input format is invalid".to_string())
                ));
            }
        }

        // Check input type specific validation
        match &validation.input_type {
            InputType::Email => {
                if !input.contains('@') || !input.contains('.') {
                    return Err(SwingBuddyError::InvalidInput("Invalid email format".to_string()));
                }
            }
            InputType::Number => {
                if input.parse::<f64>().is_err() {
                    return Err(SwingBuddyError::InvalidInput("Invalid number format".to_string()));
                }
            }
            InputType::Date => {
                if chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d").is_err() {
                    return Err(SwingBuddyError::InvalidInput("Invalid date format (YYYY-MM-DD)".to_string()));
                }
            }
            InputType::Time => {
                if chrono::NaiveTime::parse_from_str(input, "%H:%M").is_err() {
                    return Err(SwingBuddyError::InvalidInput("Invalid time format (HH:MM)".to_string()));
                }
            }
            InputType::Choice(choices) => {
                if !choices.contains(&input.to_string()) {
                    return Err(SwingBuddyError::InvalidInput(
                        format!("Invalid choice. Available options: {}", choices.join(", "))
                    ));
                }
            }
            _ => {} // No additional validation for Text, Phone, Location
        }

        Ok(())
    }

    /// Check if a scenario can be interrupted
    pub fn can_interrupt(&self, scenario_id: &str) -> bool {
        self.get_scenario(scenario_id)
            .map(|s| s.interruptible)
            .unwrap_or(true)
    }

    /// Get the current step information
    pub fn get_current_step(&self, context: &ConversationContext) -> Result<&ScenarioStep> {
        let scenario_id = context.scenario.as_ref()
            .ok_or_else(|| SwingBuddyError::InvalidInput("No active scenario".to_string()))?;

        let step_id = context.step.as_ref()
            .ok_or_else(|| SwingBuddyError::InvalidInput("No active step".to_string()))?;

        let scenario = self.get_scenario(scenario_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown scenario: {}", scenario_id)))?;

        scenario.steps.get(step_id)
            .ok_or_else(|| SwingBuddyError::InvalidInput(format!("Unknown step: {}", step_id)))
    }
}

/// Create the user onboarding scenario
fn create_onboarding_scenario() -> Scenario {
    let mut steps = HashMap::new();

    steps.insert("language_selection".to_string(), ScenarioStep {
        id: "language_selection".to_string(),
        name: "Language Selection".to_string(),
        description: "User selects their preferred language".to_string(),
        next_steps: vec!["name_input".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Choice(vec!["en".to_string(), "ru".to_string()]),
            min_length: None,
            max_length: None,
            pattern: None,
            error_message: Some("Please select a valid language".to_string()),
        }),
        skippable: false,
    });

    steps.insert("name_input".to_string(), ScenarioStep {
        id: "name_input".to_string(),
        name: "Name Input".to_string(),
        description: "User provides their name".to_string(),
        next_steps: vec!["location_input".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Text,
            min_length: Some(2),
            max_length: Some(50),
            pattern: Some(r"^[a-zA-Zа-яА-Я\s]+$".to_string()),
            error_message: Some("Name should be 2-50 characters, letters and spaces only".to_string()),
        }),
        skippable: false,
    });

    steps.insert("location_input".to_string(), ScenarioStep {
        id: "location_input".to_string(),
        name: "Location Input".to_string(),
        description: "User provides their location".to_string(),
        next_steps: vec!["welcome".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Location,
            min_length: Some(2),
            max_length: Some(100),
            pattern: None,
            error_message: Some("Please provide a valid location".to_string()),
        }),
        skippable: true,
    });

    steps.insert("welcome".to_string(), ScenarioStep {
        id: "welcome".to_string(),
        name: "Welcome".to_string(),
        description: "Show welcome message and complete onboarding".to_string(),
        next_steps: vec![],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    Scenario {
        id: "onboarding".to_string(),
        name: "User Onboarding".to_string(),
        description: "New user onboarding flow".to_string(),
        initial_step: "language_selection".to_string(),
        steps,
        max_duration: Some(3600), // 1 hour
        interruptible: false,
    }
}

/// Create the group setup scenario
fn create_group_setup_scenario() -> Scenario {
    let mut steps = HashMap::new();

    steps.insert("permission_check".to_string(), ScenarioStep {
        id: "permission_check".to_string(),
        name: "Permission Check".to_string(),
        description: "Check bot permissions in the group".to_string(),
        next_steps: vec!["configuration".to_string(), "permission_request".to_string()],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    steps.insert("permission_request".to_string(), ScenarioStep {
        id: "permission_request".to_string(),
        name: "Permission Request".to_string(),
        description: "Request necessary permissions from group admin".to_string(),
        next_steps: vec!["permission_check".to_string()],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    steps.insert("configuration".to_string(), ScenarioStep {
        id: "configuration".to_string(),
        name: "Group Configuration".to_string(),
        description: "Configure group settings".to_string(),
        next_steps: vec!["complete".to_string()],
        requires_input: true,
        validation: None,
        skippable: true,
    });

    steps.insert("complete".to_string(), ScenarioStep {
        id: "complete".to_string(),
        name: "Setup Complete".to_string(),
        description: "Group setup completed successfully".to_string(),
        next_steps: vec![],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    Scenario {
        id: "group_setup".to_string(),
        name: "Group Setup".to_string(),
        description: "Bot setup in a new group".to_string(),
        initial_step: "permission_check".to_string(),
        steps,
        max_duration: Some(1800), // 30 minutes
        interruptible: true,
    }
}

/// Create the event creation scenario
fn create_event_creation_scenario() -> Scenario {
    let mut steps = HashMap::new();

    steps.insert("title_input".to_string(), ScenarioStep {
        id: "title_input".to_string(),
        name: "Event Title".to_string(),
        description: "User provides event title".to_string(),
        next_steps: vec!["description_input".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Text,
            min_length: Some(3),
            max_length: Some(100),
            pattern: None,
            error_message: Some("Event title should be 3-100 characters".to_string()),
        }),
        skippable: false,
    });

    steps.insert("description_input".to_string(), ScenarioStep {
        id: "description_input".to_string(),
        name: "Event Description".to_string(),
        description: "User provides event description".to_string(),
        next_steps: vec!["date_input".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Text,
            min_length: Some(10),
            max_length: Some(500),
            pattern: None,
            error_message: Some("Event description should be 10-500 characters".to_string()),
        }),
        skippable: true,
    });

    steps.insert("date_input".to_string(), ScenarioStep {
        id: "date_input".to_string(),
        name: "Event Date".to_string(),
        description: "User provides event date".to_string(),
        next_steps: vec!["time_input".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Date,
            min_length: None,
            max_length: None,
            pattern: None,
            error_message: Some("Please provide a valid date (YYYY-MM-DD)".to_string()),
        }),
        skippable: false,
    });

    steps.insert("time_input".to_string(), ScenarioStep {
        id: "time_input".to_string(),
        name: "Event Time".to_string(),
        description: "User provides event time".to_string(),
        next_steps: vec!["location_input".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Time,
            min_length: None,
            max_length: None,
            pattern: None,
            error_message: Some("Please provide a valid time (HH:MM)".to_string()),
        }),
        skippable: false,
    });

    steps.insert("location_input".to_string(), ScenarioStep {
        id: "location_input".to_string(),
        name: "Event Location".to_string(),
        description: "User provides event location".to_string(),
        next_steps: vec!["confirmation".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Location,
            min_length: Some(3),
            max_length: Some(200),
            pattern: None,
            error_message: Some("Please provide a valid location".to_string()),
        }),
        skippable: false,
    });

    steps.insert("confirmation".to_string(), ScenarioStep {
        id: "confirmation".to_string(),
        name: "Event Confirmation".to_string(),
        description: "Confirm event creation".to_string(),
        next_steps: vec!["create".to_string(), "cancel".to_string()],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Choice(vec!["confirm".to_string(), "cancel".to_string()]),
            min_length: None,
            max_length: None,
            pattern: None,
            error_message: Some("Please confirm or cancel".to_string()),
        }),
        skippable: false,
    });

    steps.insert("create".to_string(), ScenarioStep {
        id: "create".to_string(),
        name: "Create Event".to_string(),
        description: "Create the event".to_string(),
        next_steps: vec![],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    steps.insert("cancel".to_string(), ScenarioStep {
        id: "cancel".to_string(),
        name: "Cancel Creation".to_string(),
        description: "Cancel event creation".to_string(),
        next_steps: vec![],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    Scenario {
        id: "event_creation".to_string(),
        name: "Event Creation".to_string(),
        description: "Create a new event".to_string(),
        initial_step: "title_input".to_string(),
        steps,
        max_duration: Some(1800), // 30 minutes
        interruptible: true,
    }
}

/// Create the admin panel scenario
fn create_admin_panel_scenario() -> Scenario {
    let mut steps = HashMap::new();

    steps.insert("main_menu".to_string(), ScenarioStep {
        id: "main_menu".to_string(),
        name: "Admin Main Menu".to_string(),
        description: "Show admin panel main menu".to_string(),
        next_steps: vec![
            "user_management".to_string(),
            "group_management".to_string(),
            "event_management".to_string(),
            "system_settings".to_string(),
            "statistics".to_string(),
        ],
        requires_input: true,
        validation: Some(StepValidation {
            input_type: InputType::Choice(vec![
                "users".to_string(),
                "groups".to_string(),
                "events".to_string(),
                "settings".to_string(),
                "stats".to_string(),
            ]),
            min_length: None,
            max_length: None,
            pattern: None,
            error_message: Some("Please select a valid option".to_string()),
        }),
        skippable: false,
    });

    steps.insert("user_management".to_string(), ScenarioStep {
        id: "user_management".to_string(),
        name: "User Management".to_string(),
        description: "Manage users".to_string(),
        next_steps: vec!["main_menu".to_string()],
        requires_input: true,
        validation: None,
        skippable: false,
    });

    steps.insert("group_management".to_string(), ScenarioStep {
        id: "group_management".to_string(),
        name: "Group Management".to_string(),
        description: "Manage groups".to_string(),
        next_steps: vec!["main_menu".to_string()],
        requires_input: true,
        validation: None,
        skippable: false,
    });

    steps.insert("event_management".to_string(), ScenarioStep {
        id: "event_management".to_string(),
        name: "Event Management".to_string(),
        description: "Manage events".to_string(),
        next_steps: vec!["main_menu".to_string()],
        requires_input: true,
        validation: None,
        skippable: false,
    });

    steps.insert("system_settings".to_string(), ScenarioStep {
        id: "system_settings".to_string(),
        name: "System Settings".to_string(),
        description: "Configure system settings".to_string(),
        next_steps: vec!["main_menu".to_string()],
        requires_input: true,
        validation: None,
        skippable: false,
    });

    steps.insert("statistics".to_string(), ScenarioStep {
        id: "statistics".to_string(),
        name: "Statistics".to_string(),
        description: "View system statistics".to_string(),
        next_steps: vec!["main_menu".to_string()],
        requires_input: false,
        validation: None,
        skippable: false,
    });

    Scenario {
        id: "admin_panel".to_string(),
        name: "Admin Panel".to_string(),
        description: "Administrative operations".to_string(),
        initial_step: "main_menu".to_string(),
        steps,
        max_duration: Some(3600), // 1 hour
        interruptible: true,
    }
}

impl Default for ScenarioManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_creation() {
        let manager = ScenarioManager::new();
        
        assert!(manager.get_scenario("onboarding").is_some());
        assert!(manager.get_scenario("group_setup").is_some());
        assert!(manager.get_scenario("event_creation").is_some());
        assert!(manager.get_scenario("admin_panel").is_some());
        assert!(manager.get_scenario("nonexistent").is_none());
    }

    #[test]
    fn test_scenario_flow() {
        let manager = ScenarioManager::new();
        let mut context = ConversationContext::new(123);
        
        // Start onboarding scenario
        manager.start_scenario(&mut context, "onboarding").unwrap();
        assert_eq!(context.scenario, Some("onboarding".to_string()));
        assert_eq!(context.step, Some("language_selection".to_string()));
        
        // Move to next step
        manager.next_step(&mut context, "name_input").unwrap();
        assert_eq!(context.step, Some("name_input".to_string()));
    }

    #[test]
    fn test_input_validation() {
        let manager = ScenarioManager::new();
        let mut context = ConversationContext::new(123);
        
        manager.start_scenario(&mut context, "onboarding").unwrap();
        
        // Valid language selection
        assert!(manager.validate_input(&context, "en").is_ok());
        assert!(manager.validate_input(&context, "ru").is_ok());
        
        // Invalid language selection
        assert!(manager.validate_input(&context, "fr").is_err());
        assert!(manager.validate_input(&context, "invalid").is_err());
    }

    #[test]
    fn test_invalid_transitions() {
        let manager = ScenarioManager::new();
        let mut context = ConversationContext::new(123);
        
        manager.start_scenario(&mut context, "onboarding").unwrap();
        
        // Try to skip to a non-adjacent step
        assert!(manager.next_step(&mut context, "welcome").is_err());
        
        // Try to go to a non-existent step
        assert!(manager.next_step(&mut context, "nonexistent").is_err());
    }
}