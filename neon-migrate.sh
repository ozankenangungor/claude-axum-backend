#!/bin/bash

# Neon PostgreSQL Migration Management Script
# Supports Neon branching and serverless optimizations

set -euo pipefail

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
NEON_BRANCH=${NEON_BRANCH:-"main"}
NEON_API_KEY=${NEON_API_KEY:-""}
NEON_PROJECT_ID=${NEON_PROJECT_ID:-""}

print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

# Check if we have a Neon database URL
check_neon_connection() {
    if [[ "$DATABASE_URL" =~ .*neon\.tech.* ]]; then
        print_info "Detected Neon PostgreSQL database"
        return 0
    else
        print_warning "Not using Neon PostgreSQL - using standard migration process"
        return 1
    fi
}

# Wait for Neon compute to be ready (cold start handling)
wait_for_neon_ready() {
    print_info "Waiting for Neon compute to be ready (handling cold start)..."
    
    local max_attempts=10
    local attempt=1
    
    while [ $attempt -le $max_attempts ]; do
        if timeout 15s psql "$DATABASE_URL" -c "SELECT 1;" >/dev/null 2>&1; then
            print_status "Neon compute is ready!"
            return 0
        fi
        
        print_info "Attempt $attempt/$max_attempts - waiting for Neon compute..."
        sleep 3
        attempt=$((attempt + 1))
    done
    
    print_error "Neon compute failed to become ready after $max_attempts attempts"
    return 1
}

# Run migrations with Neon-optimized settings
run_neon_migrations() {
    print_info "Running migrations on Neon PostgreSQL..."
    
    # Set Neon-specific environment variables
    export SQLX_OFFLINE=true
    export DATABASE_URL_TIMEOUT=30
    
    # Wait for Neon to be ready first
    if ! wait_for_neon_ready; then
        print_error "Cannot proceed with migrations - Neon compute not ready"
        return 1
    fi
    
    # Run migrations with timeout protection
    if timeout 120s cargo run --bin migrate; then
        print_status "Migrations completed successfully on Neon"
    else
        print_error "Migration timed out or failed"
        return 1
    fi
}

# Create development branch for testing
create_dev_branch() {
    if [ -z "$NEON_API_KEY" ] || [ -z "$NEON_PROJECT_ID" ]; then
        print_warning "NEON_API_KEY or NEON_PROJECT_ID not set - skipping branch creation"
        return 0
    fi
    
    local branch_name=${1:-"dev-$(date +%Y%m%d-%H%M%S)"}
    
    print_info "Creating Neon development branch: $branch_name"
    
    curl -X POST \
        "https://console.neon.tech/api/v2/projects/$NEON_PROJECT_ID/branches" \
        -H "Authorization: Bearer $NEON_API_KEY" \
        -H "Content-Type: application/json" \
        -d "{
            \"name\": \"$branch_name\",
            \"parent_branch_id\": \"main\"
        }" && print_status "Development branch '$branch_name' created"
}

# Main migration function
main() {
    echo -e "${BLUE}🗄️  Neon PostgreSQL Migration Manager${NC}"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    
    # Check if DATABASE_URL is set
    if [ -z "${DATABASE_URL:-}" ]; then
        print_error "DATABASE_URL environment variable is not set"
        exit 1
    fi
    
    # Check if we're using Neon
    if check_neon_connection; then
        # Neon-specific migration process
        run_neon_migrations
    else
        # Standard migration process
        print_info "Running standard PostgreSQL migrations..."
        cargo run --bin migrate
    fi
    
    print_status "Migration process completed!"
}

# Handle command line arguments
case "${1:-run}" in
    "run")
        main
        ;;
    "create-branch")
        create_dev_branch "${2:-}"
        ;;
    "wait")
        wait_for_neon_ready
        ;;
    "help")
        echo "Usage: $0 [command]"
        echo "Commands:"
        echo "  run           - Run migrations (default)"
        echo "  create-branch - Create a new Neon development branch"
        echo "  wait          - Wait for Neon compute to be ready"
        echo "  help          - Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac