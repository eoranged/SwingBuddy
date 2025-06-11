#!/bin/bash

# Integration test runner script for SwingBuddy
# This script sets up the environment and runs integration tests

set -e

echo "🚀 Starting SwingBuddy Integration Tests"

# Check if PostgreSQL is running
if ! pg_isready -h localhost -p 5432 >/dev/null 2>&1; then
    echo "❌ PostgreSQL is not running on localhost:5432"
    echo "Please start PostgreSQL before running integration tests"
    exit 1
fi

echo "✅ PostgreSQL is running"

# Set up environment variables
export DATABASE_URL="postgresql://test_user:test_password@localhost:5432/test_swingbuddy"

# Ensure test database exists and is clean
echo "🧹 Setting up test database..."
psql -h localhost -p 5432 -U eoranged -d test_swingbuddy -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public;" >/dev/null 2>&1
psql -h localhost -p 5432 -U eoranged -d test_swingbuddy -c "GRANT ALL ON SCHEMA public TO test_user;" >/dev/null 2>&1

# Run migrations
echo "🔄 Running database migrations..."
sqlx migrate run

echo "🧪 Running integration tests..."

# Run the tests
if [ $# -eq 0 ]; then
    # Run all integration tests
    cargo test --test integration_test
else
    # Run specific test
    cargo test --test integration_test "$1" -- --nocapture
fi

echo "✅ Integration tests completed successfully!"