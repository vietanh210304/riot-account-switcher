//! Giao diện native bằng egui (không còn HTML/JS).
//!
//! Toàn bộ thao tác nặng (chuyển tài khoản, mở game, dọn dung lượng...) chạy
//! trên luồng nền và trả kết quả về UI qua channel, để cửa sổ không bị đứng.

use crate::avatars;
use crate::core::{self, Account, ActionResult, StorageReport};
use crate::i18n::{Lang, Strings};
use eframe::egui;
use egui::{Align, Align2, Color32, Layout, RichText, Rounding, Vec2};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

const ACCENT: Color32 = Color32::from_rgb(200, 170, 110);
const ACCENT2: Color32 = Color32::from_rgb(10, 200, 185);
const BG_BASE: Color32 = Color32::from_rgb(1, 10, 19);
const BG_SURFACE: Color32 = Color32::from_rgb(9, 20, 40);
const BG_CARD: Color32 = Color32::from_rgba_premultiplied(10, 20, 40, 230);
const BG_CARD_HOVER: Color32 = Color32::from_rgba_premultiplied(14, 30, 60, 245);
const GREEN: Color32 = Color32::from_rgb(60, 200, 120);
const RED: Color32 = Color32::from_rgb(230, 90, 90);
const DIM: Color32 = Color32::from_rgb(150, 160, 175);

// ---------------------------------------------------------------------------
// Kiểu dữ liệu nội bộ UI
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
struct StatusData {
    riot_running: bool,
    match_running: bool,
    active: core::ActiveAccount,
    lol_installed: bool,
    val_installed: bool,
    version: String,
}

#[derive(PartialEq, Clone, Copy)]
enum Modal {
    None,
    Capture,
    NewSession,
    MatchWarning,
    Delete,
    Edit,
    Storage,
}

#[derive(Clone)]
enum PendingAction {
    Launch(String),
    Switch(String, String),
}

#[derive(Clone, Copy, PartialEq)]
enum ToastKind {
    Info,
    Success,
    Error,
}

struct Toast {
    msg: String,
    kind: ToastKind,
    born: f64,
}

/// Kết quả trả về từ luồng nền.
enum Bg {
    Status(StatusData),
    Accounts(Vec<Account>),
    Capture(Result<Account, String>),
    Switch(ActionResult),
    Launch(ActionResult),
    Kill(bool),
    Storage(StorageReport),
    Prune(u64, usize),
    CleanLock(bool),
    NewSession(ActionResult),
    Update(bool),
    Delete,
}

/// Chạy một tác vụ trên luồng nền và gửi kết quả về UI.
fn bg<F>(tx: &Sender<Bg>, f: F)
where
    F: FnOnce() -> Bg + Send + 'static,
{
    let tx = tx.clone();
    std::thread::spawn(move || {
        let _ = tx.send(f());
    });
}

fn fetch_status() -> StatusData {
    let ps = core::check_process_status();
    let active = core::parse_active_riot_token(None);
    let (lol, val) = core::find_installed_games();
    StatusData {
        riot_running: ps.riot_running,
        match_running: ps.match_running,
        active,
        lol_installed: lol,
        val_installed: val,
        version: core::APP_VERSION.to_string(),
    }
}

// ---------------------------------------------------------------------------
// App
// ---------------------------------------------------------------------------

pub struct App {
    lang: Lang,
    accounts: Vec<Account>,
    status: Option<StatusData>,
    storage: Option<StorageReport>,

    modal: Modal,
    pending: Option<PendingAction>,

    // Form: lưu phiên hiện tại
    cap_name: String,
    cap_note: String,
    cap_avatar: String,
    // Form: chỉnh sửa
    edit_id: String,
    edit_name: String,
    edit_tag: String,
    edit_note: String,
    edit_avatar: String,
    // Xoá
    del_id: String,
    del_label: String,
    // Phiên mới
    new_started: bool,
    // Dọn dung lượng
    storage_loading: bool,

    toasts: Vec<Toast>,
    textures: HashMap<String, egui::TextureHandle>,

    tx: Sender<Bg>,
    rx: Receiver<Bg>,
}

