#!/bin/bash

################################################################################
# FALCON Fresh Installation Script
# Clean installation of FALCON REST API for production
#
# This script will:
# 1. Create new 'falcon' user and directories
# 2. Install all dependencies (Rust, PostgreSQL client, etc.)
# 3. Setup PostgreSQL database
# 4. Clone and build FALCON
# 5. Configure systemd service
# 6. Setup firewall
# 7. Configure automatic backups
#
# Usage:
#   sudo ./fresh-install-falcon.sh
#
# Requirements:
# - Ubuntu/Debian Linux
# - Root or sudo access
# - Internet connection
################################################################################

set -e  # Exit on error

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

# Configuration
FALCON_USER="falcon"
FALCON_HOME="/home/falcon"
APP_DIR="$FALCON_HOME/FalconRest"
SERVICE_NAME="falcon-api"
DB_NAME="falcon"
DB_USER="falcon_user"
GITHUB_REPO="https://github.com/Frezneel/FalconRest.git"
BRANCH="main"  # or Development

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

print_header "FALCON Fresh Installation"

echo "This script will install FALCON REST API from scratch."
echo ""
echo "What will be installed:"
echo "  • New system user: falcon"
echo "  • Rust toolchain"
echo "  • PostgreSQL database: $DB_NAME"
echo "  • FALCON application"
echo "  • Systemd service: falcon-api"
echo "  • Firewall rules"
echo "  • Automatic backups"
echo ""
echo "Estimated time: 20-30 minutes"
echo ""

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    print_error "Please run as root or with sudo"
    echo "Usage: sudo $0"
    exit 1
fi

# Confirmation
read -p "Do you want to proceed with FRESH INSTALLATION? (yes/no): " CONFIRM
if [ "$CONFIRM" != "yes" ]; then
    echo "Installation cancelled."
    exit 0
fi

echo ""

################################################################################
# Configuration Input
################################################################################

TOTAL_STEPS=15

print_step 1 $TOTAL_STEPS "Gathering configuration"

