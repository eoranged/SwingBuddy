//! Test fixtures and data for integration tests
//!
//! This module provides test data fixtures, helper functions to load test data,
//! and database seeding utilities for complex test scenarios.

use std::collections::HashMap;
// Remove unused imports - these types are not used in fixtures
use chrono::{DateTime, Utc, Duration};

/// Test user fixtures
pub struct UserFixtures {
    pub english_user: TestUser,
    pub russian_user: TestUser,
    pub admin_user: TestUser,
    pub incomplete_user: TestUser,
}

/// Test user data structure
#[derive(Debug, Clone)]
pub struct TestUser {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub language_code: String,
    pub location: Option<String>,
    pub is_admin: bool,
}

impl TestUser {
    pub fn new(telegram_id: i64, first_name: &str) -> Self {
        Self {
            telegram_id,
            username: Some(format!("user_{}", telegram_id)),
            first_name: first_name.to_string(),
            last_name: Some("TestLastName".to_string()),
            language_code: "en".to_string(),
            location: None,
            is_admin: false,
        }
    }

    pub fn with_username(mut self, username: Option<&str>) -> Self {
        self.username = username.map(|s| s.to_string());
        self
    }

    pub fn with_language(mut self, language_code: &str) -> Self {
        self.language_code = language_code.to_string();
        self
    }

    pub fn with_location(mut self, location: Option<&str>) -> Self {
        self.location = location.map(|s| s.to_string());
        self
    }

    pub fn with_admin(mut self, is_admin: bool) -> Self {
        self.is_admin = is_admin;
        self
    }

    pub fn with_last_name(mut self, last_name: Option<&str>) -> Self {
        self.last_name = last_name.map(|s| s.to_string());
        self
    }
}

impl UserFixtures {
    pub fn new() -> Self {
        Self {
            english_user: TestUser::new(100001, "English User")
                .with_language("en")
                .with_location(Some("Moscow")),
            
            russian_user: TestUser::new(100002, "Русский Пользователь")
                .with_language("ru")
                .with_location(Some("Saint Petersburg")),
            
            admin_user: TestUser::new(555666777, "Admin User") // Matches test settings
                .with_language("en")
                .with_location(Some("Moscow"))
                .with_admin(true),
            
            incomplete_user: TestUser::new(100004, "Incomplete User")
                .with_language("en")
                .with_location(None), // No location set
        }
    }

    /// Get all test users as a vector
    pub fn all_users(&self) -> Vec<&TestUser> {
        vec![
            &self.english_user,
            &self.russian_user,
            &self.admin_user,
            &self.incomplete_user,
        ]
    }
}

/// Test event fixtures
pub struct EventFixtures {
    pub upcoming_dance: TestEvent,
    pub workshop: TestEvent,
    pub social_event: TestEvent,
    pub past_event: TestEvent,
}

/// Test event data structure
#[derive(Debug, Clone)]
pub struct TestEvent {
    pub title: String,
    pub description: Option<String>,
    pub event_date: DateTime<Utc>,
    pub location: Option<String>,
    pub max_participants: Option<i32>,
    pub created_by: i64,
}

impl TestEvent {
    pub fn new(title: &str, created_by: i64) -> Self {
        Self {
            title: title.to_string(),
            description: Some(format!("Description for {}", title)),
            event_date: Utc::now() + Duration::days(7), // Default to next week
            location: Some("Test Venue".to_string()),
            max_participants: Some(50),
            created_by,
        }
    }

    pub fn with_description(mut self, description: Option<&str>) -> Self {
        self.description = description.map(|s| s.to_string());
        self
    }

    pub fn with_date(mut self, date: DateTime<Utc>) -> Self {
        self.event_date = date;
        self
    }

    pub fn with_location(mut self, location: Option<&str>) -> Self {
        self.location = location.map(|s| s.to_string());
        self
    }

    pub fn with_max_participants(mut self, max: Option<i32>) -> Self {
        self.max_participants = max;
        self
    }
}

impl EventFixtures {
    pub fn new(admin_user_id: i64) -> Self {
        Self {
            upcoming_dance: TestEvent::new("Swing Dance Night", admin_user_id)
                .with_description(Some("Weekly swing dance social event"))
                .with_date(Utc::now() + Duration::days(3))
                .with_location(Some("Dance Studio A")),
            
            workshop: TestEvent::new("Lindy Hop Workshop", admin_user_id)
                .with_description(Some("Beginner-friendly Lindy Hop workshop"))
                .with_date(Utc::now() + Duration::days(10))
                .with_location(Some("Workshop Room B"))
                .with_max_participants(Some(20)),
            
            social_event: TestEvent::new("Swing Community Meetup", admin_user_id)
                .with_description(Some("Monthly community gathering"))
                .with_date(Utc::now() + Duration::days(14))
                .with_location(Some("Community Center")),
            
            past_event: TestEvent::new("Past Dance Event", admin_user_id)
                .with_description(Some("This event already happened"))
                .with_date(Utc::now() - Duration::days(7))
                .with_location(Some("Old Venue")),
        }
    }