impl App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let _ = core::ensure_dirs();
        apply_visuals(&cc.egui_ctx);

        let (tx, rx) = channel();

        // Luồng nền: cập nhật trạng thái Riot mỗi 3 giây.
        let tx_status = tx.clone();
        std::thread::spawn(move || loop {
            let _ = tx_status.send(Bg::Status(fetch_status()));
            std::thread::sleep(Duration::from_secs(3));
        });

        // Tải danh sách tài khoản ban đầu.
        bg(&tx, || Bg::Accounts(core::load_accounts()));

        let lang = load_lang();
        Self {
            lang,
            accounts: Vec::new(),
            status: None,
            storage: None,
            modal: Modal::None,
            pending: None,
            cap_name: String::new(),
            cap_note: String::new(),
            cap_avatar: "jinx".to_string(),
            edit_id: String::new(),
            edit_name: String::new(),
            edit_tag: String::new(),
            edit_note: String::new(),
            edit_avatar: "jinx".to_string(),
            del_id: String::new(),
            del_label: String::new(),
            new_started: false,
            storage_loading: false,
            toasts: Vec::new(),
            textures: HashMap::new(),
            tx,
            rx,
        }
    }

    fn t(&self) -> &'static Strings {
        self.lang.t()
    }

    fn toast(&mut self, ctx: &egui::Context, msg: impl Into<String>, kind: ToastKind) {
        let now = ctx.input(|i| i.time);
        self.toasts.push(Toast {
            msg: msg.into(),
            kind,
            born: now,
        });
    }

    /// Xử lý kết quả từ luồng nền.
    fn poll_bg(&mut self, ctx: &egui::Context) {
        while let Ok(msg) = self.rx.try_recv() {
            match msg {
                Bg::Status(s) => self.status = Some(s),
                Bg::Accounts(list) => self.accounts = list,
                Bg::Capture(Ok(acc)) => {
                    self.modal = Modal::None;
                    let t = self.t();
                    self.toast(ctx, format!("✓ {}", acc.riot_id()), ToastKind::Success);
                    let _ = t;
                    bg(&self.tx, || Bg::Accounts(core::load_accounts()));
                }
                Bg::Capture(Err(e)) => {
                    self.toast(ctx, e, ToastKind::Error);
                }
                Bg::Switch(res) => {
                    if res.success {
                        let t = self.t();
                        let msg = if res.already_active {
                            t.toast_already_active.to_string()
                        } else {
                            t.toast_switched.to_string()
                        };
                        self.toast(ctx, msg, ToastKind::Success);
                    } else {
                        let e = if res.error.is_empty() {
                            self.t().error_connection.to_string()
                        } else {
                            res.error
                        };
                        self.toast(ctx, e, ToastKind::Error);
                    }
                    bg(&self.tx, || Bg::Accounts(core::load_accounts()));
                }
                Bg::Launch(res) => {
                    if res.success {
                        let t = self.t();
                        let msg = if res.already_running {
                            format!("{} ✓", t.ready_play)
                        } else if res.pending && !res.message.is_empty() {
                            res.message.clone()
                        } else {
                            t.toast_switched.to_string()
                        };
                        self.toast(ctx, msg, ToastKind::Success);
                    } else {
                        let e = if res.error.is_empty() {
                            self.t().error_connection.to_string()
                        } else {
                            res.error
                        };
                        self.toast(ctx, e, ToastKind::Error);
                    }
                }
                Bg::Kill(ok) => {
                    let t = self.t();
                    let (m, k) = if ok {
                        (t.toast_killed.to_string(), ToastKind::Success)
                    } else {
                        (t.error_connection.to_string(), ToastKind::Error)
                    };
                    self.toast(ctx, m, k);
                }
                Bg::Storage(rep) => {
                    self.storage_loading = false;
                    self.storage = Some(rep);
                }
                Bg::Prune(reclaimed, files) => {
                    let t = self.t();
                    if reclaimed > 0 {
                        let msg = t.toast_pruned(self.lang, &core::human_size(reclaimed), files);
                        self.toast(ctx, msg, ToastKind::Success);
                    } else {
                        let m = t.toast_no_junk.to_string();
                        self.toast(ctx, m, ToastKind::Info);
                    }
                    self.storage_loading = true;
                    bg(&self.tx, || Bg::Storage(core::storage_report()));
                }
                Bg::CleanLock(removed) => {
                    let t = self.t();
                    let (m, k) = if removed {
                        (t.toast_lock_cleaned.to_string(), ToastKind::Success)
                    } else {
                        (t.toast_lock_absent.to_string(), ToastKind::Info)
                    };
                    self.toast(ctx, m, k);
                }
                Bg::NewSession(res) => {
                    if res.success {
                        self.new_started = true;
                    } else {
                        let e = if res.error.is_empty() {
                            self.t().error_connection.to_string()
                        } else {
                            res.error
                        };
                        self.toast(ctx, e, ToastKind::Error);
                    }
                }
                Bg::Update(ok) => {
                    if ok {
                        self.modal = Modal::None;
                        let m = self.t().toast_edit_success.to_string();
                        self.toast(ctx, m, ToastKind::Success);
                        bg(&self.tx, || Bg::Accounts(core::load_accounts()));
                    } else {
                        let m = self.t().error_connection.to_string();
                        self.toast(ctx, m, ToastKind::Error);
                    }
                }
                Bg::Delete => {
                    self.modal = Modal::None;
                    let m = self.t().toast_delete_success.to_string();
                    self.toast(ctx, m, ToastKind::Success);
                    bg(&self.tx, || Bg::Accounts(core::load_accounts()));
                }
            }
        }
    }

    fn ensure_textures(&mut self, ctx: &egui::Context) {
        for av in avatars::AVATARS.iter() {
            if self.textures.contains_key(av.id) {
                continue;
            }
            if let Some(tex) = load_avatar_texture(ctx, av.id) {
                self.textures.insert(av.id.to_string(), tex);
            }
        }
    }

    fn avatar_tex(&self, id: &str) -> Option<egui::TextureHandle> {
        self.textures.get(id).cloned()
    }

    // -- Hành động -----------------------------------------------------------

    fn request_switch(&mut self, ctx: &egui::Context, id: String, mode: &str) {
        if self.status.as_ref().map(|s| s.match_running).unwrap_or(false) {
            self.pending = Some(PendingAction::Switch(id, mode.to_string()));
            self.modal = Modal::MatchWarning;
            return;
        }
        self.execute_switch(ctx, id, mode.to_string(), false);
    }

    fn execute_switch(&mut self, ctx: &egui::Context, id: String, mode: String, force: bool) {
        let m = self.t().toast_switching.to_string();
        self.toast(ctx, m, ToastKind::Info);
        bg(&self.tx, move || {
            Bg::Switch(core::switch_to_account(&id, &mode, force))
        });
    }

    fn request_launch(&mut self, ctx: &egui::Context, product: &str) {
        if product != "client"
            && self.status.as_ref().map(|s| s.match_running).unwrap_or(false)
        {
            self.pending = Some(PendingAction::Launch(product.to_string()));
            self.modal = Modal::MatchWarning;
            return;
        }
        self.execute_launch(ctx, product.to_string(), false);
    }

    fn execute_launch(&mut self, ctx: &egui::Context, product: String, force: bool) {
        let t = self.t();
        let msg = match product.as_str() {
            "lol" => t.toast_launching_lol,
            "valorant" => t.toast_launching_val,
            _ => t.toast_launching_client,
        }
        .to_string();
        self.toast(ctx, msg, ToastKind::Info);
        bg(&self.tx, move || {
            let res = if product == "client" {
                core::open_riot_client()
            } else {
                core::launch_riot_product(&product, "live", force)
            };
            Bg::Launch(res)
        });
    }

    fn open_capture(&mut self, ctx: &egui::Context) {
        if let Some(s) = &self.status {
            if s.active.logged_in {
                self.cap_name = if s.active.tag.is_empty() {
                    s.active.name.clone()
                } else {
                    format!("{} #{}", s.active.name, s.active.tag)
                };
            } else {
                self.cap_name.clear();
            }
        }
        self.cap_note.clear();
        self.modal = Modal::Capture;
        let _ = ctx;
    }

    fn open_edit(&mut self, id: &str) {
        if let Some(acc) = self.accounts.iter().find(|a| a.id == id) {
            self.edit_id = acc.id.clone();
            self.edit_name = acc.name.clone();
            self.edit_tag = acc.tag.clone();
            self.edit_note = acc.note.clone();
            self.edit_avatar = if acc.avatar.is_empty() {
                "jinx".to_string()
            } else {
                acc.avatar.clone()
            };
            self.modal = Modal::Edit;
        }
    }

    fn open_delete(&mut self, id: &str) {
        if let Some(acc) = self.accounts.iter().find(|a| a.id == id) {
            self.del_id = acc.id.clone();
            self.del_label = acc.riot_id();
            self.modal = Modal::Delete;
        }
    }

    fn open_storage(&mut self, ctx: &egui::Context) {
        self.modal = Modal::Storage;
        self.storage = None;
        self.storage_loading = true;
        let _ = ctx;
        bg(&self.tx, || Bg::Storage(core::storage_report()));
    }

    fn is_active(&self, acc: &Account) -> bool {
        let Some(s) = &self.status else {
            return false;
        };
        if !s.active.logged_in {
            return false;
        }
        let live_name = s.active.name.to_lowercase();
        let live_tag = s.active.tag.to_lowercase();
        acc.name.to_lowercase() == live_name
            && (acc.tag.is_empty() || acc.tag.to_lowercase() == live_tag)
    }

    // -- Render --------------------------------------------------------------

    fn header(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        let lang = self.lang;
        let riot_running = self.status.as_ref().map(|s| s.riot_running).unwrap_or(false);
        let version = self
            .status
            .as_ref()
            .map(|s| s.version.clone())
            .unwrap_or_else(|| core::APP_VERSION.to_string());

        ui.horizontal(|ui| {
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add_space(6.0);
                ui.label(RichText::new("RIOT ACCOUNT SWITCHER").size(20.0).strong().color(ACCENT));
                ui.label(RichText::new(format!("{} · v{}", t.app_sub, version)).size(12.0).color(DIM));
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.add_space(4.0);
                if ui.button(RichText::new(lang.toggle_label()).size(13.0)).clicked() {
                    self.lang = self.lang.toggle();
                    save_lang(self.lang);
                }
                // Chấm trạng thái Riot
                let (dot_color, label) = if riot_running {
                    (GREEN, t.riot_running)
                } else {
                    (DIM, t.riot_closed)
                };
                ui.label(RichText::new(label).size(12.0).color(DIM));
                ui.label(RichText::new("●").size(14.0).color(dot_color));
                // Pill cảnh báo trong trận
                if self.status.as_ref().map(|s| s.match_running).unwrap_or(false) {
                    ui.label(
                        RichText::new(t.match_warning_pill)
                            .size(11.0)
                            .strong()
                            .color(RED),
                    );
                }
            });
        });
    }

    fn hero(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        let active = self.status.as_ref().map(|s| s.active.clone());
        let match_running = self.status.as_ref().map(|s| s.match_running).unwrap_or(false);
        let riot_running = self.status.as_ref().map(|s| s.riot_running).unwrap_or(false);

        let frame = egui::Frame::none()
            .fill(BG_SURFACE)
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(16.0))
            .stroke(egui::Stroke::new(1.0_f32, Color32::from_rgb(30, 45, 70)));

        frame.show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                // Avatar tài khoản đang active
                let av_id = active
                    .as_ref()
                    .and_then(|a| {
                        self.accounts
                            .iter()
                            .find(|x| x.name.to_lowercase() == a.name.to_lowercase())
                            .map(|x| x.avatar.clone())
                    })
                    .unwrap_or_else(|| "jinx".to_string());
                if let Some(tex) = self.avatar_tex(&av_id) {
                    ui.add(
                        egui::Image::new(&tex)
                            .fit_to_exact_size(Vec2::splat(64.0))
                            .rounding(Rounding::same(32.0)),
                    );
                } else {
                    let (rect, _) = ui.allocate_exact_size(Vec2::splat(64.0), egui::Sense::hover());
                    ui.painter().circle_filled(rect.center(), 32.0, BG_CARD);
                }

                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.label(RichText::new(t.active_label).size(11.0).color(DIM));
                    match &active {
                        Some(a) if a.logged_in => {
                            ui.label(
                                RichText::new(&a.name)
                                    .size(22.0)
                                    .strong()
                                    .color(Color32::WHITE),
                            );
                            let status_text = if match_running {
                                t.in_match_active
                            } else if riot_running {
                                t.ready_play
                            } else {
                                t.ready_play_client_closed
                            };
                            ui.label(RichText::new(status_text).size(12.0).color(ACCENT2));
                        }
                        _ => {
                            ui.label(RichText::new(t.no_account).size(22.0).strong().color(DIM));
                            ui.label(RichText::new(t.no_account_sub).size(12.0).color(DIM));
                        }
                    }
                });

                // Nhóm nút khởi chạy
                let (lol_ok, val_ok) = self
                    .status
                    .as_ref()
                    .map(|s| (s.lol_installed, s.val_installed))
                    .unwrap_or((true, true));
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(secondary_button(t.launch_client))
                        .clicked()
                    {
                        self.request_launch(ui.ctx(), "client");
                    }
                    let val_btn = ui.add_enabled(val_ok, primary_button(t.launch_val));
                    if val_btn.clicked() {
                        self.request_launch(ui.ctx(), "valorant");
                    }
                    if !val_ok {
                        val_btn.on_hover_text(t.game_not_installed);
                    }
                    let lol_btn = ui.add_enabled(lol_ok, primary_button(t.launch_lol));
                    if lol_btn.clicked() {
                        self.request_launch(ui.ctx(), "lol");
                    }
                    if !lol_ok {
                        lol_btn.on_hover_text(t.game_not_installed);
                    }
                });
            });
        });
    }

    fn action_bar(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        let count = self.accounts.len();
        let count_label = t.acc_count(self.lang, count);
        ui.horizontal(|ui| {
            ui.label(RichText::new(t.saved_accounts).size(16.0).strong().color(ACCENT));
            ui.label(RichText::new(count_label).size(12.0).color(DIM));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button(RichText::new(t.btn_kill_riot).size(13.0)).clicked() {
                    bg(&self.tx, || Bg::Kill(core::kill_riot_processes()));
                }
                if ui.button(RichText::new(t.btn_storage).size(13.0)).clicked() {
                    self.open_storage(ui.ctx());
                }
                if ui.button(RichText::new(t.btn_new_session).size(13.0)).clicked() {
                    self.new_started = false;
                    self.modal = Modal::NewSession;
                }
                if ui.add(primary_button(t.btn_capture)).clicked() {
                    self.open_capture(ui.ctx());
                }
            });
        });
    }

    fn account_grid(&mut self, ui: &mut egui::Ui) {
        let t = self.t();
        if self.accounts.is_empty() {
            let frame = egui::Frame::none()
                .fill(BG_CARD)
                .rounding(Rounding::same(12.0))
                .inner_margin(egui::Margin::same(28.0));
            frame.show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    ui.label(RichText::new(t.empty_title).size(18.0).color(DIM));
                    ui.add_space(6.0);
                    ui.label(RichText::new(t.empty_body).size(13.0).color(DIM));
                    ui.add_space(14.0);
                    if ui.add(primary_button(t.btn_capture)).clicked() {
                        self.open_capture(ui.ctx());
                    }
                    ui.add_space(20.0);
                });
            });
            return;
        }

        let accounts = self.accounts.clone();
        ui.horizontal_wrapped(|ui| {
            for acc in &accounts {
                self.account_card(ui, acc);
            }
        });
    }

    fn account_card(&mut self, ui: &mut egui::Ui, acc: &Account) {
        let t = self.t();
        let active = self.is_active(acc);
        let now = core::now_ts();
        let time_label = t.time_ago(self.lang, Some(acc.last_used), now);
        let av_id = if acc.avatar.is_empty() {
            "jinx".to_string()
        } else {
            acc.avatar.clone()
        };

        let stroke = if active {
            egui::Stroke::new(1.5_f32, ACCENT2)
        } else {
            egui::Stroke::new(1.0_f32, Color32::from_rgb(30, 45, 70))
        };

        let mut switch_clicked = false;
        let mut edit_clicked = false;
        let mut ctx_menu: Option<&'static str> = None;

        let frame = egui::Frame::none()
            .fill(if active { BG_CARD_HOVER } else { BG_CARD })
            .rounding(Rounding::same(12.0))
            .inner_margin(egui::Margin::same(14.0))
            .stroke(stroke);

        let card = frame.show(ui, |ui| {
            ui.set_width(330.0);
            ui.horizontal(|ui| {
                // Avatar (bấm để sửa)
                let resp = if let Some(tex) = self.avatar_tex(&av_id) {
                    ui.add(
                        egui::Image::new(&tex)
                            .fit_to_exact_size(Vec2::splat(56.0))
                            .rounding(Rounding::same(28.0))
                            .sense(egui::Sense::click()),
                    )
                } else {
                    ui.allocate_exact_size(Vec2::splat(56.0), egui::Sense::click()).1
                };
                if resp.clicked() {
                    edit_clicked = true;
                }

                ui.add_space(10.0);
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&acc.name).size(16.0).strong().color(Color32::WHITE));
                        if !acc.tag.is_empty() {
                            ui.label(RichText::new(format!("#{}", acc.tag)).size(13.0).color(DIM));
                        }
                    });
                    ui.horizontal(|ui| {
                        badge(ui, &acc.region);
                        badge(ui, &time_label);
                    });
                    let note = if acc.note.is_empty() {
                        t.default_note
                    } else {
                        &acc.note
                    };
                    ui.label(RichText::new(note).size(12.0).color(DIM));
                });
            });

            ui.add_space(8.0);
            if active {
                ui.vertical_centered(|ui| {
                    ui.label(RichText::new(t.badge_active).size(13.0).strong().color(ACCENT2));
                });
            } else if ui
                .add_sized(
                    [ui.available_width(), 32.0],
                    egui::Button::new(
                        RichText::new(t.btn_switch)
                            .size(13.0)
                            .strong()
                            .color(BG_BASE),
                    )
                    .fill(ACCENT)
                    .rounding(Rounding::same(8.0)),
                )
                .clicked()
            {
                switch_clicked = true;
            }
        });

        // Menu ngữ cảnh (chuột phải)
        card.response.context_menu(|ui| {
            if ui.button(t.ctx_switch_lol).clicked() {
                ctx_menu = Some("lol");
                ui.close_menu();
            }
            if ui.button(t.ctx_switch_val).clicked() {
                ctx_menu = Some("val");
                ui.close_menu();
            }
            if !active && ui.button(t.ctx_switch).clicked() {
                ctx_menu = Some("none");
                ui.close_menu();
            }
            ui.separator();
            if ui.button(t.ctx_edit).clicked() {
                ctx_menu = Some("edit");
                ui.close_menu();
            }
            if ui.button(t.ctx_copy).clicked() {
                ctx_menu = Some("copy");
                ui.close_menu();
            }
            ui.separator();
            if ui.button(t.ctx_delete).clicked() {
                ctx_menu = Some("delete");
                ui.close_menu();
            }
        });

        if edit_clicked {
            self.open_edit(&acc.id);
        }
        if switch_clicked {
            self.request_switch(ui.ctx(), acc.id.clone(), "none");
        }
        match ctx_menu {
            Some("lol") => self.request_switch(ui.ctx(), acc.id.clone(), "lol"),
            Some("val") => self.request_switch(ui.ctx(), acc.id.clone(), "valorant"),
            Some("none") => self.request_switch(ui.ctx(), acc.id.clone(), "none"),
            Some("edit") => self.open_edit(&acc.id),
            Some("copy") => {
                let text = acc.riot_id();
                ui.output_mut(|o| o.copied_text = text.clone());
                let msg = format!("{}{}", self.t().copy_toast_prefix, text);
                self.toast(ui.ctx(), msg, ToastKind::Success);
            }
            Some("delete") => self.open_delete(&acc.id),
            _ => {}
        }
    }

    // -- Modals --------------------------------------------------------------

    fn render_modals(&mut self, ctx: &egui::Context) {
        if self.modal == Modal::None {
            return;
        }
        // Lớp mờ nền
        egui::Area::new(egui::Id::new("modal_dim"))
            .fixed_pos(egui::Pos2::ZERO)
            .order(egui::Order::Middle)
            .show(ctx, |ui| {
                ui.painter()
                    .rect_filled(ctx.screen_rect(), 0.0, Color32::from_black_alpha(170));
            });

        match self.modal {
            Modal::Capture => self.modal_capture(ctx),
            Modal::NewSession => self.modal_new_session(ctx),
            Modal::MatchWarning => self.modal_match_warning(ctx),
            Modal::Delete => self.modal_delete(ctx),
            Modal::Edit => self.modal_edit(ctx),
            Modal::Storage => self.modal_storage(ctx),
            Modal::None => {}
        }
    }

    fn modal_window<'a>(&self, ctx: &'a egui::Context, id: &str, title: &str) -> egui::Window<'a> {
        egui::Window::new(title)
            .id(egui::Id::new(id))
            .collapsible(false)
            .resizable(false)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .default_width(460.0)
            .frame(
                egui::Frame::window(&ctx.style())
                    .fill(BG_SURFACE)
                    .rounding(Rounding::same(14.0))
                    .inner_margin(egui::Margin::same(20.0)),
            )
    }

    fn modal_capture(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let mut open = true;
        let mut submit = false;
        let mut cancel = false;

        self.modal_window(ctx, "capture", t.modal_capture_title)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(RichText::new(t.modal_capture_desc).size(12.0).color(DIM));
                ui.add_space(10.0);
                ui.label(RichText::new(t.label_display_name).size(12.0).color(ACCENT));
                ui.add(
                    egui::TextEdit::singleline(&mut self.cap_name)
                        .hint_text(t.ph_capture_name)
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(t.label_note).size(12.0).color(ACCENT));
                ui.add(
                    egui::TextEdit::singleline(&mut self.cap_note)
                        .hint_text(t.ph_capture_note)
                        .desired_width(f32::INFINITY),
                );
                ui.add_space(10.0);
                ui.label(RichText::new(t.label_avatar).size(12.0).color(ACCENT));
                avatar_grid(ui, &mut self.cap_avatar, &self.textures);

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(t.btn_cancel).clicked() {
                        cancel = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(primary_button(t.btn_confirm_save)).clicked() {
                            submit = true;
                        }
                    });
                });
            });

        if cancel || !open {
            self.modal = Modal::None;
        }
        if submit {
            let mut name = None;
            let mut tag = None;
            let raw = self.cap_name.trim().to_string();
            if let Some((n, tg)) = raw.split_once('#') {
                name = Some(n.trim().to_string());
                tag = Some(tg.trim().to_string());
            } else if !raw.is_empty() {
                name = Some(raw);
            }
            let note = self.cap_note.trim().to_string();
            let avatar = self.cap_avatar.clone();
            let theme = "lol".to_string();
            bg(&self.tx, move || {
                Bg::Capture(core::capture_current_session(name, tag, &note, &avatar, &theme))
            });
        }
    }

    fn modal_new_session(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let mut open = true;
        let mut start = false;
        let mut save = false;

        self.modal_window(ctx, "new_session", t.modal_new_title)
            .open(&mut open)
            .show(ctx, |ui| {
                for (i, step) in [t.step1, t.step2, t.step3].iter().enumerate() {
                    ui.label(
                        RichText::new(format!("{} {}: {}", t.step_prefix, i + 1, step)).size(12.5),
                    );
                    ui.add_space(4.0);
                }
                if self.new_started {
                    ui.add_space(8.0);
                    ui.label(RichText::new(t.waiting_login).size(12.0).color(ACCENT2));
                }
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(t.btn_close).clicked() {
                        self.modal = Modal::None;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.new_started {
                            if ui.add(primary_button(t.btn_save_new)).clicked() {
                                save = true;
                            }
                        } else if ui.add(primary_button(t.btn_start_login)).clicked() {
                            start = true;
                        }
                    });
                });
            });

        if !open {
            self.modal = Modal::None;
        }
        if start {
            let m = "Đang mở Riot Client để đăng nhập mới...".to_string();
            self.toast(ctx, m, ToastKind::Info);
            bg(&self.tx, || Bg::NewSession(core::prepare_new_login_session()));
        }
        if save {
            self.open_capture(ctx);
        }
    }

    fn modal_match_warning(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let mut force = false;
        let mut back = false;
        let pending = self.pending.clone();

        self.modal_window(ctx, "match_warning", t.modal_match_title)
            .show(ctx, |ui| {
                ui.label(RichText::new(t.modal_match_body1).size(13.0));
                ui.add_space(6.0);
                ui.label(RichText::new(t.modal_match_body2).size(13.0).color(RED));
                ui.add_space(6.0);
                ui.label(RichText::new(t.modal_match_body3).size(13.0));
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(t.btn_back_match).clicked() {
                        back = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(t.btn_force_switch).strong().color(Color32::WHITE),
                                )
                                .fill(RED)
                                .rounding(Rounding::same(8.0)),
                            )
                            .clicked()
                        {
                            force = true;
                        }
                    });
                });
            });

        if back {
            self.modal = Modal::None;
            self.pending = None;
        }
        if force {
            self.modal = Modal::None;
            if let Some(action) = pending {
                match action {
                    PendingAction::Launch(p) => self.execute_launch(ctx, p, true),
                    PendingAction::Switch(id, mode) => self.execute_switch(ctx, id, mode, true),
                }
            }
            self.pending = None;
        }
    }

    fn modal_delete(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let mut confirm = false;
        let mut cancel = false;
        let id = self.del_id.clone();
        let label = self.del_label.clone();

        self.modal_window(ctx, "delete", t.modal_delete_title)
            .show(ctx, |ui| {
                ui.label(
                    RichText::new(format!("{} {} {}", t.modal_delete_body1, label, t.modal_delete_body2))
                        .size(13.0),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(t.modal_delete_sub).size(12.0).color(DIM));
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(t.btn_cancel).clicked() {
                        cancel = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui
                            .add(
                                egui::Button::new(
                                    RichText::new(t.btn_confirm_delete).strong().color(Color32::WHITE),
                                )
                                .fill(RED)
                                .rounding(Rounding::same(8.0)),
                            )
                            .clicked()
                        {
                            confirm = true;
                        }
                    });
                });
            });

        if cancel {
            self.modal = Modal::None;
        }
        if confirm {
            let id = id.clone();
            bg(&self.tx, move || {
                core::delete_account(&id);
                Bg::Delete
            });
        }
    }

    fn modal_edit(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let mut save = false;
        let mut cancel = false;

        self.modal_window(ctx, "edit", t.modal_edit_title)
            .show(ctx, |ui| {
                ui.label(RichText::new(t.label_display_name).size(12.0).color(ACCENT));
                ui.add(
                    egui::TextEdit::singleline(&mut self.edit_name).desired_width(f32::INFINITY),
                );
                ui.add_space(8.0);
                ui.label(RichText::new(t.label_tagline).size(12.0).color(ACCENT));
                ui.add(egui::TextEdit::singleline(&mut self.edit_tag).desired_width(f32::INFINITY));
                ui.add_space(8.0);
                ui.label(RichText::new(t.label_note).size(12.0).color(ACCENT));
                ui.add(egui::TextEdit::singleline(&mut self.edit_note).desired_width(f32::INFINITY));
                ui.add_space(10.0);
                ui.label(RichText::new(t.label_avatar).size(12.0).color(ACCENT));
                avatar_grid(ui, &mut self.edit_avatar, &self.textures);

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(t.btn_cancel).clicked() {
                        cancel = true;
                    }
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.add(primary_button(t.btn_save_changes)).clicked() {
                            save = true;
                        }
                    });
                });
            });

        if cancel {
            self.modal = Modal::None;
        }
        if save {
            if self.edit_name.trim().is_empty() {
                let m = if self.lang == Lang::En {
                    "Please enter account name".to_string()
                } else {
                    "Vui lòng nhập tên tài khoản".to_string()
                };
                self.toast(ctx, m, ToastKind::Error);
            } else {
                let id = self.edit_id.clone();
                let name = self.edit_name.trim().to_string();
                let tag = self.edit_tag.trim().to_string();
                let note = self.edit_note.trim().to_string();
                let avatar = self.edit_avatar.clone();
                bg(&self.tx, move || {
                    let ok = core::update_account(
                        &id,
                        Some(name),
                        Some(tag),
                        Some(note),
                        Some(avatar),
                        None,
                    );
                    Bg::Update(ok)
                });
            }
        }
    }

    fn modal_storage(&mut self, ctx: &egui::Context) {
        let t = self.t();
        let mut open = true;
        let mut prune = false;
        let mut clean_lock = false;

        self.modal_window(ctx, "storage", t.modal_storage_title)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.set_width(520.0);
                ui.label(RichText::new(t.modal_storage_desc).size(12.0).color(DIM));
                ui.add_space(10.0);

                if self.storage_loading {
                    ui.label(RichText::new(t.storage_loading).size(13.0).color(ACCENT2));
                } else if let Some(rep) = &self.storage {
                    ui.horizontal(|ui| {
                        stat_box(ui, t.storage_total, &rep.total_human, ACCENT);
                        stat_box(ui, t.storage_junk, &rep.junk_human, RED);
                        stat_box(ui, t.storage_backup, &rep.backup_human, ACCENT2);
                    });
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .max_height(220.0)
                        .show(ui, |ui| {
                            for it in &rep.items {
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(if it.name.is_empty() {
                                            &it.id
                                        } else {
                                            &it.name
                                        })
                                        .size(13.0),
                                    );
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if it.junk_bytes > 0 {
                                            ui.label(
                                                RichText::new(
                                                    t.storage_item_junk(self.lang, &it.junk_human),
                                                )
                                                .size(12.0)
                                                .color(RED),
                                            );
                                        }
                                        ui.label(
                                            RichText::new(&it.size_human).size(12.0).color(DIM),
                                        );
                                    });
                                });
                                ui.separator();
                            }
                        });

                    ui.add_space(6.0);
                    if rep.junk_bytes == 0 {
                        ui.label(RichText::new(t.storage_none).size(13.0).color(GREEN));
                    }
                }

                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if ui.button(t.btn_close).clicked() {
                        self.modal = Modal::None;
                    }
                    if ui.button(t.btn_clean_lockfile).clicked() {
                        clean_lock = true;
                    }
                    let has_junk = self
                        .storage
                        .as_ref()
                        .map(|r| r.junk_bytes > 0)
                        .unwrap_or(false);
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if has_junk && ui.add(primary_button(t.btn_prune_all)).clicked() {
                            prune = true;
                        }
                    });
                });
            });

        if !open {
            self.modal = Modal::None;
        }
        if prune {
            let m = self.t().toast_pruning.to_string();
            self.toast(ctx, m, ToastKind::Info);
            bg(&self.tx, || {
                let (bytes, files) = core::prune_storage(None, true);
                Bg::Prune(bytes, files)
            });
        }
        if clean_lock {
            bg(&self.tx, || Bg::CleanLock(core::clean_live_lockfile()));
        }
    }

    fn render_toasts(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        self.toasts.retain(|t| now - t.born < 3.5);
        if self.toasts.is_empty() {
            return;
        }
        egui::Area::new(egui::Id::new("toasts"))
            .anchor(Align2::RIGHT_BOTTOM, [-16.0, -16.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.with_layout(Layout::bottom_up(Align::RIGHT), |ui| {
                    for t in self.toasts.iter().rev() {
                        let color = match t.kind {
                            ToastKind::Info => ACCENT2,
                            ToastKind::Success => GREEN,
                            ToastKind::Error => RED,
                        };
                        egui::Frame::none()
                            .fill(BG_SURFACE)
                            .rounding(Rounding::same(8.0))
                            .inner_margin(egui::Margin::symmetric(14.0, 10.0))
                            .stroke(egui::Stroke::new(1.0_f32, color))
                            .show(ui, |ui| {
                                ui.label(RichText::new(&t.msg).size(13.0).color(Color32::WHITE));
                            });
                        ui.add_space(6.0);
                    }
                });
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ensure_textures(ctx);
        self.poll_bg(ctx);
        ctx.request_repaint_after(Duration::from_millis(200));

        egui::TopBottomPanel::top("header")
            .frame(
                egui::Frame::none()
                    .fill(BG_SURFACE)
                    .inner_margin(egui::Margin::symmetric(16.0, 10.0)),
            )
            .show(ctx, |ui| self.header(ui));

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(BG_BASE).inner_margin(egui::Margin::same(16.0)))
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.hero(ui);
                    ui.add_space(14.0);
                    self.action_bar(ui);
                    ui.add_space(10.0);
                    self.account_grid(ui);
                    ui.add_space(20.0);
                });
            });

        self.render_modals(ctx);
        self.render_toasts(ctx);
    }
}

