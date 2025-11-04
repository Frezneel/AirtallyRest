# 🦅 FALCON REST API

**Flight pAssenger vaLidation and COuntiNg - REST API Backend**

Version 2.0.0 - Production-ready API with authentication and user management.

---

## 📋 Overview

FALCON REST API is a high-performance backend service built with Rust for managing airport barcode scanning operations, flight data, and user authentication.

### Key Features

- ✅ **Authentication & Authorization** - JWT-based auth with role-based access control (RBAC)
- ✅ **User Management** - Create, edit, and manage users with different roles (Admin, Supervisor, Operator)
- ✅ **Barcode Processing** - Fast PDF417 barcode parsing for boarding passes
- ✅ **Flight Management** - Track flights, passenger counts, and scanning operations
- ✅ **Offline Support** - Handle offline data sync from mobile devices
- ✅ **High Performance** - Built with Rust/Actix-web for maximum throughput
- ✅ **PostgreSQL Database** - Reliable data storage with ACID compliance
- ✅ **Security Features** - API key authentication, JWT tokens, rate limiting

---

## 🚀 Quick Start

### Prerequisites

- Rust 1.75+ (install from https://rustup.rs)
- PostgreSQL 14+
- Linux/Ubuntu server (recommended for production)
- Git

### Development Setup

```bash
# Clone repository
git clone https://github.com/Frezneel/FalconRest.git
cd FalconRest

# Copy environment file
cp .env.example .env

# Edit .env with your database credentials
nano .env

# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Create database
createdb airtally

# Run migrations
sqlx migrate run

# Build and run
cargo run --release
```

The API will be available at `http://localhost:3000`

---

## 📦 Production Deployment

### For New Installations

Follow the comprehensive guide in **[DEPLOYMENT_CHECKLIST.md](DEPLOYMENT_CHECKLIST.md)**

Key steps:
1. Create dedicated user and directories
2. Configure PostgreSQL database
3. Set environment variables (including **JWT_SECRET**)
4. Build application
5. Set up systemd service
6. Configure firewall and security

### For Existing AirTally Installations

If you already have AirTally running in production and want to migrate to FALCON:

**Option 1: Automated Migration (Recommended)**

Use the migration script for a safe, guided upgrade:

```bash
# On production server, as 'airtally' user
cd ~/AirtallyRest

# Download migration script
wget https://raw.githubusercontent.com/Frezneel/FalconRest/main/migrate-to-falcon.sh
chmod +x migrate-to-falcon.sh

# Run migration
./migrate-to-falcon.sh
```

The script will:
- ✅ Backup your current installation
- ✅ Add JWT_SECRET (critical security fix)
- ✅ Pull latest FALCON code
- ✅ Build new version
- ✅ Run database migrations
- ✅ Restart service with minimal downtime (~30 seconds)

**Option 2: Manual Migration**

Follow the detailed guide in **[MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)**

---

## 🔐 Security Configuration

### Critical: JWT_SECRET

⚠️ **WARNING**: JWT_SECRET must be set for production!

```bash
# Generate secure JWT secret
openssl rand -base64 48

# Add to .env file
echo "JWT_SECRET=<generated_secret>" >> .env
```

Without JWT_SECRET, you will see this warning:
```
JWT_SECRET not set, using default (NOT SECURE FOR PRODUCTION)
```

**Why this matters:**
- JWT tokens authenticate all users
- Default secret = anyone can create fake tokens
- Attackers can impersonate users (including admins)
- **Your entire authentication system is compromised**

### Other Security Settings

```bash
# API Key (change in production)
API_KEY=$(openssl rand -hex 32)

# Rate limiting
RATE_LIMIT_PER_MINUTE=100

# Environment
ENVIRONMENT=production

# Disable Swagger in production
ENABLE_SWAGGER=false
```

---

## 📝 Configuration Files

### .env File (Production Example)

```bash
# Database
DATABASE_URL=postgresql://falcon_user:STRONG_PASSWORD@localhost:5432/airtally

# Server
HOST=0.0.0.0
PORT=3000

# Environment
ENVIRONMENT=production
LOG_LEVEL=info

# Security (CRITICAL - MUST BE CHANGED!)
API_KEY=<openssl rand -hex 32>
JWT_SECRET=<openssl rand -base64 48>
RATE_LIMIT_PER_MINUTE=100

# Features
ENABLE_SWAGGER=false

# Database Pool
DB_MIN_CONNECTIONS=10
DB_MAX_CONNECTIONS=50
```

### Systemd Service

Located at `/etc/systemd/system/falcon-api.service` (or `airtally-api.service` for in-place migrations):

```ini
[Unit]
Description=FALCON REST API Service
After=network.target postgresql.service

[Service]
Type=simple
User=falcon
WorkingDirectory=/home/falcon/FalconRest/AirtallyRest
ExecStart=/home/falcon/FalconRest/AirtallyRest/target/release/falcon-rest
Restart=always

[Install]
WantedBy=multi-user.target
```

---

## 🛠️ Management Commands

### Service Management

```bash
# Status
sudo systemctl status falcon-api

# Start/Stop/Restart
sudo systemctl start falcon-api
sudo systemctl stop falcon-api
sudo systemctl restart falcon-api

# View logs
sudo journalctl -u falcon-api -f

# View last 100 lines
sudo journalctl -u falcon-api -n 100
```

### Database Management

```bash
# Connect to database
psql -U falcon_user -d airtally

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert

# Check migration status
sqlx migrate info

# Create backup
pg_dump -U falcon_user airtally | gzip > backup_$(date +%Y%m%d).sql.gz
```

### Updates

For production servers, use the update script:

```bash
sudo su - falcon  # or airtally for in-place setups
cd ~/FalconRest/AirtallyRest
./update-api.sh main
```

The script handles:
- Backup creation
- Service stop/start
- Code pull and build
- Migration execution
- Verification

---

## 📚 API Documentation

### Authentication

All API requests (except public endpoints) require:

```bash
# Headers
X-API-Key: <your_api_key>
Authorization: Bearer <jwt_token>
```

### Key Endpoints

```bash
# Health Check (no auth)
GET /api/starter-data/version

# Authentication
POST /api/auth/login
POST /api/auth/logout
POST /api/auth/change-password

# User Management (Admin only)
GET  /api/users
POST /api/users
GET  /api/users/:id
PUT  /api/users/:id
POST /api/users/:id/reset-password

# Roles & Permissions
GET /api/roles
GET /api/roles/:id/permissions

# Barcode Operations
POST /api/barcodes
GET  /api/barcodes
GET  /api/barcodes/:id

# Flight Management
GET  /api/flights
POST /api/flights
GET  /api/flights/:id
PUT  /api/flights/:id

# Code Data
GET /api/codes/airlines
GET /api/codes/airports
GET /api/codes/classes
```

### Full API Documentation

When `ENABLE_SWAGGER=true`, visit: `http://localhost:3000/swagger-ui`

⚠️ Disable Swagger in production for security.

---

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Test database connection
psql $DATABASE_URL -c "SELECT 1"

# Test API endpoint
curl http://localhost:3000/api/starter-data/version
```

---

## 📊 Monitoring & Troubleshooting

### Check Service Status

```bash
# Is service running?
systemctl is-active falcon-api

# Service status details
sudo systemctl status falcon-api

# Service start time
systemctl show falcon-api --property=ActiveEnterTimestamp
```

### Common Issues

#### JWT_SECRET Warning

**Symptom:** Log shows "JWT_SECRET not set, using default"

**Fix:**
```bash
# Add to .env
JWT_SECRET=$(openssl rand -base64 48)
echo "JWT_SECRET=$JWT_SECRET" >> /home/falcon/FalconRest/AirtallyRest/.env

# Restart
sudo systemctl restart falcon-api
```

#### Service Won't Start

**Check logs:**
```bash
sudo journalctl -u falcon-api -n 50
```

**Common causes:**
- Database connection failed (check DATABASE_URL)
- Port already in use (check `netstat -tlnp | grep 3000`)
- Permission issues (check file ownership)
- Binary not found (check ExecStart path)

#### Database Connection Failed

```bash
# Test connection
psql $DATABASE_URL

# Check PostgreSQL is running
sudo systemctl status postgresql

# Check if database exists
sudo -u postgres psql -l | grep airtally
```

#### High Memory Usage

```bash
# Check memory
free -h

# Check connection pool settings in .env
DB_MAX_CONNECTIONS=50  # Reduce if needed
```

---

## 🔄 Backup & Recovery

### Automatic Backups

Set up daily database backups with cron:

```bash
# Create backup script
cat > /home/falcon/backup-db.sh <<'EOF'
#!/bin/bash
BACKUP_DIR="/home/falcon/backups/database"
DATE=$(date +%Y%m%d_%H%M%S)
mkdir -p $BACKUP_DIR
pg_dump -U falcon_user airtally | gzip > $BACKUP_DIR/falcon_${DATE}.sql.gz
find $BACKUP_DIR -name "falcon_*.sql.gz" -mtime +30 -delete
EOF

chmod +x /home/falcon/backup-db.sh

# Add to crontab (daily at 2 AM)
(crontab -l 2>/dev/null; echo "0 2 * * * /home/falcon/backup-db.sh") | crontab -
```

### Restore from Backup

```bash
# Stop service
sudo systemctl stop falcon-api

# Restore database
gunzip -c backup_YYYYMMDD_HHMMSS.sql.gz | psql -U falcon_user airtally

# Start service
sudo systemctl start falcon-api
```

---

## 🏗️ Architecture

```
┌─────────────────┐
│  Mobile App     │ (Flutter)
│  (FALCON)       │
└────────┬────────┘
         │ HTTPS/HTTP
         │ JWT + API Key
         ▼
┌─────────────────┐
│  Nginx          │ (Optional)
│  Reverse Proxy  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Actix-web      │ (Rust)
│  REST API       │
│  - Auth         │
│  - Barcode      │
│  - User Mgmt    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  PostgreSQL     │
│  Database       │
│  - Users        │
│  - Flights      │
│  - Barcodes     │
└─────────────────┘
```

---

## 📖 Additional Documentation

- **[DEPLOYMENT_CHECKLIST.md](DEPLOYMENT_CHECKLIST.md)** - Complete production deployment guide
- **[MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)** - Migrate from AirTally to FALCON
- **[migrate-to-falcon.sh](migrate-to-falcon.sh)** - Automated migration script
- **[update-api.sh](update-api.sh)** - Update script for production

---

## 🤝 Support

### Logs and Diagnostics

```bash
# Export logs for troubleshooting
sudo journalctl -u falcon-api --since "1 hour ago" > logs.txt

# Check database connections
psql -U falcon_user -d airtally -c "SELECT count(*) FROM pg_stat_activity"

# Check API response time
curl -w "@-" -o /dev/null -s http://localhost:3000/api/starter-data/version <<'EOF'
time_total: %{time_total}s
EOF
```

### Emergency Contacts

- IT Team: [Contact Info]
- Database Admin: [Contact Info]
- Development Team: [Contact Info]

---

## 📄 License

Proprietary - InJourneyDHX

---

## 🎯 Version History

### Version 2.0.0 (2025-11-04)
- ✅ Complete authentication system with JWT tokens
- ✅ Role-based access control (RBAC)
- ✅ User management features
- ✅ JWT_SECRET security enhancement
- ✅ Modern UI/UX in mobile app
- ✅ Production-ready security hardening
- ✅ Comprehensive deployment documentation

### Version 1.2.2
- Offline performance improvements
- PDF417 barcode filter
- Reduced sync timeout

### Version 1.0.0
- Initial release
- Basic barcode scanning
- Flight management

---

**Built with ❤️ by InJourneyDHX Team**
