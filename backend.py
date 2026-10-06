import os
import sys
import json
import re
import time
import shutil
import base64
import subprocess
import urllib.parse
import urllib.request
import urllib.error
import ssl
from http.server import HTTPServer, SimpleHTTPRequestHandler
from pathlib import Path

# Paths
LOCALAPPDATA = os.environ.get("LOCALAPPDATA", os.path.expanduser("~\\AppData\\Local"))
APPDATA = os.environ.get("APPDATA", os.path.expanduser("~\\AppData\\Roaming"))
PROGRAMDATA = os.environ.get("ProgramData", "C:\\ProgramData")

if getattr(sys, 'frozen', False):
    BUNDLE_DIR = Path(sys._MEIPASS)
    APP_DIR = Path(sys.executable).parent
else:
    BUNDLE_DIR = Path(__file__).resolve().parent
    APP_DIR = Path(__file__).resolve().parent

DATA_DIR = APP_DIR / "data"
SNAPSHOTS_DIR = DATA_DIR / "snapshots"
BACKUPS_DIR = DATA_DIR / "backups"
ACCOUNTS_FILE = DATA_DIR / "accounts.json"
WEB_DIR = BUNDLE_DIR / "web"
if not WEB_DIR.exists():
    WEB_DIR = APP_DIR / "web"

RIOT_CLIENT_DATA = Path(LOCALAPPDATA) / "Riot Games" / "Riot Client" / "Data"
RIOT_CLIENT_CONFIG = Path(LOCALAPPDATA) / "Riot Games" / "Riot Client" / "Config"
RIOT_PRIVATE_SETTINGS = RIOT_CLIENT_DATA / "RiotGamesPrivateSettings.yaml"
TCNO_RIOT_CACHE = Path(APPDATA) / "TcNo Account Switcher" / "LoginCache" / "Riot Games"

import hashlib

def safe_id(name, tag):
    clean = re.sub(r'[\/\\:\*\?\"<>\|\s]+', '_', f"{name}_{tag}").strip('_')
    if not clean or len(clean.replace('_', '')) == 0:
        clean = "acc_" + hashlib.md5(f"{name}_{tag}".encode()).hexdigest()[:8]
    return clean.lower()
SNAPSHOTS_DIR.mkdir(parents=True, exist_ok=True)
BACKUPS_DIR.mkdir(parents=True, exist_ok=True)
if not ACCOUNTS_FILE.exists():
    ACCOUNTS_FILE.write_text("[]", encoding="utf-8")

RIOT_PROCS = [
    "league of legends.exe",
    "valorant-win64-shipping.exe",
    "leagueclientuxrender.exe",
    "leagueclientux.exe",
    "leagueclient.exe",
    "valorant.exe",
    "riot client.exe",
    "riotclientuxrender.exe",
    "riotclientux.exe",
    "riotclientservices.exe",
    "riotclientcrashhandler.exe",
]

MATCH_PROCS = [
    "league of legends.exe",
    "valorant-win64-shipping.exe"
]

def find_riot_client_services():
    installs_json = Path(PROGRAMDATA) / "Riot Games" / "RiotClientInstalls.json"
    if installs_json.exists():
        try:
            with open(installs_json, "r", encoding="utf-8") as f:
                data = json.load(f)
            for k in ("rc_default", "rc_live"):
                p = data.get(k)
                if p and Path(p).exists():
                    return str(Path(p).resolve())
        except Exception:
            pass

    for candidate in [
        r"G:\Riot Games\Riot Client\RiotClientServices.exe",
        r"C:\Riot Games\Riot Client\RiotClientServices.exe",
        r"D:\Riot Games\Riot Client\RiotClientServices.exe",
        r"E:\Riot Games\Riot Client\RiotClientServices.exe",
    ]:
        if Path(candidate).exists():
            return candidate
    return None

