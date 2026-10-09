//! Logic lõi: đường dẫn Riot, snapshot tài khoản, chuyển tài khoản, khởi chạy
//! game, dọn dung lượng. Toàn bộ đọc/ghi dữ liệu người dùng nằm ở đây.
//!
//! Ghi chú an toàn: các thao tác chỉ *đọc* phiên Riot (id_token trong
//! RiotGamesPrivateSettings.yaml). Không bao giờ in giá trị token ra log.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub const APP_VERSION: &str = "2.0.0";

/// Tên tệp KHÔNG bao giờ được lưu vào snapshot tài khoản:
///  - ClientConfiguration.json: cache cấu hình máy ~17MB, không chứa credential.
///  - lockfile: mô tả PID/port của Riot Client đang chạy; khôi phục lockfile cũ
///    khiến Riot tưởng phiên đã chết còn sống -> lỗi không mở được game.
pub const SNAPSHOT_EXCLUDE_FILES: [&str; 2] = ["ClientConfiguration.json", "lockfile"];

pub const RIOT_PROCS: [&str; 11] = [
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
];

pub const MATCH_PROCS: [&str; 2] = ["league of legends.exe", "valorant-win64-shipping.exe"];

// ---------------------------------------------------------------------------
// Đường dẫn
// ---------------------------------------------------------------------------

fn env_path(key: &str, fallback: &str) -> PathBuf {
    std::env::var_os(key)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(fallback))
}

/// Thư mục chứa dữ liệu ứng dụng (data/snapshots, data/backups, accounts.json).
pub fn data_dir() -> PathBuf {
    // Dữ liệu ứng dụng nằm trong %LOCALAPPDATA%\RiotAccountSwitcher.
    env_path("LOCALAPPDATA", "C:\\ProgramData").join("RiotAccountSwitcher")
}

pub fn snapshots_dir() -> PathBuf {
    data_dir().join("snapshots")
}

pub fn backups_dir() -> PathBuf {
    data_dir().join("backups")
}

pub fn accounts_file() -> PathBuf {
    data_dir().join("accounts.json")
}

pub fn settings_file() -> PathBuf {
    data_dir().join("settings.json")
}

pub fn riot_client_data() -> PathBuf {
    env_path("LOCALAPPDATA", "C:\\ProgramData")
        .join("Riot Games")
        .join("Riot Client")
        .join("Data")
}

pub fn riot_client_config() -> PathBuf {
    env_path("LOCALAPPDATA", "C:\\ProgramData")
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
}

pub fn riot_private_settings() -> PathBuf {
    riot_client_data().join("RiotGamesPrivateSettings.yaml")
}

pub fn live_lockfile() -> PathBuf {
    riot_client_config().join("lockfile")
}

