# SwingBuddy Telegram Bot - Technical Architecture Document

## Executive Summary

SwingBuddy is a Telegram bot designed for swing dancing community management, built in Rust using the teloxide framework. The bot facilitates event organization, user onboarding, group management, and community moderation with multi-language support and comprehensive admin functionality.

## System Overview

```mermaid
graph TB
    subgraph "External Services"
        TG[Telegram API]
        GC[Google Calendar API]
        CAS[CAS API]
    end
    
    subgraph "SwingBuddy Bot"
        WH[Webhook Handler]
        RT[Router/Dispatcher]
        SM[State Manager]
        
        subgraph "Core Modules"
            UM[User Management]
            GM[Group Management]
            EM[Event Management]
            AM[Admin Module]
            I18N[Internationalization]
        end
        
        subgraph "Services"
            AS[Auth Service]
            CS[CAS Service]
            GS[Google Service]
            NS[Notification Service]
        end
    end
    
    subgraph "Data Layer"
        PG[(PostgreSQL)]
        RD[(Redis Cache)]
        CF[Config Files]
    end
    
    TG --> WH
    WH --> RT
    RT --> SM
    SM --> UM
    SM --> GM
    SM --> EM
    SM --> AM
    
    UM --> AS
    GM --> CS
    EM --> GS
    AM --> NS
    
    AS --> PG
    CS --> CAS
    GS --> GC
    NS --> TG
    
    SM --> RD
    AM --> CF
```

## Project Structure

```
src/
├── main.rs                 # Application entry point
├── lib.rs                  # Library root
├── config/
│   ├── mod.rs             # Configuration module
│   ├── settings.rs        # Settings management
│   └── validation.rs      # Config validation
├── handlers/
│   ├── mod.rs             # Handler module root
│   ├── commands/          # Command handlers
│   │   ├── mod.rs
│   │   ├── start.rs       # /start command
│   │   ├── help.rs        # /help command
│   │   ├── events.rs      # Event-related commands
│   │   └── admin.rs       # Admin commands
│   ├── callbacks/         # Callback query handlers
│   │   ├── mod.rs
│   │   ├── language.rs    # Language selection
│   │   ├── events.rs      # Event interactions
│   │   └── admin.rs       # Admin callbacks
│   └── messages/          # Message handlers
│       ├── mod.rs
│       ├── text.rs        # Text message handling
│       └── media.rs       # Media message handling
├── services/
│   ├── mod.rs             # Services module root
│   ├── auth.rs            # Authentication service
│   ├── cas.rs             # CAS API integration
│   ├── google.rs          # Google Calendar service
│   ├── notification.rs    # Notification service
│   └── user.rs            # User management service
├── models/
│   ├── mod.rs             # Models module root
│   ├── user.rs            # User model
│   ├── group.rs           # Group model
│   ├── event.rs           # Event model
│   └── admin.rs           # Admin model
├── database/
│   ├── mod.rs             # Database module root
│   ├── connection.rs      # DB connection management
│   ├── migrations/        # SQL migrations
│   └── repositories/      # Data access layer
│       ├── mod.rs
│       ├── user.rs        # User repository
│       ├── group.rs       # Group repository
│       └── event.rs       # Event repository
├── state/
│   ├── mod.rs             # State management
│   ├── context.rs         # Conversation context
│   ├── scenarios.rs       # Scenario definitions
│   └── storage.rs         # State persistence
├── i18n/
│   ├── mod.rs             # Internationalization
│   ├── loader.rs          # Translation loader
│   └── translations/      # Translation files
│       ├── en.json
│       └── ru.json
├── utils/
│   ├── mod.rs             # Utilities
│   ├── errors.rs          # Error handling
│   ├── logging.rs         # Logging setup
│   └── helpers.rs         # Helper functions
└── middleware/
    ├── mod.rs             # Middleware root
    ├── auth.rs            # Authentication middleware
    ├── logging.rs         # Request logging
    └── rate_limit.rs      # Rate limiting
```

## Database Schema Design

