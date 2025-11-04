#!/bin/bash

################################################################################
# Cleanup Old AirTally Installation
#
# This script SAFELY removes the old AirTally installation after migrating
# to FALCON. It includes safety checks and backup creation.
#
# WARNING: This will permanently delete:
# - AirTally service
# - AirTally user and home directory
# - AirTally database (optional)
#
# Usage:
#   sudo ./cleanup-old-airtally.sh
#
# Requirements:
# - Root or sudo access
# - FALCON is installed and working
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
OLD_USER="airtally"
OLD_HOME="/home/airtally"
OLD_SERVICE="airtally-api"
OLD_DB="airtally"  # Database name if you want to delete it
BACKUP_DIR="/root/airtally_cleanup_backup_$(date +%Y%m%d_%H%M%S)"

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

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_info() {
    echo -e "${CYAN}ℹ${NC} $1"
}

################################################################################
# Pre-flight checks
################################################################################

print_header "Cleanup Old AirTally Installation"

echo -e "${RED}⚠ WARNING: This will PERMANENTLY DELETE the old AirTally installation!${NC}"
echo ""
echo "What will be removed:"
echo "  • User: $OLD_USER"
echo "  • Home directory: $OLD_HOME"
echo "  • Service: $OLD_SERVICE"
echo "  • Database: $OLD_DB (optional)"
echo ""
echo "Before proceeding, ensure:"
echo "  ✓ FALCON is installed and working"
echo "  ✓ All data has been migrated"
echo "  ✓ Mobile apps are using new FALCON API"
echo "  ✓ You have tested FALCON for at least 24-48 hours"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    print_error "Please run as root or with sudo"
    echo "Usage: sudo $0"
    exit 1
fi

# Check if FALCON is running
if ! systemctl is-active --quiet falcon-api; then
    print_error "FALCON service (falcon-api) is not running!"
    echo ""
    echo "Please ensure FALCON is installed and working before cleaning up old installation."
    exit 1
fi

print_success "FALCON service is running"

# Final confirmation
echo ""
echo -e "${YELLOW}This action CANNOT be undone!${NC}"
read -p "Type 'DELETE OLD AIRTALLY' to confirm: " CONFIRM

if [ "$CONFIRM" != "DELETE OLD AIRTALLY" ]; then
    echo "Cleanup cancelled."
    exit 0
fi

echo ""
print_info "Starting cleanup process..."

################################################################################
# Create backup
################################################################################

print_info "Creating final backup before deletion..."
mkdir -p "$BACKUP_DIR"

# Backup home directory
if [ -d "$OLD_HOME" ]; then
    print_info "Backing up $OLD_HOME..."
    tar -czf "$BACKUP_DIR/airtally_home.tar.gz" -C "$(dirname $OLD_HOME)" "$(basename $OLD_HOME)" 2>/dev/null || true
    print_success "Home directory backed up"
fi

# Backup systemd service
if [ -f "/etc/systemd/system/$OLD_SERVICE.service" ]; then
    cp "/etc/systemd/system/$OLD_SERVICE.service" "$BACKUP_DIR/"
    print_success "Service file backed up"
fi

# Backup database (if exists)
if sudo -u postgres psql -lqt | cut -d \| -f 1 | grep -qw "$OLD_DB"; then
    print_info "Backing up database $OLD_DB..."
    sudo -u postgres pg_dump "$OLD_DB" | gzip > "$BACKUP_DIR/${OLD_DB}_final.sql.gz"
    print_success "Database backed up"
fi

print_success "Backup created at: $BACKUP_DIR"

################################################################################
# Stop and disable service
################################################################################

print_info "Stopping and disabling $OLD_SERVICE service..."

if systemctl list-units --full --all | grep -q "$OLD_SERVICE"; then
    systemctl stop "$OLD_SERVICE" 2>/dev/null || true
    systemctl disable "$OLD_SERVICE" 2>/dev/null || true
    rm -f "/etc/systemd/system/$OLD_SERVICE.service"
    systemctl daemon-reload
    print_success "Service stopped and removed"
else
    print_info "Service $OLD_SERVICE not found (already removed)"
fi

################################################################################
# Remove user and home directory
################################################################################

print_info "Removing user $OLD_USER and home directory..."

if id "$OLD_USER" &>/dev/null; then
    # Kill any processes owned by user
    pkill -u "$OLD_USER" 2>/dev/null || true
    sleep 2

    # Remove user and home directory
    userdel -r "$OLD_USER" 2>/dev/null || userdel "$OLD_USER" 2>/dev/null || true

    # Force remove home directory if still exists
    if [ -d "$OLD_HOME" ]; then
        rm -rf "$OLD_HOME"
    fi

    print_success "User and home directory removed"