pub fn ensure_dirs() -> std::io::Result<()> {
    fs::create_dir_all(snapshots_dir())?;
    fs::create_dir_all(backups_dir())?;
    if !accounts_file().exists() {
        fs::write(accounts_file(), "[]")?;
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Mô hình dữ liệu
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Account {
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub sub: String,
    #[serde(default)]
    pub note: String,
    #[serde(default)]
    pub avatar: String,
    #[serde(default)]
    pub theme_pref: String,
    #[serde(default)]
    pub created_at: i64,
    #[serde(default)]
    pub last_used: i64,
}

impl Account {
    pub fn riot_id(&self) -> String {
        if self.tag.is_empty() {
            self.name.clone()
        } else {
            format!("{}#{}", self.name, self.tag)
        }
    }
}

/// Kết quả đọc phiên đăng nhập Riot đang hoạt động.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ActiveAccount {
    pub logged_in: bool,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub tag: String,
    #[serde(default)]
    pub region: String,
    #[serde(default)]
    pub sub: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error: String,
}

/// Trạng thái tiến trình Riot.
#[derive(Serialize, Clone, Debug)]
pub struct ProcessStatus {
    pub riot_running: bool,
    pub match_running: bool,
    pub active_processes: Vec<String>,
}

/// Thông tin lockfile của Riot Client đang chạy.
#[derive(Clone, Debug)]
pub struct RiotLock {
    pub port: u16,
    pub password: String,
}

/// Một dòng trong báo cáo dung lượng.
#[derive(Serialize, Clone, Debug)]
pub struct StorageItem {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub size_bytes: u64,
    pub size_human: String,
    pub junk_bytes: u64,
    pub junk_human: String,
    pub junk_files: usize,
    pub known: bool,
}

/// Báo cáo dung lượng tổng thể.
#[derive(Serialize, Clone, Debug)]
pub struct StorageReport {
    pub snapshots_dir: String,
    pub total_bytes: u64,
    pub total_human: String,
    pub junk_bytes: u64,
    pub junk_human: String,
    pub junk_files: usize,
    pub backup_bytes: u64,
    pub backup_human: String,
    pub items: Vec<StorageItem>,
    pub exclude_files: Vec<String>,
}

/// Kết quả chung của một hành động (switch/launch/...).
#[derive(Serialize, Clone, Debug, Default)]
pub struct ActionResult {
    pub success: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub error: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub message: String,
    #[serde(default)]
    pub already_running: bool,
    #[serde(default)]
    pub already_active: bool,
    #[serde(default)]
    pub pending: bool,
    #[serde(default)]
    pub conflict: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub product: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub account_id: String,
}

// ---------------------------------------------------------------------------
// Tiện ích
// ---------------------------------------------------------------------------

pub fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Chuẩn hoá tên tài khoản thành id an toàn cho tên thư mục.
pub fn safe_id(name: &str, tag: &str) -> String {
    let raw = format!("{name}_{tag}");
    let mut clean = String::new();
    for ch in raw.chars() {
        if ch.is_whitespace() || "/\\:*?\"<>|".contains(ch) {
            clean.push('_');
        } else {
            clean.push(ch);
        }
    }
    let clean = clean.trim_matches('_').to_string();
    // Chặn id rỗng/toàn dấu gạch và các id có thể gây path traversal (".", "..", "..."),
    // vì id được dùng trực tiếp làm tên thư mục trong snapshots/backups.
    let dots_only = !clean.is_empty() && clean.chars().all(|c| c == '.');
    if clean.is_empty() || clean.chars().all(|c| c == '_') || dots_only || clean.contains("..") {
        let mut hash: u64 = 1469598103934665603;
        for b in raw.as_bytes() {
            hash ^= *b as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        return format!("acc_{:08x}", (hash & 0xffff_ffff) as u32);
    }
    clean.to_lowercase()
}

/// Kiểm tra id tài khoản an toàn trước khi dùng làm tên thư mục (phòng vệ nhiều lớp).
fn is_safe_account_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && !id.contains('/')
        && !id.contains('\\')
        && !id.contains("..")
}

pub fn human_size(bytes: u64) -> String {
    let mut size = bytes as f64;
    for unit in ["B", "KB", "MB", "GB"] {
        if size < 1024.0 || unit == "GB" {
            if unit == "B" {
                return format!("{} B", bytes);
            }
            return format!("{size:.1} {unit}");
        }
        size /= 1024.0;
    }
    format!("{size:.1} GB")
}

fn is_excluded(name: &str) -> bool {
    SNAPSHOT_EXCLUDE_FILES.contains(&name)
}

/// Đếm tổng dung lượng mọi tệp trong cây thư mục.
pub fn dir_size_bytes(dir: &Path) -> u64 {
    let mut total = 0u64;
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            total += dir_size_bytes(&path);
        } else if let Ok(md) = entry.metadata() {
            total += md.len();
        }
    }
    total
}

/// Xoá toàn bộ nội dung bên trong một thư mục (giữ lại thư mục gốc).
pub fn clear_dir_contents(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let _ = fs::remove_dir_all(&path);
        } else {
            let _ = fs::remove_file(&path);
        }
    }
}

/// Copy cây thư mục, bỏ qua tệp có tên trong `SNAPSHOT_EXCLUDE_FILES`.
/// Trả về (số tệp đã copy, số tệp đã bỏ qua).
pub fn copy_tree_safe(src: &Path, dst: &Path) -> (usize, usize) {
    if !src.exists() {
        return (0, 0);
    }
    let _ = fs::create_dir_all(dst);
    let (mut copied, mut skipped) = (0usize, 0usize);
    let Ok(entries) = fs::read_dir(src) else {
        return (0, 0);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            let (c, s) = copy_tree_safe(&path, &dst.join(&name));
            copied += c;
            skipped += s;
        } else {
            if is_excluded(&name) {
                skipped += 1;
                continue;
            }
            if fs::copy(&path, dst.join(&name)).is_ok() {
                copied += 1;
            }
        }
    }
    (copied, skipped)
}

// ---------------------------------------------------------------------------
// Tài khoản (đọc/ghi)
// ---------------------------------------------------------------------------