    /// Get all test events as a vector
    pub fn all_events(&self) -> Vec<&TestEvent> {
        vec![
            &self.upcoming_dance,
            &self.workshop,
            &self.social_event,
            &self.past_event,
        ]
    }

    /// Get only upcoming events
    pub fn upcoming_events(&self) -> Vec<&TestEvent> {
        vec![
            &self.upcoming_dance,
            &self.workshop,
            &self.social_event,
        ]
    }
}

/// Test group fixtures
pub struct GroupFixtures {
    pub test_group: TestGroup,
    pub private_group: TestGroup,
    pub large_group: TestGroup,
}

/// Test group data structure
#[derive(Debug, Clone)]
pub struct TestGroup {
    pub telegram_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_by: i64,
}

impl TestGroup {
    pub fn new(telegram_id: i64, title: &str, created_by: i64) -> Self {
        Self {
            telegram_id,
            title: title.to_string(),
            description: Some(format!("Test group: {}", title)),
            is_active: true,
            created_by,
        }
    }

    pub fn with_description(mut self, description: Option<&str>) -> Self {
        self.description = description.map(|s| s.to_string());
        self
    }

    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
    }
}

impl GroupFixtures {
    pub fn new(admin_user_id: i64) -> Self {
        Self {
            test_group: TestGroup::new(-1001234567890, "Test Swing Group", admin_user_id)
                .with_description(Some("Main test group for swing dancing")),
            
            private_group: TestGroup::new(-1001234567891, "Private Dance Group", admin_user_id)
                .with_description(Some("Private group for advanced dancers")),
            
            large_group: TestGroup::new(-1001234567892, "Large Community Group", admin_user_id)
                .with_description(Some("Large community group with many members")),
        }
    }

    /// Get all test groups as a vector
    pub fn all_groups(&self) -> Vec<&TestGroup> {
        vec![
            &self.test_group,
            &self.private_group,
            &self.large_group,
        ]
    }
}

/// Complete test fixtures combining all types
pub struct TestFixtures {
    pub users: UserFixtures,
    pub events: EventFixtures,
    pub groups: GroupFixtures,
}

impl TestFixtures {
    pub fn new() -> Self {
        let users = UserFixtures::new();
        let admin_id = users.admin_user.telegram_id;
        
        Self {
            events: EventFixtures::new(admin_id),
            groups: GroupFixtures::new(admin_id),
            users,
        }
    }
}

