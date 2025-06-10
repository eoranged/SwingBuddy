# SwingBuddy Database Layer

This document describes the database layer implementation for the SwingBuddy Telegram bot.

## Overview

The database layer is built using SQLx with PostgreSQL and follows the Repository pattern for clean separation of concerns. It provides async operations with proper error handling and type safety.

## Architecture

```
src/database/
├── mod.rs              # Module exports
├── connection.rs       # Database connection management
├── service.rs          # High-level database service
├── repositories/       # Repository implementations
│   ├── mod.rs         # Repository exports
│   ├── user.rs        # User repository
│   ├── group.rs       # Group repository
│   ├── event.rs       # Event repository
│   └── admin.rs       # Admin repository
└── migrations/         # SQL migration files
    └── 001_initial_schema.sql
```

## Models

### User Model
- **Fields**: id, telegram_id, username, first_name, last_name, language_code, location, is_banned, created_at, updated_at
- **Operations**: CRUD, find by telegram_id, ban/unban, search by username pattern

### Group Model
- **Fields**: id, telegram_id, title, description, language_code, settings, is_active, created_at, updated_at
- **Operations**: CRUD, member management, group settings

### Event Model
- **Fields**: id, title, description, event_date, location, max_participants, google_calendar_id, created_by, group_id, is_active, created_at, updated_at
- **Operations**: CRUD, participant management, upcoming events, group events

### Admin Models
- **AdminSettings**: Key-value configuration storage
- **UserState**: Conversation state management
- **CasCheck**: CAS API check logging

## Database Schema

The database schema includes:

- **users**: User information and settings
- **groups**: Telegram group configuration
- **group_members**: Group membership with roles
- **events**: Event information and scheduling
- **event_participants**: Event registration tracking
- **admin_settings**: System configuration
- **user_states**: Conversation state storage
- **cas_checks**: CAS API check logs

All tables include proper indexes for performance optimization.

## Usage Examples

### Basic Setup

```rust
use swingbuddy::database::{create_pool, run_migrations, DatabaseConfig, DatabaseService};

// Create database connection
let config = DatabaseConfig {
    url: "postgresql://user:password@localhost/swingbuddy".to_string(),
    max_connections: 10,
    min_connections: 1,
    // ... other config
};

let pool = create_pool(&config).await?;
run_migrations(&pool).await?;

// Create database service
let db = DatabaseService::new(pool);
```

### User Operations

```rust
// Create a new user
let user = db.initialize_user(
    123456789,  // telegram_id
    Some("username".to_string()),
    Some("John".to_string()),
    Some("Doe".to_string())
).await?;

// Find user by telegram ID
let user = db.users.find_by_telegram_id(123456789).await?;

// Update user
let update_request = UpdateUserRequest {
    language_code: Some("ru".to_string()),
    location: Some("Moscow".to_string()),
    ..Default::default()
};
let updated_user = db.users.update(user.id, update_request).await?;
```

### Group Operations

```rust
// Create a group
let group = db.initialize_group(
    -987654321,  // telegram_id (negative for groups)
    "Swing Dance Moscow".to_string(),
    Some("Moscow swing dancing community".to_string())
).await?;

// Add user to group
let member = db.add_user_to_group(
    user.id,
    group.id,
    Some("admin".to_string())
).await?;

// Get group members
let members = db.groups.get_members(group.id).await?;
```

### Event Operations

```rust
use chrono::{Utc, Duration};

// Create an event
let event_date = Utc::now() + Duration::days(7);
let event = db.create_event(
    "Weekly Swing Practice".to_string(),
    Some("Join us for weekly swing dancing practice".to_string()),
    event_date,
    Some("Dance Studio, Moscow".to_string()),
    Some(20),  // max participants
    Some(user.id),  // created_by
    Some(group.id)  // group_id
).await?;

// Register user for event
let participant = db.register_for_event(event.id, user.id).await?;

// Get upcoming events
let upcoming = db.events.get_upcoming_events(Some(10)).await?;
```

### State Management

```rust
// Set user conversation state
let state = db.set_user_state(
    user.id,
    Some("event_creation".to_string()),  // scenario
    Some("waiting_for_title".to_string()),  // step
    Some(serde_json::json!({"temp_data": "value"})),  // data
    Some(Utc::now() + Duration::hours(1))  // expires_at
).await?;

// Get user state
let current_state = db.get_user_state(user.id).await?;

// Clear state
db.clear_user_state(user.id).await?;
```

### Admin Operations

