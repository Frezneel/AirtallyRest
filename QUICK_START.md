# 🚀 Quick Start - Deploy ke Ubuntu dalam 10 Menit

Panduan super cepat untuk deploy AirTally REST API ke Ubuntu server.

---

## Prerequisites

- Ubuntu server dengan SSH access
- Domain name (optional, bisa pakai IP)

---

## Step 1: Persiapan Server (2 menit)

```bash
# Login ke server
ssh username@your-server-ip

# Update system
sudo apt update && sudo apt upgrade -y

# Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

sudo apt install -y postgresql postgresql-contrib nginx git build-essential pkg-config libssl-dev
```

---

## Step 2: Setup Database (1 menit)

```bash
# Create database user & database
sudo -u postgres psql -c "CREATE USER airtally_user WITH PASSWORD 'ganti_password_ini';"
sudo -u postgres psql -c "CREATE DATABASE airtally OWNER airtally_user;"

# Create .pgpass untuk backup tanpa password prompt
echo "localhost:5432:airtally:airtally_user:ganti_password_ini" > ~/.pgpass
chmod 600 ~/.pgpass
```

---

## Step 3: Clone & Build (5-10 menit)

```bash
# Clone repository
cd ~
git clone https://github.com/yourusername/airtally.git
cd airtally/airtally-restapi

# Setup environment
cp .env.example .env
nano .env
# Update DATABASE_URL dengan password yang kamu buat di Step 2

# Build (akan memakan waktu 5-10 menit)
cargo build --release
```

---

## Step 4: Deploy dengan Script (1 menit)

```bash
# Buat script executable
chmod +x deploy.sh

# Run deployment
./deploy.sh

# Script akan otomatis:
# - Build aplikasi
# - Setup systemd service
# - Start service
# - Test API
```

---

## Step 5: Setup Nginx (1 menit)

```bash
# Copy nginx config
sudo cp nginx.conf.example /etc/nginx/sites-available/airtally-api

# Edit untuk ganti domain (jika ada)
sudo nano /etc/nginx/sites-available/airtally-api
# Ubah server_name api.yourdomain.com dengan domain Anda
# Atau biarkan _ untuk accept any hostname

# Enable config
sudo ln -s /etc/nginx/sites-available/airtally-api /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default

# Test & reload
sudo nginx -t
sudo systemctl reload nginx
```

---

## Step 6: Firewall & SSL (2 menit)

```bash
# Setup firewall
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# Setup SSL (jika punya domain)
sudo certbot --nginx -d api.yourdomain.com
# Follow prompts, pilih redirect HTTP to HTTPS
```

---

## ✅ Verification

```bash
# Test dari server
curl http://localhost:3000/api/flights

# Test via Nginx
curl http://localhost/api/flights

# Test dari komputer lain (ganti YOUR_SERVER_IP)
curl http://YOUR_SERVER_IP/api/flights

# Dengan domain & SSL
curl https://api.yourdomain.com/api/flights
```

**Expected Response:**
```json
{"status":"success","data":[],"total":0}
```

---

## 📱 Update Flutter App

Setelah server berjalan, update client Flutter di `lib/services/http_service.dart`:

```dart
static const String _baseUrl = 'http://YOUR_SERVER_IP/api';
// Atau dengan domain:
// static const String _baseUrl = 'https://api.yourdomain.com/api';
```

---

## 🔧 Useful Commands

```bash
# Check service status
sudo systemctl status airtally-api

# View logs
sudo journalctl -u airtally-api -f

# Restart service
sudo systemctl restart airtally-api

# Update application
cd ~/airtally/airtally-restapi
git pull origin main
./deploy.sh --update

# Backup database
./backup-database.sh
```

---

## 🆘 Troubleshooting

### Service not starting?
```bash
# Check logs
sudo journalctl -u airtally-api -n 50

# Check database connection
psql -U airtally_user -d airtally -h localhost

# Restart PostgreSQL
sudo systemctl restart postgresql
```

### API not accessible from outside?
```bash
# Check firewall
sudo ufw status

# Check Nginx
sudo nginx -t
sudo systemctl status nginx

# Check if port 3000 is listening
sudo ss -tulpn | grep 3000
```

### 502 Bad Gateway?
```bash
# Service is probably down
sudo systemctl status airtally-api
sudo systemctl restart airtally-api
```

---

## 📚 Next Steps

1. **Setup Monitoring:** Use `monitor-health.sh` with cron
2. **Setup Backups:** Schedule `backup-database.sh` daily
3. **Enable Barcode Decoder:** See `SETUP_INSTRUCTIONS.md`
4. **Read Full Guide:** `UBUNTU_DEPLOYMENT.md` untuk detailed instructions

---

**Need Help?** Check `UBUNTU_DEPLOYMENT.md` for comprehensive guide!