#!/bin/bash

##############################################################################
# AirTally REST API - Automated Deployment Script
# Version: 1.0
# Usage: ./deploy.sh [options]
# Options:
#   --update    Update existing installation
#   --backup    Backup database before deployment
#   --help      Show this help message
##############################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APP_NAME="airtally-api"
APP_DIR="$HOME/airtally/airtally-restapi"
SERVICE_NAME="airtally-api.service"
DB_NAME="airtally"
DB_USER="airtally_user"

# Functions
print_info() {
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

print_header() {
    echo ""
    echo -e "${BLUE}================================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}================================================${NC}"
    echo ""
}

check_command() {
    if ! command -v $1 &> /dev/null; then
        print_error "$1 is not installed. Please install it first."
        exit 1
    fi
}

# Parse command line arguments
UPDATE_MODE=false
BACKUP_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --update)
            UPDATE_MODE=true
            shift
            ;;
        --backup)
            BACKUP_MODE=true
            shift
            ;;
        --help)
            echo "AirTally REST API - Deployment Script"
            echo ""
            echo "Usage: ./deploy.sh [options]"
            echo ""
            echo "Options:"
            echo "  --update    Update existing installation"
            echo "  --backup    Backup database before deployment"
            echo "  --help      Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

print_header "AirTally REST API Deployment"

# Check prerequisites
print_info "Checking prerequisites..."
check_command "rustc"
check_command "cargo"
check_command "psql"
check_command "git"

print_success "All prerequisites satisfied"

# Backup database if requested
if [ "$BACKUP_MODE" = true ]; then
    print_header "Database Backup"

    BACKUP_DIR="$HOME/database_backups"
    mkdir -p "$BACKUP_DIR"

    DATE=$(date +%Y%m%d_%H%M%S)
    BACKUP_FILE="$BACKUP_DIR/${DB_NAME}_backup_$DATE.sql.gz"

    print_info "Creating database backup..."

    if [ -f "$HOME/.pgpass" ]; then
        pg_dump -U $DB_USER -h localhost $DB_NAME | gzip > "$BACKUP_FILE"
        print_success "Backup created: $BACKUP_FILE"
    else
        print_warning "No .pgpass file found. Backup skipped."
        print_info "Create ~/.pgpass with: localhost:5432:$DB_NAME:$DB_USER:your_password"
    fi
fi

# Update or fresh installation
if [ "$UPDATE_MODE" = true ]; then
    print_header "Update Mode: Pulling Latest Changes"

    cd "$APP_DIR"

    print_info "Pulling latest changes from repository..."
    git pull origin main

    print_success "Repository updated"
else
    print_header "Fresh Installation Mode"

    # Check if directory already exists
    if [ -d "$APP_DIR" ]; then
        print_warning "Application directory already exists: $APP_DIR"
        read -p "Do you want to remove it and start fresh? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            print_info "Removing existing directory..."
            rm -rf "$APP_DIR"
        else
            print_error "Deployment cancelled."
            exit 1
        fi
    fi
fi

# Build application
print_header "Building Application"

cd "$APP_DIR"

print_info "Building in release mode (this may take 5-15 minutes)..."
cargo build --release

if [ $? -eq 0 ]; then
    print_success "Build completed successfully"
else
    print_error "Build failed. Check cargo output above."
    exit 1
fi

# Verify binary
BINARY_PATH="$APP_DIR/target/release/airtally-restapi"
if [ -f "$BINARY_PATH" ]; then
    BINARY_SIZE=$(du -h "$BINARY_PATH" | cut -f1)
    print_success "Binary created: $BINARY_PATH ($BINARY_SIZE)"
else
    print_error "Binary not found at: $BINARY_PATH"
    exit 1
fi

# Create logs directory
print_info "Creating logs directory..."
mkdir -p "$APP_DIR/logs"

# Setup systemd service
print_header "Systemd Service Configuration"

SERVICE_FILE="/etc/systemd/system/$SERVICE_NAME"

if [ -f "$SERVICE_FILE" ]; then
    print_info "Service file already exists: $SERVICE_FILE"
else
    print_info "Creating systemd service file..."

    # Create service file content
    SERVICE_CONTENT="[Unit]
Description=AirTally REST API Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$USER
Group=$USER
WorkingDirectory=$APP_DIR
Environment=\"RUST_LOG=info\"
ExecStart=$BINARY_PATH
Restart=always
RestartSec=10
StandardOutput=append:$APP_DIR/logs/service.log
StandardError=append:$APP_DIR/logs/service_error.log

[Install]
WantedBy=multi-user.target"

    echo "$SERVICE_CONTENT" | sudo tee "$SERVICE_FILE" > /dev/null
    print_success "Service file created"
fi

# Reload systemd daemon
print_info "Reloading systemd daemon..."
sudo systemctl daemon-reload

# Enable service
print_info "Enabling service (auto-start on boot)..."
sudo systemctl enable $SERVICE_NAME

# Restart service
print_header "Starting Service"

print_info "Restarting $SERVICE_NAME..."
sudo systemctl restart $SERVICE_NAME

# Wait a moment for service to start
sleep 3

# Check service status
if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_success "Service is running!"
else
    print_error "Service failed to start. Checking logs..."
    sudo journalctl -u $SERVICE_NAME -n 20 --no-pager
    exit 1
fi

# Test API endpoint
print_header "Testing API"

print_info "Testing API endpoint..."
sleep 2

RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:3000/api/flights)

if [ "$RESPONSE" -eq 200 ]; then
    print_success "API is responding correctly (HTTP $RESPONSE)"
else
    print_warning "API returned HTTP $RESPONSE"
    print_info "This might be normal if there's no data yet"
fi

# Display service status
print_header "Deployment Summary"

sudo systemctl status $SERVICE_NAME --no-pager

# Display useful information
print_header "Useful Commands"

echo -e "${GREEN}Service Management:${NC}"
echo "  Start service:   sudo systemctl start $SERVICE_NAME"
echo "  Stop service:    sudo systemctl stop $SERVICE_NAME"
echo "  Restart service: sudo systemctl restart $SERVICE_NAME"
echo "  Check status:    sudo systemctl status $SERVICE_NAME"
echo ""
echo -e "${GREEN}Logs:${NC}"
echo "  View logs:       sudo journalctl -u $SERVICE_NAME -f"
echo "  Recent logs:     sudo journalctl -u $SERVICE_NAME -n 50"
echo ""
echo -e "${GREEN}Testing:${NC}"
echo "  Test API:        curl http://localhost:3000/api/flights"
echo ""
echo -e "${GREEN}Update:${NC}"
echo "  Update app:      ./deploy.sh --update"
echo "  With backup:     ./deploy.sh --update --backup"

print_success "Deployment completed successfully!"

# Display next steps
print_header "Next Steps"

echo "1. Verify application is working:"
echo "   curl http://localhost:3000/api/flights"
echo ""
echo "2. Configure Nginx reverse proxy (if not done yet):"
echo "   See UBUNTU_DEPLOYMENT.md section: Nginx Reverse Proxy"
echo ""
echo "3. Setup SSL certificate (if using domain):"
echo "   sudo certbot --nginx -d api.yourdomain.com"
echo ""
echo "4. Configure firewall:"
echo "   sudo ufw allow 80/tcp"
echo "   sudo ufw allow 443/tcp"
echo "   sudo ufw enable"
echo ""
echo "5. Setup monitoring and backups:"
echo "   See UBUNTU_DEPLOYMENT.md section: Monitoring & Logging"

echo ""
print_info "For detailed instructions, see: UBUNTU_DEPLOYMENT.md"