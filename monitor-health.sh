#!/bin/bash

##############################################################################
# AirTally API Health Monitor
# Checks API health and restarts service if needed
# Usage: ./monitor-health.sh [--restart-on-fail]
##############################################################################

# Configuration
API_URL="http://localhost:3000/api/flights"
SERVICE_NAME="airtally-api"
LOG_FILE="$HOME/monitor.log"
MAX_RETRIES=3
RETRY_DELAY=5

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Options
AUTO_RESTART=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --restart-on-fail)
            AUTO_RESTART=true
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--restart-on-fail]"
            exit 1
            ;;
    esac
done

# Function to log messages
log_message() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

# Function to check API health
check_health() {
    RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" --connect-timeout 10 --max-time 30 "$API_URL" 2>/dev/null)
    echo "$RESPONSE"
}

# Function to restart service
restart_service() {
    log_message "Attempting to restart service: $SERVICE_NAME"

    if sudo systemctl restart "$SERVICE_NAME"; then
        log_message "Service restarted successfully"
        sleep 5  # Wait for service to start
        return 0
    else
        log_message "ERROR: Failed to restart service"
        return 1
    fi
}

# Main health check logic
log_message "Starting health check..."

ATTEMPT=1
HEALTH_OK=false

while [ $ATTEMPT -le $MAX_RETRIES ]; do
    HTTP_CODE=$(check_health)

    if [ "$HTTP_CODE" -eq 200 ] || [ "$HTTP_CODE" -eq 201 ]; then
        log_message "✓ API is healthy (HTTP $HTTP_CODE) - Attempt $ATTEMPT/$MAX_RETRIES"
        HEALTH_OK=true
        break
    else
        log_message "✗ API unhealthy (HTTP $HTTP_CODE) - Attempt $ATTEMPT/$MAX_RETRIES"

        if [ $ATTEMPT -lt $MAX_RETRIES ]; then
            log_message "Waiting ${RETRY_DELAY}s before retry..."
            sleep $RETRY_DELAY
        fi
    fi

    ((ATTEMPT++))
done

# Handle unhealthy API
if [ "$HEALTH_OK" = false ]; then
    log_message "ERROR: API is down after $MAX_RETRIES attempts"

    # Check if service is running
    if sudo systemctl is-active --quiet "$SERVICE_NAME"; then
        log_message "Service is running but API is not responding"
    else
        log_message "Service is not running"
    fi

    # Auto-restart if enabled
    if [ "$AUTO_RESTART" = true ]; then
        restart_service

        # Verify restart was successful
        sleep 5
        VERIFY_CODE=$(check_health)

        if [ "$VERIFY_CODE" -eq 200 ] || [ "$VERIFY_CODE" -eq 201 ]; then
            log_message "✓ Service restarted successfully and API is healthy"
            exit 0
        else
            log_message "ERROR: Service restart failed or API still unhealthy"
            log_message "Manual intervention required!"
            exit 1
        fi
    else
        log_message "Auto-restart disabled. Manual intervention required."
        log_message "To restart: sudo systemctl restart $SERVICE_NAME"
        exit 1
    fi
else
    exit 0
fi