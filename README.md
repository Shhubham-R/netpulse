# NetPulse — Real-Time Network Traffic Visualizer

<p align="center">
  <strong>🌐 Monitor every byte leaving your machine. See who's talking, where, and how much.</strong>
</p>

NetPulse is a cross-platform desktop application that captures network packets in real-time, maps them to applications, resolves geographic locations, and presents everything in a beautiful, interactive dashboard. Built with **Tauri v2** (Rust + React/TypeScript).

---

## ✨ Features

| Feature | Description |
|---------|-------------|
| **Live Traffic Dashboard** | Real-time table of all active connections with app name, PID, protocol, remote host, country, and bandwidth |
| **Per-App Bandwidth Graph** | Stacked area chart showing upload/download per application over the last 60 seconds |
| **App Summary Panel** | Aggregated view showing each app's total traffic, connected hosts, and countries |
| **Connection History** | SQLite-backed searchable log of all past connections, filterable by app, country, date range |
| **System Tray** | Live upload/download speed display, right-click menu with quick actions |
| **Suspicious Activity Alerts** | 7 built-in detection rules: odd hours, unknown processes, foreign countries, Tor nodes, etc. |
| **Firewall-lite Block List** | Right-click to block apps or IPs; blocked attempts are tracked and flagged |
| **GeoIP World Map** | SVG world map with animated dots showing where your connections go |
| **DNS Leak Detection** | Captures all DNS queries per app to detect DNS leaks and unexpected resolvers |
| **App Trust System** | Mark apps as trusted (collapsed in UI) or untrusted (highlighted) |
| **Export Reports** | One-click export to JSON or CSV |
| **Dark/Light Theme** | Persisted preference toggle |
| **Settings Panel** | Interface selection, alert thresholds, trusted countries, data retention |

---

## 📋 Prerequisites

- **Rust** 1.75+ — [Install via rustup](https://rustup.rs/)
- **Node.js** 18+ — [Download from nodejs.org](https://nodejs.org/)
- **libpcap** (Linux) or **Npcap** (Windows) — required for packet capture
- **GeoLite2-City.mmdb** — optional, for GeoIP lookups

### Linux Dependencies

```bash
# Ubuntu/Debian
sudo apt-get install -y libpcap-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  libappindicator3-dev librsvg2-dev pkg-config

# Fedora
sudo dnf install -y libpcap-devel gtk3-devel webkit2gtk4.1-devel \
  libappindicator-gtk3-devel librsvg2-devel

# Arch
sudo pacman -S libpcap gtk3 webkit2gtk-4.1 libappindicator-gtk3 librsvg
```

### Windows Dependencies

1. Download and install [Npcap](https://npcap.com/#download)
2. During installation, check **"Install in WinPcap API-compatible Mode"**

---

## 🗺️ GeoLite2 Database Setup

NetPulse uses MaxMind's GeoLite2 database for IP geolocation. Due to licensing, you must download it yourself:

1. Create a free account at [maxmind.com](https://www.maxmind.com/en/geolite2/signup)
2. Download `GeoLite2-City.mmdb` from your account dashboard
3. Place the file at:
   - **Linux**: `~/.local/share/com.netpulse.app/GeoLite2-City.mmdb`
   - **Windows**: `%APPDATA%\com.netpulse.app\GeoLite2-City.mmdb`

> **Note**: The app works without this file — GeoIP data will simply show as "Unknown".

---

## 🚀 Installation

### Quick Setup (Linux)

```bash
git clone https://github.com/your-username/netpulse.git
cd netpulse
chmod +x scripts/setup-linux.sh
./scripts/setup-linux.sh
```

### Quick Setup (Windows)

```powershell
git clone https://github.com/your-username/netpulse.git
cd netpulse
# Run PowerShell as Administrator
.\scripts\setup-windows.ps1
```

### Manual Installation

```bash
# Install npm dependencies
npm install

# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

---

## 🔐 Privilege Setup

### Linux — Network Capture Permissions

Packet capture requires `CAP_NET_RAW`. After building:

```bash
sudo setcap cap_net_raw+ep ./src-tauri/target/release/netpulse
```

Or run with `sudo` (not recommended for daily use).

### Windows — Administrator

Run NetPulse as Administrator for packet capture to work. The app will detect missing permissions and offer to restart with elevation.

---

## 💻 Development

```bash
# Start dev server (frontend + backend hot-reload)
npm run tauri dev

# Build frontend only
npm run build

# Check Rust code
cargo check --manifest-path src-tauri/Cargo.toml

# Production build
npm run tauri build
```

---

## 🏗️ Architecture Overview

| Module | Description |
|--------|-------------|
| `capture/` | Packet capture engine using `pcap` crate with a dedicated thread, BPF filtering, and 500ms batching |
| `capture/packet_parser.rs` | Parses raw Ethernet/IP/TCP/UDP packets using `pnet` |
| `capture/process_mapper.rs` | Maps network sockets to PIDs via `/proc/net/tcp` (Linux) |
| `geo/` | GeoIP lookups via `maxminddb` with DashMap caching |
| `geo/dns_resolver.rs` | Captures and parses DNS packets, resolves hostnames |
| `db/` | SQLite database with WAL mode, batched writes, and data cleanup |
| `alerts/` | Rule-based alert engine evaluating 7 suspicious activity patterns |
| `tray/` | System tray icon with live speed tooltip and quick-action menu |
| `commands/` | 16 Tauri IPC command handlers bridging backend ↔ frontend |

The frontend uses React 18 with TypeScript, Zustand for state management, Recharts for graphs, TanStack Virtual for high-performance table rendering, and Tailwind CSS for styling.

---

## 🔧 Troubleshooting

| Problem | Solution |
|---------|----------|
| "Permission denied" on capture | Run `sudo setcap cap_net_raw+ep` on the binary, or run as root |
| "No interfaces found" | Ensure libpcap/Npcap is installed correctly |
| GeoIP shows "Unknown" | Download and place `GeoLite2-City.mmdb` in the app data directory |
| App starts but no traffic | Check that the correct network interface is selected in Settings |
| High CPU usage | Reduce the number of monitored interfaces or increase batch interval |
| Database corruption | Delete `~/.local/share/com.netpulse.app/netpulse.db` (data will be lost) |

---

## 📄 License

MIT License — see [LICENSE](LICENSE) for details.

---

<p align="center">
  Built with 🦀 Rust + ⚛️ React + 💙 TypeScript
</p>
