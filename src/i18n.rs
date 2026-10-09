//! Song ngữ VI/EN — toàn bộ chuỗi giao diện.
//!
//! Mọi chuỗi hiển thị cho người dùng nằm ở đây; không hard-code text trong `app.rs`.

/// Ngôn ngữ đang chọn.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Vi,
    En,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::Vi => Lang::En,
            Lang::En => Lang::Vi,
        }
    }

    /// Nhãn của nút đổi ngôn ngữ (hiển thị ngôn ngữ *sẽ chuyển tới*).
    pub fn toggle_label(self) -> &'static str {
        match self {
            Lang::Vi => "🌐 English",
            Lang::En => "🇻🇳 Tiếng Việt",
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::Vi => "vi",
            Lang::En => "en",
        }
    }

    pub fn from_code(code: &str) -> Self {
        if code == "en" {
            Lang::En
        } else {
            Lang::Vi
        }
    }

    pub fn t(self) -> &'static Strings {
        match self {
            Lang::Vi => &VI,
            Lang::En => &EN,
        }
    }
}

/// Bảng chuỗi tĩnh của một ngôn ngữ.
pub struct Strings {
    pub app_sub: &'static str,
    pub riot_running: &'static str,
    pub riot_closed: &'static str,
    pub match_warning_pill: &'static str,
    pub no_account: &'static str,
    pub no_account_sub: &'static str,
    pub ready_play: &'static str,
    pub ready_play_client_closed: &'static str,
    pub in_match_active: &'static str,
    pub active_label: &'static str,
    pub launch_lol: &'static str,
    pub launch_val: &'static str,
    pub launch_client: &'static str,
    pub saved_accounts: &'static str,
    pub btn_capture: &'static str,
    pub btn_new_session: &'static str,
    pub btn_kill_riot: &'static str,
    pub btn_storage: &'static str,
    pub btn_switch: &'static str,
    pub badge_active: &'static str,
    pub default_note: &'static str,
    pub btn_cancel: &'static str,
    pub btn_close: &'static str,

    pub modal_capture_title: &'static str,
    pub modal_capture_desc: &'static str,
    pub label_display_name: &'static str,
    pub ph_capture_name: &'static str,
    pub label_note: &'static str,
    pub ph_capture_note: &'static str,
    pub label_avatar: &'static str,
    pub btn_confirm_save: &'static str,

    pub modal_new_title: &'static str,
    pub step1: &'static str,
    pub step2: &'static str,
    pub step3: &'static str,
    pub step_prefix: &'static str,
    pub waiting_login: &'static str,
    pub btn_start_login: &'static str,
    pub btn_save_new: &'static str,

    pub modal_match_title: &'static str,
    pub modal_match_body1: &'static str,
    pub modal_match_body2: &'static str,
    pub modal_match_body3: &'static str,
    pub btn_back_match: &'static str,
    pub btn_force_switch: &'static str,

    pub modal_delete_title: &'static str,
    pub modal_delete_body1: &'static str,
    pub modal_delete_body2: &'static str,
    pub modal_delete_sub: &'static str,
    pub btn_confirm_delete: &'static str,

    pub modal_edit_title: &'static str,
    pub label_tagline: &'static str,
    pub btn_save_changes: &'static str,

    pub ctx_edit: &'static str,
    pub ctx_switch_lol: &'static str,
    pub ctx_switch_val: &'static str,
    pub ctx_switch: &'static str,
    pub ctx_copy: &'static str,
    pub ctx_delete: &'static str,

    pub toast_switching: &'static str,
    pub toast_switched: &'static str,
    pub toast_already_active: &'static str,
    pub toast_killed: &'static str,
    pub toast_delete_success: &'static str,
    pub toast_edit_success: &'static str,
    pub toast_launching_lol: &'static str,
    pub toast_launching_val: &'static str,
    pub toast_launching_client: &'static str,
    pub game_not_installed: &'static str,

    pub modal_storage_title: &'static str,
    pub modal_storage_desc: &'static str,
    pub storage_total: &'static str,
    pub storage_junk: &'static str,
    pub storage_backup: &'static str,
    pub storage_none: &'static str,
    pub btn_prune_all: &'static str,
    pub btn_clean_lockfile: &'static str,
    pub toast_pruning: &'static str,
    pub toast_no_junk: &'static str,
    pub toast_lock_cleaned: &'static str,
    pub toast_lock_absent: &'static str,
    pub storage_loading: &'static str,

    pub empty_title: &'static str,
    pub empty_body: &'static str,
    pub error_connection: &'static str,
    pub just_created: &'static str,
    pub just_now: &'static str,
    pub copy_toast_prefix: &'static str,
}