# Database password
echo ""
print_info "Database Configuration"
while true; do
    read -sp "Enter PostgreSQL password for falcon_user (minimum 12 chars): " DB_PASSWORD
    echo ""
    if [ ${#DB_PASSWORD} -lt 12 ]; then
        print_warning "Password too short. Minimum 12 characters."
        continue
    fi
    read -sp "Confirm password: " DB_PASSWORD_CONFIRM
    echo ""
    if [ "$DB_PASSWORD" = "$DB_PASSWORD_CONFIRM" ]; then
        break
    else
        print_warning "Passwords don't match. Try again."
    fi
done

# Generate secrets
print_info "Generating security secrets..."
API_KEY=$(openssl rand -hex 32)
JWT_SECRET=$(openssl rand -base64 48)
print_success "Secrets generated"

# Network configuration
echo ""
print_info "Network Configuration"
read -p "Enter allowed network CIDR (e.g., 10.17.6.0/24): " ALLOWED_NETWORKS
read -p "Enter API port (default: 3000): " API_PORT
API_PORT=${API_PORT:-3000}

# GitHub repo
echo ""
print_info "Repository Configuration"
read -p "Enter GitHub repository URL (press Enter for default): " REPO_INPUT
GITHUB_REPO=${REPO_INPUT:-$GITHUB_REPO}
read -p "Enter branch name (default: main): " BRANCH_INPUT
BRANCH=${BRANCH_INPUT:-$BRANCH}

print_success "Configuration complete"

################################################################################
# Step 2: Update system packages
################################################################################

print_step 2 $TOTAL_STEPS "Updating system packages"

apt update
apt upgrade -y

print_success "System packages updated"

################################################################################
# Step 3: Install dependencies
################################################################################

print_step 3 $TOTAL_STEPS "Installing system dependencies"

apt install -y \
    curl \
    git \
    build-essential \
    pkg-config \
    libssl-dev \
    postgresql-client \
    ufw

print_success "Dependencies installed"

################################################################################
# Step 4: Create falcon user
################################################################################

print_step 4 $TOTAL_STEPS "Creating falcon user"

if id "$FALCON_USER" &>/dev/null; then
    print_warning "User '$FALCON_USER' already exists"
else
    useradd -r -m -s /bin/bash -d "$FALCON_HOME" "$FALCON_USER"
    print_success "User '$FALCON_USER' created"
fi

# Create directories
mkdir -p "$FALCON_HOME/backups/database"
mkdir -p "$FALCON_HOME/logs"
chown -R "$FALCON_USER:$FALCON_USER" "$FALCON_HOME"

print_success "Directories created"

################################################################################
# Step 5: Install Rust for falcon user
################################################################################

print_step 5 $TOTAL_STEPS "Installing Rust toolchain"

sudo -u "$FALCON_USER" bash <<'RUST_INSTALL'
cd ~
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
rustc --version
RUST_INSTALL

print_success "Rust installed"

################################################################################
# Step 6: Install sqlx-cli
################################################################################

print_step 6 $TOTAL_STEPS "Installing sqlx-cli"

sudo -u "$FALCON_USER" bash <<'SQLX_INSTALL'
source $HOME/.cargo/env
cargo install sqlx-cli --no-default-features --features postgres
SQLX_INSTALL

print_success "sqlx-cli installed"

################################################################################
# Step 7: Setup PostgreSQL database
################################################################################

print_step 7 $TOTAL_STEPS "Setting up PostgreSQL database"

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    print_info "PostgreSQL not found, installing..."
    apt install -y postgresql postgresql-contrib
    systemctl enable postgresql
    systemctl start postgresql
fi

# Create database and user
sudo -u postgres psql <<SQL
-- Drop if exists (fresh install)
DROP DATABASE IF EXISTS $DB_NAME;
DROP USER IF EXISTS $DB_USER;

-- Create user and database
CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';
CREATE DATABASE $DB_NAME OWNER $DB_USER;

-- Grant privileges
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;

-- Connect and grant schema privileges
\c $DB_NAME
GRANT ALL ON SCHEMA public TO $DB_USER;
SQL

print_success "Database created: $DB_NAME"
print_success "Database user created: $DB_USER"

################################################################################
# Step 8: Clone repository
################################################################################

print_step 8 $TOTAL_STEPS "Cloning FALCON repository"

sudo -u "$FALCON_USER" bash <<CLONE
cd "$FALCON_HOME"
if [ -d "FalconRest" ]; then
    rm -rf FalconRest
fi
git clone "$GITHUB_REPO" FalconRest
cd FalconRest
git checkout "$BRANCH"
git log -1 --oneline
CLONE

print_success "Repository cloned from $GITHUB_REPO"

################################################################################
# Step 9: Configure environment
################################################################################

print_step 9 $TOTAL_STEPS "Creating .env configuration"

cat > "$APP_DIR/.env" <<ENV
# ==============================================================================
# FALCON REST API - Production Configuration
# Generated: $(date)
# ==============================================================================

# Database Configuration
DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME

# Database Connection Pool
DB_MIN_CONNECTIONS=10
DB_MAX_CONNECTIONS=50
DB_CONNECT_TIMEOUT=10
DB_ACQUIRE_TIMEOUT=30
DB_IDLE_TIMEOUT=600
DB_MAX_LIFETIME=1800
DB_TEST_ON_CHECKOUT=true

# Server Configuration
HOST=0.0.0.0
PORT=$API_PORT

# Environment
ENVIRONMENT=production

# Logging
LOG_LEVEL=info

# Security (GENERATED - KEEP SECRET!)
API_KEY=$API_KEY
JWT_SECRET=$JWT_SECRET

# Network Security
ALLOWED_NETWORKS=$ALLOWED_NETWORKS

# Rate Limiting
RATE_LIMIT_PER_MINUTE=100

# Features
ENABLE_SWAGGER=false
ENV

chown "$FALCON_USER:$FALCON_USER" "$APP_DIR/.env"
chmod 600 "$APP_DIR/.env"

print_success ".env file created"

# Save secrets to secure location
cat > "$FALCON_HOME/SECRETS.txt" <<SECRETS
FALCON API Secrets - KEEP SECURE!
Generated: $(date)
========================================

Database User: $DB_USER
Database Password: $DB_PASSWORD
Database Name: $DB_NAME

API Key: $API_KEY
JWT Secret: $JWT_SECRET

DO NOT COMMIT THIS FILE TO GIT!
DO NOT SHARE PUBLICLY!

Store this file securely and delete after saving to password manager.
SECRETS

chown "$FALCON_USER:$FALCON_USER" "$FALCON_HOME/SECRETS.txt"
chmod 600 "$FALCON_HOME/SECRETS.txt"

print_success "Secrets saved to $FALCON_HOME/SECRETS.txt"
print_warning "IMPORTANT: Save secrets to password manager and delete SECRETS.txt!"

################################################################################
# Step 10: Run database migrations
################################################################################

print_step 10 $TOTAL_STEPS "Running database migrations"

sudo -u "$FALCON_USER" bash <<MIGRATE
cd "$APP_DIR"
source $HOME/.cargo/env
export DATABASE_URL="postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME"
sqlx migrate run
MIGRATE

print_success "Migrations completed"

################################################################################
# Step 11: Build application
################################################################################

print_step 11 $TOTAL_STEPS "Building FALCON (this will take 10-15 minutes)"

print_info "Starting Rust compilation..."
sudo -u "$FALCON_USER" bash <<BUILD
cd "$APP_DIR"
source $HOME/.cargo/env
SQLX_OFFLINE=true cargo build --release
BUILD

if [ -f "$APP_DIR/target/release/falcon-rest" ]; then
    print_success "Build completed successfully"
    ls -lh "$APP_DIR/target/release/falcon-rest"
else
    print_error "Build failed - binary not found"
    exit 1
fi

################################################################################
# Step 12: Create systemd service
################################################################################

print_step 12 $TOTAL_STEPS "Creating systemd service"

cat > "/etc/systemd/system/$SERVICE_NAME.service" <<SERVICE
[Unit]
Description=FALCON REST API Service
After=network.target postgresql.service
Wants=postgresql.service

[Service]
Type=simple
User=$FALCON_USER
Group=$FALCON_USER
WorkingDirectory=$APP_DIR
Environment="RUST_LOG=info"
ExecStart=$APP_DIR/target/release/falcon-rest
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal

# Security hardening
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=$FALCON_HOME

[Install]
WantedBy=multi-user.target
SERVICE

systemctl daemon-reload
systemctl enable "$SERVICE_NAME"

print_success "Systemd service created: $SERVICE_NAME"

################################################################################
# Step 13: Configure firewall
################################################################################

print_step 13 $TOTAL_STEPS "Configuring firewall"

# Enable UFW if not enabled
if ! ufw status | grep -q "Status: active"; then
    print_info "Enabling UFW firewall..."
    ufw --force enable
fi

# Allow SSH
ufw allow 22/tcp comment 'SSH'

# Allow API port from allowed network
if [ ! -z "$ALLOWED_NETWORKS" ]; then
    ufw allow from "$ALLOWED_NETWORKS" to any port "$API_PORT" comment 'FALCON API'
    print_success "API port $API_PORT allowed from $ALLOWED_NETWORKS"
else
    ufw allow "$API_PORT/tcp" comment 'FALCON API'
    print_success "API port $API_PORT allowed from anywhere"
fi

# Reload firewall
ufw reload

print_success "Firewall configured"

################################################################################
# Step 14: Setup automatic backups
################################################################################

print_step 14 $TOTAL_STEPS "Setting up automatic backups"

cat > "$FALCON_HOME/backup-db.sh" <<'BACKUP'
#!/bin/bash
BACKUP_DIR="/home/falcon/backups/database"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR

# Load database credentials
source /home/falcon/FalconRest/.env
DB_NAME="falcon"

# Extract credentials from DATABASE_URL
DB_USER=$(echo $DATABASE_URL | sed -n 's/.*\/\/\([^:]*\):.*/\1/p')
DB_PASS=$(echo $DATABASE_URL | sed -n 's/.*:\/\/[^:]*:\([^@]*\)@.*/\1/p')

# Backup database
PGPASSWORD=$DB_PASS pg_dump -U $DB_USER $DB_NAME | gzip > $BACKUP_DIR/falcon_${DATE}.sql.gz

# Keep only last 30 days
find $BACKUP_DIR -name "falcon_*.sql.gz" -mtime +30 -delete

echo "[$(date)] Backup completed: falcon_${DATE}.sql.gz"
BACKUP

chmod +x "$FALCON_HOME/backup-db.sh"
chown "$FALCON_USER:$FALCON_USER" "$FALCON_HOME/backup-db.sh"

# Add to crontab (daily at 2 AM)
(crontab -u "$FALCON_USER" -l 2>/dev/null | grep -v backup-db.sh; echo "0 2 * * * $FALCON_HOME/backup-db.sh >> $FALCON_HOME/logs/backup.log 2>&1") | crontab -u "$FALCON_USER" -

print_success "Automatic backups configured (daily at 2 AM)"

################################################################################
# Step 15: Start service and verify
################################################################################

print_step 15 $TOTAL_STEPS "Starting service and verifying"

systemctl start "$SERVICE_NAME"
sleep 5

if systemctl is-active --quiet "$SERVICE_NAME"; then
    print_success "Service is running"
else
    print_error "Service failed to start"
    echo ""
    echo "Checking logs:"
    journalctl -u "$SERVICE_NAME" -n 30
    exit 1
fi

# Test API
print_info "Testing API endpoint..."
sleep 2
if curl -s "http://localhost:$API_PORT/api/starter-data/version" | grep -q "success"; then
    print_success "API is responding correctly"
else
    print_warning "API test returned unexpected response (may need more time)"
fi

# Check for JWT warning
print_info "Checking for JWT_SECRET warning..."
if journalctl -u "$SERVICE_NAME" -n 50 | grep -q "JWT_SECRET not set"; then
    print_error "JWT_SECRET warning found - check configuration"
else
    print_success "No JWT_SECRET warning - configuration is correct!"
fi

################################################################################
# Installation Summary
################################################################################

print_header "Installation Complete!"

echo -e "${GREEN}✓ FALCON REST API installed successfully!${NC}"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo -e "${CYAN}📋 System Information:${NC}"
echo "  • User: $FALCON_USER"
echo "  • Home: $FALCON_HOME"
echo "  • Application: $APP_DIR"
echo "  • Service: $SERVICE_NAME"
echo "  • Status: $(systemctl is-active $SERVICE_NAME)"
echo ""
echo -e "${CYAN}🔐 Database Information:${NC}"
echo "  • Database: $DB_NAME"
echo "  • User: $DB_USER"
echo "  • Connection: localhost:5432"
echo ""
echo -e "${CYAN}🌐 API Information:${NC}"
echo "  • URL: http://$(hostname -I | awk '{print $1}'):$API_PORT/api"
echo "  • Port: $API_PORT"
echo "  • Allowed Networks: $ALLOWED_NETWORKS"
echo ""
echo -e "${CYAN}🔑 Default Superuser:${NC}"
echo "  • Username: superuser"
echo "  • Password: Falcon2025!"
echo "  • ${RED}⚠ CHANGE THIS PASSWORD IMMEDIATELY!${NC}"
echo ""
echo -e "${CYAN}📱 Mobile App Configuration:${NC}"
echo "  Update in lib/config/api_config.dart:"
echo "  • baseUrl: 'http://$(hostname -I | awk '{print $1}'):$API_PORT/api'"
echo "  • apiKey: '$API_KEY'"
echo ""
echo -e "${CYAN}🔧 Useful Commands:${NC}"
echo "  • View logs: sudo journalctl -u $SERVICE_NAME -f"
echo "  • Restart: sudo systemctl restart $SERVICE_NAME"
echo "  • Status: sudo systemctl status $SERVICE_NAME"
echo "  • Stop: sudo systemctl stop $SERVICE_NAME"
echo "  • Database backup: sudo -u falcon $FALCON_HOME/backup-db.sh"
echo ""
echo -e "${CYAN}📝 Important Files:${NC}"
echo "  • Secrets: $FALCON_HOME/SECRETS.txt ${RED}(DELETE AFTER SAVING!)${NC}"
echo "  • Configuration: $APP_DIR/.env"
echo "  • Service: /etc/systemd/system/$SERVICE_NAME.service"
echo "  • Backups: $FALCON_HOME/backups/"
echo "  • Update script: $APP_DIR/update-api.sh"
echo ""
echo -e "${CYAN}📚 Documentation:${NC}"
echo "  • README: $APP_DIR/README.md"
echo "  • Deployment: $APP_DIR/DEPLOYMENT_CHECKLIST.md"
echo "  • Roles & Permissions: $APP_DIR/ROLES_AND_PERMISSIONS.md"
echo ""
echo -e "${CYAN}✅ Next Steps:${NC}"
echo "  1. Save secrets from $FALCON_HOME/SECRETS.txt to password manager"
echo "  2. Delete SECRETS.txt: sudo rm $FALCON_HOME/SECRETS.txt"
echo "  3. Update mobile app with new API URL and API key"
echo "  4. Login with superuser and CHANGE PASSWORD immediately"
echo "  5. Create additional users with appropriate roles"
echo "  6. Test barcode scanning and sync functionality"
echo "  7. Monitor logs for first 24 hours: sudo journalctl -u $SERVICE_NAME -f"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

print_success "Installation completed successfully!"
echo ""
print_info "System will display secrets file content now. Please save it!"
echo ""
read -p "Press Enter to view secrets..."
echo ""
cat "$FALCON_HOME/SECRETS.txt"
echo ""
print_warning "Save these secrets NOW! File will remain at $FALCON_HOME/SECRETS.txt"
echo ""
