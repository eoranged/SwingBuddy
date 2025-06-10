# SwingBuddy Test Suite

This directory contains the comprehensive test suite for the SwingBuddy Telegram bot. The tests are organized into different categories to ensure thorough coverage of all functionality.

## Test Structure

```
tests/
├── README.md                          # This file
├── fixtures/                          # Test data fixtures
│   └── mod.rs                         # Test data helpers and fixtures
├── helpers/                           # Test utilities and infrastructure
│   ├── mod.rs                         # Helper module exports
│   ├── test_context.rs                # Unified test context
│   ├── telegram_mock.rs               # Mock Telegram API server
│   ├── database_helper.rs             # Database test utilities
│   └── test_data.rs                   # Test data creation helpers
├── integration/                       # Integration tests
│   ├── mod.rs                         # Integration test utilities
│   ├── handlers/                      # Handler-specific tests
│   │   ├── commands/                  # Command handler tests
│   │   │   ├── start_test.rs          # /start command tests
│   │   │   ├── help_test.rs           # /help command tests
│   │   │   └── events_test.rs         # /events command tests
│   │   └── callbacks/                 # Callback handler tests
│   │       ├── language_test.rs       # Language selection tests
│   │       └── location_test.rs       # Location selection tests
│   └── scenarios/                     # End-to-end scenario tests
│       ├── onboarding_test.rs         # User onboarding scenarios
│       └── complete_user_journey_test.rs # Complete user journeys
└── [legacy test files]               # Older test files (to be migrated)
```

## Test Categories

### 1. Unit Tests
- **Location**: Throughout the `src/` directory alongside source code
- **Purpose**: Test individual functions and methods in isolation
- **Run with**: `cargo test --lib`

### 2. Integration Tests
- **Location**: `tests/integration/`
- **Purpose**: Test interactions between components and full workflows
- **Run with**: `cargo test --test integration`

### 3. Handler Tests
- **Location**: `tests/integration/handlers/`
- **Purpose**: Test specific command and callback handlers
- **Coverage**:
  - Command handlers (`/start`, `/help`, `/events`, etc.)
  - Callback handlers (language selection, location selection, etc.)
  - Error handling and edge cases
  - Different user contexts (private chat, group chat)

### 4. Scenario Tests
- **Location**: `tests/integration/scenarios/`
- **Purpose**: Test complete user workflows and journeys
- **Coverage**:
  - Complete onboarding flow
  - Multi-step user interactions
  - Cross-feature functionality
  - Error recovery scenarios

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Run only integration tests
cargo test --test integration

# Run only unit tests
cargo test --lib

# Run tests for a specific module
cargo test handlers::commands::start

# Run a specific test
cargo test test_start_command_new_user_private_chat
```

### Run Tests with Output
```bash
# Show test output
cargo test -- --nocapture

# Show test output with debug logging
RUST_LOG=debug cargo test -- --nocapture
```

### Run Tests in Serial (for database tests)
```bash
# Most integration tests use serial execution to avoid database conflicts
cargo test --test integration -- --test-threads=1
```

## Test Environment Setup

### Prerequisites
1. **PostgreSQL Database**: Tests require a PostgreSQL database for integration tests
2. **Redis** (optional): Some tests use Redis for state management
3. **Environment Variables**: Set up test-specific environment variables

### Environment Variables
```bash
# Database configuration
export TEST_DATABASE_URL="postgresql://username:password@localhost/swingbuddy_test"

# Redis configuration (optional)
export TEST_REDIS_URL="redis://localhost:6379"

# Telegram Bot Token (for mock server)
export TEST_BOT_TOKEN="12345:test_token"
```

### Database Setup
```bash
# Create test database
createdb swingbuddy_test

# Run migrations
sqlx migrate run --database-url $TEST_DATABASE_URL
```

## Test Infrastructure

### TestContext
The `TestContext` provides a unified testing environment that includes:
- **Database**: Isolated test database with automatic cleanup
- **Mock Telegram API**: Simulated Telegram Bot API responses
- **Redis**: Optional Redis instance for state management
- **Application State**: Fully configured application context
- **Fixtures**: Pre-loaded test data

### Mock Telegram API
The mock Telegram API server simulates real Telegram Bot API responses:
- **Success scenarios**: Normal API responses
- **Error scenarios**: API errors and failures
- **Timeout scenarios**: Delayed responses for timeout testing
- **Request verification**: Verify API calls were made correctly

### Test Fixtures
Pre-defined test data for consistent testing:
- **Users**: Test users with different configurations
- **Events**: Sample events for testing event functionality
- **Groups**: Test groups for group-related features

## Writing New Tests

### Integration Test Template
```rust
use serial_test::serial;
use crate::helpers::{TestContext, TestConfig};

