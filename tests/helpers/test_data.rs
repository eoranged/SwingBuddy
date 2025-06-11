//! Test data helpers for creating test objects
//! 
//! This module provides helper functions for creating test Telegram messages,
//! callback queries, users, and other test data structures.

use teloxide::types::{
    Message, User, Chat, ChatKind, MessageKind, MessageCommon, CallbackQuery, 
    InlineKeyboardButton, InlineKeyboardMarkup, UserId, ChatId, MessageId,
    ChatPrivate, ChatPublic, PublicChatKind, PublicChatSupergroup,
    MediaKind, MediaText
};
use chrono::Utc;

/// Helper function to create a test Telegram user
pub fn create_test_user(
    user_id: i64,
    username: Option<&str>,
    first_name: &str,
    last_name: Option<&str>,
    language_code: Option<&str>,
) -> User {
    User {
        id: UserId(user_id as u64),
        is_bot: false,
        first_name: first_name.to_string(),
        last_name: last_name.map(|s| s.to_string()),
        username: username.map(|s| s.to_string()),
        language_code: language_code.map(|s| s.to_string()),
        is_premium: false,
        added_to_attachment_menu: false,
    }
}

/// Helper function to create a test private chat
pub fn create_test_private_chat(
    chat_id: i64,
    username: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
) -> Chat {
    Chat {
        id: ChatId(chat_id),
        kind: ChatKind::Private(ChatPrivate {
            username: username.map(|s| s.to_string()),
            first_name: first_name.map(|s| s.to_string()),
            last_name: last_name.map(|s| s.to_string()),
        }),
    }
}

/// Helper function to create a test group chat
pub fn create_test_group_chat(chat_id: i64, title: &str) -> Chat {
    Chat {
        id: ChatId(chat_id),
        kind: ChatKind::Public(ChatPublic {
            title: Some(title.to_string()),
            kind: PublicChatKind::Supergroup(PublicChatSupergroup {
                username: None,
                is_forum: false,
            }),
        }),
    }
}

/// Helper function to create a test Telegram message
pub fn create_test_message(
    user_id: i64,
    chat_id: i64,
    text: &str,
    username: Option<&str>,
    first_name: &str,
    last_name: Option<&str>,
) -> Message {
    let user = create_test_user(user_id, username, first_name, last_name, Some("en"));
    
    let chat = if chat_id > 0 {
        create_test_private_chat(chat_id, username, Some(first_name), last_name)
    } else {
        create_test_group_chat(chat_id, "Test Group")
    };

    Message {
        id: MessageId(1),
        thread_id: None,
        from: Some(user),
        sender_chat: None,
        sender_business_bot: None,
        date: Utc::now(),
        chat,
        is_topic_message: false,
        via_bot: None,
        kind: MessageKind::Common(MessageCommon {
            author_signature: None,
            forward_origin: None,
            external_reply: None,
            quote: None,
            reply_to_story: None,
            edit_date: None,
            media_kind: MediaKind::Text(MediaText {
                text: text.to_string(),
                entities: vec![],
                link_preview_options: None,
            }),
            reply_markup: None,
            effect_id: None,
            reply_to_message: None,
            sender_boost_count: None,
            is_automatic_forward: false,
            has_protected_content: false,
            is_from_offline: false,
            business_connection_id: None,
        }),
    }
}

/// Helper function to create a simple test message with default user data
pub fn create_simple_test_message(user_id: i64, chat_id: i64, text: &str) -> Message {
    create_test_message(
        user_id,
        chat_id,
        text,
        Some("testuser"),
        "TestUser",
        Some("LastName"),
    )
}

/// Helper function to create a test callback query
pub fn create_test_callback_query(
    user_id: i64,
    chat_id: i64,
    data: &str,
    username: Option<&str>,
    first_name: &str,
    last_name: Option<&str>,
) -> CallbackQuery {
    let user = create_test_user(user_id, username, first_name, last_name, Some("en"));
    let message = create_test_message(user_id, chat_id, "Test message", username, first_name, last_name);
    
    CallbackQuery {
        id: format!("callback_{}", user_id),
        from: user,
        message: Some(teloxide::types::MaybeInaccessibleMessage::Regular(Box::new(message))),
        inline_message_id: None,
        data: Some(data.to_string()),
        game_short_name: None,
        chat_instance: "test_chat_instance".to_string(),
    }
}

