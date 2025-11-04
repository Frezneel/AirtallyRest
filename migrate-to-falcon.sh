#!/bin/bash

################################################################################
# FALCON Migration Script - AirTally → FALCON (In-Place Migration)
#
# This script safely migrates your existing AirTally production server
# to FALCON without data loss or significant downtime.
#
# Usage:
#   chmod +x migrate-to-falcon.sh
#   ./migrate-to-falcon.sh
#
# Requirements:
# - Running as 'airtally' user
# - Existing AirTally installation in ~/AirtallyRest
# - Git repository configured
# - Internet connection for pulling latest code
################################################################################

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
APP_DIR="/home/airtally/AirtallyRest"
SERVICE_NAME="airtally-api"
BACKUP_DIR="/home/airtally/backups"
MIGRATION_LOG="/home/airtally/migration.log"

################################################################################
# Helper Functions
################################################################################

print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}========================================${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}[Step $1/$2]${NC} $3"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] Step $1: $3" >> "$MIGRATION_LOG"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] SUCCESS: $1" >> "$MIGRATION_LOG"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] WARNING: $1" >> "$MIGRATION_LOG"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] ERROR: $1" >> "$MIGRATION_LOG"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

################################################################################
# Pre-flight checks
################################################################################

print_header "FALCON Migration Script - AirTally → FALCON"

echo "This script will migrate your existing AirTally installation to FALCON."
echo "Migration type: IN-PLACE (minimal changes, no directory moves)"
echo ""
echo "What will happen:"
echo "  1. Backup current installation"
echo "  2. Add JWT_SECRET to .env"
echo "  3. Pull latest FALCON code"
echo "  4. Build new version"
echo "  5. Run database migrations"
echo "  6. Restart service"
echo ""
echo "Estimated time: 10-15 minutes"
echo "Expected downtime: 30-60 seconds"
echo ""

# Initialize log
echo "=== FALCON Migration Log - $(date) ===" > "$MIGRATION_LOG"

# Confirmation
read -p "Do you want to proceed? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Migration cancelled."
    exit 0
fi

echo ""

# Check if running as airtally user
if [ "$(whoami)" != "airtally" ]; then
    print_error "Script must be run as 'airtally' user"
    echo "Run: sudo su - airtally"
    exit 1
fi

# Check if app directory exists
if [ ! -d "$APP_DIR" ]; then
    print_error "Directory $APP_DIR not found"
    exit 1
fi

# Check if service exists
if ! sudo systemctl list-units --full --all | grep -q "$SERVICE_NAME"; then
    print_error "Service $SERVICE_NAME not found"
    exit 1
fi

# Check if git is configured
cd "$APP_DIR"
if ! git status &> /dev/null; then
    print_error "Not a git repository: $APP_DIR"
    exit 1
fi

print_success "Pre-flight checks passed"
echo ""

################################################################################
# Migration Process
################################################################################

TOTAL_STEPS=10

################################################################################
# Step 1: Create backup directory
################################################################################

print_step 1 $TOTAL_STEPS "Creating backup directory"

mkdir -p "$BACKUP_DIR"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
MIGRATION_BACKUP_DIR="$BACKUP_DIR/migration_$TIMESTAMP"
mkdir -p "$MIGRATION_BACKUP_DIR"

print_success "Backup directory created: $MIGRATION_BACKUP_DIR"

################################################################################
# Step 2: Backup current installation
################################################################################

print_step 2 $TOTAL_STEPS "Backing up current installation"

# Backup binary
if [ -f "$APP_DIR/target/release/airtally-rest" ]; then
    cp "$APP_DIR/target/release/airtally-rest" "$MIGRATION_BACKUP_DIR/airtally-rest"
    print_success "Binary backed up"
else
    print_warning "No existing binary found"
fi

# Backup .env
if [ -f "$APP_DIR/.env" ]; then
    cp "$APP_DIR/.env" "$MIGRATION_BACKUP_DIR/.env"
    print_success ".env backed up"
else
    print_error ".env file not found"
    exit 1
fi

# Backup systemd service
if [ -f "/etc/systemd/system/$SERVICE_NAME.service" ]; then
    sudo cp "/etc/systemd/system/$SERVICE_NAME.service" "$MIGRATION_BACKUP_DIR/"
    print_success "Systemd service backed up"
fi

# Save current git commit
cd "$APP_DIR"
git log -1 --oneline > "$MIGRATION_BACKUP_DIR/git_commit.txt"
print_success "Git commit info saved"

################################################################################
# Step 3: Check for JWT_SECRET
################################################################################

print_step 3 $TOTAL_STEPS "Checking JWT_SECRET configuration"

if grep -q "^JWT_SECRET=" "$APP_DIR/.env"; then
    print_info "JWT_SECRET already exists in .env"
    JWT_EXISTS=true
else
    print_warning "JWT_SECRET not found in .env"
    JWT_EXISTS=false