pub fn load_accounts() -> Vec<Account> {
    match fs::read_to_string(accounts_file()) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub fn save_accounts(accounts: &[Account]) -> std::io::Result<()> {
    let _ = fs::create_dir_all(data_dir());
    let text = serde_json::to_string_pretty(accounts).unwrap_or_else(|_| "[]".to_string());
    fs::write(accounts_file(), text)
}

// ---------------------------------------------------------------------------
// Phiên Riot đang hoạt động (chỉ đọc)
// ---------------------------------------------------------------------------

fn b64url_decode(input: &str) -> Option<Vec<u8>> {
    use base64::Engine;
    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let mut s = input.trim().replace(['\n', '\r'], "");
    // Thêm padding nếu thiếu.
    while !s.len().is_multiple_of(4) {
        s.push('=');
    }
    engine.decode(s.as_bytes()).ok()
}

/// Đọc id_token trong RiotGamesPrivateSettings.yaml (chỉ đọc, không in ra ngoài).
pub fn parse_active_riot_token(path: Option<&Path>) -> ActiveAccount {
    let target = path.map(|p| p.to_path_buf()).unwrap_or_else(riot_private_settings);
    if !target.exists() {
        return ActiveAccount {
            logged_in: false,
            error: "Settings file does not exist".to_string(),
            ..Default::default()
        };
    }
    let Ok(content) = fs::read_to_string(&target) else {
        return ActiveAccount {
            logged_in: false,
            error: "Cannot read settings".to_string(),
            ..Default::default()
        };
    };

    // Tìm `id_token: "..."` (có thể xuống dòng trong chuỗi).
    let Some(token) = extract_id_token(&content) else {
        return ActiveAccount {
            logged_in: false,
            error: "No id_token found".to_string(),
            ..Default::default()
        };
    };

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return ActiveAccount {
            logged_in: false,
            error: "Invalid token".to_string(),
            ..Default::default()
        };
    }
    let Some(payload_bytes) = b64url_decode(parts[1]) else {
        return ActiveAccount {
            logged_in: false,
            error: "Invalid token payload".to_string(),
            ..Default::default()
        };
    };
    let Ok(data) = serde_json::from_slice::<serde_json::Value>(&payload_bytes) else {
        return ActiveAccount {
            logged_in: false,
            error: "Invalid token json".to_string(),
            ..Default::default()
        };
    };

    let acct = data.get("acct");
    let game_name = acct
        .and_then(|a| a.get("game_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let tag_line = acct
        .and_then(|a| a.get("tag_line"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let sub = data.get("sub").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let mut active_region = "VN2".to_string();
    if let Some(regions) = data.get("lol_region").and_then(|v| v.as_array()) {
        for r in regions {
            if r.get("active").and_then(|v| v.as_bool()).unwrap_or(false) {
                if let Some(cpid) = r.get("cpid").and_then(|v| v.as_str()) {
                    active_region = cpid.to_string();
                }
                break;
            }
        }
    }

    ActiveAccount {
        logged_in: true,
        name: game_name,
        tag: tag_line,
        region: active_region,
        sub,
        error: String::new(),
    }
}

/// Trích `id_token: "<value>"` từ nội dung YAML mà không cần parser YAML đầy đủ.
fn extract_id_token(content: &str) -> Option<String> {
    let idx = content.find("id_token:")?;
    let rest = &content[idx + "id_token:".len()..];
    let start = rest.find('"')? + 1;
    let after = &rest[start..];
    let end = after.find('"')?;
    let token: String = after[..end]
        .chars()
        .filter(|c| *c != '\n' && *c != '\r')
        .collect();
    if token.is_empty() {
        None
    } else {
        Some(token)
    }
}

// ---------------------------------------------------------------------------
// Tiến trình (qua sysinfo, không cần psutil)
// ---------------------------------------------------------------------------

pub fn get_running_processes() -> HashSet<String> {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.processes()
        .values()
        .map(|p| p.name().to_string_lossy().to_lowercase())
        .collect()
}

pub fn check_process_status() -> ProcessStatus {
    let running = get_running_processes();
    let active_processes: Vec<String> = RIOT_PROCS
        .iter()
        .filter(|p| running.contains(**p))
        .map(|p| p.to_string())
        .collect();
    ProcessStatus {
        riot_running: !active_processes.is_empty(),
        match_running: MATCH_PROCS.iter().any(|p| running.contains(*p)),
        active_processes,
    }
}

pub fn kill_riot_processes() -> bool {
    use sysinfo::{ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    let mut killed_any = false;
    for p in sys.processes().values() {
        let name = p.name().to_string_lossy().to_lowercase();
        if RIOT_PROCS.contains(&name.as_str()) {
            let _ = p.kill();
            killed_any = true;
        }
    }
    if !killed_any {
        return true;
    }
    // Chờ tối đa ~3s cho tiến trình thoát hẳn.
    for _ in 0..15 {
        std::thread::sleep(std::time::Duration::from_millis(200));
        let cur = get_running_processes();
        if !RIOT_PROCS.iter().any(|p| cur.contains(*p)) {
            return true;
        }
    }
    false
}

// ---------------------------------------------------------------------------
// Riot Client: tìm exe, lockfile, khởi chạy
// ---------------------------------------------------------------------------

pub fn find_riot_client_services() -> Option<String> {
    let installs_json = env_path("PROGRAMDATA", "C:\\ProgramData")
        .join("Riot Games")
        .join("RiotClientInstalls.json");
    if let Ok(text) = fs::read_to_string(&installs_json) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
            for key in ["rc_default", "rc_live"] {
                if let Some(p) = data.get(key).and_then(|v| v.as_str()) {
                    if Path::new(p).exists() {
                        return Some(p.to_string());
                    }
                }
            }
        }
    }
    for drive in ["G", "C", "D", "E"] {
        let candidate = format!("{drive}:\\Riot Games\\Riot Client\\RiotClientServices.exe");
        if Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}

pub fn find_installed_games() -> (bool, bool) {
    let installs_json = env_path("PROGRAMDATA", "C:\\ProgramData")
        .join("Riot Games")
        .join("RiotClientInstalls.json");
    let mut lol = false;
    let mut valorant = false;
    if let Ok(text) = fs::read_to_string(&installs_json) {
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(assoc) = data.get("associated_client").and_then(|v| v.as_object()) {
                for key in assoc.keys() {
                    let k = key.to_lowercase();
                    if k.contains("league of legends") {
                        lol = true;
                    }
                    if k.contains("valorant") {
                        valorant = true;
                    }
                }
            }
        }
    }
    (lol, valorant)
}

/// Đọc lockfile của Riot Client đang chạy (name:pid:port:password:protocol).
///
/// Bỏ qua lockfile cũ: nếu tiến trình Riot Client tương ứng đã tắt, lockfile
/// còn sót lại không đại diện cho phiên đang chạy nên không dùng để gọi API.
pub fn read_riot_lockfile() -> Option<RiotLock> {
    let path = env_path("LOCALAPPDATA", "C:\\ProgramData")
        .join("Riot Games")
        .join("Riot Client")
        .join("Config")
        .join("lockfile");
    let content = fs::read_to_string(path).ok()?;
    let parts: Vec<&str> = content.trim().split(':').collect();
    if parts.len() >= 5 {
        let pid: u32 = parts[1].parse().unwrap_or(0);
        if pid != 0 && !is_pid_alive(pid) {
            return None;
        }
        Some(RiotLock {
            port: parts[2].parse().unwrap_or(0),
            password: parts[3].to_string(),
        })
    } else {
        None
    }
}

/// Kiểm tra tiến trình còn sống theo PID (dùng sysinfo, chỉ đọc).
pub fn is_pid_alive(pid: u32) -> bool {
    use sysinfo::{Pid, ProcessesToUpdate, System};
    let mut sys = System::new();
    sys.refresh_processes(ProcessesToUpdate::All, true);
    sys.process(Pid::from_u32(pid)).is_some()
}

/// Khởi chạy RiotClientServices.exe dạng tách rời (không chờ).
fn spawn_detached(exe: &str, args: &[String]) -> std::io::Result<()> {
    let dir = Path::new(exe).parent().map(|p| p.to_path_buf());
    let mut cmd = std::process::Command::new(exe);
    cmd.args(args);
    if let Some(d) = dir {
        cmd.current_dir(d);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x0000_0008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);
    }
    cmd.spawn().map(|_| ())
}

pub fn open_riot_client() -> ActionResult {
    let Some(exe) = find_riot_client_services() else {
        return ActionResult {
            success: false,
            error: "Không tìm thấy RiotClientServices.exe".to_string(),
            ..Default::default()
        };
    };
    match spawn_detached(&exe, &[]) {
        Ok(_) => ActionResult {
            success: true,
            ..Default::default()
        },
        Err(e) => ActionResult {
            success: false,
            error: format!("Không thể mở Riot Client: {e}"),
            ..Default::default()
        },
    }
}

/// Gọi REST API nội bộ của Riot Client (self-signed TLS).
fn riot_api_post(port: u16, password: &str, path: &str) -> Result<u16, String> {
    use base64::Engine;
    let auth = base64::engine::general_purpose::STANDARD.encode(format!("riot:{password}"));
    let url = format!("https://127.0.0.1:{port}{path}");

    let tls = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .map_err(|e| e.to_string())?;
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .tls_connector(std::sync::Arc::new(tls))
        .build();

    match agent
        .post(&url)
        .set("Authorization", &format!("Basic {auth}"))
        .set("Content-Type", "application/json")
        .send_string("{}")
    {
        Ok(resp) => Ok(resp.status()),
        Err(ureq::Error::Status(code, _)) => Ok(code),
        Err(e) => Err(e.to_string()),
    }
}

/// Tập trung cửa sổ game đang chạy (best-effort trên Windows).
fn focus_game_window(product: &str) -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, SetForegroundWindow,
            ShowWindow, SW_RESTORE,
        };

        let targets: Vec<String> = if product == "league_of_legends" {
            vec!["league of legends".to_string()]
        } else {
            vec!["valorant".to_string()]
        };

        struct Search {
            targets: Vec<String>,
            found: HWND,
        }
        let mut search = Search {
            targets,
            found: HWND(std::ptr::null_mut()),
        };

        unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let search = &mut *(lparam.0 as *mut Search);
            if IsWindowVisible(hwnd).as_bool() {
                let len = GetWindowTextLengthW(hwnd);
                if len > 0 {
                    let mut buf = vec![0u16; (len + 1) as usize];
                    let copied = GetWindowTextW(hwnd, &mut buf);
                    if copied > 0 {
                        let title = String::from_utf16_lossy(&buf[..copied as usize]).to_lowercase();
                        for t in &search.targets {
                            if title.contains(t) {
                                search.found = hwnd;
                                return BOOL(0); // dừng
                            }
                        }
                    }
                }
            }
            BOOL(1)
        }

        unsafe {
            let _ = EnumWindows(Some(enum_cb), LPARAM(&mut search as *mut Search as isize));
            if !search.found.0.is_null() {
                let _ = ShowWindow(search.found, SW_RESTORE);
                let _ = SetForegroundWindow(search.found);
                return true;
            }
        }
    }
    let _ = product;
    false
}