/// Helper function to load all test fixtures into database
pub async fn load_test_fixtures(
    pool: &sqlx::PgPool,
) -> Result<TestFixtures, sqlx::Error> {
    let fixtures = TestFixtures::new();
    
    // Load users
    for user in fixtures.users.all_users() {
        let user_result = sqlx::query!(
            r#"
            INSERT INTO users (telegram_id, username, first_name, last_name, language_code, location, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            ON CONFLICT (telegram_id) DO NOTHING
            "#,
            user.telegram_id,
            user.username,
            user.first_name,
            user.last_name,
            user.language_code,
            user.location
        )
        .execute(pool)
        .await;
        
        if let Err(e) = user_result {
            eprintln!("Failed to insert user {}: {}", user.telegram_id, e);
        }
    }
    
    // Load groups
    for group in fixtures.groups.all_groups() {
        let group_result = sqlx::query!(
            r#"
            INSERT INTO groups (telegram_id, title, description, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, NOW(), NOW())
            ON CONFLICT (telegram_id) DO NOTHING
            "#,
            group.telegram_id,
            group.title,
            group.description,
            group.is_active
        )
        .execute(pool)
        .await;
        
        if let Err(e) = group_result {
            eprintln!("Failed to insert group {}: {}", group.telegram_id, e);
        }
    }
    
    // Load events
    for event in fixtures.events.all_events() {
        let event_result = sqlx::query!(
            r#"
            INSERT INTO events (title, description, event_date, location, max_participants, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW())
            "#,
            event.title,
            event.description,
            event.event_date,
            event.location,
            event.max_participants,
            event.created_by
        )
        .execute(pool)
        .await;
        
        if let Err(e) = event_result {
            eprintln!("Failed to insert event {}: {}", event.title, e);
        }
    }
    
    Ok(fixtures)
}

/// Helper function to clean up test fixtures from database
pub async fn cleanup_test_fixtures(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    // Clean up in reverse order of dependencies
    sqlx::query!("DELETE FROM events WHERE created_by IN (100001, 100002, 555666777, 100004)")
        .execute(pool)
        .await?;
    
    sqlx::query!("DELETE FROM groups WHERE telegram_id IN (-1001234567890, -1001234567891, -1001234567892)")
        .execute(pool)
        .await?;
    
    sqlx::query!("DELETE FROM users WHERE telegram_id IN (100001, 100002, 555666777, 100004)")
        .execute(pool)
        .await?;
    
    Ok(())
}

/// Helper function to create test data for specific scenarios
pub fn create_onboarding_test_data() -> Vec<(i64, &'static str, &'static str, Option<&'static str>)> {
    vec![
        (200001, "en", "John Smith", Some("Moscow")),
        (200002, "ru", "Анна Иванова", Some("Saint Petersburg")),
        (200003, "en", "Bob Johnson", None), // Skip location
        (200004, "ru", "Мария Петрова", Some("Moscow")),
    ]
}

/// Helper function to create invalid input test cases
pub fn create_invalid_input_test_cases() -> HashMap<String, Vec<String>> {
    let mut cases = HashMap::new();
    
    cases.insert("invalid_names".to_string(), vec![
        String::new(), "A".to_string(), "123".to_string(), "@#$%".to_string(), "A".repeat(100)
    ]);
    
    cases.insert("invalid_languages".to_string(), vec![
        "fr".to_string(), "de".to_string(), "es".to_string(), "invalid".to_string(), "".to_string()
    ]);
    
    cases.insert("malformed_callbacks".to_string(), vec![
        "lang".to_string(),
        "lang:".to_string(),
        "location".to_string(),
        "location:".to_string(),
        "".to_string()
    ]);
    
    cases
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_fixtures_creation() {
        let fixtures = UserFixtures::new();
        
        assert_eq!(fixtures.english_user.language_code, "en");
        assert_eq!(fixtures.russian_user.language_code, "ru");
        assert!(fixtures.admin_user.is_admin);
        assert!(fixtures.incomplete_user.location.is_none());
        
        let all_users = fixtures.all_users();
        assert_eq!(all_users.len(), 4);
    }
    
    #[test]
    fn test_event_fixtures_creation() {
        let admin_id = 555666777;
        let fixtures = EventFixtures::new(admin_id);
        
        assert_eq!(fixtures.upcoming_dance.created_by, admin_id);
        assert!(fixtures.upcoming_dance.event_date > Utc::now());
        assert!(fixtures.past_event.event_date < Utc::now());
        
        let upcoming = fixtures.upcoming_events();
        assert_eq!(upcoming.len(), 3);
        
        let all_events = fixtures.all_events();
        assert_eq!(all_events.len(), 4);
    }
    
    #[test]
    fn test_group_fixtures_creation() {
        let admin_id = 555666777;
        let fixtures = GroupFixtures::new(admin_id);
        
        assert_eq!(fixtures.test_group.created_by, admin_id);
        assert!(fixtures.test_group.is_active);
        
        let all_groups = fixtures.all_groups();
        assert_eq!(all_groups.len(), 3);
    }
    
    #[test]
    fn test_complete_fixtures_creation() {
        let fixtures = TestFixtures::new();
        
        assert_eq!(fixtures.users.admin_user.telegram_id, fixtures.events.upcoming_dance.created_by);
        assert_eq!(fixtures.users.admin_user.telegram_id, fixtures.groups.test_group.created_by);
    }
    
    #[test]
    fn test_onboarding_test_data() {
        let data = create_onboarding_test_data();
        assert_eq!(data.len(), 4);
        
        // Check that we have both languages
        assert!(data.iter().any(|(_, lang, _, _)| *lang == "en"));
        assert!(data.iter().any(|(_, lang, _, _)| *lang == "ru"));
        
        // Check that we have both location scenarios
        assert!(data.iter().any(|(_, _, _, loc)| loc.is_some()));
        assert!(data.iter().any(|(_, _, _, loc)| loc.is_none()));
    }
    
    #[test]
    fn test_invalid_input_test_cases() {
        let cases = create_invalid_input_test_cases();
        
        assert!(cases.contains_key("invalid_names"));
        assert!(cases.contains_key("invalid_languages"));
        assert!(cases.contains_key("malformed_callbacks"));
        
        assert!(!cases["invalid_names"].is_empty());
        assert!(!cases["invalid_languages"].is_empty());
        assert!(!cases["malformed_callbacks"].is_empty());
    }
}