/// Helper function to create a simple test callback query with default user data
pub fn create_simple_test_callback_query(user_id: i64, chat_id: i64, data: &str) -> CallbackQuery {
    create_test_callback_query(
        user_id,
        chat_id,
        data,
        Some("testuser"),
        "TestUser",
        Some("LastName"),
    )
}

/// Helper function to create an inline keyboard with language options
pub fn create_language_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("🇺🇸 English", "lang:en"),
            InlineKeyboardButton::callback("🇷🇺 Русский", "lang:ru"),
        ]
    ])
}

/// Helper function to create an inline keyboard with location options
pub fn create_location_keyboard() -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new(vec![
        vec![
            InlineKeyboardButton::callback("📍 Moscow", "location:Moscow"),
            InlineKeyboardButton::callback("📍 Saint Petersburg", "location:Saint Petersburg"),
        ],
        vec![
            InlineKeyboardButton::callback("⏭️ Skip", "location:skip"),
        ]
    ])
}

/// Test user data structure for creating database users
#[derive(Debug, Clone)]
pub struct TestUserData {
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub language_code: String,
    pub location: Option<String>,
}

impl TestUserData {
    /// Create a new test user data with default values
    pub fn new(telegram_id: i64, first_name: &str) -> Self {
        Self {
            telegram_id,
            username: Some(format!("user_{}", telegram_id)),
            first_name: first_name.to_string(),
            last_name: Some("TestLastName".to_string()),
            language_code: "en".to_string(),
            location: None,
        }
    }

    /// Set username
    pub fn with_username(mut self, username: Option<&str>) -> Self {
        self.username = username.map(|s| s.to_string());
        self
    }

    /// Set last name
    pub fn with_last_name(mut self, last_name: Option<&str>) -> Self {
        self.last_name = last_name.map(|s| s.to_string());
        self
    }

    /// Set language code
    pub fn with_language(mut self, language_code: &str) -> Self {
        self.language_code = language_code.to_string();
        self
    }

    /// Set location
    pub fn with_location(mut self, location: Option<&str>) -> Self {
        self.location = location.map(|s| s.to_string());
        self
    }
}

/// Test scenario data for onboarding flow testing
#[derive(Debug, Clone)]
pub struct OnboardingTestScenario {
    pub user_data: TestUserData,
    pub selected_language: String,
    pub provided_name: String,
    pub selected_location: Option<String>,
    pub should_skip_location: bool,
}

impl OnboardingTestScenario {
    /// Create a complete onboarding scenario
    pub fn complete_flow(telegram_id: i64) -> Self {
        Self {
            user_data: TestUserData::new(telegram_id, "TestUser"),
            selected_language: "en".to_string(),
            provided_name: "John Doe".to_string(),
            selected_location: Some("Moscow".to_string()),
            should_skip_location: false,
        }
    }

    /// Create an onboarding scenario with location skip
    pub fn with_location_skip(telegram_id: i64) -> Self {
        Self {
            user_data: TestUserData::new(telegram_id, "TestUser"),
            selected_language: "ru".to_string(),
            provided_name: "Иван Петров".to_string(),
            selected_location: None,
            should_skip_location: true,
        }
    }

    /// Create a Russian language onboarding scenario
    pub fn russian_flow(telegram_id: i64) -> Self {
        Self {
            user_data: TestUserData::new(telegram_id, "Тестовый Пользователь"),
            selected_language: "ru".to_string(),
            provided_name: "Анна Смирнова".to_string(),
            selected_location: Some("Saint Petersburg".to_string()),
            should_skip_location: false,
        }
    }

    /// Get the callback data for language selection
    pub fn language_callback_data(&self) -> String {
        format!("lang:{}", self.selected_language)
    }

    /// Get the callback data for location selection
    pub fn location_callback_data(&self) -> String {
        if self.should_skip_location {
            "location:skip".to_string()
        } else {
            format!("location:{}", self.selected_location.as_ref().unwrap())
        }
    }
}

/// Helper function to create multiple test users for concurrent testing
pub fn create_multiple_test_users(count: usize, base_id: i64) -> Vec<TestUserData> {
    (0..count)
        .map(|i| {
            let user_id = base_id + i as i64;
            TestUserData::new(user_id, &format!("TestUser{}", i + 1))
                .with_username(Some(&format!("testuser{}", i + 1)))
                .with_language(if i % 2 == 0 { "en" } else { "ru" })
        })
        .collect()
}

