#!/bin/bash

################################################################################
# AirTally REST API - Maintenance Script
# Script untuk backup, monitoring, dan maintenance tasks
#
# Usage:
#   chmod +x maintenance.sh
#   ./maintenance.sh [command]
#
# Commands:
#   backup      - Backup database
#   restore     - Restore database dari backup
#   logs        - View service logs
#   status      - Check service status
#   restart     - Restart service
#   clean       - Clean old logs and backups
#   monitor     - Monitor resources
################################################################################

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
APP_DIR="/home/airtally/AirtallyRest"
SERVICE_NAME="airtally-api"
BACKUP_DIR="/home/airtally/backups"
DB_NAME="airtally_prod"
DB_USER="airtally_user"

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
# Command: backup
################################################################################

cmd_backup() {
    print_header "Database Backup"

    mkdir -p "$BACKUP_DIR"

    BACKUP_FILE="$BACKUP_DIR/airtally_$(date +%Y%m%d_%H%M%S).sql"

    echo "Creating backup: $BACKUP_FILE"

    if pg_dump -h localhost -U $DB_USER $DB_NAME > "$BACKUP_FILE"; then
        print_success "Backup created successfully"
        echo "File: $BACKUP_FILE"
        echo "Size: $(du -h $BACKUP_FILE | cut -f1)"
    else
        print_error "Backup failed"
        exit 1
    fi

    # List recent backups
    echo ""
    echo "Recent backups:"
    ls -lht "$BACKUP_DIR" | head -6
}

################################################################################
# Command: restore
################################################################################

