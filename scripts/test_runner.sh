#!/bin/bash

# Todo API Test Suite Runner
# This script provides comprehensive testing capabilities for the Todo API

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to run unit tests
run_unit_tests() {
    print_status "Running unit tests..."
    cargo test unit_ --lib --bins
    if [ $? -eq 0 ]; then
        print_success "Unit tests passed!"
    else
        print_error "Unit tests failed!"
        exit 1
    fi
}

# Function to run integration tests
run_integration_tests() {
    print_status "Running integration tests..."
    cargo test integration_
    if [ $? -eq 0 ]; then
        print_success "Integration tests passed!"
    else
        print_error "Integration tests failed!"
        exit 1
    fi
}

# Function to run all tests
run_all_tests() {
    print_status "Running all tests..."
    cargo test
    if [ $? -eq 0 ]; then
        print_success "All tests passed!"
    else
        print_error "Some tests failed!"
        exit 1
    fi
}

# Function to run benchmarks
run_benchmarks() {
    print_status "Running performance benchmarks..."
    cargo bench
    if [ $? -eq 0 ]; then
        print_success "Benchmarks completed!"
    else
        print_warning "Benchmarks encountered issues!"
    fi
}

# Function to run tests with coverage
run_coverage() {
    print_status "Running tests with coverage..."
    
    # Install cargo-llvm-cov if not present
    if ! command -v cargo-llvm-cov &> /dev/null; then
        print_status "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov
    fi
    
    # Clean previous coverage data
    cargo llvm-cov clean
    
    # Run tests with coverage
    cargo llvm-cov --html --output-dir coverage/
    
    if [ $? -eq 0 ]; then
        print_success "Coverage report generated in coverage/ directory"
        print_status "Open coverage/index.html in your browser to view the report"
    else
        print_error "Coverage generation failed!"
        exit 1
    fi
}

# Function to run linting
run_lint() {
    print_status "Running linting checks..."
    
    # Run clippy
    cargo clippy -- -D warnings
    if [ $? -eq 0 ]; then
        print_success "Clippy checks passed!"
    else
        print_error "Clippy found issues!"
        exit 1
    fi
    
    # Run fmt check
    cargo fmt -- --check
    if [ $? -eq 0 ]; then
        print_success "Code formatting is correct!"
    else
        print_error "Code formatting issues found! Run 'cargo fmt' to fix."
        exit 1
    fi
}

# Function to run security audit
run_audit() {
    print_status "Running security audit..."
    
    # Install cargo-audit if not present
    if ! command -v cargo-audit &> /dev/null; then
        print_status "Installing cargo-audit..."
        cargo install cargo-audit
    fi
    
    cargo audit
    if [ $? -eq 0 ]; then
        print_success "Security audit passed!"
    else
        print_warning "Security audit found issues!"
    fi
}

# Function to show test statistics
show_test_stats() {
    print_status "Generating test statistics..."
    
    echo ""
    echo "=== Test Statistics ==="
    echo "Unit Tests:"
    cargo test unit_ --lib --bins -- --list | grep -c "test"
    
    echo "Integration Tests:"
    cargo test integration_ -- --list | grep -c "test"
    
    echo "Total Tests:"
    cargo test -- --list | grep -c "test"
    
    echo "Benchmark Tests:"
    if [ -d "benches" ]; then
        find benches -name "*.rs" -exec grep -l "fn.*benchmark" {} \; | wc -l
    else
        echo "0"
    fi
    echo ""
}

# Function to clean test artifacts
clean_artifacts() {
    print_status "Cleaning test artifacts..."
    cargo clean
    rm -rf coverage/
    rm -f *.profraw
    rm -rf target/criterion/
    print_success "Test artifacts cleaned!"
}

# Function to watch tests (requires cargo-watch)
watch_tests() {
    print_status "Starting test watcher..."
    
    if ! command -v cargo-watch &> /dev/null; then
        print_status "Installing cargo-watch..."
        cargo install cargo-watch
    fi
    
    cargo watch -x test
}

# Function to show help
show_help() {
    echo "Todo API Test Suite Runner"
    echo ""
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  unit           Run only unit tests"
    echo "  integration    Run only integration tests"
    echo "  all            Run all tests (default)"
    echo "  bench          Run performance benchmarks"
    echo "  coverage       Run tests with coverage report"
    echo "  lint           Run linting checks (clippy + fmt)"
    echo "  audit          Run security audit"
    echo "  stats          Show test statistics"
    echo "  clean          Clean test artifacts"
    echo "  watch          Watch for changes and run tests"
    echo "  ci             Run full CI pipeline (lint + audit + test + coverage)"
    echo "  help           Show this help message"
    echo ""
}

# Function to run full CI pipeline
run_ci() {
    print_status "Running full CI pipeline..."
    
    echo "Step 1/4: Linting"
    run_lint
    
    echo "Step 2/4: Security Audit"
    run_audit
    
    echo "Step 3/4: All Tests"
    run_all_tests
    
    echo "Step 4/4: Coverage"
    run_coverage
    
    print_success "CI pipeline completed successfully!"
}

# Main execution
case "${1:-all}" in
    "unit")
        run_unit_tests
        ;;
    "integration")
        run_integration_tests
        ;;
    "all")
        run_all_tests
        ;;
    "bench")
        run_benchmarks
        ;;
    "coverage")
        run_coverage
        ;;
    "lint")
        run_lint
        ;;
    "audit")
        run_audit
        ;;
    "stats")
        show_test_stats
        ;;
    "clean")
        clean_artifacts
        ;;
    "watch")
        watch_tests
        ;;
    "ci")
        run_ci
        ;;
    "help")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac