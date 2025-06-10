#!/bin/bash

# SwingBuddy Test Runner Script
# This script sets up the test environment and runs all tests with proper configuration

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
TEST_DB_NAME="swingbuddy_test"
DEFAULT_DATABASE_URL="postgresql://localhost/$TEST_DB_NAME"
DEFAULT_REDIS_URL="redis://localhost:6379"

# Functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependency() {
    local cmd=$1
    local name=$2
    
    if command -v "$cmd" >/dev/null 2>&1; then
        log_success "$name is available"
        return 0
    else
        log_error "$name is not available. Please install $name."
        return 1
    fi
}

check_service() {
    local service=$1
    local check_cmd=$2
    
    if eval "$check_cmd" >/dev/null 2>&1; then
        log_success "$service is running"
        return 0
    else
        log_warning "$service is not running or not accessible"
        return 1
    fi
}

setup_database() {
    log_info "Setting up test database..."
    
    # Check if database exists
    if psql -lqt | cut -d \| -f 1 | grep -qw "$TEST_DB_NAME"; then
        log_info "Test database '$TEST_DB_NAME' already exists"
    else
        log_info "Creating test database '$TEST_DB_NAME'..."
        createdb "$TEST_DB_NAME" || {
            log_error "Failed to create test database"
            return 1
        }
        log_success "Test database created"
    fi
    
    # Run migrations
    log_info "Running database migrations..."
    export DATABASE_URL="${TEST_DATABASE_URL:-$DEFAULT_DATABASE_URL}"
    
    if command -v sqlx >/dev/null 2>&1; then
        sqlx migrate run --database-url "$DATABASE_URL" || {
            log_error "Failed to run migrations"
            return 1
        }
        log_success "Database migrations completed"
    else
        log_warning "sqlx-cli not found. Skipping migrations."
        log_info "Install with: cargo install sqlx-cli"
    fi
}

cleanup_database() {
    log_info "Cleaning up test database..."
    
    # Clean test data but keep schema
    export DATABASE_URL="${TEST_DATABASE_URL:-$DEFAULT_DATABASE_URL}"
    
    psql "$DATABASE_URL" -c "
        TRUNCATE TABLE users, events, groups, admins RESTART IDENTITY CASCADE;
    " 2>/dev/null || log_warning "Could not clean test data (tables may not exist yet)"
}

validate_environment() {
    log_info "Validating test environment..."
    
    # Check required tools
    local deps_ok=true
    
    check_dependency "cargo" "Rust/Cargo" || deps_ok=false
    check_dependency "psql" "PostgreSQL client" || deps_ok=false
    
    if [ "$deps_ok" = false ]; then
        log_error "Missing required dependencies"
        return 1
    fi
    
    # Check services
    local services_ok=true
    
    check_service "PostgreSQL" "pg_isready" || {
        log_error "PostgreSQL is required for tests"
        services_ok=false
    }
    
    # Redis is optional
    check_service "Redis" "redis-cli ping" || {
        log_warning "Redis is not available. Some tests may be skipped."
    }
    
    if [ "$services_ok" = false ]; then
        log_error "Required services are not available"
        return 1
    fi
    
    log_success "Environment validation completed"
}

setup_environment() {
    log_info "Setting up test environment variables..."
    
    # Set default environment variables if not already set
    export TEST_DATABASE_URL="${TEST_DATABASE_URL:-$DEFAULT_DATABASE_URL}"
    export TEST_REDIS_URL="${TEST_REDIS_URL:-$DEFAULT_REDIS_URL}"
    export TEST_BOT_TOKEN="${TEST_BOT_TOKEN:-12345:test_token}"
    export RUST_LOG="${RUST_LOG:-info}"
    
    log_info "Environment variables:"
    log_info "  TEST_DATABASE_URL: $TEST_DATABASE_URL"
    log_info "  TEST_REDIS_URL: $TEST_REDIS_URL"
    log_info "  TEST_BOT_TOKEN: ${TEST_BOT_TOKEN:0:10}..."
    log_info "  RUST_LOG: $RUST_LOG"
}

run_tests() {
    local test_type="$1"
    local test_args="${@:2}"
    
    cd "$PROJECT_ROOT"
    
    case "$test_type" in
        "unit")
            log_info "Running unit tests..."
            cargo test --lib $test_args
            ;;
        "integration")
            log_info "Running integration tests..."
            cargo test --test integration $test_args
            ;;
        "all")
            log_info "Running all tests..."
            cargo test $test_args
            ;;
        "specific")
            log_info "Running specific test: $test_args"
            cargo test $test_args
            ;;
        *)
            log_error "Unknown test type: $test_type"
            return 1
            ;;
    esac
}

show_usage() {
    echo "Usage: $0 [OPTIONS] [TEST_TYPE] [TEST_ARGS...]"
    echo ""
    echo "Test Types:"
    echo "  unit         Run unit tests only"
    echo "  integration  Run integration tests only"
    echo "  all          Run all tests (default)"
    echo "  specific     Run specific test (provide test name as argument)"
    echo ""
    echo "Options:"
    echo "  --setup-only     Only setup environment, don't run tests"
    echo "  --cleanup-only   Only cleanup test data"
    echo "  --no-setup       Skip environment setup"
    echo "  --no-cleanup     Skip cleanup after tests"
    echo "  --verbose        Show detailed test output"
    echo "  --help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                                    # Run all tests"
    echo "  $0 unit                              # Run unit tests only"
    echo "  $0 integration                       # Run integration tests only"
    echo "  $0 specific test_start_command       # Run specific test"
    echo "  $0 --verbose all                     # Run all tests with verbose output"
    echo "  $0 --setup-only                      # Setup environment only"
    echo ""
    echo "Environment Variables:"
    echo "  TEST_DATABASE_URL   PostgreSQL connection string"
    echo "  TEST_REDIS_URL      Redis connection string"
    echo "  TEST_BOT_TOKEN      Telegram bot token for tests"
    echo "  RUST_LOG           Logging level (debug, info, warn, error)"
}

# Main script
main() {
    local setup_only=false
    local cleanup_only=false
    local no_setup=false
    local no_cleanup=false
    local verbose=false
    local test_type="all"
    local test_args=""
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --setup-only)
                setup_only=true
                shift
                ;;
            --cleanup-only)
                cleanup_only=true
                shift
                ;;
            --no-setup)
                no_setup=true
                shift
                ;;
            --no-cleanup)
                no_cleanup=true
                shift
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --help)
                show_usage
                exit 0
                ;;
            unit|integration|all|specific)
                test_type="$1"
                shift
                test_args="$*"
                break
                ;;
            *)
                test_args="$*"
                break
                ;;
        esac
    done
    
    # Add verbose flag to test args if requested
    if [ "$verbose" = true ]; then
        test_args="$test_args -- --nocapture"
    fi
    
    log_info "SwingBuddy Test Runner"
    log_info "======================"
    
    # Handle cleanup-only mode
    if [ "$cleanup_only" = true ]; then
        setup_environment
        cleanup_database
        log_success "Cleanup completed"
        exit 0
    fi
    
    # Setup environment
    if [ "$no_setup" = false ]; then
        validate_environment || exit 1
        setup_environment
        setup_database || exit 1
        cleanup_database  # Clean any existing test data
    fi
    
    # Handle setup-only mode
    if [ "$setup_only" = true ]; then
        log_success "Environment setup completed"
        exit 0
    fi
    
    # Run tests
    log_info "Starting test execution..."
    local test_start_time=$(date +%s)
    
    if run_tests "$test_type" $test_args; then
        local test_end_time=$(date +%s)
        local test_duration=$((test_end_time - test_start_time))
        
        log_success "All tests completed successfully in ${test_duration}s"
        
        # Cleanup after tests
        if [ "$no_cleanup" = false ]; then
            cleanup_database
            log_success "Test cleanup completed"
        fi
        
        exit 0
    else
        log_error "Tests failed"
        
        # Cleanup even on failure
        if [ "$no_cleanup" = false ]; then
            cleanup_database
        fi
        
        exit 1
    fi
}

# Run main function with all arguments
main "$@"