```sql
-- Users table
CREATE TABLE users (
    id BIGINT PRIMARY KEY,
    telegram_id BIGINT UNIQUE NOT NULL,
    username VARCHAR(255),
    first_name VARCHAR(255),
    last_name VARCHAR(255),
    language_code VARCHAR(10) DEFAULT 'en',
    location VARCHAR(255),
    is_banned BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Groups table
CREATE TABLE groups (
    id BIGINT PRIMARY KEY,
    telegram_id BIGINT UNIQUE NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    language_code VARCHAR(10) DEFAULT 'en',
    settings JSONB DEFAULT '{}',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Group members table
CREATE TABLE group_members (
    id BIGSERIAL PRIMARY KEY,
    group_id BIGINT REFERENCES groups(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member',
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(group_id, user_id)
);

-- Events table
CREATE TABLE events (
    id BIGSERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    event_date TIMESTAMP WITH TIME ZONE NOT NULL,
    location VARCHAR(255),
    max_participants INTEGER,
    google_calendar_id VARCHAR(255),
    created_by BIGINT REFERENCES users(id),
    group_id BIGINT REFERENCES groups(id),
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Event participants table
CREATE TABLE event_participants (
    id BIGSERIAL PRIMARY KEY,
    event_id BIGINT REFERENCES events(id) ON DELETE CASCADE,
    user_id BIGINT REFERENCES users(id) ON DELETE CASCADE,
    status VARCHAR(50) DEFAULT 'registered',
    registered_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(event_id, user_id)
);

-- Admin settings table
CREATE TABLE admin_settings (
    id BIGSERIAL PRIMARY KEY,
    key VARCHAR(255) UNIQUE NOT NULL,
    value JSONB NOT NULL,
    updated_by BIGINT REFERENCES users(id),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User states table (for conversation context)
CREATE TABLE user_states (
    user_id BIGINT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    scenario VARCHAR(100),
    step VARCHAR(100),
    data JSONB DEFAULT '{}',
    expires_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- CAS check logs table
CREATE TABLE cas_checks (
    id BIGSERIAL PRIMARY KEY,
    user_id BIGINT NOT NULL,
    telegram_id BIGINT NOT NULL,
    is_banned BOOLEAN NOT NULL,
    ban_reason TEXT,
    checked_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_users_telegram_id ON users(telegram_id);
CREATE INDEX idx_groups_telegram_id ON groups(telegram_id);
CREATE INDEX idx_events_date ON events(event_date);
CREATE INDEX idx_events_group ON events(group_id);
CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_user_states_scenario ON user_states(scenario);
```

## Configuration Management

**TOML Configuration Structure:**

```toml
[bot]
token = "YOUR_BOT_TOKEN"
webhook_url = "https://your-domain.com/webhook"
admin_ids = [123456789, 987654321]

[database]
url = "postgresql://user:password@localhost/swingbuddy"
max_connections = 10
min_connections = 1

[redis]
url = "redis://localhost:6379"
prefix = "swingbuddy:"
ttl_seconds = 3600

[google]
service_account_path = "/path/to/service-account.json"
calendar_id = "your-calendar-id@group.calendar.google.com"

[cas]
api_url = "https://api.cas.chat"
timeout_seconds = 5
auto_ban = true

[i18n]
default_language = "en"
supported_languages = ["en", "ru"]

[logging]
level = "info"
file_path = "/var/log/swingbuddy.log"
max_file_size = "10MB"
max_files = 5

[features]
cas_protection = true
google_calendar = true
admin_panel = true
```

## State Management Architecture

```mermaid
stateDiagram-v2
    [*] --> Idle
    
    Idle --> NewUserOnboarding: /start (new user)
    Idle --> GroupSetup: Bot added to group
    Idle --> EventManagement: Event commands
    Idle --> AdminPanel: Admin commands
    
    state NewUserOnboarding {
        [*] --> LanguageSelection
        LanguageSelection --> NameInput
        NameInput --> LocationInput
        LocationInput --> Welcome
        Welcome --> [*]
    }
    
    state GroupSetup {
        [*] --> PermissionCheck
        PermissionCheck --> ConfigurationFlow
        ConfigurationFlow --> GroupReady
        GroupReady --> [*]
    }
    
    state EventManagement {
        [*] --> EventList
        EventList --> EventDetails
        EventList --> CreateEvent
        EventDetails --> RegisterForEvent
        CreateEvent --> EventCreated
        RegisterForEvent --> [*]
        EventCreated --> [*]
    }
    
    state AdminPanel {
        [*] --> AdminMenu
        AdminMenu --> UserManagement
        AdminMenu --> GroupManagement
        AdminMenu --> EventManagement
        AdminMenu --> SystemSettings
        UserManagement --> [*]
        GroupManagement --> [*]
        EventManagement --> [*]
        SystemSettings --> [*]
    }
    
    NewUserOnboarding --> Idle
    GroupSetup --> Idle
    EventManagement --> Idle
    AdminPanel --> Idle
```