// ---------------------------------------------------------------------------
// Tiện ích giao diện
// ---------------------------------------------------------------------------

fn primary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).size(13.0).strong().color(BG_BASE))
        .fill(ACCENT)
        .rounding(Rounding::same(8.0))
}

fn secondary_button(text: &str) -> egui::Button<'static> {
    egui::Button::new(RichText::new(text.to_string()).size(13.0).color(ACCENT2))
        .fill(BG_CARD)
        .rounding(Rounding::same(8.0))
        .stroke(egui::Stroke::new(1.0_f32, ACCENT2))
}

fn badge(ui: &mut egui::Ui, text: &str) {
    egui::Frame::none()
        .fill(Color32::from_rgb(20, 35, 60))
        .rounding(Rounding::same(6.0))
        .inner_margin(egui::Margin::symmetric(7.0, 2.0))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(11.0).color(ACCENT));
        });
}

fn stat_box(ui: &mut egui::Ui, label: &str, value: &str, color: Color32) {
    egui::Frame::none()
        .fill(BG_CARD)
        .rounding(Rounding::same(8.0))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.set_min_width(140.0);
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(11.0).color(DIM));
                ui.label(RichText::new(value).size(16.0).strong().color(color));
            });
        });
}

/// Lưới chọn avatar (chọn id vào `selected`).
fn avatar_grid(
    ui: &mut egui::Ui,
    selected: &mut String,
    textures: &HashMap<String, egui::TextureHandle>,
) {
    egui::ScrollArea::vertical()
        .id_salt("avatar_grid")
        .max_height(170.0)
        .show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                for av in avatars::AVATARS.iter() {
                    let is_sel = selected == av.id;
                    let border = if is_sel { ACCENT2 } else { Color32::TRANSPARENT };
                    let frame = egui::Frame::none()
                        .stroke(egui::Stroke::new(2.0_f32, border))
                        .rounding(Rounding::same(24.0))
                        .inner_margin(egui::Margin::same(2.0));
                    let resp = frame
                        .show(ui, |ui| {
                            if let Some(tex) = textures.get(av.id) {
                                ui.add(
                                    egui::Image::new(tex)
                                        .fit_to_exact_size(Vec2::splat(44.0))
                                        .rounding(Rounding::same(22.0)),
                                )
                                .on_hover_text(format!("{} ({})", av.name, av.game))
                            } else {
                                ui.allocate_exact_size(Vec2::splat(44.0), egui::Sense::hover()).1
                            }
                        })
                        .response;
                    if resp.interact(egui::Sense::click()).clicked() {
                        *selected = av.id.to_string();
                    }
                }
            });
        });
}