/// Helper function to create test messages for invalid name inputs
pub fn create_invalid_name_test_cases() -> Vec<(String, &'static str)> {
    vec![
        ("A".to_string(), "Too short name"),
        ("".to_string(), "Empty name"),
        ("123".to_string(), "Numbers only"),
        ("@#$%".to_string(), "Special characters only"),
        ("A".repeat(60), "Too long name"),
        ("Test123".to_string(), "Name with numbers"),
        ("Test@User".to_string(), "Name with special characters"),
    ]
}

/// Helper function to create test messages for valid name inputs
pub fn create_valid_name_test_cases() -> Vec<(&'static str, &'static str)> {
    vec![
        ("John", "Simple English name"),
        ("John Doe", "English name with space"),
        ("Иван", "Simple Russian name"),
        ("Анна Петрова", "Russian name with space"),
        ("José María", "Name with accents"),
        ("李小明", "Chinese name"),
        ("محمد علي", "Arabic name"),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_user() {
        let user = create_test_user(123, Some("testuser"), "Test", Some("User"), Some("en"));
        
        assert_eq!(user.id.0, 123);
        assert_eq!(user.username, Some("testuser".to_string()));
        assert_eq!(user.first_name, "Test");
        assert_eq!(user.last_name, Some("User".to_string()));
        assert_eq!(user.language_code, Some("en".to_string()));
        assert!(!user.is_bot);
    }

    #[test]
    fn test_create_test_message() {
        let message = create_simple_test_message(123, 123, "Hello");
        
        assert_eq!(message.from.as_ref().unwrap().id.0, 123);
        assert_eq!(message.chat.id.0, 123);
        
        if let MessageKind::Common(common) = &message.kind {
            if let MediaKind::Text(text) = &common.media_kind {
                assert_eq!(text.text, "Hello");
            } else {
                panic!("Expected text message");
            }
        } else {
            panic!("Expected common message");
        }
    }

    #[test]
    fn test_create_test_callback_query() {
        let callback = create_simple_test_callback_query(123, 123, "test:data");
        
        assert_eq!(callback.from.id.0, 123);
        assert_eq!(callback.data, Some("test:data".to_string()));
        assert!(callback.message.is_some());
    }

    #[test]
    fn test_onboarding_test_scenario() {
        let scenario = OnboardingTestScenario::complete_flow(123);
        
        assert_eq!(scenario.user_data.telegram_id, 123);
        assert_eq!(scenario.selected_language, "en");
        assert_eq!(scenario.provided_name, "John Doe");
        assert_eq!(scenario.selected_location, Some("Moscow".to_string()));
        assert!(!scenario.should_skip_location);
        
        assert_eq!(scenario.language_callback_data(), "lang:en");
        assert_eq!(scenario.location_callback_data(), "location:Moscow");
    }

    #[test]
    fn test_onboarding_scenario_with_skip() {
        let scenario = OnboardingTestScenario::with_location_skip(456);
        
        assert_eq!(scenario.user_data.telegram_id, 456);
        assert_eq!(scenario.selected_language, "ru");
        assert!(scenario.should_skip_location);
        assert_eq!(scenario.location_callback_data(), "location:skip");
    }

    #[test]
    fn test_create_multiple_test_users() {
        let users = create_multiple_test_users(3, 1000);
        
        assert_eq!(users.len(), 3);
        assert_eq!(users[0].telegram_id, 1000);
        assert_eq!(users[1].telegram_id, 1001);
        assert_eq!(users[2].telegram_id, 1002);
        
        assert_eq!(users[0].language_code, "en");
        assert_eq!(users[1].language_code, "ru");
        assert_eq!(users[2].language_code, "en");
    }

    #[test]
    fn test_invalid_name_test_cases() {
        let cases = create_invalid_name_test_cases();
        assert!(!cases.is_empty());
        
        // Check that we have various types of invalid inputs
        let names: Vec<String> = cases.iter().map(|(name, _)| name.clone()).collect();
        assert!(names.contains(&"A".to_string())); // Too short
        assert!(names.contains(&"".to_string())); // Empty
        assert!(names.contains(&"123".to_string())); // Numbers only
    }

    #[test]
    fn test_valid_name_test_cases() {
        let cases = create_valid_name_test_cases();
        assert!(!cases.is_empty());
        
        // Check that we have various types of valid inputs
        let names: Vec<&str> = cases.iter().map(|(name, _)| *name).collect();
        assert!(names.contains(&"John")); // Simple English
        assert!(names.contains(&"Иван")); // Simple Russian
        assert!(names.contains(&"John Doe")); // With space
    }
}