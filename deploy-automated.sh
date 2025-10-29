#!/bin/bash

################################################################################
# AirTally REST API - Automated Deployment Script
# Target: Ubuntu 22.04 LTS
# Server: 10.17.6.155
# GitHub: https://github.com/Frezneel/AirtallyRest
#
# Usage:
#   chmod +x deploy-automated.sh
#   ./deploy-automated.sh
#
# Note: Jalankan sebagai user yang punya sudo access
################################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
APP_USER="airtally"
APP_DIR="/home/${APP_USER}/AirtallyRest"
GITHUB_REPO="https://github.com/Frezneel/AirtallyRest.git"
DB_NAME="airtally_prod"
DB_USER="airtally_user"
SERVICE_NAME="airtally-api"

################################################################################
# Helper Functions
################################################################################

print_header() {
    echo ""
    echo -e "${BLUE}======================================================${NC}"
    echo -e "${BLUE}  $1${NC}"
    echo -e "${BLUE}======================================================${NC}"
    echo ""
}

print_step() {
    echo -e "${GREEN}[STEP $1/$2]${NC} $3"
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

check_command() {
    if command -v $1 &> /dev/null; then
        print_success "$1 sudah terinstall"
        return 0
    else
        print_warning "$1 belum terinstall"
        return 1
    fi
}

prompt_input() {
    local prompt="$1"
    local var_name="$2"
    local default="$3"

    if [ -n "$default" ]; then
        read -p "$prompt [$default]: " input
        input=${input:-$default}
    else
        read -p "$prompt: " input
    fi

    eval $var_name="'$input'"
}

prompt_password() {
    local prompt="$1"
    local var_name="$2"

    read -s -p "$prompt: " password
    echo ""
    read -s -p "Confirm password: " password_confirm
    echo ""

    if [ "$password" != "$password_confirm" ]; then
        print_error "Password tidak cocok!"
        exit 1
    fi

    eval $var_name="'$password'"
}

################################################################################
# Main Script
################################################################################

print_header "AirTally REST API - Automated Deployment"

echo "Script ini akan menginstall dan configure:"
echo "  - Rust toolchain"
echo "  - PostgreSQL database"
echo "  - Nginx reverse proxy"
echo "  - AirTally REST API service"
echo ""
echo "Target server: 10.17.6.155"
echo ""
read -p "Lanjutkan? (y/n): " confirm

if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
    echo "Deployment dibatalkan"
    exit 0
fi

TOTAL_STEPS=12

################################################################################
# Step 1: Check prerequisites
################################################################################

print_step 1 $TOTAL_STEPS "Checking prerequisites"

if [ "$EUID" -eq 0 ]; then
    print_error "Jangan jalankan script ini sebagai root!"
    print_warning "Login sebagai user biasa yang punya sudo access"
    exit 1
fi

if ! sudo -n true 2>/dev/null; then
    print_warning "Script ini membutuhkan sudo access"
    sudo -v
fi

print_success "Prerequisites OK"

################################################################################
# Step 2: Collect configuration
################################################################################

print_step 2 $TOTAL_STEPS "Collecting configuration"

echo ""
echo "Konfigurasi Database:"
prompt_password "Masukkan password untuk database user '$DB_USER'" DB_PASSWORD

echo ""
print_success "Configuration collected"

################################################################################
# Step 3: Update system
################################################################################

print_step 3 $TOTAL_STEPS "Updating system packages"

sudo apt update -qq
sudo apt upgrade -y -qq
sudo apt install -y curl wget git build-essential pkg-config libssl-dev

print_success "System updated"

################################################################################
# Step 4: Create application user
################################################################################

print_step 4 $TOTAL_STEPS "Creating application user"

if id "$APP_USER" &>/dev/null; then
    print_warning "User $APP_USER sudah ada, skip..."
else
    sudo useradd -m -s /bin/bash $APP_USER
    print_success "User $APP_USER dibuat"
fi

################################################################################
# Step 5: Install Rust
################################################################################

print_step 5 $TOTAL_STEPS "Installing Rust"

if sudo -u $APP_USER bash -c "command -v rustc" &>/dev/null; then
    print_warning "Rust sudah terinstall, skip..."
else
    sudo -u $APP_USER bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"
    print_success "Rust installed"
fi

################################################################################
# Step 6: Install PostgreSQL
################################################################################

print_step 6 $TOTAL_STEPS "Installing PostgreSQL"

if check_command psql; then
    print_warning "PostgreSQL sudah terinstall, skip..."
else
    sudo apt install -y postgresql postgresql-contrib
    sudo systemctl start postgresql
    sudo systemctl enable postgresql
    print_success "PostgreSQL installed"
fi

################################################################################
# Step 7: Setup database
################################################################################

print_step 7 $TOTAL_STEPS "Setting up database"

# Check if user exists
if sudo -u postgres psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='$DB_USER'" | grep -q 1; then
    print_warning "Database user sudah ada, skip..."
else
    sudo -u postgres psql << EOF
CREATE USER $DB_USER WITH PASSWORD '$DB_PASSWORD';
CREATE DATABASE $DB_NAME OWNER $DB_USER;
GRANT ALL PRIVILEGES ON DATABASE $DB_NAME TO $DB_USER;
EOF
    print_success "Database created"
fi

# Test connection
if PGPASSWORD=$DB_PASSWORD psql -h localhost -U $DB_USER -d $DB_NAME -c "SELECT 1" &>/dev/null; then
    print_success "Database connection OK"
else
    print_error "Database connection failed"
    exit 1
fi

################################################################################
# Step 8: Install Nginx
################################################################################

print_step 8 $TOTAL_STEPS "Installing Nginx"

if check_command nginx; then
    print_warning "Nginx sudah terinstall, skip..."
else
    sudo apt install -y nginx
    sudo systemctl start nginx
    sudo systemctl enable nginx
    print_success "Nginx installed"
fi

################################################################################
# Step 9: Clone and build application
################################################################################

print_step 9 $TOTAL_STEPS "Cloning and building application (this may take 10-15 minutes)"

if [ -d "$APP_DIR" ]; then
    print_warning "Directory $APP_DIR sudah ada"
    read -p "Pull latest changes? (y/n): " pull_confirm
    if [ "$pull_confirm" = "y" ] || [ "$pull_confirm" = "Y" ]; then
        sudo -u $APP_USER bash -c "cd $APP_DIR && git pull"
        print_success "Code updated"
    fi
else
    sudo -u $APP_USER git clone $GITHUB_REPO $APP_DIR
    print_success "Repository cloned"
fi

# Install sqlx-cli
if sudo -u $APP_USER bash -c "command -v sqlx" &>/dev/null; then
    print_warning "sqlx-cli sudah terinstall, skip..."
else
    print_warning "Installing sqlx-cli (this may take 5-10 minutes)..."
    sudo -u $APP_USER bash -c "source \$HOME/.cargo/env && cargo install sqlx-cli --no-default-features --features postgres"
    print_success "sqlx-cli installed"
fi

# Create .env file
print_warning "Creating .env file..."
sudo -u $APP_USER bash -c "cat > $APP_DIR/.env" << EOF
# Database Configuration
DATABASE_URL=postgresql://$DB_USER:$DB_PASSWORD@localhost:5432/$DB_NAME

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=3000

# Environment
ENVIRONMENT=production

# Logging
LOG_LEVEL=info

# Features
ENABLE_SWAGGER=false

# Security
RATE_LIMIT_PER_MINUTE=100
EOF

sudo -u $APP_USER chmod 600 $APP_DIR/.env
print_success ".env file created"

# Build application
print_warning "Building application (this will take 10-15 minutes)..."
sudo -u $APP_USER bash -c "source \$HOME/.cargo/env && cd $APP_DIR && cargo build --release"
print_success "Application built"

# Run migrations
print_warning "Running database migrations..."
sudo -u $APP_USER bash -c "source \$HOME/.cargo/env && cd $APP_DIR && sqlx migrate run"
print_success "Migrations completed"

################################################################################
# Step 10: Create systemd service
################################################################################

print_step 10 $TOTAL_STEPS "Creating systemd service"

sudo tee /etc/systemd/system/$SERVICE_NAME.service > /dev/null << EOF
[Unit]
Description=AirTally REST API Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=$APP_USER
Group=$APP_USER
WorkingDirectory=$APP_DIR
Environment="RUST_LOG=info"
ExecStart=$APP_DIR/target/release/airtally-rest
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=$SERVICE_NAME

# Security
NoNewPrivileges=true
PrivateTmp=true

[Install]
WantedBy=multi-user.target
EOF

sudo systemctl daemon-reload
sudo systemctl enable $SERVICE_NAME
sudo systemctl start $SERVICE_NAME

sleep 3

if sudo systemctl is-active --quiet $SERVICE_NAME; then
    print_success "Service started successfully"
else
    print_error "Service failed to start"
    echo "Check logs dengan: sudo journalctl -u $SERVICE_NAME -n 50"
    exit 1
fi

################################################################################
# Step 11: Configure Nginx
################################################################################

print_step 11 $TOTAL_STEPS "Configuring Nginx"

sudo tee /etc/nginx/sites-available/$SERVICE_NAME > /dev/null << 'EOF'
server {
    listen 80;
    server_name 10.17.6.155;

    # Logs
    access_log /var/log/nginx/airtally-api.access.log;
    error_log /var/log/nginx/airtally-api.error.log;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;

    # Max upload size
    client_max_body_size 10M;

    # API endpoints
    location /api/ {
        proxy_pass http://127.0.0.1:3000/api/;
        proxy_http_version 1.1;

        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check
    location /health {
        proxy_pass http://127.0.0.1:3000/health;
        access_log off;
    }

    # Root
    location / {
        return 200 '{"status":"ok","service":"AirTally REST API"}';
        add_header Content-Type application/json;
    }
}
EOF

# Enable site
sudo ln -sf /etc/nginx/sites-available/$SERVICE_NAME /etc/nginx/sites-enabled/

# Test configuration
if sudo nginx -t 2>&1 | grep -q "successful"; then
    print_success "Nginx configuration OK"
    sudo systemctl reload nginx
else
    print_error "Nginx configuration error"
    exit 1
fi

################################################################################
# Step 12: Configure firewall
################################################################################

print_step 12 $TOTAL_STEPS "Configuring firewall"

sudo ufw --force enable
sudo ufw allow 22/tcp  # SSH
sudo ufw allow 80/tcp  # HTTP

print_success "Firewall configured"

################################################################################
# Testing
################################################################################

print_header "Testing Installation"

echo "Testing API..."
sleep 2

if curl -s http://localhost:3000/api/starter-data/version | grep -q "success"; then
    print_success "API test passed (direct)"
else
    print_error "API test failed (direct)"
fi

if curl -s http://10.17.6.155/api/starter-data/version | grep -q "success"; then
    print_success "API test passed (via Nginx)"
else
    print_error "API test failed (via Nginx)"
fi

################################################################################
# Summary
################################################################################

print_header "Deployment Complete!"

echo -e "${GREEN}Installation berhasil!${NC}"
echo ""
echo "📋 Informasi Penting:"
echo "  • API URL: http://10.17.6.155/api"
echo "  • Health Check: http://10.17.6.155/health"
echo "  • Version: http://10.17.6.155/api/starter-data/version"
echo ""
echo "📁 Path Penting:"
echo "  • App Directory: $APP_DIR"
echo "  • Binary: $APP_DIR/target/release/airtally-rest"
echo "  • Config: $APP_DIR/.env"
echo ""
echo "🔧 Useful Commands:"
echo "  • View logs: sudo journalctl -u $SERVICE_NAME -f"
echo "  • Restart: sudo systemctl restart $SERVICE_NAME"
echo "  • Status: sudo systemctl status $SERVICE_NAME"
echo ""
echo "📱 Next Steps:"
echo "  1. Test API dari browser: http://10.17.6.155/api/starter-data/version"
echo "  2. Update Flutter app untuk connect ke: http://10.17.6.155/api"
echo "  3. Backup database: pg_dump -h localhost -U $DB_USER $DB_NAME > backup.sql"
echo ""
echo "📖 Documentation:"
echo "  • Manual Guide: $APP_DIR/DEPLOYMENT_MANUAL.md"
echo "  • Update Script: $APP_DIR/update-api.sh"
echo ""

print_success "Deployment selesai!"