fn load_avatar_texture(ctx: &egui::Context, id: &str) -> Option<egui::TextureHandle> {
    // Chỉ nạp texture cho avatar có trong danh mục (tránh path tùy ý từ dữ liệu lưu).
    avatars::find(id)?;
    let path = avatars::avatar_path(id);
    let img = image::open(&path).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let color = egui::ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
    Some(ctx.load_texture(id, color, egui::TextureOptions::LINEAR))
}

fn apply_visuals(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = BG_BASE;
    visuals.window_fill = BG_SURFACE;
    visuals.override_text_color = Some(Color32::from_rgb(220, 225, 235));
    visuals.widgets.noninteractive.bg_fill = BG_SURFACE;
    visuals.widgets.inactive.bg_fill = Color32::from_rgb(20, 35, 60);
    visuals.widgets.hovered.bg_fill = Color32::from_rgb(30, 50, 80);
    visuals.widgets.active.bg_fill = Color32::from_rgb(40, 60, 95);
    visuals.selection.bg_fill = ACCENT2;
    ctx.set_visuals(visuals);
}

fn settings_path() -> std::path::PathBuf {
    core::settings_file()
}

fn load_lang() -> Lang {
    if let Ok(text) = std::fs::read_to_string(settings_path()) {
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            if let Some(code) = v.get("lang").and_then(|x| x.as_str()) {
                return Lang::from_code(code);
            }
        }
    }
    Lang::Vi
}

fn save_lang(lang: Lang) {
    let _ = std::fs::create_dir_all(core::data_dir());
    let v = serde_json::json!({ "lang": lang.code() });
    let _ = std::fs::write(settings_path(), v.to_string());
}