## Internationalization Strategy

**Translation Management:**

```rust
// i18n/mod.rs
use std::collections::HashMap;
use serde_json::Value;

pub struct I18n {
    translations: HashMap<String, HashMap<String, Value>>,
    default_lang: String,
}

impl I18n {
    pub fn new(default_lang: String) -> Self {
        Self {
            translations: HashMap::new(),
            default_lang,
        }
    }
    
    pub fn t(&self, key: &str, lang: &str, params: Option<&HashMap<String, String>>) -> String {
        let lang = if self.translations.contains_key(lang) {
            lang
        } else {
            &self.default_lang
        };
        
        // Translation logic with parameter substitution
        // ...
    }
}
```

**Translation Files Structure:**

```json
// i18n/translations/en.json
{
  "commands": {
    "start": {
      "welcome": "Welcome to SwingBuddy! 💃🕺",
      "choose_language": "Please choose your language:"
    },
    "events": {
      "list_title": "Upcoming Events",
      "no_events": "No upcoming events found.",
      "register_success": "Successfully registered for {event_name}!"
    }
  },
  "errors": {
    "generic": "Something went wrong. Please try again.",
    "permission_denied": "You don't have permission to perform this action."
  }
}
```

## API Integration Patterns

**CAS API Integration:**

```rust
// services/cas.rs
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct CasResponse {
    pub ok: bool,
    pub result: Option<CasResult>,
}

#[derive(Debug, Deserialize)]
pub struct CasResult {
    pub offenses: u32,
    pub messages: Vec<String>,
    pub time_added: Option<String>,
}

pub struct CasService {
    client: Client,
    api_url: String,
    timeout: Duration,
}

impl CasService {
    pub async fn check_user(&self, user_id: i64) -> Result<CasResponse, CasError> {
        let url = format!("{}/check?user_id={}", self.api_url, user_id);
        
        let response = self.client
            .get(&url)
            .timeout(self.timeout)
            .send()
            .await?
            .json::<CasResponse>()
            .await?;
            
        Ok(response)
    }
}
```

**Google Calendar Integration:**

```rust
// services/google.rs
use google_calendar3::{CalendarHub, oauth2, hyper, hyper_rustls};

pub struct GoogleCalendarService {
    hub: CalendarHub<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
    calendar_id: String,
}

impl GoogleCalendarService {
    pub async fn create_event(&self, event: &Event) -> Result<String, GoogleError> {
        let calendar_event = google_calendar3::api::Event {
            summary: Some(event.title.clone()),
            description: event.description.clone(),
            start: Some(google_calendar3::api::EventDateTime {
                date_time: Some(event.event_date.to_rfc3339()),
                ..Default::default()
            }),
            location: event.location.clone(),
            ..Default::default()
        };
        
        let result = self.hub
            .events()
            .insert(calendar_event, &self.calendar_id)
            .doit()
            .await?;
            
        Ok(result.1.id.unwrap_or_default())
    }
}
```

## Error Handling Strategy

```rust
// utils/errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SwingBuddyError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Telegram API error: {0}")]
    Telegram(#[from] teloxide::RequestError),
    
    #[error("CAS API error: {0}")]
    Cas(#[from] CasError),
    
    #[error("Google Calendar error: {0}")]
    Google(#[from] GoogleError),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: i64 },
    
    #[error("Invalid state transition: {from} -> {to}")]
    InvalidStateTransition { from: String, to: String },
}

pub type Result<T> = std::result::Result<T, SwingBuddyError>;
```

## Logging Strategy

