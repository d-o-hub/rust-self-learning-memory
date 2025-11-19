#!/bin/bash
# Memory CLI Deployment Script
# This script helps deploy memory-cli in various environments

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        echo "linux"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    elif [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "win32" ]]; then
        echo "windows"
    else
        echo "unknown"
    fi
}

# Setup directories
setup_directories() {
    log_info "Setting up directories..."

    mkdir -p "$SCRIPT_DIR/config"
    mkdir -p "$SCRIPT_DIR/data"
    mkdir -p "$SCRIPT_DIR/backups"
    mkdir -p "$SCRIPT_DIR/logs"

    # Set permissions
    if [[ "$(detect_os)" == "linux" ]]; then
        chmod 755 "$SCRIPT_DIR/config"
        chmod 755 "$SCRIPT_DIR/data"
        chmod 755 "$SCRIPT_DIR/backups"
        chmod 755 "$SCRIPT_DIR/logs"
    fi

    log_success "Directories created"
}

# Setup configuration
setup_config() {
    log_info "Setting up configuration..."

    if [[ ! -f "$SCRIPT_DIR/config/memory-cli.toml" ]]; then
        cp "$SCRIPT_DIR/config/memory-cli.toml" "$SCRIPT_DIR/config/memory-cli.toml.example" 2>/dev/null || true
        log_warn "Configuration file not found. Please create $SCRIPT_DIR/config/memory-cli.toml"
        log_info "You can use the example configuration as a starting point"
    else
        log_success "Configuration file exists"
    fi

    if [[ ! -f "$SCRIPT_DIR/.env" ]]; then
        cp "$SCRIPT_DIR/.env.example" "$SCRIPT_DIR/.env" 2>/dev/null || true
        log_warn "Environment file not found. Please create $SCRIPT_DIR/.env"
        log_info "You can copy from .env.example and fill in your values"
    else
        log_success "Environment file exists"
    fi
}

# Build the application
build_app() {
    log_info "Building memory-cli..."

    if ! command_exists cargo; then
        log_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi

    cd "$PROJECT_ROOT"

    # Build with full features
    cargo build --release --features full

    if [[ ! -f "$PROJECT_ROOT/target/release/memory-cli" ]]; then
        log_error "Build failed"
        exit 1
    fi

    log_success "Build completed"
}

# Install the binary
install_binary() {
    local install_path="$1"

    if [[ -z "$install_path" ]]; then
        case "$(detect_os)" in
            "linux")
                install_path="/usr/local/bin"
                ;;
            "macos")
                install_path="/usr/local/bin"
                ;;
            *)
                install_path="$HOME/.local/bin"
                ;;
        esac
    fi

    log_info "Installing binary to $install_path..."

    # Create directory if it doesn't exist
    sudo mkdir -p "$install_path" 2>/dev/null || mkdir -p "$install_path"

    # Copy binary
    if [[ -w "$install_path" ]]; then
        cp "$PROJECT_ROOT/target/release/memory-cli" "$install_path/"
    else
        sudo cp "$PROJECT_ROOT/target/release/memory-cli" "$install_path/"
    fi

    # Make executable
    if [[ -w "$install_path" ]]; then
        chmod +x "$install_path/memory-cli"
    else
        sudo chmod +x "$install_path/memory-cli"
    fi

    log_success "Binary installed to $install_path/memory-cli"
}

