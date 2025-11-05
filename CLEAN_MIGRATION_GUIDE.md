# 🔄 Clean Migration Guide: Full Fresh Install

## Overview

Panduan ini untuk **fresh install FALCON dari awal** dengan menghapus instalasi AirTally yang lama. Cocok untuk situasi:
- Data masih sedikit/baru mulai
- Ingin clean start dengan struktur baru
- Migrasi data tidak terlalu kompleks

---

## ⚠️ Important Information

### What This Migration Does:

✅ **Creates:**
- New user: `falcon`
- New directory: `/home/falcon/FalconRest`
- New database: `falcon` (or keeps `airtally` if you prefer)
- New service: `falcon-api`
- Fresh secrets: JWT_SECRET, API_KEY

✅ **Removes (after cleanup):**
- Old user: `airtally`
- Old directory: `/home/airtally/AirtallyRest`
- Old service: `airtally-api`
- Old database: `airtally` (optional)

### Prerequisites:

- Ubuntu/Debian Linux server
- Root/sudo access
- Internet connection
- PostgreSQL installed (or will be installed by script)
- Git access to repository

---

## 🚀 Migration Steps

### Step 1: Backup Old Data (If Needed)

If you have any important data in old AirTally:

```bash
# Login to server
ssh user@your-server

# Become airtally user
sudo su - airtally

# Backup database
pg_dump -U postgres airtally | gzip > ~/airtally_backup_$(date +%Y%m%d).sql.gz

# Backup any important files
tar -czf ~/airtally_files_backup.tar.gz ~/AirtallyRest/.env ~/AirtallyRest/uploads/ 2>/dev/null

# Copy backups to safe location
exit  # Back to your user
scp airtally@localhost:~/airtally_backup_*.gz /backup/location/
```

### Step 2: Download Fresh Install Script

```bash
# As your regular user with sudo access
cd /tmp

# Download script (or copy from repository)
wget https://raw.githubusercontent.com/Frezneel/FalconRest/main/fresh-install-falcon.sh

# Or if you have repo cloned:
# cp /path/to/repo/fresh-install-falcon.sh .

# Make executable
chmod +x fresh-install-falcon.sh
```

### Step 3: Run Fresh Install

```bash
# Run as root
sudo ./fresh-install-falcon.sh
```

The script will ask you for:

1. **Database Password** (minimum 12 characters)
   - Example: `FalconDB2025!SecurePass`
   - Confirm password

2. **API Port** (default: 3000)
   - Press Enter for default
   - Or enter custom port like 8080

3. **GitHub Repository URL**
   - Press Enter for default
   - Or enter your fork URL

4. **Branch Name** (default: main)
   - Press Enter for default
   - Or enter `Development` or other branch

**Script will then:**
- ✅ Install all dependencies (15-20 minutes)
- ✅ Create falcon user and directories
- ✅ Setup PostgreSQL database
- ✅ Clone repository
- ✅ Generate secure secrets (JWT_SECRET, API_KEY)
- ✅ Build FALCON application
- ✅ Create systemd service
- ✅ Configure firewall
- ✅ Setup automatic backups
- ✅ Start and verify service

### Step 4: Save Secrets

At the end of installation, script will display secrets file:

```bash
# Secrets will be shown on screen
# COPY THESE TO PASSWORD MANAGER:

API Key: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
JWT Secret: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
Database Password: FalconDB2025!SecurePass
```

**IMPORTANT:**
- Save to password manager immediately!
- File location: `/home/falcon/SECRETS.txt`
- Delete after saving: `sudo rm /home/falcon/SECRETS.txt`

### Step 5: Update Mobile App

Update `lib/config/api_config.dart`:

```dart
class ApiConfig {
  // Update with your server IP
  static const String baseUrl = 'http://10.17.6.155:3000/api';

  // Use API key from secrets file
  static const String apiKey = 'your_generated_api_key_here';

  // ... rest of config
}
```

Build and test mobile app:

```bash
flutter clean
flutter pub get
flutter build apk --release
```

### Step 6: Test FALCON

```bash
# Check service status
sudo systemctl status falcon-api

# View logs
sudo journalctl -u falcon-api -f

# Test API
curl http://localhost:3000/api/starter-data/version

# Should return:
# {"success":true,"message":"Starter data version retrieved",...}
```

**Test from mobile app:**
- Login with default credentials:
  - Username: `superuser`
  - Password: `Falcon2025!`
- Test barcode scanning
- Test flight list
- Test sync
- Test user management (admin only)

### Step 7: Change Default Password

**CRITICAL SECURITY STEP:**

1. Login to mobile app as `superuser`
2. Go to User Management
3. Click on superuser → Reset Password
4. Set new strong password (minimum 8 characters)
5. Logout and login with new password