cmd_restore() {
    print_header "Database Restore"

    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A $BACKUP_DIR)" ]; then
        print_error "No backups found in $BACKUP_DIR"
        exit 1
    fi

    echo "Available backups:"
    ls -1t "$BACKUP_DIR"/*.sql | head -10

    echo ""
    read -p "Enter backup filename: " backup_file

    if [ ! -f "$BACKUP_DIR/$backup_file" ]; then
        print_error "Backup file not found"
        exit 1
    fi

    echo ""
    print_warning "WARNING: This will overwrite current database!"
    read -p "Continue? (yes/no): " confirm

    if [ "$confirm" != "yes" ]; then
        echo "Restore cancelled"
        exit 0
    fi

    # Stop service
    echo "Stopping service..."
    sudo systemctl stop $SERVICE_NAME

    # Restore
    echo "Restoring database..."
    if psql -h localhost -U $DB_USER -d $DB_NAME < "$BACKUP_DIR/$backup_file"; then
        print_success "Database restored"
    else
        print_error "Restore failed"
        exit 1
    fi

    # Start service
    echo "Starting service..."
    sudo systemctl start $SERVICE_NAME

    print_success "Restore complete"
}

################################################################################
# Command: logs
################################################################################

cmd_logs() {
    print_header "Service Logs"

    echo "Showing last 50 lines (Ctrl+C to exit)"
    echo ""

    sudo journalctl -u $SERVICE_NAME -n 50 --no-pager

    echo ""
    read -p "Follow logs in real-time? (y/n): " follow

    if [ "$follow" = "y" ] || [ "$follow" = "Y" ]; then
        sudo journalctl -u $SERVICE_NAME -f
    fi
}

################################################################################
# Command: status
################################################################################

cmd_status() {
    print_header "Service Status"

    # Service status
    echo "Service Status:"
    sudo systemctl status $SERVICE_NAME --no-pager

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # API test
    echo "API Test:"
    if curl -s http://localhost:3000/api/starter-data/version | grep -q "success"; then
        print_success "API is responding"
    else
        print_error "API is not responding"
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Database connection
    echo "Database Connection:"
    if PGPASSWORD=$(grep DATABASE_URL $APP_DIR/.env | cut -d: -f3 | cut -d@ -f1) \
       psql -h localhost -U $DB_USER -d $DB_NAME -c "SELECT 1" &>/dev/null; then
        print_success "Database connection OK"
    else
        print_error "Database connection failed"
    fi

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Database statistics
    echo "Database Statistics:"
    PGPASSWORD=$(grep DATABASE_URL $APP_DIR/.env | cut -d: -f3 | cut -d@ -f1) \
    psql -h localhost -U $DB_USER -d $DB_NAME << EOF
SELECT 'Flights' as table_name, COUNT(*) as count FROM flights
UNION ALL
SELECT 'Scan Data', COUNT(*) FROM scan_data
UNION ALL
SELECT 'Decoded Barcodes', COUNT(*) FROM decode_barcode;
EOF

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Disk usage
    echo "Disk Usage:"
    df -h | grep -E "Filesystem|/$"

    echo ""
    echo "App Directory Size:"
    du -sh $APP_DIR
}

################################################################################
# Command: restart
################################################################################

cmd_restart() {
    print_header "Restart Service"

    echo "Restarting $SERVICE_NAME..."

    sudo systemctl restart $SERVICE_NAME
    sleep 3

    if sudo systemctl is-active --quiet $SERVICE_NAME; then
        print_success "Service restarted successfully"

        # Test API
        if curl -s http://localhost:3000/api/starter-data/version | grep -q "success"; then
            print_success "API is responding"
        else
            print_warning "API not responding yet (may need more time)"
        fi
    else
        print_error "Service failed to start"
        echo ""
        echo "Recent logs:"
        sudo journalctl -u $SERVICE_NAME -n 20
        exit 1
    fi
}

################################################################################
# Command: clean
################################################################################

cmd_clean() {
    print_header "Cleanup Old Files"

    echo "This will remove:"
    echo "  - Backups older than 30 days"
    echo "  - Old binary backups (keep last 5)"
    echo "  - Cargo build cache"
    echo ""
    read -p "Continue? (y/n): " confirm

    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo "Cleanup cancelled"
        exit 0
    fi

    # Clean old database backups
    echo "Cleaning old database backups..."
    if [ -d "$BACKUP_DIR" ]; then
        find "$BACKUP_DIR" -name "airtally_*.sql" -type f -mtime +30 -delete
        print_success "Old database backups removed"
    fi

    # Keep only last 5 binary backups
    echo "Cleaning old binary backups..."
    if [ -d "$BACKUP_DIR" ]; then
        ls -t "$BACKUP_DIR"/airtally-rest.* 2>/dev/null | tail -n +6 | xargs -r rm
        print_success "Old binary backups removed"
    fi

    # Clean cargo cache
    echo "Cleaning cargo build cache..."
    if [ -d "$APP_DIR/target" ]; then
        cd "$APP_DIR"
        cargo clean
        print_success "Cargo cache cleaned"
    fi

    # Show disk space saved
    echo ""
    echo "Current disk usage:"
    df -h | grep -E "Filesystem|/$"
}

################################################################################
# Command: monitor
################################################################################

cmd_monitor() {
    print_header "Resource Monitor"

    echo "System Resources:"
    echo ""

    # CPU and Memory
    echo "CPU & Memory:"
    top -bn1 | head -5

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Process info
    echo "AirTally Process:"
    ps aux | grep airtally-rest | grep -v grep || echo "Process not running"

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Database connections
    echo "Database Connections:"
    PGPASSWORD=$(grep DATABASE_URL $APP_DIR/.env | cut -d: -f3 | cut -d@ -f1) \
    psql -h localhost -U $DB_USER -d $DB_NAME -c "SELECT count(*) as connections FROM pg_stat_activity WHERE datname='$DB_NAME';"

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    # Network connections
    echo "Network Connections (port 3000):"
    sudo netstat -tn | grep :3000 | wc -l

    echo ""
    read -p "Show detailed network connections? (y/n): " show_net

    if [ "$show_net" = "y" ] || [ "$show_net" = "Y" ]; then
        sudo netstat -tn | grep :3000
    fi
}

################################################################################
# Command: help
################################################################################

cmd_help() {
    print_header "AirTally Maintenance Script"

    echo "Usage: ./maintenance.sh [command]"
    echo ""
    echo "Available commands:"
    echo "  backup      - Backup database ke $BACKUP_DIR"
    echo "  restore     - Restore database dari backup"
    echo "  logs        - View dan follow service logs"
    echo "  status      - Check service, API, dan database status"
    echo "  restart     - Restart service"
    echo "  clean       - Cleanup old backups dan cache"
    echo "  monitor     - Monitor CPU, memory, dan connections"
    echo "  help        - Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./maintenance.sh backup"
    echo "  ./maintenance.sh status"
    echo "  ./maintenance.sh logs"
}

################################################################################
# Main
################################################################################

# Check if running as airtally user (except for help)
if [ "$1" != "help" ] && [ "$(whoami)" != "airtally" ]; then
    print_error "Script harus dijalankan sebagai user 'airtally'"
    echo "Jalankan: sudo su - airtally"
    exit 1
fi

# Parse command
case "$1" in
    backup)
        cmd_backup
        ;;
    restore)
        cmd_restore
        ;;
    logs)
        cmd_logs
        ;;
    status)
        cmd_status
        ;;
    restart)
        cmd_restart
        ;;
    clean)
        cmd_clean
        ;;
    monitor)
        cmd_monitor
        ;;
    help|"")
        cmd_help
        ;;
    *)
        print_error "Unknown command: $1"
        echo ""
        cmd_help
        exit 1
        ;;
esac