# Setup systemd service (Linux only)
setup_systemd() {
    if [[ "$(detect_os)" != "linux" ]]; then
        log_info "Skipping systemd setup (not on Linux)"
        return
    fi

    if ! command_exists systemctl; then
        log_info "systemctl not found, skipping systemd setup"
        return
    fi

    log_info "Setting up systemd service..."

    # Create memory user if it doesn't exist
    if ! id -u memory >/dev/null 2>&1; then
        sudo useradd --create-home --shell /bin/bash memory
        log_info "Created memory user"
    fi

    # Create directories with proper ownership
    sudo mkdir -p /etc/memory-cli
    sudo mkdir -p /var/lib/memory-cli
    sudo mkdir -p /var/log/memory-cli
    sudo mkdir -p /var/backup/memory-cli

    sudo chown -R memory:memory /etc/memory-cli
    sudo chown -R memory:memory /var/lib/memory-cli
    sudo chown -R memory:memory /var/log/memory-cli
    sudo chown -R memory:memory /var/backup/memory-cli

    # Copy configuration
    if [[ -f "$SCRIPT_DIR/config/memory-cli.toml" ]]; then
        sudo cp "$SCRIPT_DIR/config/memory-cli.toml" /etc/memory-cli/
        sudo chown memory:memory /etc/memory-cli/memory-cli.toml
    fi

    if [[ -f "$SCRIPT_DIR/.env" ]]; then
        sudo cp "$SCRIPT_DIR/.env" /etc/memory-cli/environment
        sudo chown memory:memory /etc/memory-cli/environment
        sudo chmod 600 /etc/memory-cli/environment
    fi

    # Copy systemd service file
    sudo cp "$SCRIPT_DIR/systemd/memory-cli.service" /etc/systemd/system/

    # Reload systemd and enable service
    sudo systemctl daemon-reload
    sudo systemctl enable memory-cli

    log_success "Systemd service configured"
    log_info "Use 'sudo systemctl start memory-cli' to start the service"
}

# Setup Docker deployment
setup_docker() {
    if ! command_exists docker; then
        log_error "Docker is not installed"
        return
    fi

    log_info "Setting up Docker deployment..."

    if [[ -f "$SCRIPT_DIR/docker/docker-compose.yml" ]]; then
        cd "$SCRIPT_DIR/docker"
        docker-compose config >/dev/null
        log_success "Docker Compose configuration is valid"
        log_info "Use 'docker-compose up -d' to start services"
    else
        log_warn "Docker Compose configuration not found"
    fi
}

# Run initial validation
run_validation() {
    log_info "Running initial validation..."

    if command_exists memory-cli; then
        if memory-cli config validate >/dev/null 2>&1; then
            log_success "Configuration validation passed"
        else
            log_warn "Configuration validation failed - please check your config"
        fi
    else
        log_warn "memory-cli not found in PATH - install it first"
    fi
}

# Main deployment function
main() {
    local install_path=""
    local skip_build=false
    local skip_install=false
    local setup_systemd=false
    local setup_docker=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --install-path)
                install_path="$2"
                shift 2
                ;;
            --skip-build)
                skip_build=true
                shift
                ;;
            --skip-install)
                skip_install=true
                shift
                ;;
            --systemd)
                setup_systemd=true
                shift
                ;;
            --docker)
                setup_docker=true
                shift
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --install-path PATH    Install binary to custom path"
                echo "  --skip-build          Skip building the application"
                echo "  --skip-install        Skip installing the binary"
                echo "  --systemd            Setup systemd service (Linux only)"
                echo "  --docker             Setup Docker deployment"
                echo "  --help               Show this help"
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done

    log_info "Starting Memory CLI deployment..."

    # Setup directories
    setup_directories

    # Setup configuration
    setup_config

    # Build application
    if [[ "$skip_build" != true ]]; then
        build_app
    fi

    # Install binary
    if [[ "$skip_install" != true ]]; then
        install_binary "$install_path"
    fi

    # Setup systemd
    if [[ "$setup_systemd" == true ]]; then
        setup_systemd
    fi

    # Setup Docker
    if [[ "$setup_docker" == true ]]; then
        setup_docker
    fi

    # Run validation
    run_validation

    log_success "Deployment completed!"
    log_info ""
    log_info "Next steps:"
    log_info "1. Configure your database credentials in config/memory-cli.toml or .env"
    log_info "2. Run 'memory-cli config validate' to verify configuration"
    log_info "3. Run 'memory-cli health check' to verify connectivity"
    log_info "4. Start using the CLI with 'memory-cli --help'"
}

# Run main function with all arguments
main "$@"