### Step 8: Create Additional Users

As admin, create users for your team:

1. Go to User Management → Create User
2. Fill in details:
   - Username
   - Email
   - Full Name
   - Password (min 8 chars)
   - Role (Admin/Supervisor/Operator)
3. Test login with new users

### Step 9: Monitor for 24-48 Hours

```bash
# Watch logs for errors
sudo journalctl -u falcon-api -f

# Check service stays up
sudo systemctl status falcon-api

# Monitor memory usage
free -h

# Check database connections
sudo -u postgres psql -c "SELECT count(*) FROM pg_stat_activity WHERE datname='falcon';"
```

### Step 10: Cleanup Old Installation (Optional)

**Only after confirming FALCON works perfectly for 24-48 hours!**

```bash
# Download cleanup script
cd /tmp
wget https://raw.githubusercontent.com/Frezneel/FalconRest/main/cleanup-old-airtally.sh
chmod +x cleanup-old-airtally.sh

# Run cleanup
sudo ./cleanup-old-airtally.sh
```

Cleanup script will:
- ✅ Create final backup of old installation
- ✅ Stop and remove old service
- ✅ Remove old user and directories
- ✅ Clean up firewall rules
- ✅ Optionally remove old database
- ✅ Verify FALCON still works after cleanup

---

## 📊 Comparison: Before vs After

| Aspect | Before (AirTally) | After (FALCON) |
|--------|------------------|----------------|
| **User** | airtally | falcon |
| **Home Dir** | /home/airtally | /home/falcon |
| **App Dir** | ~/AirtallyRest | ~/FalconRest |
| **Service** | airtally-api | falcon-api |
| **Binary** | airtally-rest | falcon-rest |
| **Database** | airtally | falcon (or kept as airtally) |
| **JWT_SECRET** | ❌ Not set (WARNING!) | ✅ Secure random |
| **API Key** | Default/weak | ✅ Secure random |

---

## 🔐 Security Improvements

### Old Setup Issues:
- ❌ JWT_SECRET using default value
- ❌ Weak API key
- ⚠️ No firewall configuration
- ⚠️ No automatic backups

### New FALCON Setup:
- ✅ Strong JWT_SECRET (48 bytes random)
- ✅ Strong API Key (32 bytes hex)
- ✅ Firewall configured (UFW)
- ✅ Open API access (secured by API Key + JWT)
- ✅ Automatic daily backups
- ✅ Systemd service hardening
- ✅ Proper file permissions

---

## 🛠️ Useful Commands

### Service Management

```bash
# Status
sudo systemctl status falcon-api

# Start/Stop/Restart
sudo systemctl restart falcon-api

# View logs (live)
sudo journalctl -u falcon-api -f

# View last 100 lines
sudo journalctl -u falcon-api -n 100

# View logs since yesterday
sudo journalctl -u falcon-api --since yesterday
```

### Database Management

```bash
# Connect to database
sudo -u postgres psql falcon

# List tables
\dt

# Count users
SELECT COUNT(*) FROM users;

# Count flights
SELECT COUNT(*) FROM flights;

# Exit
\q
```

### Backup Management

```bash
# Manual backup
sudo -u falcon /home/falcon/backup-db.sh

# List backups
ls -lh /home/falcon/backups/database/

# Restore from backup
gunzip -c /home/falcon/backups/database/falcon_YYYYMMDD_HHMMSS.sql.gz | sudo -u postgres psql falcon
```

### Update FALCON

```bash
# Switch to falcon user
sudo su - falcon

# Navigate to app directory
cd ~/FalconRest/AirtallyRest

# Make update script executable
chmod +x update-api.sh

# Run update
./update-api.sh main
```

---

## 🚨 Troubleshooting

### Issue 1: Fresh Install Script Fails

**Check logs:**
```bash
# If script fails, check what went wrong
journalctl -xe

# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check if ports are available
sudo netstat -tlnp | grep 3000
```

**Common fixes:**
```bash
# If PostgreSQL not installed
sudo apt install postgresql postgresql-contrib

# If port 3000 is in use
# Change API_PORT in script or stop conflicting service
```

### Issue 2: Service Won't Start

```bash
# Check logs
sudo journalctl -u falcon-api -n 50

# Common issues:
# - Binary not found: Check /home/falcon/FalconRest/AirtallyRest/target/release/falcon-rest
# - Database connection: Check DATABASE_URL in .env
# - Permission denied: Check file ownership (should be falcon:falcon)
```

**Fix permissions:**
```bash
sudo chown -R falcon:falcon /home/falcon/FalconRest
sudo chmod +x /home/falcon/FalconRest/AirtallyRest/target/release/falcon-rest
```

### Issue 3: API Not Responding