fn is_game_running(product: &str) -> bool {
    let running = get_running_processes();
    if product == "league_of_legends" {
        [
            "league of legends.exe",
            "leagueclient.exe",
            "leagueclientux.exe",
            "leagueclientuxrender.exe",
        ]
        .iter()
        .any(|p| running.contains(*p))
    } else {
        ["valorant.exe", "valorant-win64-shipping.exe"]
            .iter()
            .any(|p| running.contains(*p))
    }
}

fn normalize_product(product: &str) -> &'static str {
    if product == "league_of_legends" || product == "lol" {
        "league_of_legends"
    } else {
        "valorant"
    }
}

/// Khởi chạy một sản phẩm Riot (LoL/Valorant) qua REST API hoặc CLI.
pub fn launch_riot_product(product: &str, patchline: &str, force: bool) -> ActionResult {
    let prod_id = normalize_product(product);
    let other_prod = if prod_id == "league_of_legends" {
        "valorant"
    } else {
        "league_of_legends"
    };
    let prod_label = if prod_id == "league_of_legends" {
        "Liên Minh Huyền Thoại"
    } else {
        "VALORANT"
    };
    let other_label = if prod_id == "league_of_legends" {
        "VALORANT"
    } else {
        "Liên Minh Huyền Thoại"
    };

    // 1. Game đã chạy -> focus.
    if is_game_running(prod_id) {
        let _ = focus_game_window(prod_id);
        if let Some(lock) = read_riot_lockfile() {
            let _ = riot_api_post(
                lock.port,
                &lock.password,
                "/product-launcher/v1/default-product/focus",
            );
        }
        return ActionResult {
            success: true,
            already_running: true,
            product: prod_id.to_string(),
            ..Default::default()
        };
    }

    // 2. Xung đột với game kia (Vanguard chỉ cho 1 game).
    if is_game_running(other_prod) {
        let match_status = check_process_status();
        if match_status.match_running && !force {
            return ActionResult {
                success: false,
                conflict: true,
                error: format!("Đang có trận đấu {other_label} diễn ra! Không thể mở {prod_label} lúc này."),
                ..Default::default()
            };
        } else if force {
            kill_riot_processes();
            std::thread::sleep(std::time::Duration::from_secs(1));
        } else {
            return ActionResult {
                success: false,
                conflict: true,
                error: format!("{other_label} đang chạy trên máy. Riot Vanguard chỉ cho phép mở 1 game cùng lúc. Vui lòng đóng {other_label} trước!"),
                ..Default::default()
            };
        }
    }

    // 3. Thử kích hoạt qua REST API nếu Riot Client đang chạy.
    let mut api_triggered = false;
    if let Some(lock) = read_riot_lockfile() {
        let path = format!("/product-launcher/v1/products/{prod_id}/patchlines/{patchline}");
        match riot_api_post(lock.port, &lock.password, &path) {
            Ok(200) | Ok(204) => api_triggered = true,
            Ok(423) => {
                // Đã được launch sẵn.
                focus_game_window(prod_id);
                return ActionResult {
                    success: true,
                    already_running: true,
                    product: prod_id.to_string(),
                    ..Default::default()
                };
            }
            _ => {}
        }
    }

    // 3b. Fallback: khởi chạy qua CLI.
    if !api_triggered {
        let Some(exe) = find_riot_client_services() else {
            return ActionResult {
                success: false,
                error: "Không tìm thấy RiotClientServices.exe".to_string(),
                ..Default::default()
            };
        };
        let args = vec![
            format!("--launch-product={prod_id}"),
            format!("--launch-patchline={patchline}"),
        ];
        if let Err(e) = spawn_detached(&exe, &args) {
            return ActionResult {
                success: false,
                error: format!("Lỗi khởi chạy {prod_id} qua Riot Client: {e}"),
                ..Default::default()
            };
        }
    }

    // 4. Chờ tối đa ~8s để xác nhận game thực sự mở.
    for _ in 0..16 {
        std::thread::sleep(std::time::Duration::from_millis(500));
        if is_game_running(prod_id) {
            focus_game_window(prod_id);
            return ActionResult {
                success: true,
                product: prod_id.to_string(),
                ..Default::default()
            };
        }
    }

    // 5. Fallback: Riot Client có mở không?
    if read_riot_lockfile().is_some() {
        let _ = open_riot_client();
        ActionResult {
            success: true,
            pending: true,
            message: format!("Đã gửi lệnh mở {prod_label} tới Riot Client. Cửa sổ Riot Client đang hiển thị để bạn vào game."),
            ..Default::default()
        }
    } else {
        ActionResult {
            success: false,
            error: format!("Không thể khởi chạy {prod_label}. Hãy mở Riot Client để kiểm tra cập nhật game."),
            ..Default::default()
        }
    }
}