#[tokio::test]
#[serial]
async fn test_your_functionality() {
    let config = TestConfig {
        use_database: true,
        use_redis: true,
        setup_default_mocks: true,
        bot_token: None,
    };
    
    let ctx = TestContext::new_with_config(config).await
        .expect("Failed to create test context");
    
    // Your test logic here
    
    ctx.cleanup().await.expect("Failed to cleanup test context");
}
```

### Using Test Helpers
```rust
use crate::helpers::{create_simple_test_message, create_simple_test_callback_query};

// Create test message
let message = create_simple_test_message(user_id, chat_id, "/start");

// Create test callback
let callback = create_simple_test_callback_query(user_id, chat_id, "lang:en");
```

### Using Test Fixtures
```rust
use crate::fixtures::{load_test_fixtures, TestFixtures};

// Load all test fixtures
let fixtures = load_test_fixtures(ctx.db_pool()).await?;

// Use specific fixture data
let admin_user = &fixtures.users.admin_user;
```

## Test Best Practices

### 1. Test Isolation
- Each test should be independent and not rely on other tests
- Use `#[serial]` attribute for tests that modify shared resources
- Clean up test data after each test

### 2. Comprehensive Coverage
- Test both success and failure scenarios
- Test edge cases and boundary conditions
- Test different user contexts (new users, existing users, admins)

### 3. Realistic Test Data
- Use realistic user names, locations, and input data
- Test with different languages and character sets
- Include both valid and invalid input scenarios

### 4. Error Handling
- Test error scenarios explicitly
- Verify graceful error handling
- Test recovery from error states

### 5. Performance Considerations
- Keep tests fast and efficient
- Use mocks for external dependencies
- Avoid unnecessary database operations

## Debugging Tests

### Common Issues

#### Database Connection Errors
```bash
# Check database is running
pg_isready

# Verify connection string
psql $TEST_DATABASE_URL
```

#### Redis Connection Errors
```bash
# Check Redis is running
redis-cli ping

# Verify Redis URL
redis-cli -u $TEST_REDIS_URL ping
```

#### Test Timeouts
- Increase timeout for slow tests
- Check for deadlocks in concurrent tests
- Verify mock server responses

### Debug Logging
```bash
# Enable debug logging for tests
RUST_LOG=debug cargo test test_name -- --nocapture

# Enable trace logging for detailed output
RUST_LOG=trace cargo test test_name -- --nocapture
```

### Test Data Inspection
```bash
# Connect to test database to inspect data
psql $TEST_DATABASE_URL

# Check test tables
\dt
SELECT * FROM users;
SELECT * FROM events;
```

## Continuous Integration

### GitHub Actions
The test suite is configured to run automatically on:
- Pull requests
- Pushes to main branch
- Scheduled runs (daily)

### Test Matrix
Tests run against multiple configurations:
- Different Rust versions
- Different PostgreSQL versions
- With and without Redis

## Contributing

### Adding New Tests
1. **Identify test category**: Unit, integration, or scenario test
2. **Choose appropriate location**: Follow the directory structure
3. **Use existing helpers**: Leverage test infrastructure
4. **Follow naming conventions**: Use descriptive test names
5. **Add documentation**: Document complex test scenarios

### Test Naming Conventions
- Use descriptive names: `test_start_command_new_user_private_chat`
- Include context: `test_language_callback_invalid_language`
- Specify scenario: `test_onboarding_with_location_skip`

### Code Review Checklist
- [ ] Tests cover both success and failure cases
- [ ] Tests are properly isolated and use `#[serial]` when needed
- [ ] Test data is cleaned up properly
- [ ] Tests have descriptive names and documentation
- [ ] Tests follow existing patterns and conventions

## Troubleshooting

### Test Failures

#### "Database connection failed"
- Ensure PostgreSQL is running
- Check `TEST_DATABASE_URL` environment variable
- Verify database exists and migrations are applied

#### "Redis connection failed"
- Ensure Redis is running (if using Redis tests)
- Check `TEST_REDIS_URL` environment variable
- Some tests can run without Redis

#### "Mock server errors"
- Check for port conflicts
- Verify mock server setup in test configuration
- Review mock response configurations

#### "Test data conflicts"
- Ensure tests use unique user IDs
- Check for proper test cleanup
- Use `#[serial]` for tests that modify shared state

### Performance Issues

#### Slow test execution
- Run tests in parallel where possible
- Use minimal test configurations
- Optimize database operations

#### Memory usage
- Clean up test resources properly
- Avoid creating unnecessary test data
- Monitor test resource usage

For additional help, check the project documentation or create an issue in the repository.