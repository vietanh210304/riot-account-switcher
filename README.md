# ⚡ Riot Account Switcher (LoL & VALORANT)

<p align="center">
  <img src="web/assets/icon_256.png" width="128" height="128" alt="Riot Account Switcher Logo">
  <br>
  <strong>High-speed, privacy-first Riot Games account switcher for Windows</strong>
  <br>
  <em>Designed for League of Legends & VALORANT players</em>
</p>

<p align="center">
  <strong>English</strong> | <a href="README_vi.md">Tiếng Việt</a>
</p>

<p align="center">
  <a href="https://github.com/vietanh210304/riot-account-switcher/releases/latest">
    <img src="https://img.shields.io/github/v/release/vietanh210304/riot-account-switcher?style=flat-square&color=c8aa6e" alt="Latest Release">
  </a>
  <img src="https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011-0ac8b9?style=flat-square" alt="Platform">
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT-ff4655?style=flat-square" alt="License">
  </a>
</p>

---

## 🌟 Key Features

- ⚡ **One-Click Instant Account Swapping:** Effortlessly switch between Riot Games accounts in seconds without re-entering credentials or 2FA codes.
- 🎯 **Reliable Game Launching via Local Lockfile REST API:** Directly trigger game launches via Riot Client's internal HTTPS REST API (`POST /product-launcher/v1/products/.../patchlines/live`), bypassing background process locks.
- 🌐 **Bilingual Support (English & Tiếng Việt):** Instant on-the-fly language toggle between English and Vietnamese across all app screens and dialogs.
- 🖱️ **Right-Click Context Menu & Inline Editing:**
  - Edit display Riot ID & tagline (`#tag`).
  - Add account notes.
  - Choose from **24 official offline avatars** (12 LoL champions + 12 VALORANT agents).
  - Quick-copy Riot ID to clipboard.
  - Safe account removal with styled in-app confirmation dialog.
- 🛡️ **100% Privacy & Local Storage:** Session tokens and snapshot data remain strictly on your local PC. No telemetry, no remote servers, no data collection.

---

## 📥 Download Latest Release

👉 Grab the pre-built packages from **[Releases](https://github.com/vietanh210304/riot-account-switcher/releases/latest)**:

- 📦 **Installer (`RiotAccountSwitcher-Setup.exe`):** Standard Windows installer that sets up Start Menu shortcuts, Desktop icon, and an uninstaller in Windows Settings.
- 🚀 **Portable (`RiotAccountSwitcher-Portable.exe`):** Single-file standalone executable with zero installation required. Run directly from anywhere!

---

## 🛠️ Development & Building from Source

### Prerequisites:
- Windows 10 / 11 (64-bit)
- Python 3.10+
- Microsoft Edge WebView2 Runtime (pre-installed on Windows 10/11)

### Setup & Installation:
```bash
git clone https://github.com/vietanh210304/riot-account-switcher.git
cd riot-account-switcher
pip install pywebview pyinstaller psutil pillow
```

### Run in Development Mode:
```bash
python app.py
```

### Build Single-File `.exe` with PyInstaller:
```bash
pyinstaller --noconsole --onefile --icon "assets/icon.ico" --name "RiotAccountSwitcher" --add-data "web;web" --add-data "assets;assets" --add-data "backend.py;." app.py -y
```
The compiled binary will be generated at `dist/RiotAccountSwitcher.exe`.

---

## ⚖️ License & Disclaimer

Distributed under the [MIT License](LICENSE).

*Disclaimer:* Riot Account Switcher is an unofficial utility and is not affiliated with, endorsed by, or sponsored by Riot Games, Inc. Riot Games, League of Legends, and VALORANT are trademarks or registered trademarks of Riot Games, Inc.
