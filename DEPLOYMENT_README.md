# 🚀 AirTally REST API - Deployment Files

Complete deployment toolkit untuk deploy AirTally REST API ke Ubuntu Server.

---

## 📁 Deployment Files

### 📖 Documentation

| File | Description |
|------|-------------|
| `QUICK_START.md` | 10-minute quick deployment guide (recommended untuk first-time deploy) |
| `UBUNTU_DEPLOYMENT.md` | Comprehensive deployment guide dengan semua detail |
| `SETUP_INSTRUCTIONS.md` | Local development setup guide |
| `AirTally_REST_API_Documentation.md` | Complete API documentation |

### 🔧 Scripts

| File | Description | Usage |
|------|-------------|-------|
| `deploy.sh` | Automated deployment script | `./deploy.sh [--update] [--backup]` |
| `backup-database.sh` | Database backup script | `./backup-database.sh [--retention DAYS]` |
| `monitor-health.sh` | API health monitor | `./monitor-health.sh [--restart-on-fail]` |

### ⚙️ Configuration Files

| File | Description | Target Location |
|------|-------------|-----------------|
| `airtally-api.service` | Systemd service unit | `/etc/systemd/system/airtally-api.service` |
| `nginx.conf.example` | Nginx reverse proxy config | `/etc/nginx/sites-available/airtally-api` |
| `.env.example` | Environment variables template | `.env` (copy & edit) |

---

## 🎯 Quick Deployment

### For Beginners (Recommended)

1. **Read Quick Start:**
   ```bash
   cat QUICK_START.md
   ```

2. **Follow the 6 steps** - takes ~10-15 minutes total

### For Experienced Users

1. **Run automated deployment:**
   ```bash
   chmod +x deploy.sh
   ./deploy.sh
   ```

2. **Setup Nginx & SSL** (see UBUNTU_DEPLOYMENT.md sections 7-8)

---

## 📋 Deployment Checklist

### Pre-Deployment
- [ ] Ubuntu server accessible via SSH
- [ ] Domain name configured (optional)
- [ ] Basic server knowledge (SSH, command line)

### Initial Deployment
- [ ] Rust installed
- [ ] PostgreSQL installed and configured
- [ ] Database created with correct credentials
- [ ] `.env` file configured with production values
- [ ] Application built in release mode
- [ ] Systemd service created and enabled
- [ ] Service running successfully

### Network & Security
- [ ] Nginx installed and configured
- [ ] SSL certificate obtained (if using domain)
- [ ] Firewall configured (UFW)
- [ ] API accessible from public internet

### Operations
- [ ] Database backup scheduled (cron)
- [ ] Health monitoring setup
- [ ] Log rotation configured
- [ ] Service auto-start verified (test reboot)

---

## 🛠️ Scripts Usage

### 1. Deployment Script

**First deployment:**
```bash
./deploy.sh
```

**Update existing installation:**
```bash
./deploy.sh --update
```

**Update with database backup:**
```bash
./deploy.sh --update --backup
```

### 2. Database Backup

**Manual backup:**
```bash
./backup-database.sh
```

**With custom retention:**
```bash
./backup-database.sh --retention 60  # Keep 60 days
```

**Schedule daily backup (2 AM):**
```bash
crontab -e
# Add line:
0 2 * * * /home/airtally/airtally/airtally-restapi/backup-database.sh >> /home/airtally/backup.log 2>&1
```

### 3. Health Monitoring

**Check health once:**
```bash
./monitor-health.sh
```

**Check and auto-restart if down:**
```bash
./monitor-health.sh --restart-on-fail
```

**Schedule monitoring (every 5 minutes):**
```bash
crontab -e
# Add line:
*/5 * * * * /home/airtally/airtally/airtally-restapi/monitor-health.sh --restart-on-fail >> /home/airtally/monitor.log 2>&1
```

---

## 🔄 Update Workflow

### Minor Updates (Code Changes Only)

```bash
cd ~/airtally/airtally-restapi
git pull origin main
./deploy.sh --update
```

### Major Updates (With Database Changes)

```bash
cd ~/airtally/airtally-restapi

# Backup first
./backup-database.sh

# Pull changes
git pull origin main

# Deploy
./deploy.sh --update

# Verify
curl http://localhost:3000/api/flights
```

### Rollback (If Needed)

```bash
# Stop service
sudo systemctl stop airtally-api

# Restore database
gunzip ~/database_backups/airtally_backup_YYYYMMDD_HHMMSS.sql.gz
PGPASSWORD='your_password' psql -U airtally_user -h localhost -d airtally < airtally_backup_YYYYMMDD_HHMMSS.sql

# Checkout previous version
cd ~/airtally/airtally-restapi
git checkout <previous_commit_hash>

# Rebuild
cargo build --release

# Start service
sudo systemctl start airtally-api
```