pub static VI: Strings = Strings {
    app_sub: "Chuyển tài khoản LMHT & Valorant",
    riot_running: "Riot Client đang chạy",
    riot_closed: "Riot Client đã tắt",
    match_warning_pill: "ĐANG TRONG TRẬN",
    no_account: "Chưa phát hiện",
    no_account_sub: "⚪ Hãy chọn một tài khoản bên dưới để kích hoạt",
    ready_play: "🟢 Sẵn sàng chiến game",
    ready_play_client_closed: "⚪ Sẵn sàng (Riot Client sẽ tự mở)",
    in_match_active: "⚠️ Đang trong trận đấu!",
    active_label: "Tài khoản đang kích hoạt",
    launch_lol: "⚔️ VÀO LIÊN MINH HUYỀN THOẠI",
    launch_val: "🎯 VÀO VALORANT",
    launch_client: "⚡ Mở Riot Client",
    saved_accounts: "Danh sách tài khoản",
    btn_capture: "⚡ Lưu phiên hiện tại",
    btn_new_session: "+ Thêm tài khoản mới",
    btn_kill_riot: "🛑 Tắt Riot",
    btn_storage: "🧹 Dọn dung lượng",
    btn_switch: "⚡ Chuyển sang tài khoản này",
    badge_active: "✔ ĐANG KÍCH HOẠT",
    default_note: "Tài khoản Riot",
    btn_cancel: "Hủy",
    btn_close: "Đóng",

    modal_capture_title: "Lưu tài khoản hiện tại",
    modal_capture_desc: "Switcher sẽ đọc phiên đăng nhập Riot đang mở trên máy (đã tích 'Duy trì đăng nhập') và tạo snapshot để đổi nhanh bất cứ lúc nào.",
    label_display_name: "Tên hiển thị / Riot ID",
    ph_capture_name: "Tự động nhận diện từ Riot...",
    label_note: "Ghi chú tài khoản",
    ph_capture_note: "Nhập ghi chú cho tài khoản này...",
    label_avatar: "Chọn Avatar đại diện",
    btn_confirm_save: "Xác nhận lưu",

    modal_new_title: "Thêm tài khoản mới",
    step1: "Bấm 'Mở trang đăng nhập' bên dưới để Switcher làm sạch phiên cũ và khởi chạy Riot Client.",
    step2: "Đăng nhập tài khoản mới và TÍCH VÀO 'Duy trì đăng nhập' (Stay signed in).",
    step3: "Quay lại đây bấm 'Đã đăng nhập - Lưu ngay' để hoàn tất.",
    step_prefix: "Bước",
    waiting_login: "Đang chờ bạn đăng nhập trên Riot Client...",
    btn_start_login: "1. Mở trang đăng nhập",
    btn_save_new: "2. Đã đăng nhập - Lưu ngay",

    modal_match_title: "⚠️ Cảnh báo đang trong trận đấu",
    modal_match_body1: "Hệ thống phát hiện bạn ĐANG TRONG MỘT TRẬN ĐẤU (LMHT hoặc VALORANT).",
    modal_match_body2: "Nếu đổi tài khoản ngay bây giờ, game sẽ bị đóng đột ngột và bạn có thể bị xử phạt AFK / mất điểm xếp hạng.",
    modal_match_body3: "Bạn có chắc chắn muốn buộc đóng trận và chuyển tài khoản không?",
    btn_back_match: "Quay lại trận đấu",
    btn_force_switch: "Vẫn buộc đóng & Đổi",

    modal_delete_title: "🗑️ Xác nhận xóa tài khoản",
    modal_delete_body1: "Bạn có chắc chắn muốn xóa tài khoản",
    modal_delete_body2: "khỏi ứng dụng?",
    modal_delete_sub: "Toàn bộ dữ liệu phiên đăng nhập và bản sao lưu của tài khoản này trên máy sẽ bị xóa vĩnh viễn.",
    btn_confirm_delete: "Xác nhận xóa",

    modal_edit_title: "Chỉnh sửa tài khoản",
    label_tagline: "Tagline (sau dấu #)",
    btn_save_changes: "Lưu thay đổi",

    ctx_edit: "Đổi tên & Avatar",
    ctx_switch_lol: "Chuyển & Vào LMHT",
    ctx_switch_val: "Chuyển & Vào VALORANT",
    ctx_switch: "Chỉ chuyển tài khoản",
    ctx_copy: "Sao chép Riot ID",
    ctx_delete: "Xóa tài khoản",

    toast_switching: "Đang kiểm tra và chuyển tài khoản...",
    toast_switched: "Đã chuyển sang tài khoản thành công!",
    toast_already_active: "Tài khoản này đang được kích hoạt sẵn trên máy!",
    toast_killed: "Đã đóng toàn bộ tiến trình Riot!",
    toast_delete_success: "Đã xóa tài khoản khỏi ứng dụng!",
    toast_edit_success: "Đã cập nhật tên và avatar thành công!",
    toast_launching_lol: "Đang mở Liên Minh Huyền Thoại...",
    toast_launching_val: "Đang mở VALORANT...",
    toast_launching_client: "Đang mở Riot Client...",
    game_not_installed: "Game này chưa được cài trên máy",

    modal_storage_title: "🧹 Dọn dung lượng lưu trữ",
    modal_storage_desc: "Các bản snapshot cũ có thể chứa tệp cấu hình máy (ClientConfiguration.json ~17MB) và lockfile cũ. Các tệp này không phải dữ liệu tài khoản và có thể xóa an toàn.",
    storage_total: "Tổng dung lượng snapshot",
    storage_junk: "Dung lượng rác có thể dọn",
    storage_backup: "Dung lượng bản sao lưu",
    storage_none: "Không có tệp rác nào cần dọn. Tuyệt vời! 🎉",
    btn_prune_all: "Dọn tất cả",
    btn_clean_lockfile: "Xóa lockfile cũ (sửa lỗi mở game)",
    toast_pruning: "Đang dọn dung lượng...",
    toast_no_junk: "Không có gì để dọn.",
    toast_lock_cleaned: "Đã xóa lockfile cũ. Bạn có thể mở lại game!",
    toast_lock_absent: "Không tìm thấy lockfile cũ.",
    storage_loading: "Đang tính dung lượng...",

    empty_title: "Chưa có tài khoản nào được lưu",
    empty_body: "Hãy bấm 'Lưu phiên hiện tại' nếu bạn đang mở Riot Client để tạo snapshot tài khoản đầu tiên.",
    error_connection: "Lỗi kết nối",
    just_created: "Mới tạo",
    just_now: "Vừa xong",
    copy_toast_prefix: "Đã copy: ",
};