```bash
# Check if service is running
sudo systemctl status falcon-api

# Check if listening on port
sudo netstat -tlnp | grep 3000

# Test locally
curl http://localhost:3000/api/starter-data/version

# Check firewall
sudo ufw status
```

### Issue 4: Database Connection Failed

```bash
# Check PostgreSQL is running
sudo systemctl status postgresql

# Check database exists
sudo -u postgres psql -l | grep falcon

# Check .env DATABASE_URL
cat /home/falcon/FalconRest/AirtallyRest/.env | grep DATABASE_URL

# Test connection manually
psql "postgresql://falcon_user:PASSWORD@localhost:5432/falcon"
```

### Issue 5: JWT_SECRET Warning Still Appears

```bash
# Check if JWT_SECRET is in .env
cat /home/falcon/FalconRest/AirtallyRest/.env | grep JWT_SECRET

# If missing or empty, regenerate
JWT_SECRET=$(openssl rand -base64 48)
echo "JWT_SECRET=$JWT_SECRET" >> /home/falcon/FalconRest/AirtallyRest/.env

# Restart service
sudo systemctl restart falcon-api
```

---

## 📱 Mobile App Integration

### Update API Configuration

**File:** `lib/config/api_config.dart`

```dart
class ApiConfig {
  // Your server IP address (check with: hostname -I)
  static const String baseUrl = 'http://YOUR_SERVER_IP:3000/api';

  // API Key from /home/falcon/SECRETS.txt
  static const String apiKey = 'YOUR_GENERATED_API_KEY';

  // Timeouts
  static const Duration connectionTimeout = Duration(seconds: 30);
  static const Duration receiveTimeout = Duration(seconds: 30);
}
```

### Find Your Server IP

```bash
# On server, run:
hostname -I

# Or:
ip addr show | grep "inet " | grep -v 127.0.0.1
```

### Test API from Mobile Network

```bash
# From your phone's network, test if API is reachable
curl http://YOUR_SERVER_IP:3000/api/starter-data/version

# If this fails:
# 1. Check firewall allows your phone's IP
# 2. Check server is accessible from outside (not just localhost)
# 3. Check HOST=0.0.0.0 in .env (not 127.0.0.1)
```

---

## ✅ Post-Installation Checklist

After installation is complete, verify:

### Security
- [ ] JWT_SECRET is set and unique (no warning in logs)
- [ ] API_KEY is generated and saved
- [ ] Default superuser password changed
- [ ] SECRETS.txt file deleted after saving
- [ ] Firewall configured and active
- [ ] API accessible from network (test with curl)

### Functionality
- [ ] Service starts automatically on boot
- [ ] API responds to health check
- [ ] Mobile app can login
- [ ] Barcode scanning works
- [ ] Flight list displays
- [ ] Data sync works (online/offline)
- [ ] User management works (admin)

### Monitoring
- [ ] Logs are accessible and readable
- [ ] No errors in service logs
- [ ] Database connections are stable
- [ ] Automatic backups are scheduled (crontab)

### Documentation
- [ ] Team knows new API URL
- [ ] Team has access to new API key
- [ ] Emergency contacts documented
- [ ] Rollback procedure understood

---

## 🔄 Rollback Procedure

If you need to revert to old AirTally (before running cleanup script):

```bash
# 1. Stop FALCON
sudo systemctl stop falcon-api
sudo systemctl disable falcon-api

# 2. Start old AirTally (if not cleaned up yet)
sudo systemctl start airtally-api

# 3. Update mobile app back to old API URL and key

# 4. Remove FALCON (optional)
sudo systemctl stop falcon-api
sudo rm /etc/systemd/system/falcon-api.service
sudo systemctl daemon-reload
sudo userdel -r falcon
```

---

## 📞 Support

### Log Collection for Support

If you need help, collect these logs:

```bash
# Service logs
sudo journalctl -u falcon-api -n 200 > falcon-logs.txt

# System info
uname -a >> system-info.txt
free -h >> system-info.txt
df -h >> system-info.txt

# Database status
sudo -u postgres psql -c "\l" > db-status.txt
sudo -u postgres psql falcon -c "\dt" >> db-status.txt

# Compress and send
tar -czf falcon-support-$(date +%Y%m%d).tar.gz falcon-logs.txt system-info.txt db-status.txt
```

---

## 📚 Additional Resources

- **[README.md](README.md)** - Complete API documentation
- **[DEPLOYMENT_CHECKLIST.md](DEPLOYMENT_CHECKLIST.md)** - Detailed deployment guide
- **[MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)** - In-place migration option
- **[update-api.sh](update-api.sh)** - Update script for future updates

---

**Version**: 2.0.0
**Last Updated**: 2025-11-04
**Author**: InJourneyDHX Team