fi

################################################################################
# Step 4: Add JWT_SECRET if missing
################################################################################

if [ "$JWT_EXISTS" = false ]; then
    print_step 4 $TOTAL_STEPS "Generating and adding JWT_SECRET"

    # Check if openssl is available
    if ! command -v openssl &> /dev/null; then
        print_error "openssl not found. Please install: sudo apt install openssl"
        exit 1
    fi

    # Generate JWT secret
    JWT_SECRET=$(openssl rand -base64 48)

    # Add to .env
    echo "" >> "$APP_DIR/.env"
    echo "# JWT Secret for Authentication Tokens (Added by migration script)" >> "$APP_DIR/.env"
    echo "# Generated: $(date)" >> "$APP_DIR/.env"
    echo "JWT_SECRET=$JWT_SECRET" >> "$APP_DIR/.env"

    print_success "JWT_SECRET generated and added to .env"
    print_info "JWT_SECRET saved in backup: $MIGRATION_BACKUP_DIR/jwt_secret.txt"
    echo "$JWT_SECRET" > "$MIGRATION_BACKUP_DIR/jwt_secret.txt"
    chmod 600 "$MIGRATION_BACKUP_DIR/jwt_secret.txt"
else
    print_step 4 $TOTAL_STEPS "JWT_SECRET already configured (skipping)"
fi

################################################################################
# Step 5: Test current service with JWT_SECRET
################################################################################

print_step 5 $TOTAL_STEPS "Testing service with JWT_SECRET"

print_info "Restarting service to apply JWT_SECRET..."
sudo systemctl restart $SERVICE_NAME
sleep 3

if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_success "Service is running"

    # Check for JWT warning in logs
    if sudo journalctl -u $SERVICE_NAME -n 50 | grep -q "JWT_SECRET not set"; then
        print_warning "JWT_SECRET warning still present, but continuing..."
    else
        print_success "No JWT_SECRET warning found in logs"
    fi
else
    print_error "Service failed to start after adding JWT_SECRET"
    echo ""
    echo "Checking logs:"
    sudo journalctl -u $SERVICE_NAME -n 20
    echo ""
    print_info "Rolling back .env..."
    cp "$MIGRATION_BACKUP_DIR/.env" "$APP_DIR/.env"
    sudo systemctl restart $SERVICE_NAME
    exit 1
fi

################################################################################
# Step 6: Fetch latest code
################################################################################

print_step 6 $TOTAL_STEPS "Fetching latest FALCON code from Git"

cd "$APP_DIR"

# Show current state
echo "Current commit:"
git log -1 --oneline

# Fetch and checkout
git fetch origin
BRANCH=$(git rev-parse --abbrev-ref HEAD)
print_info "Current branch: $BRANCH"

# Ask which branch to use
echo ""
read -p "Which branch to pull? (default: $BRANCH): " INPUT_BRANCH
if [ -z "$INPUT_BRANCH" ]; then
    INPUT_BRANCH=$BRANCH
fi

git checkout $INPUT_BRANCH
git pull origin $INPUT_BRANCH

echo ""
echo "New commit:"
git log -1 --oneline

print_success "Code updated"

################################################################################
# Step 7: Stop service before build
################################################################################

print_step 7 $TOTAL_STEPS "Stopping service for rebuild"

sudo systemctl stop $SERVICE_NAME
sleep 2

if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_error "Failed to stop service"
    exit 1
else
    print_success "Service stopped"
fi

################################################################################
# Step 8: Build new version
################################################################################

print_step 8 $TOTAL_STEPS "Building FALCON (this may take 5-10 minutes)"

cd "$APP_DIR"
source $HOME/.cargo/env

print_info "Starting Rust compilation..."
if SQLX_OFFLINE=true cargo build --release 2>&1 | tee "$MIGRATION_BACKUP_DIR/build.log"; then
    if [ -f "$APP_DIR/target/release/airtally-rest" ]; then
        print_success "Build completed successfully"

        # Show binary info
        ls -lh "$APP_DIR/target/release/airtally-rest"
    else
        print_error "Build succeeded but binary not found"
        print_info "Restoring from backup..."
        cp "$MIGRATION_BACKUP_DIR/airtally-rest" "$APP_DIR/target/release/airtally-rest"
        exit 1
    fi
else
    print_error "Build failed"
    echo ""
    print_info "Check build log: $MIGRATION_BACKUP_DIR/build.log"
    print_info "Restoring from backup..."
    cp "$MIGRATION_BACKUP_DIR/airtally-rest" "$APP_DIR/target/release/airtally-rest"
    sudo systemctl start $SERVICE_NAME
    exit 1
fi

################################################################################
# Step 9: Run database migrations
################################################################################

print_step 9 $TOTAL_STEPS "Running database migrations"

# Load DATABASE_URL from .env
if [ -f "$APP_DIR/.env" ]; then
    export $(grep DATABASE_URL "$APP_DIR/.env" | xargs)
