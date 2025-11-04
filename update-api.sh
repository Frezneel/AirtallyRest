#!/bin/bash

################################################################################
# FALCON REST API - Update Script
# Untuk update aplikasi dari GitHub tanpa full reinstall
#
# Usage:
#   chmod +x update-api.sh
#   ./update-api.sh [branch]
#
# Default branch: main
################################################################################

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
APP_DIR="/home/falcon/FalconRest"
SERVICE_NAME="falcon-api"
BRANCH="${1:-main}"
BACKUP_DIR="/home/falcon/backups"

################################################################################
# Helper Functions
################################################################################

print_header() {
    echo ""
    echo -e "${BLUE}======================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}======================================${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}[$1/$2]${NC} $3"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

################################################################################
# Pre-flight checks
################################################################################

print_header "FALCON API Update Script"

echo "Branch: $BRANCH"
echo "Date: $(date)"
echo ""

# Check if running as falcon user
if [ "$(whoami)" != "falcon" ]; then
    print_error "Script harus dijalankan sebagai user 'falcon'"
    echo "Jalankan: sudo su - falcon"
    exit 1
fi

# Check if app directory exists
if [ ! -d "$APP_DIR" ]; then
    print_error "Directory $APP_DIR tidak ditemukan"
    exit 1
fi

# Check if service exists
if ! sudo systemctl list-units --full --all | grep -q "$SERVICE_NAME"; then
    print_error "Service $SERVICE_NAME tidak ditemukan"
    exit 1
fi

################################################################################
# Update Process
################################################################################

TOTAL_STEPS=9

################################################################################
# Step 1: Create backup directory
################################################################################

print_step 1 $TOTAL_STEPS "Creating backup directory"

mkdir -p "$BACKUP_DIR"
print_success "Backup directory ready"

################################################################################
# Step 2: Backup current binary
################################################################################

print_step 2 $TOTAL_STEPS "Backing up current binary"

if [ -f "$APP_DIR/target/release/falcon-rest" ]; then
    BACKUP_FILE="$BACKUP_DIR/falcon-rest.$(date +%Y%m%d_%H%M%S)"
    cp "$APP_DIR/target/release/falcon-rest" "$BACKUP_FILE"
    print_success "Backup created: $BACKUP_FILE"
else
    print_warning "No existing binary found, skipping backup"
fi

################################################################################
# Step 3: Stop service
################################################################################

print_step 3 $TOTAL_STEPS "Stopping service"

sudo systemctl stop $SERVICE_NAME
sleep 2

if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_error "Failed to stop service"
    exit 1
else
    print_success "Service stopped"
fi

################################################################################
# Step 4: Pull latest changes
################################################################################

print_step 4 $TOTAL_STEPS "Pulling latest changes from GitHub"

cd "$APP_DIR"

# Fetch all branches
git fetch origin

# Show current commit
echo "Current commit:"
git log -1 --oneline

# Checkout and pull
git checkout $BRANCH
git pull origin $BRANCH

# Show new commit
echo ""
echo "New commit:"
git log -1 --oneline

print_success "Code updated"

################################################################################
# Step 5: Show recent changes
################################################################################

print_step 5 $TOTAL_STEPS "Recent changes"

echo ""
git log -5 --oneline --decorate
echo ""

################################################################################
# Step 6: Build application
################################################################################

print_step 6 $TOTAL_STEPS "Building application (this may take several minutes)"

source $HOME/.cargo/env
# Use SQLx offline mode (doesn't require database connection during build)
SQLX_OFFLINE=true cargo build --release

if [ -f "$APP_DIR/target/release/falcon-rest" ]; then
    print_success "Build completed"
else
    print_error "Build failed - binary not found"
    echo ""
    echo "Restoring from backup..."
    if [ -f "$BACKUP_FILE" ]; then
        cp "$BACKUP_FILE" "$APP_DIR/target/release/falcon-rest"
        print_success "Backup restored"
    fi
    exit 1
fi

################################################################################
# Step 7: Run migrations
################################################################################

print_step 7 $TOTAL_STEPS "Running database migrations"

# Load DATABASE_URL from .env
if [ -f "$APP_DIR/.env" ]; then
    export $(grep DATABASE_URL "$APP_DIR/.env" | xargs)
fi

if sqlx migrate run 2>&1 | grep -q "No migrations"; then
    print_warning "No new migrations"
else
    print_success "Migrations completed"
fi

################################################################################
# Step 8: Start service
################################################################################

print_step 8 $TOTAL_STEPS "Starting service"

sudo systemctl start $SERVICE_NAME
sleep 3

################################################################################
# Step 9: Verify service
################################################################################

print_step 9 $TOTAL_STEPS "Verifying service"

if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_success "Service is running"
else
    print_error "Service failed to start"
    echo ""
    echo "Checking logs:"
    sudo journalctl -u $SERVICE_NAME -n 20
    echo ""
    echo "Restoring from backup..."
    if [ -f "$BACKUP_FILE" ]; then
        sudo systemctl stop $SERVICE_NAME
        cp "$BACKUP_FILE" "$APP_DIR/target/release/falcon-rest"
        sudo systemctl start $SERVICE_NAME
        print_success "Backup restored"
    fi
    exit 1
fi

# Test API
sleep 2
if curl -s http://localhost:3000/api/starter-data/version | grep -q "success"; then
    print_success "API is responding"
else
    print_warning "API not responding (may need more time)"
fi

################################################################################
# Summary
################################################################################

print_header "Update Complete!"

echo -e "${GREEN}Update berhasil!${NC}"
echo ""
echo "📋 Status:"
echo "  • Service: $(sudo systemctl is-active $SERVICE_NAME)"
echo "  • API: http://10.17.6.155/api"
echo ""
echo "🔧 Commands:"
echo "  • View logs: sudo journalctl -u $SERVICE_NAME -f"
echo "  • Restart: sudo systemctl restart $SERVICE_NAME"
echo "  • Rollback: cp $BACKUP_FILE $APP_DIR/target/release/falcon-rest"
echo ""

print_success "Update selesai!"
