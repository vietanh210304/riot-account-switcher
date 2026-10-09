//! Riot Account Switcher — bản viết lại bằng Rust (egui native UI).
//!
//! Chỉ giữ tính năng chuyển tài khoản Riot (LoL & Valorant); đã bỏ hoàn toàn
//! tính năng import từ TcNo Account Switcher.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod avatars;
mod core;
mod i18n;

use std::path::PathBuf;

/// Thư mục chứa tài nguyên tĩnh (avatars/, icon...).
pub fn assets_dir() -> PathBuf {
    let mut candidates: Vec<PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            candidates.push(dir.join("assets"));
            candidates.push(dir.join("web").join("assets"));
        }
    }
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("web").join("assets"));
    candidates.push(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"));
    for c in candidates {
        if c.exists() {
            return c;
        }
    }
    PathBuf::from("assets")
}

fn main() -> eframe::Result<()> {
    let _ = core::ensure_dirs();

    let icon = load_icon();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1180.0, 780.0])
            .with_min_inner_size([960.0, 640.0])
            .with_title("Riot Account Switcher - LoL & Valorant")
            .with_icon(icon),
        ..Default::default()
    };

    eframe::run_native(
        "Riot Account Switcher - LoL & Valorant",
        native_options,
        Box::new(|cc| Ok(Box::new(app::App::new(cc)))),
    )
}

/// Nạp icon cửa sổ từ assets/icon.ico hoặc web/assets/icon.png.
fn load_icon() -> egui::IconData {
    let dir = assets_dir();
    for name in ["icon.ico", "icon.png"] {
        let path = dir.join(name);
        if let Ok(img) = image::open(&path) {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            return egui::IconData {
                rgba: rgba.into_raw(),
                width: w,
                height: h,
            };
        }
    }
    egui::IconData::default()
}