pub static EN: Strings = Strings {
    app_sub: "LoL & Valorant Fast Swap",
    riot_running: "Riot Client Running",
    riot_closed: "Riot Client Closed",
    match_warning_pill: "IN-MATCH ACTIVE",
    no_account: "Not Detected",
    no_account_sub: "⚪ Select an account below to activate",
    ready_play: "🟢 Ready to Play",
    ready_play_client_closed: "⚪ Ready (Riot Client will auto-start)",
    in_match_active: "⚠️ Active match in progress!",
    active_label: "Currently Active Account",
    launch_lol: "⚔️ LAUNCH LEAGUE OF LEGENDS",
    launch_val: "🎯 LAUNCH VALORANT",
    launch_client: "⚡ Open Riot Client",
    saved_accounts: "Saved Accounts",
    btn_capture: "⚡ Capture Current Session",
    btn_new_session: "+ Add New Account",
    btn_kill_riot: "🛑 Close Riot",
    btn_storage: "🧹 Clean Up Storage",
    btn_switch: "⚡ Switch to this account",
    badge_active: "✔ ACTIVE NOW",
    default_note: "Riot Account",
    btn_cancel: "Cancel",
    btn_close: "Close",

    modal_capture_title: "Capture Current Account",
    modal_capture_desc: "Switcher will read the active Riot session on this PC ('Stay signed in' must be checked) and create a snapshot for fast switching anytime.",
    label_display_name: "Display Name / Riot ID",
    ph_capture_name: "Auto-detected from Riot Client...",
    label_note: "Account Note",
    ph_capture_note: "Enter a note for this account...",
    label_avatar: "Select Profile Avatar",
    btn_confirm_save: "Save Account",

    modal_new_title: "Add New Account",
    step1: "Click 'Launch Login Window' below to clear the active session and launch Riot Client.",
    step2: "Log into your account and CHECK 'Stay signed in'.",
    step3: "Return here and click 'Signed In - Save Now' to finish.",
    step_prefix: "Step",
    waiting_login: "Waiting for you to sign in on Riot Client...",
    btn_start_login: "1. Launch Login Window",
    btn_save_new: "2. Signed In - Save Now",

    modal_match_title: "⚠️ Active Match Warning",
    modal_match_body1: "System detected an ACTIVE MATCH in progress (LoL or VALORANT).",
    modal_match_body2: "Switching accounts now will terminate the game immediately and may cause AFK penalties or rank loss.",
    modal_match_body3: "Are you sure you want to force close the game and switch?",
    btn_back_match: "Return to Game",
    btn_force_switch: "Force Close & Switch",

    modal_delete_title: "🗑️ Confirm Delete Account",
    modal_delete_body1: "Are you sure you want to delete account",
    modal_delete_body2: "from this app?",
    modal_delete_sub: "All saved session data and backups for this account will be permanently deleted from your computer.",
    btn_confirm_delete: "Confirm Delete",

    modal_edit_title: "Edit Account",
    label_tagline: "Tagline (after #)",
    btn_save_changes: "Save Changes",

    ctx_edit: "Edit Name & Avatar",
    ctx_switch_lol: "Switch & Launch LoL",
    ctx_switch_val: "Switch & Launch VALORANT",
    ctx_switch: "Switch Account Only",
    ctx_copy: "Copy Riot ID",
    ctx_delete: "Delete Account",

    toast_switching: "Verifying and switching account...",
    toast_switched: "Switched account successfully!",
    toast_already_active: "This account is already active on this PC!",
    toast_killed: "Stopped all Riot Games processes!",
    toast_delete_success: "Account deleted from app!",
    toast_edit_success: "Account name & avatar updated!",
    toast_launching_lol: "Launching League of Legends...",
    toast_launching_val: "Launching VALORANT...",
    toast_launching_client: "Launching Riot Client...",
    game_not_installed: "This game is not installed on this PC",

    modal_storage_title: "🧹 Clean Up Storage",
    modal_storage_desc: "Old snapshots may contain machine config files (ClientConfiguration.json ~17MB) and stale lockfiles. These are not account data and can be safely removed.",
    storage_total: "Total snapshot size",
    storage_junk: "Reclaimable junk size",
    storage_backup: "Backup size",
    storage_none: "No junk files to clean. All good! 🎉",
    btn_prune_all: "Clean All",
    btn_clean_lockfile: "Remove stale lockfile (fixes game launch)",
    toast_pruning: "Cleaning up storage...",
    toast_no_junk: "Nothing to clean.",
    toast_lock_cleaned: "Stale lockfile removed. You can launch the game again!",
    toast_lock_absent: "No stale lockfile found.",
    storage_loading: "Calculating size...",

    empty_title: "No accounts saved yet",
    empty_body: "Click 'Capture Current Session' if Riot Client is open to create your first account snapshot.",
    error_connection: "Connection error",
    just_created: "Just created",
    just_now: "Just now",
    copy_toast_prefix: "Copied: ",
};