```rust
// utils/logging.rs
use tracing::{info, warn, error, debug};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging(config: &LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    let file_appender = tracing_appender::rolling::daily(&config.file_path, "swingbuddy.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.level))
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
        .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
        .init();
        
    Ok(())
}

// Structured logging for key events
pub fn log_user_action(user_id: i64, action: &str, details: Option<&str>) {
    info!(
        user_id = user_id,
        action = action,
        details = details,
        "User action performed"
    );
}

pub fn log_cas_check(user_id: i64, is_banned: bool, reason: Option<&str>) {
    if is_banned {
        warn!(
            user_id = user_id,
            reason = reason,
            "CAS check: User is banned"
        );
    } else {
        debug!(user_id = user_id, "CAS check: User is clean");
    }
}
```

## Deployment Architecture

```mermaid
graph TB
    subgraph "Production Environment"
        subgraph "Application Layer"
            APP[SwingBuddy Bot]
            NGINX[Nginx Reverse Proxy]
        end
        
        subgraph "Data Layer"
            PG[(PostgreSQL)]
            REDIS[(Redis)]
        end
        
        subgraph "External Services"
            TG[Telegram API]
            GC[Google Calendar]
            CAS[CAS API]
        end
        
        subgraph "Monitoring"
            LOGS[Log Files]
            METRICS[Metrics Collection]
        end
    end
    
    NGINX --> APP
    APP --> PG
    APP --> REDIS
    APP --> TG
    APP --> GC
    APP --> CAS
    APP --> LOGS
    APP --> METRICS
```

## Security Considerations

1. **Authentication & Authorization:**
   - Admin user verification via Telegram ID whitelist
   - Role-based access control for group operations
   - Rate limiting to prevent abuse

2. **Data Protection:**
   - Encrypted database connections
   - Secure storage of API keys and tokens
   - Regular security updates

3. **CAS Integration:**
   - Automatic spam user detection
   - Configurable auto-ban functionality
   - Audit logging for all CAS actions

## Performance Optimization

1. **Caching Strategy:**
   - Redis for user session states
   - Cached translation files
   - Event data caching with TTL

2. **Database Optimization:**
   - Proper indexing for frequent queries
   - Connection pooling
   - Query optimization

3. **Resource Management:**
   - Async/await throughout the application
   - Efficient memory usage with streaming
   - Background task processing

## Testing Strategy

```rust
// tests/integration/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_onboarding_flow() {
        // Test complete user onboarding scenario
    }
    
    #[tokio::test]
    async fn test_cas_integration() {
        // Test CAS API integration
    }
    
    #[tokio::test]
    async fn test_event_creation() {
        // Test event creation and Google Calendar integration
    }
}
```

## Monitoring and Observability

1. **Metrics Collection:**
   - User activity metrics
   - API response times
   - Error rates and types

2. **Health Checks:**
   - Database connectivity
   - External API availability
   - Redis connection status

3. **Alerting:**
   - Critical error notifications
   - Performance degradation alerts
   - Security incident notifications

## Implementation Roadmap

### Phase 1: Core Infrastructure
- [ ] Project structure setup
- [ ] Database schema implementation
- [ ] Configuration management
- [ ] Basic error handling and logging
- [ ] Teloxide bot framework integration

### Phase 2: User Management
- [ ] User registration and onboarding flow
- [ ] Language selection and i18n implementation
- [ ] State management for conversations
- [ ] CAS API integration for spam protection

### Phase 3: Group Management
- [ ] Group setup and configuration
- [ ] Permission system implementation
- [ ] Member management functionality
- [ ] Group-specific settings

### Phase 4: Event Management
- [ ] Event creation and management
- [ ] Google Calendar integration
- [ ] Event registration system
- [ ] Notification system

### Phase 5: Admin Features
- [ ] Admin panel implementation
- [ ] User and group management tools
- [ ] System configuration interface
- [ ] Monitoring and analytics

### Phase 6: Production Deployment
- [ ] Production environment setup
- [ ] CI/CD pipeline implementation
- [ ] Monitoring and alerting setup
- [ ] Performance optimization
- [ ] Security hardening

## Conclusion

This architecture provides a solid foundation for the SwingBuddy Telegram bot with emphasis on modularity, maintainability, and extensibility. The design supports the specified requirements while allowing for future enhancements and scaling as the swing dancing community grows.

The modular structure ensures that new features can be easily added without affecting existing functionality, while the comprehensive error handling and logging strategies provide visibility into system operations and facilitate debugging and maintenance.