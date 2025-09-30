# 🎯 START HERE - AirTally REST API Deployment

**Selamat datang!** Ini adalah starting point untuk deploy AirTally REST API.

---

## 🚀 Saya Mau Deploy ke Ubuntu!

### Pertama Kali Deploy? Mulai Di Sini:

1. **Baca:** [`QUICK_START.md`](./QUICK_START.md)
   - ⏱️ Waktu: ~10-15 menit
   - 📝 6 langkah mudah dari nol sampai production
   - ✅ Best untuk pemula

2. **Jalankan:** `deploy.sh`
   ```bash
   chmod +x deploy.sh
   ./deploy.sh
   ```

3. **Done!** API kamu sudah running! 🎉

### Sudah Deploy, Mau Update?

```bash
cd ~/airtally/airtally-restapi
git pull origin main
./deploy.sh --update
```

---

## 📚 Documentation Overview

| File | Untuk Siapa | Waktu Baca | Kapan Digunakan |
|------|------------|-----------|-----------------|
| **QUICK_START.md** | 🟢 Semua orang | 5 menit | First time deployment |
| **UBUNTU_DEPLOYMENT.md** | 🟡 Advanced users | 20 menit | Detailed setup & troubleshooting |
| **DEPLOYMENT_README.md** | 🟡 DevOps | 15 menit | Operations & maintenance |
| **SETUP_INSTRUCTIONS.md** | 🔵 Developers | 10 menit | Local development |
| **AirTally_REST_API_Documentation.md** | 🔵 Developers | 30 menit | API reference |

### 🎨 Legend:
- 🟢 **Beginner-friendly** - Cocok untuk yang baru mulai
- 🟡 **Intermediate** - Butuh basic server knowledge
- 🔵 **Advanced** - Untuk developers/DevOps

---

## 🛠️ Scripts & Tools

### Automated Scripts

| Script | Fungsi | Usage |
|--------|--------|-------|
| `deploy.sh` | Deploy/update aplikasi | `./deploy.sh [--update] [--backup]` |
| `backup-database.sh` | Backup database | `./backup-database.sh` |
| `monitor-health.sh` | Monitor API health | `./monitor-health.sh [--restart-on-fail]` |

### Configuration Files

| File | Fungsi | Copy To |
|------|--------|---------|
| `airtally-api.service` | Systemd service | `/etc/systemd/system/` |
| `nginx.conf.example` | Nginx config | `/etc/nginx/sites-available/` |
| `.env.example` | Environment vars | `.env` (edit first!) |

---

## 🎯 Quick Navigation

### "Saya mau..."

**Deploy untuk pertama kali:**
→ Baca: [`QUICK_START.md`](./QUICK_START.md)

**Update aplikasi yang sudah jalan:**
```bash
./deploy.sh --update
```

**Setup SSL/HTTPS:**
→ Lihat: [`UBUNTU_DEPLOYMENT.md`](./UBUNTU_DEPLOYMENT.md) - Section 8

**Backup database:**
```bash
./backup-database.sh
```

**Troubleshooting API tidak jalan:**
→ Lihat: [`UBUNTU_DEPLOYMENT.md`](./UBUNTU_DEPLOYMENT.md) - Section 12

**Monitoring API:**
```bash
./monitor-health.sh --restart-on-fail
```

**Lihat API documentation:**
→ Baca: [`AirTally_REST_API_Documentation.md`](./AirTally_REST_API_Documentation.md)

**Setup development lokal:**
→ Baca: [`SETUP_INSTRUCTIONS.md`](./SETUP_INSTRUCTIONS.md)

---

## ⚡ Super Quick Deploy (For Impatient People)

Punya Ubuntu server dan mau cepat? Copy-paste ini:

```bash
# 1. Install dependencies
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh && source "$HOME/.cargo/env"
sudo apt update && sudo apt install -y postgresql postgresql-contrib nginx git build-essential pkg-config libssl-dev

# 2. Setup database
sudo -u postgres psql -c "CREATE USER airtally_user WITH PASSWORD 'change_this_password';"
sudo -u postgres psql -c "CREATE DATABASE airtally OWNER airtally_user;"

# 3. Clone & setup
cd ~ && git clone YOUR_REPO_URL airtally
cd airtally/airtally-restapi
cp .env.example .env
nano .env  # Update DATABASE_URL password

# 4. Deploy!
chmod +x deploy.sh && ./deploy.sh

# 5. Setup Nginx
sudo cp nginx.conf.example /etc/nginx/sites-available/airtally-api
sudo ln -s /etc/nginx/sites-available/airtally-api /etc/nginx/sites-enabled/
sudo rm /etc/nginx/sites-enabled/default
sudo nginx -t && sudo systemctl reload nginx

# 6. Firewall
sudo ufw allow 22/tcp && sudo ufw allow 80/tcp && sudo ufw allow 443/tcp && sudo ufw enable

# 7. Test!
curl http://localhost/api/flights
```

**⚠️ Warning:** Ini untuk testing only! Production harus ikuti full guide di UBUNTU_DEPLOYMENT.md

---

