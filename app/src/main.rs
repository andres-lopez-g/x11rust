#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

mod app;
mod art;
mod hardware;
mod model;
mod theme;

use app::{Tab, X11App};
use eframe::egui;
use std::path::PathBuf;
use std::sync::Arc;

fn rounded(x: i32, y: i32, x0: i32, y0: i32, x1: i32, y1: i32, r: i32) -> bool {
    if x < x0 || x > x1 || y < y0 || y > y1 {
        return false;
    }
    let cx = if x < x0 + r {
        x0 + r
    } else if x > x1 - r {
        x1 - r
    } else {
        x
    };
    let cy = if y < y0 + r {
        y0 + r
    } else if y > y1 - r {
        y1 - r
    } else {
        y
    };
    let dx = x - cx;
    let dy = y - cy;
    dx * dx + dy * dy <= r * r
}

fn app_icon() -> egui::IconData {
    let size = 64usize;
    let mut rgba = vec![0u8; size * size * 4];
    for y in 0..size {
        for x in 0..size {
            let (xi, yi) = (x as i32, y as i32);
            let mut color = [0u8, 0, 0, 0];
            if rounded(xi, yi, 1, 1, 62, 62, 14) {
                color = [16, 21, 29, 255];
            }
            if rounded(xi, yi, 22, 12, 42, 54, 10) {
                color = [242, 246, 250, 255];
            }
            if rounded(xi, yi, 28, 18, 36, 30, 4) {
                color = [26, 33, 43, 255];
            }
            if rounded(xi, yi, 26, 34, 38, 36, 1) {
                color = [255, 92, 77, 255];
            }
            let i = (y * size + x) * 4;
            rgba[i..i + 4].copy_from_slice(&color);
        }
    }
    egui::IconData {
        rgba,
        width: size as u32,
        height: size as u32,
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let mut shot: Option<PathBuf> = None;
    let mut initial_tab = Tab::Overview;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--screenshot" && i + 1 < args.len() {
            shot = Some(PathBuf::from(&args[i + 1]));
            i += 2;
        } else if let Some(rest) = args[i].strip_prefix("--screenshot=") {
            shot = Some(PathBuf::from(rest));
            i += 1;
        } else if args[i] == "--tab" && i + 1 < args.len() {
            if let Some(tab) = Tab::from_arg(&args[i + 1]) {
                initial_tab = tab;
            }
            i += 2;
        } else if let Some(rest) = args[i].strip_prefix("--tab=") {
            if let Some(tab) = Tab::from_arg(rest) {
                initial_tab = tab;
            }
            i += 1;
        } else {
            i += 1;
        }
    }

    let screenshot_mode = shot.is_some();
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([1180.0, 780.0])
        .with_min_inner_size([1100.0, 760.0])
        .with_title("X11 Studio")
        .with_app_id("x11-studio")
        .with_icon(Arc::new(app_icon()));
    if screenshot_mode {
        viewport = viewport.with_active(false);
        #[cfg(target_os = "windows")]
        {
            viewport = viewport.with_window_level(egui::WindowLevel::AlwaysOnBottom);
        }
    }

    let mut options = eframe::NativeOptions {
        viewport,
        centered: true,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    // On macOS a screenshot run must not activate the app or steal focus.
    #[cfg(target_os = "macos")]
    if screenshot_mode {
        use winit::platform::macos::EventLoopBuilderExtMacOS;
        options.event_loop_builder = Some(Box::new(|builder| {
            builder.with_activate_ignoring_other_apps(false);
        }));
    }

    let shot_for_app = shot.clone();
    eframe::run_native(
        "X11 Studio",
        options,
        Box::new(move |cc| Ok(Box::new(X11App::new(cc, shot_for_app.clone(), initial_tab)))),
    )
}