// ---------------------------------------------------------------------------
// Snapshot & chuyển tài khoản
// ---------------------------------------------------------------------------

/// Sao lưu phiên live hiện tại vào data/backups/last_active.
pub fn backup_live_session() {
    let target = backups_dir().join("last_active");
    if target.exists() {
        let _ = fs::remove_dir_all(&target);
    }
    let _ = fs::create_dir_all(&target);
    if riot_client_data().exists() {
        copy_tree_safe(&riot_client_data(), &target.join("Data"));
    }
    if riot_client_config().exists() {
        copy_tree_safe(&riot_client_config(), &target.join("Config"));
    }
}

/// Lưu phiên đăng nhập đang mở thành snapshot tài khoản.
pub fn capture_current_session(
    custom_name: Option<String>,
    custom_tag: Option<String>,
    note: &str,
    avatar: &str,
    theme_pref: &str,
) -> Result<Account, String> {
    let parsed = parse_active_riot_token(None);
    let has_custom = custom_name.as_deref().map(|s| !s.is_empty()).unwrap_or(false);
    if !parsed.logged_in && !(has_custom && riot_private_settings().exists()) {
        return Err("Không tìm thấy phiên đăng nhập Riot có ghi nhớ (Stay signed in).".to_string());
    }

    let name = custom_name
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| if parsed.name.is_empty() { "Account".into() } else { parsed.name.clone() });
    let tag = custom_tag
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| parsed.tag.clone());
    let region = if parsed.region.is_empty() {
        "VN2".to_string()
    } else {
        parsed.region.clone()
    };
    let sub = parsed.sub.clone();

    let clean_id = safe_id(&name, &tag);
    let snap_dir = snapshots_dir().join(&clean_id);
    if snap_dir.exists() {
        let _ = fs::remove_dir_all(&snap_dir);
    }
    let _ = fs::create_dir_all(&snap_dir);

    copy_tree_safe(&riot_client_data(), &snap_dir.join("Data"));
    copy_tree_safe(&riot_client_config(), &snap_dir.join("Config"));

    // Phòng ngừa: không bao giờ để lockfile lọt vào snapshot.
    for stray in [
        snap_dir.join("Config").join("lockfile"),
        snap_dir.join("Data").join("lockfile"),
    ] {
        let _ = fs::remove_file(stray);
    }

    let mut accounts = load_accounts();
    let now = now_ts();
    let existing = accounts.iter_mut().find(|a| a.id == clean_id);
    if let Some(acc) = existing {
        acc.name = name.clone();
        acc.tag = tag.clone();
        acc.region = region.clone();
        acc.sub = sub.clone();
        acc.last_updated_now(now);
        if !note.is_empty() {
            acc.note = note.to_string();
        }
        if !avatar.is_empty() {
            acc.avatar = avatar.to_string();
        }
        if !theme_pref.is_empty() {
            acc.theme_pref = theme_pref.to_string();
        }
    } else {
        accounts.push(Account {
            id: clean_id.clone(),
            name: name.clone(),
            tag: tag.clone(),
            region: region.clone(),
            sub: sub.clone(),
            note: if note.is_empty() {
                "Tài khoản Riot".to_string()
            } else {
                note.to_string()
            },
            avatar: if avatar.is_empty() { "jinx".into() } else { avatar.into() },
            theme_pref: if theme_pref.is_empty() {
                "lol".into()
            } else {
                theme_pref.into()
            },
            created_at: now,
            last_used: now,
        });
    }
    save_accounts(&accounts).map_err(|e| e.to_string())?;

    Ok(Account {
        id: clean_id,
        name,
        tag,
        region,
        sub,
        note: note.to_string(),
        avatar: avatar.to_string(),
        theme_pref: theme_pref.to_string(),
        created_at: now,
        last_used: now,
    })
}

