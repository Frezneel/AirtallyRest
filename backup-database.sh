#!/bin/bash

##############################################################################
# AirTally Database Backup Script
# Automatically backs up PostgreSQL database with compression
# Usage: ./backup-database.sh [--retention DAYS]
##############################################################################

# Configuration
BACKUP_DIR="$HOME/database_backups"
DB_NAME="airtally"
DB_USER="airtally_user"
DB_HOST="localhost"
RETENTION_DAYS=30  # Default retention

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --retention)
            RETENTION_DAYS="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--retention DAYS]"
            exit 1
            ;;
    esac
done

# Create backup directory if not exists
mkdir -p "$BACKUP_DIR"

# Generate timestamp
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="$BACKUP_DIR/${DB_NAME}_backup_$DATE.sql.gz"

echo -e "${GREEN}[INFO]${NC} Starting database backup..."
echo -e "${GREEN}[INFO]${NC} Database: $DB_NAME"
echo -e "${GREEN}[INFO]${NC} Backup file: $BACKUP_FILE"

# Check if .pgpass exists for passwordless authentication
if [ ! -f "$HOME/.pgpass" ]; then
    echo -e "${YELLOW}[WARNING]${NC} ~/.pgpass file not found"
    echo -e "${YELLOW}[WARNING]${NC} Create it with: echo 'localhost:5432:$DB_NAME:$DB_USER:your_password' > ~/.pgpass && chmod 600 ~/.pgpass"
    exit 1
fi

# Perform backup with compression
if pg_dump -U "$DB_USER" -h "$DB_HOST" "$DB_NAME" | gzip > "$BACKUP_FILE"; then
    # Get backup file size
    BACKUP_SIZE=$(du -h "$BACKUP_FILE" | cut -f1)

    echo -e "${GREEN}[SUCCESS]${NC} Backup completed successfully"
    echo -e "${GREEN}[INFO]${NC} Backup size: $BACKUP_SIZE"
    echo -e "${GREEN}[INFO]${NC} Location: $BACKUP_FILE"

    # Clean old backups
    echo -e "${GREEN}[INFO]${NC} Cleaning backups older than $RETENTION_DAYS days..."
    DELETED_COUNT=$(find "$BACKUP_DIR" -name "${DB_NAME}_backup_*.sql.gz" -mtime +$RETENTION_DAYS -delete -print | wc -l)

    if [ "$DELETED_COUNT" -gt 0 ]; then
        echo -e "${GREEN}[INFO]${NC} Deleted $DELETED_COUNT old backup(s)"
    else
        echo -e "${GREEN}[INFO]${NC} No old backups to delete"
    fi

    # List recent backups
    echo ""
    echo -e "${GREEN}[INFO]${NC} Recent backups:"
    ls -lth "$BACKUP_DIR" | head -n 6

    exit 0
else
    echo -e "${RED}[ERROR]${NC} Backup failed!"
    exit 1
fi