-- Initial database schema for SwingBuddy Telegram bot
-- Based on the architecture document

-- Users table
CREATE TABLE users (
    id BIGSERIAL PRIMARY KEY,
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
    id BIGSERIAL PRIMARY KEY,
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
CREATE INDEX idx_events_created_by ON events(created_by);
CREATE INDEX idx_group_members_group ON group_members(group_id);
CREATE INDEX idx_group_members_user ON group_members(user_id);
CREATE INDEX idx_event_participants_event ON event_participants(event_id);
CREATE INDEX idx_event_participants_user ON event_participants(user_id);
CREATE INDEX idx_user_states_scenario ON user_states(scenario);
CREATE INDEX idx_user_states_expires ON user_states(expires_at);
CREATE INDEX idx_cas_checks_user ON cas_checks(user_id);
CREATE INDEX idx_cas_checks_telegram ON cas_checks(telegram_id);
CREATE INDEX idx_cas_checks_date ON cas_checks(checked_at);

-- Additional indexes for common queries
CREATE INDEX idx_users_banned ON users(is_banned);
CREATE INDEX idx_groups_active ON groups(is_active);
CREATE INDEX idx_events_active ON events(is_active);
CREATE INDEX idx_events_upcoming ON events(event_date) WHERE is_active = true AND event_date > NOW();