impl Account {
    fn last_updated_now(&mut self, now: i64) {
        self.last_used = now;
    }
}

/// Chuyển sang một tài khoản đã lưu.
pub fn switch_to_account(account_id: &str, launch_mode: &str, force: bool) -> ActionResult {
    if !is_safe_account_id(account_id) {
        return ActionResult {
            success: false,
            error: "ID tài khoản không hợp lệ.".to_string(),
            ..Default::default()
        };
    }
    let snap_dir = snapshots_dir().join(account_id);
    if !snap_dir.exists() {
        return ActionResult {
            success: false,
            error: format!("Không tìm thấy dữ liệu sao lưu cho tài khoản {account_id}."),
            ..Default::default()
        };
    }

    let mut accounts = load_accounts();
    let Some(target) = accounts.iter().find(|a| a.id == account_id).cloned() else {
        return ActionResult {
            success: false,
            error: "Không tìm thấy thông tin tài khoản".to_string(),
            ..Default::default()
        };
    };

    // Kiểm tra tài khoản đích đã active sẵn chưa.
    let active = parse_active_riot_token(None);
    let mut is_already_active = false;
    if active.logged_in {
        let cur_name = active.name.to_lowercase();
        let cur_tag = active.tag.to_lowercase();
        let tgt_name = target.name.to_lowercase();
        let tgt_tag = target.tag.to_lowercase();
        if cur_name == tgt_name && (tgt_tag.is_empty() || cur_tag == tgt_tag) {
            is_already_active = true;
        }
    }

    if !is_already_active {
        kill_riot_processes();
        backup_live_session();

        let _ = fs::create_dir_all(riot_client_data());
        clear_dir_contents(&riot_client_data());
        if riot_client_config().exists() {
            clear_dir_contents(&riot_client_config());
        }

        let snap_data = snap_dir.join("Data");
        let snap_config = snap_dir.join("Config");
        if snap_data.exists() {
            copy_tree_safe(&snap_data, &riot_client_data());
        }
        if snap_config.exists() {
            copy_tree_safe(&snap_config, &riot_client_config());
        }

        // Không mang lockfile cũ vào profile live.
        let _ = fs::remove_file(live_lockfile());
    }

    for acc in accounts.iter_mut() {
        if acc.id == account_id {
            acc.last_used = now_ts();
            break;
        }
    }
    let _ = save_accounts(&accounts);

    let mut result = match launch_mode {
        "lol" => launch_riot_product("league_of_legends", "live", force),
        "valorant" => launch_riot_product("valorant", "live", force),
        "client" => open_riot_client(),
        _ => {
            if !is_already_active {
                let _ = open_riot_client();
            }
            ActionResult {
                success: true,
                ..Default::default()
            }
        }
    };
    result.account_id = account_id.to_string();
    result.already_active = is_already_active;
    result
}

