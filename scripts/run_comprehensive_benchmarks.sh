#!/usr/bin/env bash
# Comprehensive benchmark runner for rust-self-learning-memory
# Runs all benchmark suites with proper resource management and reporting

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
BENCH_TIMEOUT=1800  # 30 minutes max per benchmark suite
MEMORY_LIMIT="4G"   # Memory limit for benchmarks

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Benchmark suites to run
BENCHMARK_SUITES=(
    "episode_lifecycle:Episode lifecycle operations"
    "pattern_extraction:Pattern extraction and matching"
    "storage_operations:Basic storage operations (redb only)"
    "concurrent_operations:Concurrent read/write operations"
    "memory_pressure:Memory usage and pressure testing"
    "scalability:Scalability across different dimensions"
    "multi_backend_comparison:Performance comparison across storage backends"
)

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

check_dependencies() {
    log_info "Checking dependencies..."

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo not found. Please install Rust."
        exit 1
    fi

    if ! command -v jq &> /dev/null; then
        log_warning "jq not found. JSON processing will be limited."
    fi

    log_success "Dependencies check passed"
}

setup_environment() {
    log_info "Setting up benchmark environment..."

    cd "$PROJECT_ROOT"

    # Clean previous results
    rm -rf target/criterion.bak
    [ -d target/criterion ] && mv target/criterion target/criterion.bak

    # Set resource limits for consistent benchmarking
    ulimit -m $(($(echo $MEMORY_LIMIT | sed 's/G/*1024*1024/'))) 2>/dev/null || true
    ulimit -v $(($(echo $MEMORY_LIMIT | sed 's/G/*1024*1024/'))) 2>/dev/null || true

    # Build benchmark dependencies
    log_info "Building benchmark dependencies..."
    cargo build --release --bins

    log_success "Environment setup complete"
}

run_benchmark_suite() {
    local suite_name="$1"
    local suite_description="$2"

    log_info "Running $suite_name: $suite_description"

    local start_time=$(date +%s)
    local output_file="target/criterion/${suite_name}/benchmark_output.log"

    mkdir -p "target/criterion/${suite_name}"

    # Run benchmark with timeout and capture output
    if timeout $BENCH_TIMEOUT cargo bench -p memory-benches --bench "$suite_name" --quiet > "$output_file" 2>&1; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))

        log_success "$suite_name completed in ${duration}s"

        # Extract key metrics from output
        extract_benchmark_metrics "$suite_name" "$output_file"

        return 0
    else
        local exit_code=$?
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))

        if [ $exit_code -eq 124 ]; then
            log_warning "$suite_name timed out after ${duration}s"
        else
            log_error "$suite_name failed after ${duration}s"
        fi

        echo "Last 20 lines of output:" >&2
        tail -20 "$output_file" >&2

        return 1
    fi
}

extract_benchmark_metrics() {
    local suite_name="$1"
    local output_file="$2"

    local metrics_file="target/criterion/${suite_name}/metrics.json"

    # Extract basic timing information from Criterion output
    {
        echo "{"
        echo "  \"suite\": \"$suite_name\","
        echo "  \"timestamp\": \"$(date -Iseconds)\","

        # Try to extract some basic metrics from the output
        if grep -q "time:" "$output_file"; then
            echo "  \"has_timing_data\": true,"
        else
            echo "  \"has_timing_data\": false,"
        fi

        echo "  \"output_size\": $(stat -f%z "$output_file" 2>/dev/null || stat -c%s "$output_file" 2>/dev/null || echo 0)"
        echo "}"
    } > "$metrics_file"
}