def find_installed_games():
    games = {"lol": False, "valorant": False}
    installs_json = Path(PROGRAMDATA) / "Riot Games" / "RiotClientInstalls.json"
    if installs_json.exists():
        try:
            with open(installs_json, "r", encoding="utf-8") as f:
                data = json.load(f)
            assoc = data.get("associated_client", {})
            for path_key in assoc.keys():
                pk_lower = path_key.lower()
                if "league of legends" in pk_lower:
                    games["lol"] = True
                if "valorant" in pk_lower:
                    games["valorant"] = True
        except Exception:
            pass
    return games

def get_running_processes():
    try:
        import psutil
        running = set()
        for p in psutil.process_iter(["name"]):
            try:
                name = p.info["name"]
                if name:
                    running.add(name.lower())
            except (psutil.NoSuchProcess, psutil.AccessDenied):
                pass
        return running
    except ImportError:
        out = subprocess.run(["tasklist", "/FO", "CSV", "/NH"], capture_output=True, text=True, errors="ignore").stdout
        running = set()
        for line in out.splitlines():
            parts = line.split('","')
            if parts and len(parts) > 0:
                pname = parts[0].strip('"').lower()
                running.add(pname)
        return running

def check_process_status():
    running = get_running_processes()
    riot_running = any(p in running for p in RIOT_PROCS)
    match_running = any(p in running for p in MATCH_PROCS)
    active_procs = [p for p in RIOT_PROCS if p in running]
    return {
        "riot_running": riot_running,
        "match_running": match_running,
        "active_processes": active_procs,
    }

def kill_riot_processes(force=True):
    running = get_running_processes()
    to_kill = [p for p in RIOT_PROCS if p in running]
    if not to_kill:
        return True
    for p in to_kill:
        subprocess.run(["taskkill", "/F", "/T", "/IM", p], capture_output=True, text=True, errors="ignore")
    # Wait up to 3s for cleanup
    for _ in range(15):
        time.sleep(0.2)
        cur = get_running_processes()
        if not any(p in cur for p in RIOT_PROCS):
            return True
    return False

def parse_active_riot_token(settings_file_path=None):
    target = Path(settings_file_path) if settings_file_path else RIOT_PRIVATE_SETTINGS
    if not target.exists():
        return {"logged_in": False, "reason": "Settings file does not exist"}
    try:
        with open(target, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()
        match = re.search(r"id_token:\s*\"([^\"]+)\"", content)
        if not match:
            return {"logged_in": False, "reason": "No id_token found"}
        token = match.group(1).replace("\n", "").replace("\r", "")
        parts = token.split(".")
        if len(parts) < 2:
            return {"logged_in": False, "reason": "Invalid token"}
        payload = parts[1]
        payload += "=" * ((4 - len(payload) % 4) % 4)
        data = json.loads(base64.urlsafe_b64decode(payload.encode()))
        acct = data.get("acct", {})
        game_name = acct.get("game_name", "Unknown")
        tag_line = acct.get("tag_line", "")
        sub = data.get("sub", "")
        regions = data.get("lol_region", [])
        active_region = "VN2"
        if isinstance(regions, list):
            for r in regions:
                if isinstance(r, dict) and r.get("active"):
                    active_region = r.get("cpid", "VN2")
                    break
        return {
            "logged_in": True,
            "name": game_name,
            "tag": tag_line,
            "region": active_region,
            "sub": sub,
            "display": f"{game_name} #{tag_line}" if tag_line else game_name
        }
    except Exception as e:
        return {"logged_in": False, "error": str(e)}

def load_accounts():
    try:
        with open(ACCOUNTS_FILE, "r", encoding="utf-8") as f:
            return json.load(f)
    except Exception:
        return []

def save_accounts(accounts):
    with open(ACCOUNTS_FILE, "w", encoding="utf-8") as f:
        json.dump(accounts, f, indent=2, ensure_ascii=False)

def copy_tree_safe(src_dir, dst_dir):
    src = Path(src_dir)
    dst = Path(dst_dir)
    if not src.exists():
        return
    dst.mkdir(parents=True, exist_ok=True)
    for root, dirs, files in os.walk(src):
        rel = os.path.relpath(root, src)
        dest_subdir = dst / rel if rel != "." else dst
        dest_subdir.mkdir(parents=True, exist_ok=True)
        for f in files:
            s_file = Path(root) / f
            d_file = dest_subdir / f
            try:
                shutil.copy2(s_file, d_file)
            except Exception:
                pass

def clear_dir_contents(directory):
    d = Path(directory)
    if not d.exists():
        return
    for item in d.iterdir():
        try:
            if item.is_dir():
                shutil.rmtree(item, ignore_errors=True)
            else:
                item.unlink(missing_ok=True)
        except Exception:
            pass

def backup_live_session():
    target = BACKUPS_DIR / "last_active"
    if target.exists():
        shutil.rmtree(target, ignore_errors=True)
    target.mkdir(parents=True, exist_ok=True)
    if RIOT_CLIENT_DATA.exists():
        copy_tree_safe(RIOT_CLIENT_DATA, target / "Data")
    if RIOT_CLIENT_CONFIG.exists():
        copy_tree_safe(RIOT_CLIENT_CONFIG, target / "Config")

def capture_current_session(account_id=None, custom_name=None, custom_tag=None, note="", avatar="jinx", theme_pref="lol"):
    parsed = parse_active_riot_token()
    if not parsed.get("logged_in") and not (custom_name and RIOT_PRIVATE_SETTINGS.exists()):
        return {"success": False, "error": "Không tìm thấy phiên đăng nhập Riot có ghi nhớ (Stay signed in)."}

    name = custom_name or parsed.get("name", "Account")
    tag = custom_tag or parsed.get("tag", "Riot")
    region = parsed.get("region", "VN2")
    sub = parsed.get("sub", "")

    clean_id = account_id or safe_id(name, tag)
    snap_dir = SNAPSHOTS_DIR / clean_id
    if snap_dir.exists():
        shutil.rmtree(snap_dir, ignore_errors=True)
    snap_dir.mkdir(parents=True, exist_ok=True)

    copy_tree_safe(RIOT_CLIENT_DATA, snap_dir / "Data")
    copy_tree_safe(RIOT_CLIENT_CONFIG, snap_dir / "Config")

    accounts = load_accounts()
    existing = next((a for a in accounts if a["id"] == clean_id), None)
    now_ts = int(time.time())

    if existing:
        existing["name"] = name
        existing["tag"] = tag
        existing["region"] = region
        existing["sub"] = sub
        existing["last_updated"] = now_ts
        if note: existing["note"] = note
        if avatar: existing["avatar"] = avatar
        if theme_pref: existing["theme_pref"] = theme_pref
    else:
        accounts.append({
            "id": clean_id,
            "name": name,
            "tag": tag,
            "region": region,
            "sub": sub,
            "note": note or "Tài khoản Riot",
            "avatar": avatar or "jinx",
            "theme_pref": theme_pref or "lol",
            "created_at": now_ts,
            "last_used": now_ts,
        })
    save_accounts(accounts)
    return {"success": True, "account": {"id": clean_id, "name": name, "tag": tag, "region": region}}

def read_riot_lockfile():
    lock_path = Path(LOCALAPPDATA) / "Riot Games" / "Riot Client" / "Config" / "lockfile"
    if not lock_path.exists():
        return None
    try:
        content = lock_path.read_text(encoding="utf-8").strip()
        parts = content.split(":")
        if len(parts) >= 5:
            return {
                "name": parts[0],
                "pid": int(parts[1]),
                "port": int(parts[2]),
                "password": parts[3],
                "protocol": parts[4]
            }
    except Exception:
        pass
    return None

def open_riot_client():
    exe = find_riot_client_services()
    if not exe:
        return {"success": False, "error": "Không tìm thấy RiotClientServices.exe"}
    DETACHED_PROCESS = 0x00000008
    CREATE_NEW_PROCESS_GROUP = 0x00000200
    subprocess.Popen([exe], cwd=str(Path(exe).parent), creationflags=DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP, close_fds=True)
    return {"success": True}

def focus_game_window(product="league_of_legends"):
    try:
        import ctypes
        prod_id = "league_of_legends" if product in ("lol", "league_of_legends") else "valorant"
        user32 = ctypes.windll.user32
        target_titles = ["League of Legends (TM) Client", "League of Legends"] if prod_id == "league_of_legends" else ["VALORANT"]

        found_hwnd = None
        def enum_windows_callback(hwnd, extra):
            nonlocal found_hwnd
            if user32.IsWindowVisible(hwnd):
                length = user32.GetWindowTextLengthW(hwnd)
                if length > 0:
                    buff = ctypes.create_unicode_buffer(length + 1)
                    user32.GetWindowTextW(hwnd, buff, length + 1)
                    title = buff.value
                    for t in target_titles:
                        if t.lower() in title.lower():
                            found_hwnd = hwnd
                            return False
            return True

        CMPFUNC = ctypes.WINFUNCTYPE(ctypes.c_bool, ctypes.c_void_p, ctypes.c_void_p)
        user32.EnumWindows(CMPFUNC(enum_windows_callback), 0)

        if found_hwnd:
            user32.ShowWindow(found_hwnd, 9)  # SW_RESTORE
            user32.SetForegroundWindow(found_hwnd)
            return True
    except Exception:
        pass
    return False

def is_game_running(product="league_of_legends"):
    prod_id = "league_of_legends" if product in ("lol", "league_of_legends") else "valorant"
    running = get_running_processes()
    if prod_id == "league_of_legends":
        lol_procs = ["league of legends.exe", "leagueclient.exe", "leagueclientux.exe", "leagueclientuxrender.exe"]
        return any(p in running for p in lol_procs)
    else:
        val_procs = ["valorant.exe", "valorant-win64-shipping.exe"]
        return any(p in running for p in val_procs)

def launch_riot_product(product="league_of_legends", patchline="live", force=False):
    # Normalize product
    prod_id = "league_of_legends" if product in ("lol", "league_of_legends") else "valorant"
    other_prod = "valorant" if prod_id == "league_of_legends" else "league_of_legends"
    prod_label = "Liên Minh Huyền Thoại" if prod_id == "league_of_legends" else "VALORANT"
    other_label = "VALORANT" if prod_id == "league_of_legends" else "Liên Minh Huyền Thoại"

    # 1. If this exact game is already running, focus window and return already_running
    if is_game_running(prod_id):
        focused = focus_game_window(prod_id)
        # Also try Riot Client's default product focus API
        lock = read_riot_lockfile()
        if lock:
            try:
                auth = base64.b64encode(f"riot:{lock['password']}".encode()).decode()
                ctx = ssl.create_default_context()
                ctx.check_hostname = False
                ctx.verify_mode = ssl.CERT_NONE
                f_url = f"https://127.0.0.1:{lock['port']}/product-launcher/v1/default-product/focus"
                f_req = urllib.request.Request(f_url, headers={"Authorization": f"Basic {auth}", "Content-Type": "application/json"}, method="POST", data=b"{}")
                urllib.request.urlopen(f_req, context=ctx, timeout=2)
            except Exception:
                pass
        return {"success": True, "already_running": True, "product": prod_id, "focused": focused}

    # 2. Check conflict with the other game (Riot Vanguard restricts 2 games running simultaneously)
    if is_game_running(other_prod):
        match_status = check_process_status()
        if match_status.get("match_running") and not force:
            return {
                "success": False,
                "conflict": True,
                "error": f"Đang có trận đấu {other_label} diễn ra! Không thể mở {prod_label} lúc này."
            }
        elif force:
            kill_riot_processes(force=True)
            time.sleep(1)
        else:
            return {
                "success": False,
                "conflict": True,
                "error": f"{other_label} đang chạy trên máy. Riot Vanguard chỉ cho phép mở 1 game cùng lúc. Vui lòng đóng {other_label} trước!"
            }

    # 3. Check if Riot Client is running
    lock = read_riot_lockfile()
    api_triggered = False

    # 3a. If Riot Client is running, trigger launch via its internal REST API
    if lock:
        try:
            auth = base64.b64encode(f"riot:{lock['password']}".encode()).decode()
            ctx = ssl.create_default_context()
            ctx.check_hostname = False
            ctx.verify_mode = ssl.CERT_NONE
            url = f"https://127.0.0.1:{lock['port']}/product-launcher/v1/products/{prod_id}/patchlines/{patchline}"
            req = urllib.request.Request(
                url,
                headers={"Authorization": f"Basic {auth}", "Content-Type": "application/json"},
                method="POST",
                data=b"{}"
            )
            with urllib.request.urlopen(req, context=ctx, timeout=5) as resp:
                if resp.status in (200, 204):
                    api_triggered = True
        except urllib.error.HTTPError as e:
            if e.code == 423:
                # already launched
                focus_game_window(prod_id)
                return {"success": True, "already_running": True, "product": prod_id}
        except Exception:
            pass

    # 3b. If REST API wasn't triggered (Riot Client cold start or API fallback), use CLI
    if not api_triggered:
        exe = find_riot_client_services()
        if not exe:
            return {"success": False, "error": "Không tìm thấy RiotClientServices.exe"}
        try:
            DETACHED_PROCESS = 0x00000008
            CREATE_NEW_PROCESS_GROUP = 0x00000200
            subprocess.Popen(
                [exe, f"--launch-product={prod_id}", f"--launch-patchline={patchline}"],
                cwd=str(Path(exe).parent),
                creationflags=DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP,
                close_fds=True
            )
        except Exception as e:
            return {"success": False, "error": f"Lỗi khởi chạy {prod_id} qua Riot Client: {e}"}

    # 4. Polling verification loop (up to 8 seconds) to verify the game actually launched
    for _ in range(16):
        time.sleep(0.5)
        if is_game_running(prod_id):
            focus_game_window(prod_id)
            return {"success": True, "product": prod_id}

    # 5. Fallback check: Did Riot Client at least open?
    cur_lock = read_riot_lockfile()
    if cur_lock:
        open_riot_client()
        return {
            "success": True,
            "pending": True,
            "message": f"Đã gửi lệnh mở {prod_label} tới Riot Client. Cửa sổ Riot Client đang hiển thị để bạn vào game."
        }
    else:
        return {
            "success": False,
            "error": f"Không thể khởi chạy {prod_label}. Hãy mở Riot Client để kiểm tra cập nhật game."
        }

def switch_to_account(account_id, launch_mode="none", force=False):
    snap_dir = SNAPSHOTS_DIR / account_id
    if not snap_dir.exists():
        return {"success": False, "error": f"Không tìm thấy dữ liệu sao lưu cho tài khoản {account_id}."}

    accounts = load_accounts()
    target_acc = next((a for a in accounts if a["id"] == account_id), None)
    if not target_acc:
        return {"success": False, "error": "Không tìm thấy thông tin tài khoản"}

    # Check if target account is ALREADY active on the PC
    active = parse_active_riot_token()
    is_already_active = False
    if active.get("logged_in"):
        cur_name = active.get("name", "").lower()
        cur_tag = active.get("tag", "").lower()
        tgt_name = target_acc.get("name", "").lower()
        tgt_tag = target_acc.get("tag", "").lower()
        if cur_name == tgt_name and (not tgt_tag or cur_tag == tgt_tag):
            is_already_active = True

    # Only kill & swap if NOT already active!
    if not is_already_active:
        kill_riot_processes(force=True)
        backup_live_session()

        RIOT_CLIENT_DATA.mkdir(parents=True, exist_ok=True)
        clear_dir_contents(RIOT_CLIENT_DATA)
        if RIOT_CLIENT_CONFIG.exists():
            clear_dir_contents(RIOT_CLIENT_CONFIG)

        snap_data = snap_dir / "Data"
        snap_config = snap_dir / "Config"
        if snap_data.exists():
            copy_tree_safe(snap_data, RIOT_CLIENT_DATA)
        if snap_config.exists():
            copy_tree_safe(snap_config, RIOT_CLIENT_CONFIG)

    # Update last_used
    for a in accounts:
        if a["id"] == account_id:
            a["last_used"] = int(time.time())
            break
    save_accounts(accounts)

    # Launch if requested
    if launch_mode == "lol":
        res = launch_riot_product("league_of_legends", "live")
        return {**res, "account_id": account_id, "already_active": is_already_active}
    elif launch_mode == "valorant":
        res = launch_riot_product("valorant", "live")
        return {**res, "account_id": account_id, "already_active": is_already_active}
    elif launch_mode == "client":
        res = open_riot_client()
        return {**res, "account_id": account_id, "already_active": is_already_active}

    # If simple switch without specific game, ensure Riot Client opens to show the newly activated account
    if not is_already_active:
        open_riot_client()

    return {"success": True, "account_id": account_id, "already_active": is_already_active}

def prepare_new_login_session():
    # 1. Kill Riot
    kill_riot_processes(force=True)
    # 2. Backup live session
    backup_live_session()
    # 3. Clear live data so fresh login screen appears
    if RIOT_CLIENT_DATA.exists():
        clear_dir_contents(RIOT_CLIENT_DATA)
    # 4. Start Riot Client
    exe = find_riot_client_services()
    if not exe:
        return {"success": False, "error": "Không tìm thấy RiotClientServices.exe"}
    DETACHED_PROCESS = 0x00000008
    CREATE_NEW_PROCESS_GROUP = 0x00000200
    subprocess.Popen([exe], cwd=str(Path(exe).parent), creationflags=DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP, close_fds=True)
    return {"success": True, "message": "Đã khởi chạy Riot Client. Hãy đăng nhập và nhớ tích 'Duy trì đăng nhập'."}

def import_from_tcno():
    if not TCNO_RIOT_CACHE.exists():
        return {"success": False, "imported": 0, "message": "Không tìm thấy thư mục lưu trữ của TcNo Account Switcher."}

    imported_count = 0
    accounts = load_accounts()
    existing_ids = {a["id"] for a in accounts}

    for item in TCNO_RIOT_CACHE.iterdir():
        if not item.is_dir() or item.name == "Shortcuts":
            continue

        # Look for yaml
        candidates = [
            item / "RiotClientPrivateSettings.yaml" / "RiotGamesPrivateSettings.yaml",
            item / "RiotGamesPrivateSettings.yaml",
            item / "Data" / "RiotGamesPrivateSettings.yaml",
        ]
        found_yaml = None
        for c in candidates:
            if c.exists():
                found_yaml = c
                break

        if not found_yaml:
            continue

        parsed = parse_active_riot_token(found_yaml)
        name = parsed.get("name") if parsed.get("logged_in") else item.name
        tag = parsed.get("tag", "")
        region = parsed.get("region", "VN2")
        sub = parsed.get("sub", "")

        acc_id = safe_id(name, tag) if tag else safe_id(item.name, "")
        snap_dir = SNAPSHOTS_DIR / acc_id
        snap_dir.mkdir(parents=True, exist_ok=True)

        # Copy Data
        data_source = item / "RiotClientPrivateSettings.yaml"
        if data_source.is_dir():
            copy_tree_safe(data_source, snap_dir / "Data")
        elif (item / "Data").is_dir():
            copy_tree_safe(item / "Data", snap_dir / "Data")
        else:
            (snap_dir / "Data").mkdir(exist_ok=True)
            shutil.copy2(found_yaml, snap_dir / "Data" / "RiotGamesPrivateSettings.yaml")

        # Copy Config
        config_source = item / "RiotClientSettings.yaml"
        if config_source.is_dir():
            copy_tree_safe(config_source, snap_dir / "Config")
        elif (item / "Config").is_dir():
            copy_tree_safe(item / "Config", snap_dir / "Config")

        if acc_id not in existing_ids:
            accounts.append({
                "id": acc_id,
                "name": name,
                "tag": tag,
                "region": region,
                "sub": sub,
                "note": f"Imported từ TcNo ({item.name})",
                "avatar": "jinx" if imported_count % 2 == 0 else "reyna",
                "theme_pref": "lol" if imported_count % 2 == 0 else "valorant",
                "created_at": int(time.time()),
                "last_used": int(time.time()),
            })
            existing_ids.add(acc_id)
            imported_count += 1

    save_accounts(accounts)
    return {"success": True, "imported": imported_count, "total": len(accounts)}

# HTTP Handler
class RiotSwitcherHandler(SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(WEB_DIR), **kwargs)

    def log_message(self, format, *args):
        pass # Quiet mode

    def send_json(self, data, status=200):
        body = json.dumps(data, ensure_ascii=False).encode("utf-8")
        self.send_response(status)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        self.send_response(200)
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, DELETE, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")
        self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path

        if path == "/api/status":
            proc_stat = check_process_status()
            active_acc = parse_active_riot_token()
            client_exe = find_riot_client_services()
            installed_games = find_installed_games()
            self.send_json({
                **proc_stat,
                "active_account": active_acc,
                "client_exe": client_exe,
                "installed_games": installed_games,
            })
            return

        if path == "/api/accounts":
            accounts = load_accounts()
            self.send_json({"accounts": accounts})
            return

        if path == "/api/games":
            self.send_json(find_installed_games())
            return

        return super().do_GET()

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        length = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(length).decode("utf-8") if length > 0 else "{}"
        try:
            payload = json.loads(body)
        except Exception:
            payload = {}

        if path == "/api/accounts/capture":
            res = capture_current_session(
                custom_name=payload.get("name"),
                custom_tag=payload.get("tag"),
                note=payload.get("note", ""),
                avatar=payload.get("avatar", "jinx"),
                theme_pref=payload.get("theme_pref", "lol")
            )
            self.send_json(res)
            return

        if path == "/api/accounts/switch":
            acc_id = payload.get("id")
            launch_mode = payload.get("launch", "none")
            force = payload.get("force", False)
            res = switch_to_account(acc_id, launch_mode, force=force)
            self.send_json(res)
            return

        if path == "/api/launch":
            product = payload.get("product", "lol")
            force = payload.get("force", False)
            if product == "client":
                res = open_riot_client()
            else:
                prod_id = "league_of_legends" if product in ("lol", "league_of_legends") else "valorant"
                res = launch_riot_product(prod_id, "live", force=force)
            self.send_json(res)
            return

        if path == "/api/accounts/new-session":
            res = prepare_new_login_session()
            self.send_json(res)
            return

        if path == "/api/accounts/update":
            acc_id = payload.get("id")
            accounts = load_accounts()
            found = False
            for a in accounts:
                if a["id"] == acc_id:
                    if "note" in payload: a["note"] = payload["note"]
                    if "avatar" in payload: a["avatar"] = payload["avatar"]
                    if "name" in payload and payload["name"]: a["name"] = payload["name"]
                    if "tag" in payload and payload["tag"]: a["tag"] = payload["tag"]
                    if "theme_pref" in payload: a["theme_pref"] = payload["theme_pref"]
                    found = True
                    break
            if found:
                save_accounts(accounts)
                self.send_json({"success": True})
            else:
                self.send_json({"success": False, "error": "Account not found"}, 404)
            return

        if path == "/api/accounts/delete":
            acc_id = payload.get("id")
            accounts = load_accounts()
            accounts = [a for a in accounts if a["id"] != acc_id]
            save_accounts(accounts)
            snap_dir = SNAPSHOTS_DIR / acc_id
            if snap_dir.exists():
                shutil.rmtree(snap_dir, ignore_errors=True)
            self.send_json({"success": True})
            return

        if path == "/api/import-tcno":
            res = import_from_tcno()
            self.send_json(res)
            return

        if path == "/api/kill-riot":
            ok = kill_riot_processes(force=True)
            self.send_json({"success": ok})
            return

        self.send_json({"error": "Endpoint not found"}, 404)

def run_server(port=48200):
    server_address = ("127.0.0.1", port)
    try:
        httpd = HTTPServer(server_address, RiotSwitcherHandler)
        print(f"Riot Switcher running at http://127.0.0.1:{port}")
        # Auto import from TcNo on first startup if accounts empty
        if not load_accounts():
            print("Auto-importing accounts from TcNo...")
            import_from_tcno()
        httpd.serve_forever()
    except OSError as e:
        print(f"Port {port} error: {e}")

if __name__ == "__main__":
    port = 48200
    if len(sys.argv) > 1 and sys.argv[1].isdigit():
        port = int(sys.argv[1])
    run_server(port)