/// Chuẩn bị phiên đăng nhập mới: tắt Riot, sao lưu, xoá dữ liệu live, mở Riot Client.
pub fn prepare_new_login_session() -> ActionResult {
    kill_riot_processes();
    backup_live_session();
    if riot_client_data().exists() {
        clear_dir_contents(&riot_client_data());
    }
    let Some(exe) = find_riot_client_services() else {
        return ActionResult {
            success: false,
            error: "Không tìm thấy RiotClientServices.exe".to_string(),
            ..Default::default()
        };
    };
    match spawn_detached(&exe, &[]) {
        Ok(_) => ActionResult {
            success: true,
            message: "Đã khởi chạy Riot Client. Hãy đăng nhập và nhớ tích 'Duy trì đăng nhập'.".to_string(),
            ..Default::default()
        },
        Err(e) => ActionResult {
            success: false,
            error: format!("Không thể mở Riot Client: {e}"),
            ..Default::default()
        },
    }
}

// ---------------------------------------------------------------------------
// Dọn dung lượng
// ---------------------------------------------------------------------------

pub fn storage_report() -> StorageReport {
    let accounts = load_accounts();
    let mut total_snap_bytes = 0u64;
    let mut junk_bytes = 0u64;
    let mut junk_files = 0usize;
    let mut items = Vec::new();

    let snap_root = snapshots_dir();
    if let Ok(entries) = fs::read_dir(&snap_root) {
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        dirs.sort();
        for snap in dirs {
            let id = snap
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            let size = dir_size_bytes(&snap);
            total_snap_bytes += size;
            let (snap_junk, snap_junk_files) = junk_in_tree(&snap);
            junk_bytes += snap_junk;
            junk_files += snap_junk_files;
            let acc = accounts.iter().find(|a| a.id == id);
            items.push(StorageItem {
                id: id.clone(),
                name: acc.map(|a| a.name.clone()).unwrap_or_else(|| id.clone()),
                tag: acc.map(|a| a.tag.clone()).unwrap_or_default(),
                size_bytes: size,
                size_human: human_size(size),
                junk_bytes: snap_junk,
                junk_human: human_size(snap_junk),
                junk_files: snap_junk_files,
                known: acc.is_some(),
            });
        }
    }

    let backup_bytes = dir_size_bytes(&backups_dir());

    StorageReport {
        snapshots_dir: snap_root.to_string_lossy().to_string(),
        total_bytes: total_snap_bytes,
        total_human: human_size(total_snap_bytes),
        junk_bytes,
        junk_human: human_size(junk_bytes),
        junk_files,
        backup_bytes,
        backup_human: human_size(backup_bytes),
        items,
        exclude_files: SNAPSHOT_EXCLUDE_FILES.iter().map(|s| s.to_string()).collect(),
    }
}