else
    print_info "User $OLD_USER not found (already removed)"
fi

################################################################################
# Clean up firewall rules
################################################################################

print_info "Cleaning up firewall rules..."

# Remove any airtally-specific rules
ufw status numbered | grep -i airtally | cut -d'[' -f2 | cut -d']' -f1 | sort -rn | while read -r rule_num; do
    if [ ! -z "$rule_num" ]; then
        echo "y" | ufw delete "$rule_num" 2>/dev/null || true
    fi
done

print_success "Firewall rules cleaned"

################################################################################
# Optional: Remove database
################################################################################

echo ""
print_warning "Database Cleanup"
echo "The old database '$OLD_DB' still exists."
echo ""
echo "Options:"
echo "  1. Keep database (you can delete it later manually)"
echo "  2. Delete database permanently"
echo ""
read -p "Enter your choice (1 or 2): " DB_CHOICE

if [ "$DB_CHOICE" = "2" ]; then
    print_info "Removing database $OLD_DB..."

    sudo -u postgres psql <<SQL
DROP DATABASE IF EXISTS $OLD_DB;
DROP USER IF EXISTS airtally_user;
SQL

    print_success "Database removed"
else
    print_info "Database kept. To remove manually later:"
    echo "  sudo -u postgres psql -c 'DROP DATABASE $OLD_DB;'"
fi

################################################################################
# Clean up cron jobs
################################################################################

print_info "Cleaning up cron jobs..."

# Remove any airtally cron jobs
if crontab -u "$OLD_USER" -l 2>/dev/null | grep -q airtally; then
    crontab -u "$OLD_USER" -r 2>/dev/null || true
    print_success "Cron jobs removed"
else
    print_info "No cron jobs found"
fi

################################################################################
# Verify FALCON is still working
################################################################################

print_info "Verifying FALCON is still working..."

sleep 2

if systemctl is-active --quiet falcon-api; then
    print_success "FALCON service is running"
else
    print_error "FALCON service is not running!"
    echo ""
    echo "Please check FALCON service immediately:"
    echo "  sudo systemctl status falcon-api"
    echo "  sudo journalctl -u falcon-api -n 50"
fi

# Test API
if curl -s http://localhost:3000/api/starter-data/version | grep -q "success"; then
    print_success "FALCON API is responding"
else
    print_warning "FALCON API not responding (may need more time)"
fi

################################################################################
# Cleanup Summary
################################################################################

print_header "Cleanup Complete!"

echo -e "${GREEN}✓ Old AirTally installation removed successfully!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${CYAN}📋 What was removed:${NC}"
echo "  • User: $OLD_USER"
echo "  • Home directory: $OLD_HOME"
echo "  • Service: $OLD_SERVICE"
echo "  • Database: $([ "$DB_CHOICE" = "2" ] && echo "Removed" || echo "Kept")"
echo ""
echo -e "${CYAN}💾 Backup Location:${NC}"
echo "  $BACKUP_DIR"
echo ""
echo "  Contents:"
echo "    • airtally_home.tar.gz (home directory)"
echo "    • $OLD_SERVICE.service (systemd service)"
if [ "$DB_CHOICE" != "2" ] && [ -f "$BACKUP_DIR/${OLD_DB}_final.sql.gz" ]; then
echo "    • ${OLD_DB}_final.sql.gz (database backup)"
fi
echo ""
echo -e "${CYAN}✅ FALCON Status:${NC}"
echo "  • Service: $(systemctl is-active falcon-api)"
echo "  • User: falcon"
echo "  • Directory: /home/falcon/FalconRest"
echo ""
echo -e "${CYAN}🔧 If you need to restore:${NC}"
echo "  1. Restore home directory:"
echo "     sudo tar -xzf $BACKUP_DIR/airtally_home.tar.gz -C /"
echo ""
echo "  2. Restore database:"
if [ -f "$BACKUP_DIR/${OLD_DB}_final.sql.gz" ]; then
echo "     gunzip -c $BACKUP_DIR/${OLD_DB}_final.sql.gz | sudo -u postgres psql $OLD_DB"
else
echo "     (database backup not available)"
fi
echo ""
echo -e "${CYAN}📝 Recommendations:${NC}"
echo "  • Keep backup for 30 days, then delete"
echo "  • Monitor FALCON for next 24 hours"
echo "  • Test all mobile app features"
echo "  • Update any documentation referencing old paths"
echo ""
echo -e "${CYAN}🗑️  Delete backup after 30 days:${NC}"
echo "  sudo rm -rf $BACKUP_DIR"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

print_success "Cleanup completed successfully!"
echo ""