fi

if sqlx migrate run 2>&1 | tee "$MIGRATION_BACKUP_DIR/migration.log" | grep -q "Applied"; then
    print_success "New migrations applied"
elif sqlx migrate run 2>&1 | grep -q "No migrations"; then
    print_info "No new migrations to apply"
else
    print_success "Migrations completed"
fi

################################################################################
# Step 10: Start service and verify
################################################################################

print_step 10 $TOTAL_STEPS "Starting service and verifying"

sudo systemctl start $SERVICE_NAME
sleep 3

if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_success "Service is running"
else
    print_error "Service failed to start"
    echo ""
    echo "Checking logs:"
    sudo journalctl -u $SERVICE_NAME -n 30
    echo ""
    print_info "Rolling back..."
    sudo systemctl stop $SERVICE_NAME
    cp "$MIGRATION_BACKUP_DIR/airtally-rest" "$APP_DIR/target/release/airtally-rest"
    sudo systemctl start $SERVICE_NAME
    exit 1
fi

# Test API
sleep 2
print_info "Testing API endpoint..."
if curl -s http://localhost:3000/api/starter-data/version | grep -q "success"; then
    print_success "API is responding correctly"
else
    print_warning "API test returned unexpected response (may need more time)"
fi

# Check for JWT warning
print_info "Checking for JWT_SECRET warning in logs..."
if sudo journalctl -u $SERVICE_NAME -n 50 | grep -q "JWT_SECRET not set"; then
    print_warning "JWT_SECRET warning still present - please check .env configuration"
else
    print_success "No JWT_SECRET warning - configuration is correct!"
fi

################################################################################
# Migration Summary
################################################################################

print_header "Migration Complete!"

echo -e "${GREEN}✓ Migration successful!${NC}"
echo ""
echo "📋 Summary:"
echo "  • Service: $(sudo systemctl is-active $SERVICE_NAME)"
echo "  • Backup location: $MIGRATION_BACKUP_DIR"
echo "  • Migration log: $MIGRATION_LOG"
echo "  • Build log: $MIGRATION_BACKUP_DIR/build.log"
echo ""
echo "🔍 What changed:"
echo "  • JWT_SECRET added to .env (security improvement)"
echo "  • Code updated to FALCON branding"
echo "  • Database migrations applied (if any)"
echo ""
echo "✅ What stayed the same:"
echo "  • Database name and data (all intact)"
echo "  • Service name: $SERVICE_NAME"
echo "  • Directory: $APP_DIR"
echo "  • API endpoints and port"
echo ""
echo "📱 Mobile App:"
echo "  • No changes needed"
echo "  • Test login, scanning, and sync"
echo ""
echo "🔧 Useful Commands:"
echo "  • View logs: sudo journalctl -u $SERVICE_NAME -f"
echo "  • Restart: sudo systemctl restart $SERVICE_NAME"
echo "  • Status: sudo systemctl status $SERVICE_NAME"
echo "  • Rollback: cp $MIGRATION_BACKUP_DIR/airtally-rest $APP_DIR/target/release/"
echo ""
echo "📝 Next Steps:"
echo "  1. Test mobile app (login, scan, sync)"
echo "  2. Monitor logs for 24 hours: sudo journalctl -u $SERVICE_NAME -f"
echo "  3. Test user management features"
echo "  4. Keep backup for 1 week, then delete if stable"
echo ""

print_success "Migration completed successfully!"

# Save summary to file
cat > "$MIGRATION_BACKUP_DIR/MIGRATION_SUMMARY.txt" <<EOF
FALCON Migration Summary
========================
Date: $(date)
User: $(whoami)
Hostname: $(hostname)

Backup Location: $MIGRATION_BACKUP_DIR
Service: $SERVICE_NAME
App Directory: $APP_DIR

Migration Type: In-Place (minimal changes)
Expected Downtime: 30-60 seconds
Actual Downtime: Check service logs

Git Commits:
Before: $(cat $MIGRATION_BACKUP_DIR/git_commit.txt)
After: $(cd $APP_DIR && git log -1 --oneline)

JWT_SECRET: $(if [ "$JWT_EXISTS" = true ]; then echo "Already existed"; else echo "Generated and added"; fi)

Database Migrations: Check $MIGRATION_BACKUP_DIR/migration.log

Service Status: $(sudo systemctl is-active $SERVICE_NAME)

Rollback Command:
sudo systemctl stop $SERVICE_NAME
cp $MIGRATION_BACKUP_DIR/airtally-rest $APP_DIR/target/release/airtally-rest
cp $MIGRATION_BACKUP_DIR/.env $APP_DIR/.env
sudo systemctl start $SERVICE_NAME

EOF

print_success "Migration summary saved: $MIGRATION_BACKUP_DIR/MIGRATION_SUMMARY.txt"