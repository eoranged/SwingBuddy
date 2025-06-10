# SwingBuddy Telegram Bot 🕺💃

A comprehensive Telegram bot for swing dancing community management, built with Rust using the teloxide framework.

## Features

### 🎯 Core Functionality
- **User Onboarding**: Multi-language user registration with profile management
- **Group Management**: Automated group setup with permission verification
- **Event Calendars**: Integration with Google Calendar for dance events
- **Spam Protection**: CAS API integration for automatic user moderation
- **Admin Panel**: Comprehensive administration tools for community managers

### 🌍 Multi-Language Support
- **English** and **Russian** translations
- Dynamic language detection from Telegram user settings
- Easy extensibility for additional languages

### 🏗️ Architecture
- **Modular Design**: Clean separation of concerns with pluggable components
- **Async/Await**: Full async implementation using tokio
- **Database**: PostgreSQL with SQLx for type-safe queries
- **Caching**: Redis integration for performance optimization
- **State Management**: Context-aware conversation flows

## Quick Start

### Prerequisites
- Rust 1.70+ 
- PostgreSQL 13+
- Redis 6+
- Telegram Bot Token (from [@BotFather](https://t.me/botfather))

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/your-username/SwingBuddy.git
   cd SwingBuddy
   ```

2. **Set up the database**
   ```bash
   # Create PostgreSQL database
   createdb swingbuddy
   
   # Start Redis server
   redis-server
   ```

3. **Configure the bot**
   ```bash
   cp config.toml.example config.toml
   # Edit config.toml with your settings
   ```

4. **Run database migrations**
   ```bash
   cargo run --bin migrate
   ```

5. **Start the bot**
   ```bash
   cargo run
   ```

## Configuration

The bot is configured via `config.toml`. See [`config.toml.example`](config.toml.example) for all available options.

### Essential Settings

```toml
[bot]
token = "YOUR_BOT_TOKEN"

[database]
url = "postgresql://username:password@localhost/swingbuddy"

[redis]
url = "redis://localhost:6379"

[admins]
user_ids = [123456789, 987654321]  # Telegram user IDs of bot admins
```

## Usage

### User Commands
- `/start` - Begin user onboarding process
- `/help` - Show available commands
- `/events` - Browse dance events and calendars

### Admin Commands
- `/admin` - Access admin panel (admin only)
- `/stats` - Show bot statistics (admin only)

### User Onboarding Flow
1. **Language Selection**: Choose preferred language (English/Russian)
2. **Name Input**: Provide display name (defaults to Telegram name)
3. **Location**: Select city (Moscow, Saint Petersburg, or custom)
4. **Welcome**: Complete setup with personalized welcome message

### Group Setup
When added to a group, the bot will:
1. Check for required permissions (admin rights, delete messages, ban users)
2. Show setup instructions if permissions are missing
3. Allow language configuration for the group
4. Enable CAS protection for new members

## Development

### Project Structure

```
src/
├── config/          # Configuration management
├── database/        # Database models and repositories
├── handlers/        # Telegram update handlers
│   ├── commands/    # Command handlers (/start, /help, etc.)
│   ├── callbacks/   # Inline keyboard callbacks
│   └── messages/    # Message and member update handlers
├── i18n/           # Internationalization system
├── middleware/     # Request middleware
├── models/         # Data models
├── services/       # Business logic services
├── state/          # Conversation state management
└── utils/          # Utility functions

translations/       # Translation files (en.json, ru.json)
migrations/         # Database migration files
```

### Key Components

#### Services Layer
- **UserService**: User registration and profile management
- **CASService**: Spam protection via CAS API
- **GoogleService**: Calendar integration
- **AuthService**: Permission and role management
- **NotificationService**: Message formatting and delivery

#### State Management
- **ScenarioManager**: Conversation flow orchestration
- **StateStorage**: Redis-backed state persistence
- **ConversationContext**: User interaction context

#### Database Layer
- **Repositories**: Data access layer with CRUD operations
- **Models**: Type-safe data structures
- **Migrations**: Database schema versioning

### Adding New Features

1. **Create a new service** in `src/services/`
2. **Add database models** in `src/models/` if needed
3. **Implement handlers** in `src/handlers/`
4. **Add translations** in `translations/`
5. **Update configuration** if required

### Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out html

# Run specific test module
cargo test services::user
```

## Deployment

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/SwingBuddy /usr/local/bin/
CMD ["SwingBuddy"]
```

### Environment Variables

```bash
export SWINGBUDDY_BOT__TOKEN="your_bot_token"
export SWINGBUDDY_DATABASE__URL="postgresql://..."
export SWINGBUDDY_REDIS__URL="redis://..."
```

## API Integration

### CAS API
The bot integrates with [CAS (Combot Anti-Spam)](https://cas.chat/) for automatic spam protection:
- Checks new group members against CAS database
- Automatically bans users listed in CAS
- Caches results in Redis for performance

### Google Calendar
Integration with Google Calendar API for event management:
- Create and manage dance events
- Generate calendar sharing URLs
- Export events in iCal format

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Ensure all tests pass (`cargo test`)
- Add documentation for public APIs
- Use conventional commit messages

## Architecture

The bot follows a modular, service-oriented architecture:

- **Handlers**: Process Telegram updates and route to appropriate services
- **Services**: Implement business logic and external API integration
- **Repositories**: Provide data access abstraction
- **State Management**: Handle conversation flows and user context
- **Middleware**: Cross-cutting concerns (auth, logging, rate limiting)

For detailed architecture documentation, see [`ARCHITECTURE.md`](ARCHITECTURE.md).

## Database Schema

The bot uses PostgreSQL with the following main tables:
- `users` - User profiles and preferences
- `groups` - Group configurations and settings
- `events` - Dance events and calendar entries
- `admin_settings` - System configuration
- `user_states` - Conversation state (also cached in Redis)

For complete schema details, see [`DATABASE_README.md`](DATABASE_README.md).

## Monitoring and Logging

The bot includes comprehensive logging and monitoring:
- Structured logging with tracing
- Health checks for all services
- Performance metrics
- Error tracking and alerting

## Security

- Input validation and sanitization
- Rate limiting on API calls
- Secure configuration management
- Role-based access control
- CAS integration for spam protection

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- 📖 [Documentation](https://github.com/your-username/SwingBuddy/wiki)
- 🐛 [Issue Tracker](https://github.com/your-username/SwingBuddy/issues)
- 💬 [Discussions](https://github.com/your-username/SwingBuddy/discussions)

## Acknowledgments

- [teloxide](https://github.com/teloxide/teloxide) - Telegram bot framework
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [CAS API](https://cas.chat/) - Anti-spam service
- Swing dancing community for inspiration and feedback

---

Made with ❤️ for the swing dancing community