fn junk_in_tree(dir: &Path) -> (u64, usize) {
    let (mut bytes, mut count) = (0u64, 0usize);
    let Ok(entries) = fs::read_dir(dir) else {
        return (0, 0);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let (b, c) = junk_in_tree(&path);
            bytes += b;
            count += c;
        } else {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_excluded(&name) {
                if let Ok(md) = entry.metadata() {
                    bytes += md.len();
                }
                count += 1;
            }
        }
    }
    (bytes, count)
}

/// Xoá tệp rác (ClientConfiguration.json / lockfile) khỏi snapshot + backup.
pub fn prune_storage(account_ids: Option<Vec<String>>, include_backups: bool) -> (u64, usize) {
    let mut reclaimed = 0u64;
    let mut removed = 0usize;

    let mut targets: Vec<PathBuf> = Vec::new();
    if let Some(ids) = account_ids {
        for id in ids {
            let d = snapshots_dir().join(id);
            if d.is_dir() {
                targets.push(d);
            }
        }
    } else if let Ok(entries) = fs::read_dir(snapshots_dir()) {
        targets = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
    }

    for snap in targets {
        let (b, c) = remove_junk_in_tree(&snap);
        reclaimed += b;
        removed += c;
    }
    if include_backups {
        let (b, c) = remove_junk_in_tree(&backups_dir());
        reclaimed += b;
        removed += c;
    }
    (reclaimed, removed)
}

fn remove_junk_in_tree(dir: &Path) -> (u64, usize) {
    let (mut bytes, mut count) = (0u64, 0usize);
    let Ok(entries) = fs::read_dir(dir) else {
        return (0, 0);
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let (b, c) = remove_junk_in_tree(&path);
            bytes += b;
            count += c;
        } else {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_excluded(&name) {
                let sz = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    bytes += sz;
                    count += 1;
                }
            }
        }
    }
    (bytes, count)
}

/// Xoá lockfile live cũ (PID/port của Riot Client đã chết).
pub fn clean_live_lockfile() -> bool {
    let lock = live_lockfile();
    let existed = lock.exists();
    let _ = fs::remove_file(&lock);
    existed
}

/// Xoá tài khoản khỏi danh sách và xoá snapshot tương ứng.
pub fn delete_account(account_id: &str) {
    if !is_safe_account_id(account_id) {
        return;
    }
    let accounts = load_accounts();
    let remaining: Vec<Account> = accounts.into_iter().filter(|a| a.id != account_id).collect();
    let _ = save_accounts(&remaining);
    let snap_dir = snapshots_dir().join(account_id);
    if snap_dir.exists() {
        let _ = fs::remove_dir_all(snap_dir);
    }
}

/// Cập nhật thông tin tài khoản (note/avatar/name/tag/theme).
pub fn update_account(
    account_id: &str,
    name: Option<String>,
    tag: Option<String>,
    note: Option<String>,
    avatar: Option<String>,
    theme_pref: Option<String>,
) -> bool {
    let mut accounts = load_accounts();
    let mut found = false;
    for acc in accounts.iter_mut() {
        if acc.id == account_id {
            if let Some(v) = note {
                acc.note = v;
            }
            if let Some(v) = avatar {
                acc.avatar = v;
            }
            if let Some(v) = name.filter(|s| !s.is_empty()) {
                acc.name = v;
            }
            if let Some(v) = tag.filter(|s| !s.is_empty()) {
                acc.tag = v;
            }
            if let Some(v) = theme_pref {
                acc.theme_pref = v;
            }
            found = true;
            break;
        }
    }
    if found {
        let _ = save_accounts(&accounts);
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_id_sanitizes() {
        assert_eq!(safe_id("Hello World", "VN2"), "hello_world_vn2");
        assert!(safe_id("", "").starts_with("acc_"));
    }

    #[test]
    fn human_size_formats() {
        assert_eq!(human_size(0), "0 B");
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1024), "1.0 KB");
        assert_eq!(human_size(1024 * 1024), "1.0 MB");
    }

    #[test]
    fn extract_id_token_works() {
        let yaml = "foo: bar\nid_token: \"abc.def.ghi\"\nbaz: 1\n";
        assert_eq!(extract_id_token(yaml).as_deref(), Some("abc.def.ghi"));
    }

    #[test]
    fn excluded_files_listed() {
        assert!(is_excluded("lockfile"));
        assert!(is_excluded("ClientConfiguration.json"));
        assert!(!is_excluded("RiotGamesPrivateSettings.yaml"));
    }
}