impl Strings {
    /// "3 tài khoản" / "3 accounts".
    pub fn acc_count(&self, lang: Lang, n: usize) -> String {
        match lang {
            Lang::Vi => format!("{n} tài khoản"),
            Lang::En => {
                if n == 1 {
                    "1 account".to_string()
                } else {
                    format!("{n} accounts")
                }
            }
        }
    }

    /// "↳ Rác: 17.0 MB" / "↳ Junk: 17.0 MB".
    pub fn storage_item_junk(&self, lang: Lang, human: &str) -> String {
        match lang {
            Lang::Vi => format!("↳ Rác: {human}"),
            Lang::En => format!("↳ Junk: {human}"),
        }
    }

    /// "Đã dọn 38.0 MB (12 tệp) thành công!" / "Reclaimed 38.0 MB (12 files) successfully!".
    pub fn toast_pruned(&self, lang: Lang, human: &str, files: usize) -> String {
        match lang {
            Lang::Vi => format!("Đã dọn {human} ({files} tệp) thành công!"),
            Lang::En => format!("Reclaimed {human} ({files} files) successfully!"),
        }
    }

    /// Khoảng thời gian tương đối từ một mốc unix (giây).
    pub fn time_ago(&self, lang: Lang, ts: Option<i64>, now: i64) -> String {
        let Some(ts) = ts else {
            return self.just_created.to_string();
        };
        let diff = now - ts;
        if diff < 60 {
            return self.just_now.to_string();
        }
        if diff < 3600 {
            let m = diff / 60;
            return match lang {
                Lang::Vi => format!("{m} phút trước"),
                Lang::En => format!("{m}m ago"),
            };
        }
        if diff < 86400 {
            let h = diff / 3600;
            return match lang {
                Lang::Vi => format!("{h} giờ trước"),
                Lang::En => format!("{h}h ago"),
            };
        }
        let d = diff / 86400;
        match lang {
            Lang::Vi => format!("{d} ngày trước"),
            Lang::En => format!("{d}d ago"),
        }
    }
}