## 🏗️ Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                    Internet                          │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│  Nginx (Reverse Proxy + SSL/TLS)                    │
│  - Port 80/443                                       │
│  - HTTPS termination                                 │
│  - Rate limiting                                     │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│  AirTally REST API (Rust/Axum)                      │
│  - Port 3000 (internal)                              │
│  - Managed by systemd                                │
│  - Auto-restart on failure                           │
└─────────────────────┬───────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────────┐
│  PostgreSQL Database                                 │
│  - Port 5432 (localhost only)                        │
│  - Daily automated backups                           │
└─────────────────────────────────────────────────────┘
```

---

## 📊 Deployment Flowchart

```
START
  │
  ├─→ [Install Dependencies] → Rust, PostgreSQL, Nginx
  │
  ├─→ [Setup Database] → Create user & database
  │
  ├─→ [Clone Repository] → Get source code
  │
  ├─→ [Configure .env] → Set database credentials
  │
  ├─→ [Build Application] → cargo build --release
  │
  ├─→ [Setup Systemd] → Auto-start service
  │
  ├─→ [Configure Nginx] → Reverse proxy
  │
  ├─→ [Setup SSL] → Let's Encrypt (optional)
  │
  ├─→ [Configure Firewall] → UFW rules
  │
  ├─→ [Setup Monitoring] → Health checks & backups
  │
  └─→ [DONE] → API is live! 🎉
```

---

## 🔧 System Requirements

### Minimum
- **OS:** Ubuntu 20.04/22.04/24.04 LTS
- **RAM:** 1GB
- **Storage:** 10GB
- **CPU:** 1 core
- **Network:** Public IP

### Recommended (Production)
- **RAM:** 2GB+
- **Storage:** 20GB+ SSD
- **CPU:** 2+ cores
- **Network:** Static IP + Domain name

---

## ✅ Pre-Flight Checklist

Before deployment, make sure you have:

- [ ] Ubuntu server with SSH access
- [ ] Root or sudo privileges
- [ ] Basic command line knowledge
- [ ] Domain name (optional but recommended)
- [ ] 15-30 minutes of your time

---

## 🆘 Need Help?

### Quick Fixes

**Service not starting?**
```bash
sudo journalctl -u airtally-api -n 50
```

**API not accessible?**
```bash
sudo systemctl status airtally-api
sudo ss -tulpn | grep 3000
```

**502 Bad Gateway?**
```bash
sudo systemctl restart airtally-api
sudo systemctl restart nginx
```

### Detailed Help

1. **Check Troubleshooting:** [`UBUNTU_DEPLOYMENT.md`](./UBUNTU_DEPLOYMENT.md) - Section 12
2. **Read FAQ:** [`DEPLOYMENT_README.md`](./DEPLOYMENT_README.md) - Troubleshooting section
3. **Check Logs:**
   ```bash
   sudo journalctl -u airtally-api -f
   sudo tail -f /var/log/nginx/error.log
   ```

---

## 📞 Support

- 📖 **Documentation:** Read the docs above
- 🐛 **Bug Reports:** Open GitHub issue
- 💬 **Questions:** GitHub Discussions
- 📧 **Email:** your-support-email@example.com

---

## 🎓 Learning Path

### 1. Beginner (0-2 hours)
1. Read QUICK_START.md
2. Deploy to server
3. Test basic API calls

### 2. Intermediate (2-5 hours)
1. Read UBUNTU_DEPLOYMENT.md
2. Setup SSL/HTTPS
3. Configure monitoring
4. Setup automated backups

### 3. Advanced (5-10 hours)
1. Read DEPLOYMENT_README.md
2. Performance tuning
3. Security hardening
4. High availability setup

---

## 🎯 Success Criteria

Your deployment is successful when:

✅ Service is running: `sudo systemctl status airtally-api`
✅ API responds: `curl http://localhost:3000/api/flights`
✅ Accessible via Nginx: `curl http://YOUR_IP/api/flights`
✅ HTTPS working (if domain): `curl https://api.yourdomain.com/api/flights`
✅ Auto-starts on reboot (test it!)
✅ Backups scheduled
✅ Monitoring setup

---

## 🚦 What's Next?

After successful deployment:

1. **Test API endpoints** - See API documentation
2. **Update Flutter app** - Change API URL in code
3. **Setup monitoring** - Use monitor-health.sh
4. **Schedule backups** - Use backup-database.sh
5. **Monitor logs** - Check for errors
6. **Read security guide** - Harden your server

---

## 📝 Deployment Commands Reference

```bash
# Service management
sudo systemctl start|stop|restart|status airtally-api

# View logs
sudo journalctl -u airtally-api -f

# Update application
cd ~/airtally/airtally-restapi && ./deploy.sh --update

# Backup database
./backup-database.sh

# Monitor health
./monitor-health.sh --restart-on-fail

# Nginx
sudo systemctl reload nginx
sudo nginx -t

# Test API
curl http://localhost:3000/api/flights
```

---

**Ready to deploy?** → Start with [`QUICK_START.md`](./QUICK_START.md)

**Questions?** → Check [`UBUNTU_DEPLOYMENT.md`](./UBUNTU_DEPLOYMENT.md)

**Good luck! 🚀**