generate_report() {
    log_info "Generating comprehensive benchmark report..."

    local report_file="benchmark_report_$(date +%Y%m%d_%H%M%S).md"

    {
        echo "# Comprehensive Benchmark Report"
        echo ""
        echo "Generated: $(date)"
        echo "Project: rust-self-learning-memory"
        echo "Commit: $(git rev-parse HEAD 2>/dev/null || echo 'unknown')"
        echo ""

        echo "## Environment"
        echo ""
        echo "- **OS**: $(uname -s) $(uname -r)"
        echo "- **CPU**: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || grep -m1 'model name' /proc/cpuinfo | cut -d: -f2 | xargs 2>/dev/null || echo 'Unknown')"
        echo "- **Memory**: $(sysctl -n hw.memsize 2>/dev/null || grep MemTotal /proc/meminfo | awk '{print $2/1024/1024 " GB"}' 2>/dev/null || echo 'Unknown')"
        echo "- **Rust**: $(cargo --version)"
        echo ""

        echo "## Benchmark Suites"
        echo ""

        local total_suites=${#BENCHMARK_SUITES[@]}
        local completed_suites=0
        local failed_suites=0

        for suite_info in "${BENCHMARK_SUITES[@]}"; do
            IFS=':' read -r suite_name suite_description <<< "$suite_info"

            if [ -d "target/criterion/${suite_name}" ]; then
                echo "### ✅ $suite_name"
                echo "**Description**: $suite_description"
                echo "**Status**: Completed"

                if [ -f "target/criterion/${suite_name}/metrics.json" ]; then
                    echo "**Metrics**: Available"
                fi

                echo ""
                ((completed_suites++))
            else
                echo "### ❌ $suite_name"
                echo "**Description**: $suite_description"
                echo "**Status**: Failed or not run"
                echo ""
                ((failed_suites++))
            fi
        done

        echo "## Summary"
        echo ""
        echo "- **Total Suites**: $total_suites"
        echo "- **Completed**: $completed_suites"
        echo "- **Failed**: $failed_suites"
        echo ""

        if [ $completed_suites -gt 0 ]; then
            echo "## Recommendations"
            echo ""
            echo "1. **Performance Regression Detection**: Compare results with previous runs"
            echo "2. **Memory Analysis**: Review memory pressure benchmarks for optimization opportunities"
            echo "3. **Scalability Testing**: Analyze concurrent and scalability benchmarks for bottlenecks"
            echo "4. **Backend Comparison**: Use multi-backend results to inform storage decisions"
            echo ""

            echo "## Next Steps"
            echo ""
            echo "1. Run benchmarks on target deployment environment"
            echo "2. Set up automated performance regression alerts"
            echo "3. Profile memory usage in production scenarios"
            echo "4. Monitor concurrent operation performance under real load"
            echo ""
        fi

        echo "---"
        echo "Report generated by comprehensive benchmark runner"
    } > "$report_file"

    log_success "Report generated: $report_file"
}

main() {
    log_info "Starting comprehensive benchmark suite for rust-self-learning-memory"

    check_dependencies
    setup_environment

    local failed_suites=0
    local total_suites=${#BENCHMARK_SUITES[@]}

    for suite_info in "${BENCHMARK_SUITES[@]}"; do
        IFS=':' read -r suite_name suite_description <<< "$suite_info"

        if run_benchmark_suite "$suite_name" "$suite_description"; then
            log_success "Suite $suite_name completed successfully"
        else
            log_error "Suite $suite_name failed"
            ((failed_suites++))
        fi

        echo ""
    done

    generate_report

    echo ""
    log_info "Benchmark suite completed"
    log_info "Total suites: $total_suites"
    log_info "Successful: $((total_suites - failed_suites))"
    log_info "Failed: $failed_suites"

    if [ $failed_suites -eq 0 ]; then
        log_success "All benchmark suites completed successfully!"
        exit 0
    else
        log_warning "$failed_suites benchmark suite(s) failed"
        exit 1
    fi
}

# Allow running specific benchmark suites
if [ $# -gt 0 ]; then
    case "$1" in
        "list")
            echo "Available benchmark suites:"
            for suite_info in "${BENCHMARK_SUITES[@]}"; do
                IFS=':' read -r suite_name suite_description <<< "$suite_info"
                echo "  $suite_name - $suite_description"
            done
            exit 0
            ;;
        "run")
            if [ -z "$2" ]; then
                log_error "Please specify a benchmark suite to run"
                echo "Use '$0 list' to see available suites"
                exit 1
            fi

            for suite_info in "${BENCHMARK_SUITES[@]}"; do
                IFS=':' read -r suite_name suite_description <<< "$suite_info"
                if [ "$suite_name" = "$2" ]; then
                    check_dependencies
                    setup_environment
                    if run_benchmark_suite "$suite_name" "$suite_description"; then
                        log_success "Suite $suite_name completed successfully"
                        exit 0
                    else
                        log_error "Suite $suite_name failed"
                        exit 1
                    fi
                fi
            done

            log_error "Benchmark suite '$2' not found"
            echo "Use '$0 list' to see available suites"
            exit 1
            ;;
        *)
            log_error "Unknown command: $1"
            echo "Usage: $0 [list|run <suite_name>]"
            echo "Run without arguments to execute all benchmark suites"
            exit 1
            ;;
    esac
else
    main
fi