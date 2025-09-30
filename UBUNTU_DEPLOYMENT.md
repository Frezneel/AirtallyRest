# 🚀 AirTally REST API - Ubuntu Deployment Guide

Panduan lengkap deploy REST API Rust/Axum + PostgreSQL ke Ubuntu Server (20.04/22.04/24.04).

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Server Preparation](#server-preparation)
3. [Install Dependencies](#install-dependencies)
4. [Database Setup](#database-setup)
5. [Application Deployment](#application-deployment)
6. [Systemd Service Setup](#systemd-service-setup)
7. [Nginx Reverse Proxy](#nginx-reverse-proxy)
8. [SSL/TLS Configuration](#ssltls-configuration)
9. [Firewall Configuration](#firewall-configuration)
10. [Monitoring & Logging](#monitoring--logging)
11. [Maintenance](#maintenance)
12. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Server Requirements:
- **OS**: Ubuntu 20.04 LTS / 22.04 LTS / 24.04 LTS
- **RAM**: Minimum 1GB (recommended 2GB+)
- **Storage**: Minimum 10GB free space
- **CPU**: 1 core (recommended 2+ cores)
- **Network**: Public IP address (untuk production)

### Domain (Optional but recommended):
- Domain name pointing to your server IP
- Example: `api.airtally.com` atau `airtally.yourdomain.com`

---

## Server Preparation

### 1. Login ke Server

```bash
ssh username@your-server-ip

# Atau jika menggunakan SSH key:
ssh -i /path/to/key.pem username@your-server-ip
```

### 2. Update System

```bash
sudo apt update
sudo apt upgrade -y
sudo apt autoremove -y
```

### 3. Create Deployment User (Recommended)

```bash
# Create user untuk menjalankan aplikasi
sudo adduser airtally

# Add to sudo group (optional)
sudo usermod -aG sudo airtally

# Switch to airtally user
su - airtally
```

---

## Install Dependencies

### 1. Install Rust

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Option 1 (default installation)
# Press 1 and Enter

# Load Rust environment
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

**Expected output:**
```
rustc 1.82.0 (f6e511eec 2024-10-15)
cargo 1.82.0 (8f40fc59f 2024-10-07)
```

### 2. Install PostgreSQL

```bash
# Install PostgreSQL 15
sudo apt install -y postgresql postgresql-contrib

# Start PostgreSQL service
sudo systemctl start postgresql
sudo systemctl enable postgresql

# Check status
sudo systemctl status postgresql
```

### 3. Install Nginx (Reverse Proxy)

```bash
sudo apt install -y nginx

# Start Nginx
sudo systemctl start nginx
sudo systemctl enable nginx

# Check status
sudo systemctl status nginx
```

### 4. Install Additional Tools

```bash
# Git untuk clone repository
sudo apt install -y git

# Build essentials
sudo apt install -y build-essential pkg-config libssl-dev

# Certbot untuk SSL (optional, akan dibahas nanti)
sudo apt install -y certbot python3-certbot-nginx
```

---

## Database Setup

### 1. Create PostgreSQL User & Database

```bash
# Switch to postgres user
sudo -i -u postgres

# Create database user
createuser airtally_user -P
# Masukkan password (contoh: airtally_secure_password_123)

# Create database
createdb airtally -O airtally_user

# Exit postgres user
exit
```

### 2. Configure PostgreSQL Access

```bash
# Edit pg_hba.conf untuk allow password authentication
sudo nano /etc/postgresql/15/main/pg_hba.conf
```

**Tambahkan atau ubah line berikut:**
```conf
# TYPE  DATABASE        USER            ADDRESS                 METHOD
local   all             postgres                                peer
local   all             airtally_user                           md5
host    all             airtally_user   127.0.0.1/32            md5
host    all             airtally_user   ::1/128                 md5
```

**Restart PostgreSQL:**
```bash
sudo systemctl restart postgresql
```

### 3. Test Database Connection

```bash
psql -U airtally_user -d airtally -h localhost
# Masukkan password yang dibuat tadi

# Test query
\dt  # List tables (masih kosong)
\q   # Exit
```

---

## Application Deployment

### 1. Clone Repository

```bash
# Ke home directory
cd ~

# Clone repository (ganti dengan URL repo Anda)
git clone https://github.com/yourusername/airtally.git

# Atau jika upload manual, gunakan scp:
# scp -r /path/to/airtally-restapi username@server-ip:~/

cd ~/airtally/airtally-restapi
```

### 2. Configure Environment Variables

```bash
# Copy .env example
cp .env.example .env

# Edit .env file
nano .env
```

**Update .env dengan kredensial production:**
```env
# Database Configuration
DATABASE_URL="postgres://airtally_user:airtally_secure_password_123@localhost:5432/airtally"

# Server Configuration (optional)
RUST_LOG=info
SERVER_HOST=0.0.0.0
SERVER_PORT=3000
```

**Simpan file:** `Ctrl + O`, Enter, `Ctrl + X`

### 3. Build Application (Release Mode)

```bash
# Build dengan optimasi production
cargo build --release

# Ini akan memakan waktu 5-15 menit tergantung spek server
# Binary akan tersimpan di: target/release/airtally-restapi
```

**Verify build:**
```bash
ls -lh target/release/airtally-restapi
# Should show executable file ~10-20MB
```

### 4. Run Database Migrations

```bash
# Test run aplikasi (migrations akan otomatis berjalan)
./target/release/airtally-restapi

# Expected output:
# Successfully connected to the database
# Database migrations ran successfully
# Server listening on 0.0.0.0:3000

# Stop dengan Ctrl + C
```

### 5. Test Application Locally

```bash
# Run aplikasi di background
./target/release/airtally-restapi &

# Test API
curl http://localhost:3000/api/flights

# Expected: {"status":"success","data":[],"total":0}

# Kill background process
pkill airtally-restapi
```

---

## Systemd Service Setup

Membuat systemd service agar aplikasi auto-start saat server restart.

### 1. Create Service File

```bash
sudo nano /etc/systemd/system/airtally-api.service
```

**Paste konfigurasi berikut:**
```ini
[Unit]
Description=AirTally REST API Service
After=network.target postgresql.service
Requires=postgresql.service

[Service]
Type=simple
User=airtally
Group=airtally
WorkingDirectory=/home/airtally/airtally/airtally-restapi
Environment="RUST_LOG=info"
ExecStart=/home/airtally/airtally/airtally-restapi/target/release/airtally-restapi
Restart=always
RestartSec=10
StandardOutput=append:/home/airtally/airtally/airtally-restapi/logs/service.log
StandardError=append:/home/airtally/airtally/airtally-restapi/logs/service_error.log

[Install]
WantedBy=multi-user.target
```

**Simpan:** `Ctrl + O`, Enter, `Ctrl + X`

### 2. Create Logs Directory

```bash
mkdir -p ~/airtally/airtally-restapi/logs
```

### 3. Enable and Start Service

```bash
# Reload systemd daemon
sudo systemctl daemon-reload

# Enable service (auto-start on boot)
sudo systemctl enable airtally-api

# Start service
sudo systemctl start airtally-api

# Check status
sudo systemctl status airtally-api
```

**Expected status:**
```
● airtally-api.service - AirTally REST API Service
     Loaded: loaded (/etc/systemd/system/airtally-api.service; enabled)
     Active: active (running) since ...
```

### 4. Verify Service is Running

```bash
# Check if port 3000 is listening
sudo ss -tulpn | grep 3000

# Test API
curl http://localhost:3000/api/flights
```

### 5. Useful Service Commands

```bash
# Start service
sudo systemctl start airtally-api

# Stop service
sudo systemctl stop airtally-api

# Restart service
sudo systemctl restart airtally-api

# Check status
sudo systemctl status airtally-api

# View logs (real-time)
sudo journalctl -u airtally-api -f

# View recent logs
sudo journalctl -u airtally-api -n 50
```

---

## Nginx Reverse Proxy

Setup Nginx sebagai reverse proxy untuk expose API ke public dengan domain.

### 1. Remove Default Nginx Config

```bash
sudo rm /etc/nginx/sites-enabled/default
```

### 2. Create Nginx Configuration

```bash
sudo nano /etc/nginx/sites-available/airtally-api
```

**Paste konfigurasi berikut:**

#### Option A: Without Domain (Direct IP)

```nginx
server {
    listen 80;
    listen [::]:80;
    server_name _;  # Accept any hostname

    # API Location
    location /api {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
```

#### Option B: With Domain

```nginx
server {
    listen 80;
    listen [::]:80;
    server_name api.yourdomain.com;  # Ganti dengan domain Anda

    # API Location
    location /api {
        proxy_pass http://127.0.0.1:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_cache_bypass $http_upgrade;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # Health check endpoint
    location /health {
        access_log off;
        return 200 "healthy\n";
        add_header Content-Type text/plain;
    }
}
```

**Simpan:** `Ctrl + O`, Enter, `Ctrl + X`

### 3. Enable Configuration

```bash
# Create symbolic link
sudo ln -s /etc/nginx/sites-available/airtally-api /etc/nginx/sites-enabled/

# Test Nginx configuration
sudo nginx -t

# Expected output:
# nginx: configuration file /etc/nginx/nginx.conf test is successful

# Reload Nginx
sudo systemctl reload nginx
```

### 4. Test Nginx Proxy

```bash
# Test dari server
curl http://localhost/api/flights

# Test dari komputer lain (ganti dengan IP server)
curl http://YOUR_SERVER_IP/api/flights
```

---

## SSL/TLS Configuration

Setup HTTPS dengan Let's Encrypt (FREE SSL certificate).

**Requirement:** Domain name harus sudah pointing ke server IP Anda!

### 1. Install Certbot

```bash
# Already installed from earlier, verify:
certbot --version
```

### 2. Obtain SSL Certificate

```bash
# Untuk domain dengan Nginx
sudo certbot --nginx -d api.yourdomain.com

# Follow prompts:
# - Email: your@email.com
# - Agree to TOS: Y
# - Share email: N (optional)
# - Redirect HTTP to HTTPS: 2 (Yes, recommended)
```

**Certbot akan otomatis:**
- Generate SSL certificate
- Update Nginx configuration
- Setup auto-renewal

### 3. Verify SSL Configuration

```bash
# Test HTTPS
curl https://api.yourdomain.com/api/flights

# Check certificate
sudo certbot certificates
```

### 4. Auto-Renewal Test

```bash
# Test renewal (dry-run)
sudo certbot renew --dry-run

# Expected: "Congratulations, all renewals succeeded"
```

**Note:** Certbot auto-renewal sudah di-setup via systemd timer. Certificate akan auto-renew sebelum expired.

### 5. Manual Renewal (if needed)

```bash
sudo certbot renew
sudo systemctl reload nginx
```

---

## Firewall Configuration

Setup UFW (Uncomplicated Firewall) untuk security.

### 1. Install and Enable UFW

```bash
# Install UFW (usually pre-installed)
sudo apt install -y ufw

# Set default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing
```

### 2. Allow Required Ports

```bash
# SSH (IMPORTANT: Allow SSH before enabling firewall!)
sudo ufw allow 22/tcp

# HTTP
sudo ufw allow 80/tcp

# HTTPS
sudo ufw allow 443/tcp

# PostgreSQL (only if remote access needed)
# sudo ufw allow from YOUR_IP_ADDRESS to any port 5432
```

### 3. Enable Firewall

```bash
# Enable UFW
sudo ufw enable

# Check status
sudo ufw status verbose
```

**Expected output:**
```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
80/tcp                     ALLOW       Anywhere
443/tcp                    ALLOW       Anywhere
```

### 4. Additional Security (Optional)

```bash
# Limit SSH connections (prevent brute-force)
sudo ufw limit 22/tcp

# Allow specific IP only for SSH
# sudo ufw delete allow 22/tcp
# sudo ufw allow from YOUR_IP_ADDRESS to any port 22
```

---

## Monitoring & Logging

### 1. View Application Logs

```bash
# Real-time logs
sudo journalctl -u airtally-api -f

# Recent logs (last 100 lines)
sudo journalctl -u airtally-api -n 100

# Logs from specific time
sudo journalctl -u airtally-api --since "1 hour ago"

# Logs with priority (errors only)
sudo journalctl -u airtally-api -p err

# Export logs
sudo journalctl -u airtally-api > ~/airtally_logs.txt
```

### 2. View Nginx Logs

```bash
# Access logs
sudo tail -f /var/log/nginx/access.log

# Error logs
sudo tail -f /var/log/nginx/error.log
```

### 3. Monitor System Resources

```bash
# CPU and Memory usage
htop

# Or use top
top

# Disk usage
df -h

# Application-specific process
ps aux | grep airtally-restapi
```

### 4. Setup Log Rotation (Recommended)

```bash
# Create logrotate config
sudo nano /etc/logrotate.d/airtally-api
```

**Paste:**
```
/home/airtally/airtally/airtally-restapi/logs/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0644 airtally airtally
    sharedscripts
    postrotate
        systemctl reload airtally-api > /dev/null 2>&1 || true
    endscript
}
```

### 5. Health Check Monitoring (Optional)

Create simple monitoring script:

```bash
nano ~/monitor-airtally.sh
```

**Paste:**
```bash
#!/bin/bash
URL="http://localhost:3000/api/flights"
RESPONSE=$(curl -s -o /dev/null -w "%{http_code}" $URL)

if [ $RESPONSE -eq 200 ]; then
    echo "$(date): API is healthy (HTTP $RESPONSE)"
else
    echo "$(date): API is down or unhealthy (HTTP $RESPONSE)"
    # Optional: Restart service
    # sudo systemctl restart airtally-api
fi
```

**Make executable:**
```bash
chmod +x ~/monitor-airtally.sh
```

**Add to cron (check every 5 minutes):**
```bash
crontab -e

# Add line:
*/5 * * * * /home/airtally/monitor-airtally.sh >> /home/airtally/monitor.log 2>&1
```

---

## Maintenance

### 1. Update Application

```bash
# Navigate to project directory
cd ~/airtally/airtally-restapi

# Pull latest changes
git pull origin main

# Rebuild application
cargo build --release

# Restart service
sudo systemctl restart airtally-api

# Verify
sudo systemctl status airtally-api
```

### 2. Database Backup

**Create backup script:**
```bash
nano ~/backup-database.sh
```

**Paste:**
```bash
#!/bin/bash
BACKUP_DIR="$HOME/database_backups"
DATE=$(date +%Y%m%d_%H%M%S)
DB_NAME="airtally"
DB_USER="airtally_user"

mkdir -p $BACKUP_DIR

# Backup database
PGPASSWORD='airtally_secure_password_123' pg_dump -U $DB_USER -h localhost $DB_NAME | gzip > "$BACKUP_DIR/airtally_backup_$DATE.sql.gz"

# Keep only last 30 days of backups
find $BACKUP_DIR -name "airtally_backup_*.sql.gz" -mtime +30 -delete

echo "Backup completed: airtally_backup_$DATE.sql.gz"
```

**Make executable:**
```bash
chmod +x ~/backup-database.sh
```

**Schedule daily backup (2 AM):**
```bash
crontab -e

# Add line:
0 2 * * * /home/airtally/backup-database.sh >> /home/airtally/backup.log 2>&1
```

**Test backup:**
```bash
./backup-database.sh
ls -lh ~/database_backups/
```

### 3. Database Restore

```bash
# Extract backup
gunzip airtally_backup_YYYYMMDD_HHMMSS.sql.gz

# Restore
PGPASSWORD='airtally_secure_password_123' psql -U airtally_user -h localhost -d airtally < airtally_backup_YYYYMMDD_HHMMSS.sql
```

### 4. Check Disk Space

```bash
# Overall disk usage
df -h

# Application directory size
du -sh ~/airtally

# Database size
sudo -u postgres psql -c "SELECT pg_size_pretty(pg_database_size('airtally'));"

# Clear old build artifacts
cd ~/airtally/airtally-restapi
cargo clean
```

---

## Troubleshooting

### Service Won't Start

```bash
# Check detailed error
sudo journalctl -u airtally-api -n 50 --no-pager

# Common issues:
# 1. Database connection failed
#    - Check .env DATABASE_URL
#    - Verify PostgreSQL is running: sudo systemctl status postgresql
#    - Test connection: psql -U airtally_user -d airtally -h localhost

# 2. Port already in use
#    - Find process: sudo lsof -i :3000
#    - Kill process: sudo kill -9 PID

# 3. Permission denied
#    - Check file ownership: ls -l ~/airtally/airtally-restapi/target/release/airtally-restapi
#    - Fix: sudo chown -R airtally:airtally ~/airtally
```

### Database Connection Issues

```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Test connection manually
psql -U airtally_user -d airtally -h localhost

# Check pg_hba.conf
sudo cat /etc/postgresql/15/main/pg_hba.conf | grep airtally_user

# Restart PostgreSQL
sudo systemctl restart postgresql
```

### Nginx Issues

```bash
# Test Nginx configuration
sudo nginx -t

# Check Nginx error log
sudo tail -f /var/log/nginx/error.log

# Restart Nginx
sudo systemctl restart nginx

# Check if Nginx is listening
sudo ss -tulpn | grep nginx
```

### SSL Certificate Issues

```bash
# Check certificate status
sudo certbot certificates

# Renew certificate manually
sudo certbot renew --force-renewal

# Check certificate expiry
echo | openssl s_client -servername api.yourdomain.com -connect api.yourdomain.com:443 2>/dev/null | openssl x509 -noout -dates
```

### High CPU/Memory Usage

```bash
# Check resource usage
htop

# Check application process
ps aux | grep airtally-restapi

# Restart service
sudo systemctl restart airtally-api

# Check for memory leaks in logs
sudo journalctl -u airtally-api | grep -i "memory\|panic"
```

### API Returns 502 Bad Gateway

```bash
# Check if application is running
sudo systemctl status airtally-api

# Check if port 3000 is listening
sudo ss -tulpn | grep 3000

# Check application logs
sudo journalctl -u airtally-api -n 50

# Restart both services
sudo systemctl restart airtally-api
sudo systemctl restart nginx
```

---

## 🎯 Production Checklist

Before going to production, verify:

- [ ] PostgreSQL is running and accessible
- [ ] Database migrations completed successfully
- [ ] `.env` file has production credentials (NOT the example ones!)
- [ ] Application builds successfully in release mode
- [ ] Systemd service is enabled and running
- [ ] Nginx reverse proxy configured
- [ ] SSL certificate installed (if using domain)
- [ ] Firewall configured (UFW enabled)
- [ ] Database backup scheduled (cron job)
- [ ] Log rotation configured
- [ ] Health check monitoring setup
- [ ] Application accessible from public IP/domain
- [ ] API endpoints return expected responses
- [ ] Error logging working properly
- [ ] Service auto-starts after reboot (test it!)

---

## 🔗 Useful Commands Reference

```bash
# Application
sudo systemctl start|stop|restart|status airtally-api
sudo journalctl -u airtally-api -f

# Database
sudo systemctl start|stop|restart|status postgresql
psql -U airtally_user -d airtally -h localhost

# Nginx
sudo systemctl start|stop|restart|status nginx
sudo nginx -t

# Firewall
sudo ufw status
sudo ufw allow|deny PORT

# SSL
sudo certbot renew
sudo certbot certificates

# Monitoring
htop
df -h
du -sh /path/to/directory
```

---

## 📚 Additional Resources

- [Rust Official Documentation](https://www.rust-lang.org/learn)
- [Axum Framework Guide](https://docs.rs/axum/latest/axum/)
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [Nginx Documentation](https://nginx.org/en/docs/)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [UFW Guide](https://help.ubuntu.com/community/UFW)

---

**Deployment Guide Version:** 1.0
**Last Updated:** 2025-09-30
**Tested on:** Ubuntu 22.04 LTS

**Questions or Issues?** Check the [Troubleshooting](#troubleshooting) section or open an issue on GitHub.