```rust
// Record CAS check
let cas_check = db.record_cas_check(
    user.id,
    user.telegram_id,
    false,  // is_banned
    None    // ban_reason
).await?;

// Get system statistics
let stats = db.get_system_stats().await?;

// Cleanup expired data
let cleanup_result = db.cleanup_expired_data().await?;
```

## Repository Pattern

Each repository provides:

- **CRUD operations**: create, read, update, delete
- **Specialized queries**: find by specific fields, complex joins
- **Pagination support**: for listing operations
- **Async operations**: all methods are async
- **Error handling**: proper error propagation

### Repository Methods

#### UserRepository
- `create(request)` - Create new user
- `find_by_id(id)` - Find by primary key
- `find_by_telegram_id(telegram_id)` - Find by Telegram ID
- `update(id, request)` - Update user
- `delete(id)` - Delete user
- `list(limit, offset)` - List with pagination
- `count()` - Count total users
- `set_ban_status(id, is_banned)` - Ban/unban user
- `get_banned_users()` - Get all banned users

#### GroupRepository
- `create(request)` - Create new group
- `find_by_id(id)` - Find by primary key
- `find_by_telegram_id(telegram_id)` - Find by Telegram ID
- `update(id, request)` - Update group
- `delete(id)` - Delete group
- `add_member(request)` - Add member to group
- `remove_member(group_id, user_id)` - Remove member
- `get_members(group_id)` - Get group members
- `is_member(group_id, user_id)` - Check membership
- `update_member_role(group_id, user_id, role)` - Update member role
- `get_user_groups(user_id)` - Get user's groups
- `get_active_groups()` - Get active groups

#### EventRepository
- `create(request)` - Create new event
- `find_by_id(id)` - Find by primary key
- `update(id, request)` - Update event
- `delete(id)` - Delete event
- `list(limit, offset)` - List with pagination
- `get_upcoming_events(limit)` - Get upcoming events
- `get_group_events(group_id)` - Get group's events
- `register_participant(request)` - Register participant
- `unregister_participant(event_id, user_id)` - Unregister
- `get_participants(event_id)` - Get event participants
- `is_registered(event_id, user_id)` - Check registration
- `get_participant_count(event_id)` - Count participants
- `get_user_events(user_id)` - Get user's created events
- `get_user_registered_events(user_id)` - Get user's registrations

#### AdminRepository
- `create_setting(request)` - Create admin setting
- `get_setting(key)` - Get setting by key
- `update_setting(key, request)` - Update setting
- `delete_setting(key)` - Delete setting
- `list_settings()` - List all settings
- `upsert_user_state(request)` - Create/update user state
- `get_user_state(user_id)` - Get user state
- `update_user_state(user_id, request)` - Update user state
- `delete_user_state(user_id)` - Delete user state
- `clean_expired_states()` - Clean expired states
- `create_cas_check(request)` - Create CAS check record
- `get_latest_cas_check(user_id)` - Get latest CAS check
- `get_user_cas_checks(user_id)` - Get user's CAS checks
- `get_banned_users_from_cas()` - Get banned users from CAS
- `clean_old_cas_checks(keep_days)` - Clean old CAS checks
- `get_stats()` - Get system statistics

## Error Handling

All database operations return `Result<T, SwingBuddyError>` where `SwingBuddyError` includes:

- `Database(sqlx::Error)` - Database-specific errors
- `UserNotFound { user_id }` - User not found errors
- `Config(String)` - Configuration/validation errors

## Performance Considerations

- **Connection Pooling**: Configurable connection pool with min/max connections
- **Indexes**: Proper database indexes for common queries
- **Pagination**: Built-in pagination support for large datasets
- **Async Operations**: Non-blocking database operations
- **Query Optimization**: Efficient SQL queries with proper joins

## Migration Management

Database migrations are managed using SQLx migrations:

```bash
# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add migration_name
```

The initial schema migration (`001_initial_schema.sql`) creates all required tables with proper constraints and indexes.

## Testing

Repository tests can be run with a test database:

```bash
# Set test database URL
export DATABASE_URL="postgresql://test_user:test_pass@localhost/swingbuddy_test"

# Run tests
cargo test
```

## Configuration

Database configuration is handled through the `DatabaseConfig` struct:

```rust
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Option<Duration>,
    pub max_lifetime: Option<Duration>,
}
```

This provides a complete, production-ready database layer for the SwingBuddy Telegram bot with proper error handling, type safety, and performance optimization.