---

## 🔍 Monitoring & Maintenance

### View Logs

```bash
# Real-time application logs
sudo journalctl -u airtally-api -f

# Recent errors only
sudo journalctl -u airtally-api -p err -n 50

# Nginx access logs
sudo tail -f /var/log/nginx/access.log

# Nginx error logs
sudo tail -f /var/log/nginx/error.log
```

### Check Service Status

```bash
# Service status
sudo systemctl status airtally-api

# Is service running?
sudo systemctl is-active airtally-api

# Is service enabled on boot?
sudo systemctl is-enabled airtally-api
```

### Resource Monitoring

```bash
# CPU and memory usage
htop

# Disk usage
df -h

# Database size
sudo -u postgres psql -c "SELECT pg_size_pretty(pg_database_size('airtally'));"

# Application process
ps aux | grep airtally-restapi
```

---

## 🆘 Troubleshooting

### Service Won't Start

```bash
# Check logs
sudo journalctl -u airtally-api -n 50

# Verify binary exists
ls -la ~/airtally/airtally-restapi/target/release/airtally-restapi

# Check .env file
cat ~/airtally/airtally-restapi/.env

# Test database connection
psql -U airtally_user -d airtally -h localhost
```

### API Not Accessible

```bash
# Check if service is running
sudo systemctl status airtally-api

# Check if port is listening
sudo ss -tulpn | grep 3000

# Test locally
curl http://localhost:3000/api/flights

# Check Nginx
sudo nginx -t
sudo systemctl status nginx

# Check firewall
sudo ufw status
```

### High Memory/CPU Usage

```bash
# Check resources
htop

# Restart service
sudo systemctl restart airtally-api

# Check for memory leaks in logs
sudo journalctl -u airtally-api | grep -i "memory\|panic"
```

### Database Issues

```bash
# Check PostgreSQL status
sudo systemctl status postgresql

# Check database connections
sudo -u postgres psql -c "SELECT * FROM pg_stat_activity WHERE datname = 'airtally';"

# Restart PostgreSQL
sudo systemctl restart postgresql
```

---

## 📊 Performance Tuning

### PostgreSQL Optimization

Edit `/etc/postgresql/15/main/postgresql.conf`:

```conf
# Increase max connections
max_connections = 200

# Increase shared buffers (25% of RAM)
shared_buffers = 512MB

# Increase work memory
work_mem = 8MB

# Enable query planner statistics
shared_preload_libraries = 'pg_stat_statements'
```

Restart PostgreSQL:
```bash
sudo systemctl restart postgresql
```

### Nginx Optimization

For high traffic, edit `/etc/nginx/nginx.conf`:

```nginx
worker_processes auto;
worker_connections 1024;

# Enable gzip compression
gzip on;
gzip_types application/json;
gzip_min_length 1000;
```

Reload Nginx:
```bash
sudo systemctl reload nginx
```

---

## 🔐 Security Best Practices

1. **Change default passwords** in `.env` file
2. **Use SSL/TLS** (Let's Encrypt) for production
3. **Enable UFW firewall** with only necessary ports
4. **Regular updates:**
   ```bash
   sudo apt update && sudo apt upgrade -y
   ```
5. **Setup fail2ban** to prevent brute-force attacks:
   ```bash
   sudo apt install fail2ban
   sudo systemctl enable fail2ban
   ```
6. **Limit SSH access** to specific IPs if possible
7. **Regular database backups** (automated via cron)
8. **Monitor logs** for suspicious activity

---

## 📞 Support & Resources

### Documentation
- [QUICK_START.md](./QUICK_START.md) - Quick deployment guide
- [UBUNTU_DEPLOYMENT.md](./UBUNTU_DEPLOYMENT.md) - Comprehensive guide
- [AirTally_REST_API_Documentation.md](./AirTally_REST_API_Documentation.md) - API docs

### External Resources
- [Rust Documentation](https://www.rust-lang.org/learn)
- [Axum Framework](https://docs.rs/axum/latest/axum/)
- [PostgreSQL Docs](https://www.postgresql.org/docs/)
- [Nginx Documentation](https://nginx.org/en/docs/)

### Community
- GitHub Issues: [Report issues](https://github.com/yourusername/airtally/issues)
- Discussions: [Ask questions](https://github.com/yourusername/airtally/discussions)

---

## 📝 Version History

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2025-09-30 | Initial deployment toolkit |

---

## 📄 License

[Your License Here]

---

**Happy Deploying! 🚀**

For questions or issues, please refer to the troubleshooting section or open an issue on GitHub.