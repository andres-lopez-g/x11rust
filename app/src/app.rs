use crate::art::{MouseArt, mouse_art};
use crate::hardware::X11Hardware;
use crate::model::*;
use crate::theme::{
    self, Theme, danger_button, ghost_button, icon_button, mix, pill, primary_button,
    secondary_button, section_heading, status_dot, toggle, with_alpha,
};
use eframe::egui::{
    self, Align, Align2, Color32, ComboBox, CornerRadius, FontId, Frame, Id, Layout, Margin, Rect,
    RichText, ScrollArea, Sense, Stroke, StrokeKind, TextEdit, Ui, Vec2, pos2, vec2,
};
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Tab {
    Overview,
    Buttons,
    Dpi,
    Lighting,
    Performance,
    Power,
    Macros,
    Profiles,
    Settings,
}

impl Tab {
    pub const ALL: [Tab; 9] = [
        Tab::Overview,
        Tab::Buttons,
        Tab::Dpi,
        Tab::Lighting,
        Tab::Performance,
        Tab::Power,
        Tab::Macros,
        Tab::Profiles,
        Tab::Settings,
    ];

    pub fn label(self, lang: Lang) -> &'static str {
        match self {
            Self::Overview => lang.t("Resumen", "Overview"),
            Self::Buttons => lang.t("Botones", "Buttons"),
            Self::Dpi => "DPI",
            Self::Lighting => lang.t("Iluminación", "Lighting"),
            Self::Performance => lang.t("Rendimiento", "Performance"),
            Self::Power => lang.t("Energía", "Power"),
            Self::Macros => lang.t("Macros", "Macros"),
            Self::Profiles => lang.t("Perfiles", "Profiles"),
            Self::Settings => lang.t("Ajustes", "Settings"),
        }
    }

    pub fn subtitle(self, lang: Lang) -> &'static str {
        match self {
            Self::Overview => lang.t(
                "Todo tu X11, de un vistazo y sin rodeos.",
                "Your X11 at a glance, without the clutter.",
            ),
            Self::Buttons => lang.t(
                "Asigna la acción perfecta a cada botón.",
                "Give every button the action that fits.",
            ),
            Self::Dpi => lang.t(
                "Ocho etapas, seis activas, de 50 a 22 000 DPI.",
                "Eight stages, six active, from 50 to 22,000 DPI.",
            ),
            Self::Lighting => lang.t(
                "Color, modo y ritmo con vista previa en vivo.",
                "Color, mode and rhythm with a live preview.",
            ),
            Self::Performance => lang.t(
                "Sondeo, respuesta y precisión, en tus manos.",
                "Polling, response and precision in your hands.",
            ),
            Self::Power => lang.t(
                "Cuida la batería sin perder velocidad.",
                "Take care of the battery without losing speed.",
            ),
            Self::Macros => lang.t(
                "Automatiza secuencias en segundos.",
                "Automate sequences in seconds.",
            ),
            Self::Profiles => lang.t(
                "Una configuración lista para cada juego.",
                "A ready setup for every game.",
            ),
            Self::Settings => lang.t(
                "Haz que la app se sienta tuya.",
                "Make the app feel like yours.",
            ),
        }
    }

    pub fn from_arg(value: &str) -> Option<Tab> {
        match value.to_ascii_lowercase().as_str() {
            "overview" | "resumen" => Some(Tab::Overview),
            "buttons" | "botones" => Some(Tab::Buttons),
            "dpi" => Some(Tab::Dpi),
            "lighting" | "iluminacion" | "iluminación" => Some(Tab::Lighting),
            "performance" | "rendimiento" => Some(Tab::Performance),
            "power" | "energia" | "energía" => Some(Tab::Power),
            "macros" => Some(Tab::Macros),
            "profiles" | "perfiles" => Some(Tab::Profiles),
            "settings" | "ajustes" => Some(Tab::Settings),
            _ => None,
        }
    }

    pub fn badge(self, lang: Lang) -> &'static str {
        match self {
            Self::Overview => "",
            Self::Buttons => lang.t("5 botones", "5 buttons"),
            Self::Dpi => lang.t("8 ranuras · 6 activas", "8 slots · 6 active"),
            Self::Lighting => lang.t("7 modos", "7 modes"),
            Self::Performance => "1000 Hz",
            Self::Power => lang.t("Batería", "Battery"),
            Self::Macros => lang.t("Secuencias", "Sequences"),
            Self::Profiles => lang.t("3 perfiles", "3 profiles"),
            Self::Settings => lang.t("Local", "Local"),
        }
    }
}

struct Toast {
    text: String,
    tone: ToastTone,
    born: f64,
}

#[derive(Clone, Copy)]
enum ToastTone {
    Success,
    Warning,
    Info,
}

struct CommandPicker {
    button: usize,
    search: String,
    category: Option<CmdCategory>,
}

struct ShortcutDraft {
    button: usize,
    ctrl: bool,
    shift: bool,
    alt: bool,
    win: bool,
    key: String,
}

struct RenameState {
    index: usize,
    name: String,
}

struct MacroRecording {
    macro_index: usize,
    last_time: f64,
}

pub struct X11App {
    pub ledger: Ledger,
    saved: Ledger,
    pub tab: Tab,
    selected_button: usize,
    selected_dpi: usize,
    selected_macro: usize,
    picker: Option<CommandPicker>,
    shortcut_editor: Option<ShortcutDraft>,
    rename_profile: Option<RenameState>,
    delete_profile: Option<usize>,
    delete_macro: Option<usize>,
    reset_confirm: bool,
    toast: Option<Toast>,
    recording: Option<MacroRecording>,
    frames: u32,
    shot: Option<PathBuf>,
    shot_requested: bool,
    shot_done: Arc<AtomicBool>,
    screenshot_mode: bool,
    started: f64,
    last_accent: Accent,
    exit_frames: u32,
    hardware: X11Hardware,
}

impl X11App {
    pub fn new(cc: &eframe::CreationContext<'_>, shot: Option<PathBuf>, initial_tab: Tab) -> Self {
        let screenshot_mode = shot.is_some();
        let ledger = if screenshot_mode {
            Ledger::defaults(Lang::Es)
        } else {
            Ledger::load_or_default()
        };
        theme::install_fonts(&cc.egui_ctx);
        cc.egui_ctx.set_theme(egui::Theme::Dark);
        let theme = Theme::new(ledger.settings.accent);
        theme::install_style(&cc.egui_ctx, &theme);
        if screenshot_mode {
            cc.egui_ctx.all_styles_mut(|style| {
                style.interaction.tooltip_delay = 9_999.0;
            });
        }
        let mut hardware = X11Hardware::new();
        if !screenshot_mode {
            hardware.connect();
        }
        Self {
            saved: ledger.clone(),
            last_accent: ledger.settings.accent,
            ledger,
            tab: initial_tab,
            selected_button: 0,
            selected_dpi: 0,
            selected_macro: 0,
            picker: None,
            shortcut_editor: None,
            rename_profile: None,
            delete_profile: None,
            delete_macro: None,
            reset_confirm: false,
            toast: None,
            recording: None,
            frames: 0,
            shot,
            shot_requested: false,
            shot_done: Arc::new(AtomicBool::new(false)),
            screenshot_mode,
            started: 0.0,
            exit_frames: 0,
            hardware,
        }
    }

    fn lang(&self) -> Lang {
        self.ledger.settings.lang
    }

    fn dirty(&self) -> bool {
        self.ledger.profiles != self.saved.profiles || self.ledger.macros != self.saved.macros
    }

    fn notify(&mut self, text: impl Into<String>, tone: ToastTone, time: f64) {
        self.toast = Some(Toast {
            text: text.into(),
            tone,
            born: time,
        });
    }

    fn apply(&mut self, time: f64) {
        let lang = self.lang();
        self.ledger.sanitize();
        self.saved = self.ledger.clone();
        let saved = self.ledger.save();
        if self.hardware.connected() {
            let profile = self.ledger.profile().clone();
            match self.hardware.apply_profile(&profile) {
                Ok(()) => self.notify(
                    lang.t(
                        "Aplicado al X11. El ratón ya usa esta configuración.",
                        "Applied to the X11. The mouse is using this setup.",
                    ),
                    ToastTone::Success,
                    time,
                ),
                Err(err) => self.notify(
                    format!(
                        "{} ({err})",
                        lang.t(
                            "No se pudo enviar al ratón; se guardó en el equipo.",
                            "Could not send to the mouse; saved on this computer.",
                        )
                    ),
                    ToastTone::Warning,
                    time,
                ),
            }
        } else if saved.is_ok() {
            self.notify(
                lang.t(
                    "Guardado en el equipo. Conecta el X11 para aplicarlo al ratón.",
                    "Saved on this computer. Connect the X11 to apply it to the mouse.",
                ),
                ToastTone::Warning,
                time,
            );
        } else {
            self.notify(
                lang.t(
                    "Se aplicó en la vista previa, pero no se pudo guardar.",
                    "Applied to the preview, but saving failed.",
                ),
                ToastTone::Warning,
                time,
            );
        }
    }

    fn discard(&mut self, time: f64) {
        let lang = self.lang();
        self.ledger.profiles = self.saved.profiles.clone();
        self.ledger.macros = self.saved.macros.clone();
        self.notify(
            lang.t("Cambios descartados.", "Changes discarded."),
            ToastTone::Info,
            time,
        );
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context, time: f64) {
        if self.screenshot_mode {
            return;
        }
        let lang = self.lang();
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::S)) {
            self.apply(time);
        }
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            self.discard(time);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if self.recording.is_some() {
                self.recording = None;
                self.notify(
                    lang.t("Grabación detenida.", "Recording stopped."),
                    ToastTone::Info,
                    time,
                );
            } else if self.picker.is_some() {
                self.picker = None;
            } else if self.shortcut_editor.is_some() {
                self.shortcut_editor = None;
            }
        }
    }

    fn handle_recording(&mut self, ctx: &egui::Context, time: f64) {
        let Some(rec) = self.recording.as_mut() else {
            return;
        };
        let events = ctx.input(|i| i.events.clone());
        let mut additions: Vec<MacroAction> = Vec::new();
        let elapsed_ms = ((time - rec.last_time) * 1000.0).clamp(0.0, 5000.0) as u16;
        let mut had_key = false;
        let mut stop = false;
        for event in events {
            if let egui::Event::Key {
                key,
                pressed,
                repeat,
                ..
            } = event
            {
                if repeat {
                    continue;
                }
                if key == egui::Key::Escape {
                    stop = true;
                    continue;
                }
                had_key = true;
                let name = format!("{key:?}");
                additions.push(if pressed {
                    MacroAction::KeyDown(name)
                } else {
                    MacroAction::KeyUp(name)
                });
            }
        }
        if had_key && elapsed_ms > 8 {
            additions.insert(0, MacroAction::Delay(elapsed_ms.max(10)));
        }
        if !additions.is_empty() {
            if let Some(mac) = self.ledger.macros.get_mut(rec.macro_index) {
                mac.actions.extend(additions);
            }
            rec.last_time = time;
        }
        if stop {
            self.recording = None;
            let lang = self.lang();
            self.notify(
                lang.t("Grabación terminada.", "Recording finished."),
                ToastTone::Success,
                time,
            );
        }
    }

    fn handle_screenshot(&mut self, ctx: &egui::Context) {
        let Some(path) = self.shot.clone() else {
            return;
        };
        ctx.request_repaint();
        ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
        if self.frames >= 120 && !self.shot_requested {
            self.shot_requested = true;
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let done = self.shot_done.clone();
            ctx.request_screenshot(move |image| {
                let width = image.width() as u32;
                let height = image.height() as u32;
                if let Some(buffer) =
                    image::RgbaImage::from_raw(width, height, image.as_raw().to_vec())
                {
                    let _ = buffer.save_with_format(&path, image::ImageFormat::Png);
                }
                done.store(true, Ordering::SeqCst);
            });
        }
        if self.shot_done.load(Ordering::SeqCst) {
            self.exit_frames += 1;
            if self.exit_frames > 3 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        } else if self.frames > 220 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn draw_sidebar(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let rect = ui.max_rect();
        ui.painter()
            .rect_filled(rect, CornerRadius::ZERO, theme.sidebar);
        ui.painter().line_segment(
            [
                pos2(rect.right() - 0.5, rect.top()),
                pos2(rect.right() - 0.5, rect.bottom()),
            ],
            Stroke::new(1.0, theme.stroke_soft),
        );
        ui.add_space(2.0);

        // Brand mark.
        ui.horizontal(|ui| {
            let (mark_rect, _) = ui.allocate_exact_size(vec2(38.0, 38.0), Sense::hover());
            theme::app_mark(ui.painter(), mark_rect, theme);
            ui.add_space(4.0);
            ui.vertical(|ui| {
                ui.add_space(2.0);
                ui.label(
                    RichText::new("X11 Studio")
                        .size(16.0)
                        .color(theme.text)
                        .strong(),
                );
                ui.label(
                    RichText::new(lang.t("CENTRO DE CONTROL", "CONTROL CENTER"))
                        .size(8.0)
                        .color(theme.dim),
                );
            });
        });
        ui.add_space(20.0);
        ui.label(
            RichText::new(lang.t("CONFIGURACIÓN", "CONFIGURATION"))
                .size(8.5)
                .color(theme.dim),
        );
        ui.add_space(6.0);
        for tab in Tab::ALL {
            if self.nav_item(ui, theme, tab) {
                self.tab = tab;
            }
        }

        let remaining = ui.available_height();
        if remaining > 150.0 {
            ui.add_space(remaining - 150.0);
        }

        // Device mini card.
        let connected = self.hardware.connected();
        let device_state = if connected {
            lang.t("Conectado", "Connected")
        } else {
            lang.t("Vista previa", "Preview")
        };
        let state_color = if connected {
            theme.success
        } else {
            theme.warning
        };
        Frame::NONE
            .fill(theme.panel)
            .stroke(Stroke::new(1.0, theme.stroke_soft))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(12))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (icon_rect, _) = ui.allocate_exact_size(vec2(34.0, 34.0), Sense::hover());
                    ui.painter().rect_filled(
                        icon_rect,
                        CornerRadius::same(10),
                        with_alpha(theme.accent, 20),
                    );
                    ui.painter().text(
                        icon_rect.center(),
                        Align2::CENTER_CENTER,
                        "X11",
                        FontId::proportional(10.0),
                        theme.accent,
                    );
                    ui.vertical(|ui| {
                        ui.label(RichText::new("X11 Wireless").size(12.5).color(theme.text));
                        ui.horizontal(|ui| {
                            let (dot, _) = ui.allocate_exact_size(vec2(7.0, 7.0), Sense::hover());
                            ui.painter().circle_filled(dot.center(), 3.0, state_color);
                            ui.label(RichText::new(device_state).size(9.5).color(theme.muted));
                        });
                    });
                });
                ui.add_space(9.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(lang.t("Batería", "Battery"))
                            .size(9.5)
                            .color(theme.dim),
                    );
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(RichText::new("82 %").size(9.5).color(theme.text));
                    });
                });
                let (bar, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 5.0), Sense::hover());
                ui.painter().rect_filled(
                    bar,
                    CornerRadius::same(3),
                    Color32::from_rgb(0x26, 0x30, 0x3D),
                );
                ui.painter().rect_filled(
                    Rect::from_min_size(bar.min, vec2(bar.width() * 0.82, bar.height())),
                    CornerRadius::same(3),
                    theme.success,
                );
            });

        // Language switch.
        ui.add_space(10.0);
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(lang.t("Idioma", "Language"))
                    .size(10.0)
                    .color(theme.dim),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                for l in [Lang::En, Lang::Es] {
                    let selected = lang == l;
                    let text = if l == Lang::Es { "ES" } else { "EN" };
                    if choice_chip(ui, theme, text, selected).clicked() {
                        if self.ledger.settings.lang != l {
                            self.ledger.settings.lang = l;
                            let _ = self.ledger.save();
                        }
                    }
                }
            });
        });
        let _ = time;
    }

    fn nav_item(&mut self, ui: &mut Ui, theme: &Theme, tab: Tab) -> bool {
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::click());
        let active = self.tab == tab;
        let hover = response.hovered();
        let t = ui.ctx().animate_bool(response.id, hover || active);
        let fill = if active {
            with_alpha(theme.accent, 24)
        } else if hover {
            with_alpha(theme.text, 8)
        } else {
            Color32::TRANSPARENT
        };
        ui.painter().rect_filled(rect, CornerRadius::same(10), fill);
        if active {
            ui.painter().rect_filled(
                Rect::from_min_size(pos2(rect.left(), rect.center().y - 9.0), vec2(3.0, 18.0)),
                CornerRadius::same(2),
                theme.accent,
            );
        }
        let icon_color = if active {
            theme.accent
        } else {
            mix(theme.muted, theme.text, t * 0.4)
        };
        let icon_rect =
            Rect::from_center_size(pos2(rect.left() + 17.0, rect.center().y), vec2(18.0, 18.0));
        theme::paint_nav_icon(ui.painter(), icon_rect, tab, icon_color);
        ui.painter().text(
            pos2(rect.left() + 36.0, rect.center().y),
            Align2::LEFT_CENTER,
            tab.label(self.lang()),
            FontId::proportional(13.0),
            if active { theme.text } else { theme.muted },
        );
        if hover {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        response.clicked()
    }

    fn draw_status_bar(&mut self, ui: &mut Ui, theme: &Theme) {
        let lang = self.lang();
        let dirty = self.dirty();
        ui.horizontal(|ui| {
            status_dot(
                ui,
                if dirty { theme.warning } else { theme.success },
                if dirty {
                    lang.t(
                        "Cambios pendientes de aplicar",
                        "Changes waiting to be applied",
                    )
                } else {
                    lang.t("Todo sincronizado", "Everything in sync")
                },
                theme,
            );
            ui.label(RichText::new("·").size(11.0).color(theme.dim));
            ui.label(
                RichText::new(if self.hardware.connected() {
                    lang.t("X11 conectado por HID", "X11 connected over HID")
                } else {
                    lang.t(
                        "Los ajustes se guardan en este equipo",
                        "Settings are saved on this computer",
                    )
                })
                .size(10.0)
                .color(theme.dim),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(format!("X11 Studio {}", APP_VERSION))
                        .size(10.0)
                        .color(theme.dim),
                );
                ui.label(RichText::new("·").size(10.0).color(theme.dim));
                ui.label(
                    RichText::new(if self.hardware.connected() {
                        lang.t("Dispositivo activo", "Device active")
                    } else {
                        lang.t("Sin dispositivo", "No device")
                    })
                    .size(10.0)
                    .color(if self.hardware.connected() {
                        theme.success
                    } else {
                        theme.warning
                    }),
                );
            });
        });
    }

    fn draw_header(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let dirty = self.dirty();
        let profile_name = self.ledger.profile().name.clone();
        let profile_count = self.ledger.profiles.len();
        let mut requested: Option<usize> = None;
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(self.tab.label(lang))
                        .size(25.0)
                        .color(theme.text)
                        .strong(),
                );
                ui.label(
                    RichText::new(self.tab.subtitle(lang))
                        .size(12.0)
                        .color(theme.muted),
                );
            });
            let available = ui.available_width();
            ui.allocate_ui_with_layout(
                vec2(available, 38.0),
                Layout::right_to_left(Align::Center),
                |ui| {
                    if primary_button(ui, theme, lang.t("Aplicar cambios", "Apply changes"))
                        .clicked()
                    {
                        if dirty {
                            self.apply(time);
                        } else {
                            self.notify(
                                lang.t("Todo está al día.", "Everything is already up to date."),
                                ToastTone::Info,
                                time,
                            );
                        }
                    }
                    if dirty && ghost_button(ui, theme, lang.t("Descartar", "Discard")).clicked() {
                        self.discard(time);
                    }
                    ComboBox::from_id_salt("profile_switch")
                        .width(152.0)
                        .selected_text(
                            RichText::new(format!(
                                "{}  ●  {}",
                                lang.t("Perfil", "Profile"),
                                profile_name
                            ))
                            .size(11.5)
                            .color(theme.text),
                        )
                        .show_ui(ui, |ui| {
                            for i in 0..profile_count {
                                let name = self.ledger.profiles[i].name.clone();
                                if ui.selectable_label(i == self.ledger.active, name).clicked() {
                                    requested = Some(i);
                                }
                            }
                        });
                    if dirty {
                        pill(ui, theme, lang.t("SIN APLICAR", "UNSAVED"), theme.warning);
                    } else {
                        pill(ui, theme, lang.t("SINCRONIZADO", "IN SYNC"), theme.success);
                    }
                },
            );
        });
        if let Some(index) = requested {
            self.ledger.active = index;
            self.selected_button = 0;
            self.selected_dpi = 0;
        }
    }

    fn draw_main(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let dirty = self.dirty();
        egui::Panel::bottom("status_bar")
            .exact_size(32.0)
            .resizable(false)
            .frame(
                Frame::NONE
                    .fill(theme.sidebar)
                    .inner_margin(Margin::symmetric(16, 0)),
            )
            .show(ui, |ui| {
                self.draw_status_bar(ui, theme);
            });
        egui::Panel::left("sidebar")
            .exact_size(224.0)
            .resizable(false)
            .frame(
                Frame::NONE
                    .fill(theme.sidebar)
                    .inner_margin(Margin::symmetric(14, 16)),
            )
            .show(ui, |ui| {
                self.draw_sidebar(ui, theme, time);
            });
        egui::CentralPanel::default()
            .frame(
                Frame::NONE
                    .fill(theme.bg)
                    .inner_margin(Margin::symmetric(26, 18)),
            )
            .show(ui, |ui| {
                theme::background_gradient(ui, theme);
                self.draw_header(ui, theme, time);
                ui.add_space(14.0);
                ScrollArea::vertical()
                    .id_salt(("content_scroll", self.tab))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        match self.tab {
                            Tab::Overview => self.screen_overview(ui, theme, time),
                            Tab::Buttons => self.screen_buttons(ui, theme, time),
                            Tab::Dpi => self.screen_dpi(ui, theme, time),
                            Tab::Lighting => self.screen_lighting(ui, theme, time),
                            Tab::Performance => self.screen_performance(ui, theme, time),
                            Tab::Power => self.screen_power(ui, theme, time),
                            Tab::Macros => self.screen_macros(ui, theme, time),
                            Tab::Profiles => self.screen_profiles(ui, theme, time),
                            Tab::Settings => self.screen_settings(ui, theme, time),
                        }
                        ui.add_space(18.0);
                    });
                let _ = (lang, dirty);
            });
    }

    fn screen_overview(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let profile = self.ledger.profile().clone();
        let current_dpi = profile
            .dpi
            .iter()
            .find(|s| s.enabled)
            .map(|s| s.dpi)
            .unwrap_or(1_600);
        let polling = profile.performance.polling_hz();
        let profile_name = profile.name.clone();
        let profile_count = self.ledger.profiles.len();
        let mut go: Option<Tab> = None;

        let available = ui.available_width();
        let right_w = 292.0;
        let gap = 16.0;
        let left_w = (available - right_w - gap).max(420.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(
                vec2(left_w, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    // Hero card.
                    let hero = Frame::NONE
                        .fill(mix(theme.card, theme.accent, 0.065))
                        .stroke(Stroke::new(1.0, with_alpha(theme.accent, 55)))
                        .corner_radius(CornerRadius::same(20))
                        .inner_margin(Margin::same(22));
                    hero.show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        let content_rect = ui.max_rect();
                        let clip = ui.painter().with_clip_rect(content_rect);
                        let t = time as f32;
                        theme::paint_glow(
                            &clip,
                            pos2(content_rect.right() - 100.0, content_rect.top() + 55.0),
                            150.0,
                            theme.accent,
                            0.15,
                        );
                        ui.horizontal_top(|ui| {
                            let left = (ui.available_width() - 230.0).max(240.0);
                            ui.allocate_ui_with_layout(
                                vec2(left, 0.0),
                                Layout::top_down(Align::Min),
                                |ui| {
                                    ui.horizontal(|ui| {
                                        if self.hardware.connected() {
                                            pill(
                                                ui,
                                                theme,
                                                lang.t(
                                                    "CONECTADO · 2,4 GHz",
                                                    "CONNECTED · 2.4 GHz",
                                                ),
                                                theme.success,
                                            );
                                        } else {
                                            pill(
                                                ui,
                                                theme,
                                                lang.t(
                                                    "VISTA PREVIA · 2,4 GHz",
                                                    "PREVIEW · 2.4 GHz",
                                                ),
                                                theme.warning,
                                            );
                                        }
                                        pill(ui, theme, "X11", theme.accent);
                                    });
                                    ui.add_space(12.0);
                                    ui.label(
                                        RichText::new(lang.t("Tu X11, a tu manera.", "Your X11, your way."))
                                            .size(29.0)
                                            .color(theme.text)
                                            .strong(),
                                    );
                                    ui.add_space(2.0);
                                    ui.label(
                                        RichText::new(lang.t(
                                            "Una interfaz clara para ajustar lo que importa: botones, DPI, luz y batería.",
                                            "A clear interface to tune what matters: buttons, DPI, light and battery.",
                                        ))
                                        .size(12.5)
                                        .color(theme.muted),
                                    );
                                    ui.add_space(14.0);
                                    ui.horizontal(|ui| {
                                        if primary_button(
                                            ui,
                                            theme,
                                            lang.t("Editar botones", "Edit buttons"),
                                        )
                                        .clicked()
                                        {
                                            go = Some(Tab::Buttons);
                                        }
                                        if secondary_button(ui, theme, lang.t("Ver DPI", "View DPI"))
                                            .clicked()
                                        {
                                            go = Some(Tab::Dpi);
                                        }
                                    });
                                    ui.add_space(18.0);
                                    ui.horizontal(|ui| {
                                        hero_metric(ui, theme, "DPI", &fmt_count(current_dpi));
                                        ui.add_space(6.0);
                                        hero_metric(ui, theme, lang.t("SONDEO", "POLLING"), &format!("{polling} Hz"));
                                        ui.add_space(6.0);
                                        hero_metric(ui, theme, lang.t("PERFIL", "PROFILE"), &profile_name);
                                    });
                                },
                            );
                            ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                                let _ = t;
                                let art = mouse_art(
                                    ui,
                                    vec2(225.0, 255.0),
                                    MouseArt {
                                        theme,
                                        glow: Some((theme.accent, 0.8)),
                                        selected: None,
                                        numbered: false,
                                        caption: None,
                                    },
                                );
                                let _ = art;
                            });
                        });
                    });

                    ui.add_space(16.0);
                    ui.label(
                        RichText::new(lang.t("ACCESOS RÁPIDOS", "QUICK ACCESS"))
                            .size(8.5)
                            .color(theme.dim),
                    );
                    ui.add_space(2.0);
                    let quick_w = (ui.available_width() - 36.0) / 4.0;
                    let macro_sub = format!(
                        "{} {}",
                        self.ledger.macros.len(),
                        lang.t("guardadas", "saved")
                    );
                    ui.horizontal(|ui| {
                        for (icon, tab, title, sub) in [
                            (
                                "1",
                                Tab::Buttons,
                                lang.t("Botones", "Buttons"),
                                lang.t("5 acciones", "5 actions"),
                            ),
                            (
                                "2",
                                Tab::Dpi,
                                "DPI",
                                lang.t("8 ranuras · 6 activas", "8 slots · 6 active"),
                            ),
                            (
                                "3",
                                Tab::Lighting,
                                lang.t("Iluminación", "Lighting"),
                                lang.t("7 modos", "7 modes"),
                            ),
                            (
                                "4",
                                Tab::Macros,
                                lang.t("Macros", "Macros"),
                                macro_sub.as_str(),
                            ),
                        ] {
                            if quick_action(ui, theme, vec2(quick_w, 74.0), icon, title, sub) {
                                go = Some(tab);
                            }
                            ui.add_space(4.0);
                        }
                    });

                    ui.add_space(16.0);
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(16))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(lang.t("PERFIL ACTIVO", "ACTIVE PROFILE"))
                                            .size(8.5)
                                            .color(theme.dim),
                                    );
                                    ui.label(
                                        RichText::new(profile_name.clone())
                                            .size(17.0)
                                            .color(theme.text)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {} · {} Hz · {}",
                                            profile_count,
                                            lang.t("perfiles", "profiles"),
                                            polling,
                                            profile.light.mode_label(lang)
                                        ))
                                        .size(11.0)
                                        .color(theme.muted),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if secondary_button(
                                        ui,
                                        theme,
                                        lang.t("Gestionar perfiles", "Manage profiles"),
                                    )
                                    .clicked()
                                    {
                                        go = Some(Tab::Profiles);
                                    }
                                });
                            });
                        });
                },
            );
            ui.add_space(gap);
            ui.allocate_ui_with_layout(
                vec2(right_w, 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    // Device status.
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Estado del dispositivo", "Device status"))
                                        .size(13.5)
                                        .color(theme.text)
                                        .strong(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if self.hardware.connected() {
                                        status_dot(
                                            ui,
                                            theme.success,
                                            lang.t("Conectado", "Connected"),
                                            theme,
                                        );
                                    } else {
                                        status_dot(
                                            ui,
                                            theme.warning,
                                            lang.t("Vista previa", "Preview"),
                                            theme,
                                        );
                                    }
                                });
                            });
                            ui.add_space(10.0);
                            ui.vertical_centered(|ui| {
                                theme::battery_ring(ui, theme, 82.0, 118.0);
                            });
                            ui.add_space(4.0);
                            ui.vertical_centered(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Batería estimada", "Estimated battery"))
                                        .size(10.5)
                                        .color(theme.muted),
                                );
                                ui.label(
                                    RichText::new(if self.hardware.connected() {
                                        lang.t("Lectura del X11 conectado.", "Reading from the connected X11.")
                                    } else {
                                        lang.t(
                                            "Valor de vista previa hasta conectar el ratón.",
                                            "Preview value until the mouse is connected.",
                                        )
                                    })
                                    .size(9.0)
                                    .color(theme.dim),
                                );
                            });
                            ui.add_space(12.0);
                            theme::divider(ui, theme);
                            ui.add_space(8.0);
                            info_row(ui, theme, lang.t("Conexión", "Connection"), "2,4 GHz");
                            info_row(ui, theme, lang.t("Sondeo", "Polling"), &format!("{polling} Hz"));
                            info_row(ui, theme, lang.t("Firmware", "Firmware"), "—");
                            info_row(
                                ui,
                                theme,
                                lang.t("Última sincronización", "Last sync"),
                                if self.dirty() {
                                    lang.t("Pendiente", "Pending")
                                } else {
                                    lang.t("Ahora", "Just now")
                                },
                            );
                            ui.add_space(10.0);
                            if secondary_button(ui, theme, lang.t("Sincronizar", "Sync")).clicked() {
                                if self.hardware.connect() {
                                    let profile = self.ledger.profile().clone();
                                    match self.hardware.apply_profile(&profile) {
                                        Ok(()) => self.notify(
                                            lang.t(
                                                "X11 conectado y configuración aplicada.",
                                                "X11 connected and settings applied.",
                                            ),
                                            ToastTone::Success,
                                            time,
                                        ),
                                        Err(err) => self.notify(
                                            format!(
                                                "{} ({err})",
                                                lang.t(
                                                    "Conectado, pero no se pudo aplicar.",
                                                    "Connected, but applying failed."
                                                )
                                            ),
                                            ToastTone::Warning,
                                            time,
                                        ),
                                    }
                                } else {
                                    self.notify(
                                        lang.t(
                                            "No se encontró el X11. Revisa el receptor 2,4 GHz.",
                                            "X11 not found. Check the 2.4 GHz receiver.",
                                        ),
                                        ToastTone::Warning,
                                        time,
                                    );
                                }
                            }
                        });

                    ui.add_space(14.0);
                    Frame::NONE
                        .fill(with_alpha(theme.accent, 14))
                        .stroke(Stroke::new(1.0, with_alpha(theme.accent, 45)))
                        .corner_radius(CornerRadius::same(16))
                        .inner_margin(Margin::same(16))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(lang.t("Consejo", "Tip"))
                                    .size(10.0)
                                    .color(theme.accent),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(lang.t(
                                    "Para jugar, prueba 1000 Hz con 4 ms de respuesta y la iluminación al 60 %.",
                                    "For gaming, try 1000 Hz with 4 ms response and lighting at 60%.",
                                ))
                                .size(11.0)
                                .color(theme.text),
                            );
                        });
                },
            );
        });

        if let Some(tab) = go {
            self.tab = tab;
        }
    }

    fn screen_buttons(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let names = button_names(lang);
        let hardware = button_hardware_ids();
        let mut go_picker: Option<usize> = None;
        let available = ui.available_width();
        let left_w = (available * 0.40).clamp(330.0, 430.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                Frame::NONE
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.stroke_soft))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(lang.t("Mapa del ratón", "Mouse map"))
                                    .size(13.5)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                pill(ui, theme, lang.t("5 botones", "5 buttons"), theme.accent);
                            });
                        });
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new(lang.t(
                                "Haz clic en un número para editar ese botón.",
                                "Click a number to edit that button.",
                            ))
                            .size(10.5)
                            .color(theme.muted),
                        );
                        let art = mouse_art(
                            ui,
                            vec2(ui.available_width(), 470.0),
                            MouseArt {
                                theme,
                                glow: Some((theme.accent, 0.55)),
                                selected: Some(self.selected_button),
                                numbered: true,
                                caption: None,
                            },
                        );
                        if let Some(index) = art.clicked {
                            self.selected_button = index;
                        }
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            let name = names[self.selected_button];
                            ui.label(
                                RichText::new(format!(
                                    "{} {} · {}",
                                    lang.t("Botón", "Button"),
                                    self.selected_button + 1,
                                    name
                                ))
                                .size(11.5)
                                .color(theme.text),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "HID {}",
                                        hardware[self.selected_button]
                                    ))
                                    .size(9.5)
                                    .color(theme.dim),
                                );
                            });
                        });
                    });
            });
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(16))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Asignaciones", "Assignments"))
                                        .size(13.5)
                                        .color(theme.text)
                                        .strong(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if ghost_button(ui, theme, lang.t("Restablecer", "Reset"))
                                        .clicked()
                                    {
                                        self.ledger.profile_mut().buttons = default_buttons();
                                        self.notify(
                                            lang.t("Botones restablecidos.", "Buttons reset."),
                                            ToastTone::Info,
                                            time,
                                        );
                                    }
                                });
                            });
                            ui.add_space(6.0);
                            for i in 0..BUTTON_COUNT {
                                let command = self.ledger.profile().buttons[i].command.clone();
                                let selected = i == self.selected_button;
                                if button_row(
                                    ui,
                                    theme,
                                    i,
                                    names[i],
                                    hardware[i],
                                    &command.label(lang),
                                    command.category().label(lang),
                                    command.code(),
                                    selected,
                                ) {
                                    self.selected_button = i;
                                    go_picker = Some(i);
                                }
                            }
                        });

                    ui.add_space(14.0);
                    let mut command = self.ledger.profile().buttons[self.selected_button]
                        .command
                        .clone();
                    let original = command.clone();
                    Frame::NONE
                        .fill(theme.panel)
                        .stroke(Stroke::new(1.0, with_alpha(theme.accent, 40)))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(lang.t("Acción asignada", "Assigned action"))
                                            .size(8.5)
                                            .color(theme.dim),
                                    );
                                    ui.label(
                                        RichText::new(command.label(lang))
                                            .size(18.0)
                                            .color(theme.text)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} · 0x{:02X} · {} {}",
                                            command.category().label(lang),
                                            command.code(),
                                            lang.t("Botón", "Button"),
                                            self.selected_button + 1
                                        ))
                                        .size(10.5)
                                        .color(theme.muted),
                                    );
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if primary_button(
                                        ui,
                                        theme,
                                        lang.t("Cambiar acción", "Change action"),
                                    )
                                    .clicked()
                                    {
                                        go_picker = Some(self.selected_button);
                                    }
                                });
                            });
                            if let Command::Macro { mode, loops, .. } = &mut command {
                                ui.add_space(12.0);
                                theme::divider(ui, theme);
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        RichText::new(lang.t("Repetición", "Repeat"))
                                            .size(11.0)
                                            .color(theme.muted),
                                    );
                                    ComboBox::from_id_salt("button_macro_mode")
                                        .width(176.0)
                                        .selected_text(mode.label(lang))
                                        .show_ui(ui, |ui| {
                                            for m in [
                                                MacroMode::Repeat,
                                                MacroMode::HoldRelease,
                                                MacroMode::UntilKey,
                                            ] {
                                                ui.selectable_value(mode, m.clone(), m.label(lang));
                                            }
                                        });
                                    ui.add_space(8.0);
                                    ui.label(
                                        RichText::new(lang.t("Veces", "Times"))
                                            .size(11.0)
                                            .color(theme.muted),
                                    );
                                    ui.add(
                                        egui::DragValue::new(loops)
                                            .range(1..=99)
                                            .speed(1.0)
                                            .fixed_decimals(0),
                                    );
                                });
                            }
                            if let Command::Shortcut { .. } = &command {
                                ui.add_space(12.0);
                                if secondary_button(
                                    ui,
                                    theme,
                                    lang.t("Editar combinación", "Edit shortcut"),
                                )
                                .clicked()
                                {
                                    if let Command::Shortcut {
                                        ctrl,
                                        shift,
                                        alt,
                                        win,
                                        key,
                                    } = &command
                                    {
                                        self.shortcut_editor = Some(ShortcutDraft {
                                            button: self.selected_button,
                                            ctrl: *ctrl,
                                            shift: *shift,
                                            alt: *alt,
                                            win: *win,
                                            key: key.clone(),
                                        });
                                    }
                                }
                            }
                            if matches!(command, Command::Disabled) {
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new(lang.t(
                                        "Este botón no enviará ninguna acción.",
                                        "This button will not send any action.",
                                    ))
                                    .size(10.5)
                                    .color(theme.warning),
                                );
                            }
                            if matches!(command, Command::Macro { .. }) {
                                ui.add_space(10.0);
                                ui.label(
                                    RichText::new(lang.t(
                                        "Las macros se editan en la pestaña Macros.",
                                        "Macros are edited in the Macros tab.",
                                    ))
                                    .size(10.0)
                                    .color(theme.dim),
                                );
                            }
                        });
                    if command != original {
                        // FUN_004a09f0 refuses a map with no Left Click.
                        let keeps_left =
                            matches!(command, Command::LeftClick)
                                || self.ledger.profile().buttons.iter().enumerate().any(
                                    |(i, b)| {
                                        i != self.selected_button
                                            && matches!(b.command, Command::LeftClick)
                                    },
                                );
                        if keeps_left {
                            self.ledger.profile_mut().buttons[self.selected_button].command =
                                command;
                        } else {
                            self.notify(
                                lang.t(
                                    "Al menos un botón debe ser «Clic izquierdo».",
                                    "At least one button must be \"Left click\".",
                                ),
                                ToastTone::Warning,
                                time,
                            );
                        }
                    }
                },
            );
        });

        if let Some(index) = go_picker {
            self.selected_button = index;
            self.picker = Some(CommandPicker {
                button: index,
                search: String::new(),
                category: None,
            });
        }
    }

    fn screen_dpi(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        ui.horizontal(|ui| {
            pill(ui, theme, "50 – 22 000 DPI", theme.cyan);
            pill(
                ui,
                theme,
                lang.t("8 ranuras · 6 activas", "8 slots · 6 active"),
                theme.accent,
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                ui.label(
                    RichText::new(lang.t(
                        "El X11 ajusta el DPI en pasos de 50 hasta 10 000 y de 100 por encima.",
                        "The X11 steps DPI by 50 up to 10,000 and by 100 above it.",
                    ))
                    .size(10.0)
                    .color(theme.dim),
                );
            });
        });
        ui.add_space(10.0);

        let all_stages = self.ledger.profile().dpi.clone();
        let mut stages = all_stages.clone();
        for row in 0..3 {
            ui.columns(3, |cols| {
                for col in 0..3 {
                    let index = row * 3 + col;
                    if let Some(stage) = stages.get_mut(index) {
                        dpi_stage_card(&mut cols[col], theme, lang, index, stage);
                    }
                }
            });
            if row < 2 {
                ui.add_space(8.0);
            }
        }
        if stages != all_stages {
            self.ledger.profile_mut().dpi = stages.clone();
        }

        ui.add_space(14.0);
        let available = ui.available_width();
        let left_w = (available - 300.0 - 16.0).max(420.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                Frame::NONE
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.stroke_soft))
                    .corner_radius(CornerRadius::same(16))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        section_heading(
                            ui,
                            theme,
                            lang.t("Vista previa del ciclo", "Cycle preview"),
                            Some(lang.t(
                                "Así se recorren tus etapas al pulsar el botón DPI.",
                                "This is how your stages cycle on the DPI button.",
                            )),
                        );
                        ui.add_space(12.0);
                        ui.horizontal_wrapped(|ui| {
                            for (i, stage) in stages.iter().enumerate() {
                                if !stage.enabled {
                                    continue;
                                }
                                Frame::NONE
                                    .fill(with_alpha(
                                        Color32::from_rgb(
                                            stage.color[0],
                                            stage.color[1],
                                            stage.color[2],
                                        ),
                                        20,
                                    ))
                                    .stroke(Stroke::new(
                                        1.0,
                                        with_alpha(
                                            Color32::from_rgb(
                                                stage.color[0],
                                                stage.color[1],
                                                stage.color[2],
                                            ),
                                            110,
                                        ),
                                    ))
                                    .corner_radius(CornerRadius::same(10))
                                    .inner_margin(Margin::symmetric(10, 6))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let (dot, _) = ui.allocate_exact_size(
                                                vec2(8.0, 8.0),
                                                Sense::hover(),
                                            );
                                            ui.painter().circle_filled(
                                                dot.center(),
                                                3.5,
                                                Color32::from_rgb(
                                                    stage.color[0],
                                                    stage.color[1],
                                                    stage.color[2],
                                                ),
                                            );
                                            ui.label(
                                                RichText::new(fmt_count(stage.dpi))
                                                    .size(11.5)
                                                    .color(theme.text),
                                            );
                                        });
                                    });
                                if i + 1 < stages.iter().filter(|s| s.enabled).count() {
                                    ui.label(RichText::new("→").size(11.0).color(theme.dim));
                                }
                            }
                        });
                        ui.add_space(12.0);
                        theme::divider(ui, theme);
                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(lang.t("Plantillas rápidas", "Quick presets"))
                                    .size(11.0)
                                    .color(theme.muted),
                            );
                            ui.add_space(4.0);
                            if choice_chip(ui, theme, lang.t("Precisión", "Precision"), false)
                                .clicked()
                            {
                                apply_dpi_preset(
                                    &mut stages,
                                    &[400, 800, 1_200, 1_600, 2_400, 3_200],
                                );
                                self.ledger.profile_mut().dpi = stages.clone();
                            }
                            let factory = factory_dpi();
                            if choice_chip(
                                ui,
                                theme,
                                lang.t("Fábrica", "Factory"),
                                stages == factory,
                            )
                            .clicked()
                            {
                                stages = factory_dpi();
                                self.ledger.profile_mut().dpi = stages.clone();
                            }
                            if choice_chip(ui, theme, lang.t("Velocidad", "Speed"), false).clicked()
                            {
                                apply_dpi_preset(
                                    &mut stages,
                                    &[800, 1_600, 3_200, 6_400, 12_000, 22_000],
                                );
                                self.ledger.profile_mut().dpi = stages.clone();
                            }
                        });
                    });
            });
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(mix(theme.card, theme.accent, 0.05))
                        .stroke(Stroke::new(1.0, with_alpha(theme.accent, 45)))
                        .corner_radius(CornerRadius::same(16))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(lang.t("Etapas activas", "Active stages"))
                                    .size(12.0)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.add_space(8.0);
                            let active = stages.iter().filter(|s| s.enabled).count();
                            ui.label(
                                RichText::new(format!("{active} / {}", DPI_STAGE_COUNT))
                                    .size(26.0)
                                    .color(theme.accent)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(lang.t(
                                    "Las etapas desactivadas no entran en el ciclo.",
                                    "Disabled stages are skipped in the cycle.",
                                ))
                                .size(10.0)
                                .color(theme.muted),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(format!(
                                    "{}: {}",
                                    lang.t("Etapa de arranque", "Start-up stage"),
                                    DPI_ACTIVE_DEFAULT + 1
                                ))
                                .size(10.0)
                                .color(theme.dim),
                            );
                            ui.add_space(12.0);
                            if danger_button(ui, theme, lang.t("Restablecer DPI", "Reset DPI"))
                                .clicked()
                            {
                                self.ledger.profile_mut().dpi = factory_dpi();
                                self.notify(
                                    lang.t("DPI restablecido.", "DPI reset."),
                                    ToastTone::Info,
                                    time,
                                );
                            }
                        });
                },
            );
        });
    }

    fn screen_lighting(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let profile = self.ledger.profile().clone();
        let mut light = profile.light.clone();
        let original = light.clone();
        let preview = lighting_preview(&light, &profile.dpi, time);
        let available = ui.available_width();
        let left_w = (available * 0.43).clamp(350.0, 470.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                Frame::NONE
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.stroke_soft))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(lang.t("Vista previa", "Preview"))
                                    .size(13.5)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                pill(ui, theme, light.mode_label(lang), preview.0);
                            });
                        });
                        let _ = mouse_art(
                            ui,
                            vec2(ui.available_width(), 470.0),
                            MouseArt {
                                theme,
                                glow: Some(preview),
                                selected: None,
                                numbered: false,
                                caption: Some(
                                    lang.t(
                                        "La luz cambia contigo, en tiempo real.",
                                        "Light changes with you, in real time.",
                                    )
                                    .into(),
                                ),
                            },
                        );
                    });
            });
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(lang.t("Modo de iluminación", "Lighting mode"))
                                    .size(13.5)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.add_space(8.0);
                            ui.columns(2, |cols| {
                                for mode in 0..LIGHT_MODE_COUNT {
                                    let col = mode % 2;
                                    let label = light_mode_label(mode, lang);
                                    if mode_card(
                                        &mut cols[col],
                                        theme,
                                        mode,
                                        label,
                                        light.mode == mode,
                                        time,
                                    ) {
                                        light.mode = mode;
                                    }
                                }
                            });
                            ui.add_space(14.0);
                            theme::divider(ui, theme);
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Brillo", "Brightness"))
                                        .size(11.5)
                                        .color(theme.muted),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{}/8", light.brightness))
                                            .size(11.0)
                                            .color(theme.text),
                                    );
                                });
                            });
                            let mut brightness = light.brightness as u32;
                            let _ = theme::slider_u32(
                                ui,
                                theme,
                                &mut brightness,
                                1,
                                8,
                                ui.available_width(),
                            );
                            light.brightness = brightness.clamp(1, 8) as u8;
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Velocidad", "Speed"))
                                        .size(11.5)
                                        .color(theme.muted),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(format!("{}/8", light.speed))
                                            .size(11.0)
                                            .color(theme.text),
                                    );
                                });
                            });
                            let mut speed = light.speed as u32;
                            let _ = theme::slider_u32(
                                ui,
                                theme,
                                &mut speed,
                                4,
                                8,
                                ui.available_width(),
                            );
                            light.speed = speed.clamp(4, 8) as u8;
                            ui.add_space(12.0);
                            theme::divider(ui, theme);
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Color", "Color"))
                                        .size(11.5)
                                        .color(theme.muted),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.color_edit_button_srgb(&mut light.color);
                                });
                            });
                            ui.add_space(8.0);
                            ui.horizontal_wrapped(|ui| {
                                for rgb in COLOR_PRESETS {
                                    if color_swatch(ui, theme, rgb, light.color == rgb) {
                                        light.color = rgb;
                                    }
                                }
                            });
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(lang.t(
                                    "El modo DPI usa el color de cada etapa.",
                                    "DPI mode uses the color of each stage.",
                                ))
                                .size(10.0)
                                .color(theme.dim),
                            );
                        });
                },
            );
        });

        if light != original {
            self.ledger.profile_mut().light = light;
        }
    }

    fn screen_performance(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let profile = self.ledger.profile().clone();
        let mut perf = profile.performance.clone();
        let original = perf.clone();
        let latency = 1000.0 / perf.polling_hz() as f32 + perf.debounce as f32;

        Frame::NONE
            .fill(theme.card)
            .stroke(Stroke::new(1.0, theme.stroke_soft))
            .corner_radius(CornerRadius::same(18))
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(lang.t("Tasa de sondeo", "Polling rate"))
                                .size(13.5)
                                .color(theme.text)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(lang.t(
                                "Más Hz significa menos latencia y algo más de consumo.",
                                "More Hz means less latency and a little more power use.",
                            ))
                            .size(10.5)
                            .color(theme.muted),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        pill(ui, theme, format!("≈ {:.0} ms", latency), theme.success);
                    });
                });
                ui.add_space(12.0);
                let card_w = (ui.available_width() - 36.0) / 4.0;
                ui.horizontal(|ui| {
                    for (i, hz) in POLLING_RATES.iter().enumerate() {
                        if polling_card(
                            ui,
                            theme,
                            lang,
                            vec2(card_w, 94.0),
                            i,
                            *hz,
                            perf.polling == i,
                        )
                        .clicked()
                        {
                            perf.polling = i;
                        }
                        ui.add_space(4.0);
                    }
                });
            });

        ui.add_space(14.0);
        ui.columns(2, |cols| {
            Frame::NONE
                .fill(theme.card)
                .stroke(Stroke::new(1.0, theme.stroke_soft))
                .corner_radius(CornerRadius::same(18))
                .inner_margin(Margin::same(18))
                .show(&mut cols[0], |ui| {
                    ui.set_width(ui.available_width());
                    section_heading(
                        ui,
                        theme,
                        lang.t("Respuesta de clic", "Click response"),
                        Some(lang.t(
                            "Tiempo de debounce: menos es más rápido, más es más estable.",
                            "Debounce time: lower is faster, higher is steadier.",
                        )),
                    );
                    ui.add_space(16.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("{} ms", perf.debounce))
                                .size(30.0)
                                .color(theme.accent)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(lang.t("actuales", "current"))
                                .size(10.0)
                                .color(theme.dim),
                        );
                    });
                    ui.add_space(6.0);
                    let mut debounce = perf.debounce as u32;
                    let _ =
                        theme::slider_u32(ui, theme, &mut debounce, 2, 25, ui.available_width());
                    perf.debounce = debounce.clamp(2, 25) as u8;
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("2 ms").size(9.5).color(theme.dim));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(RichText::new("25 ms").size(9.5).color(theme.dim));
                        });
                    });
                    ui.add_space(10.0);
                    ui.label(
                        RichText::new(lang.t(
                            "Estimación: la latencia total combina sondeo y debounce.",
                            "Estimate: total latency combines polling and debounce.",
                        ))
                        .size(9.5)
                        .color(theme.dim),
                    );
                });

            Frame::NONE
                .fill(theme.card)
                .stroke(Stroke::new(1.0, theme.stroke_soft))
                .corner_radius(CornerRadius::same(18))
                .inner_margin(Margin::same(18))
                .show(&mut cols[1], |ui| {
                    ui.set_width(ui.available_width());
                    section_heading(
                        ui,
                        theme,
                        lang.t("Precisión", "Precision"),
                        Some(lang.t(
                            "Opciones finas para un movimiento limpio.",
                            "Fine options for clean movement.",
                        )),
                    );
                    ui.add_space(10.0);
                    toggle_setting(
                        ui,
                        theme,
                        lang.t("Control de ondulación", "Ripple control"),
                        lang.t("Suaviza movimientos rápidos.", "Smooths fast movement."),
                        &mut perf.ripple,
                    );
                    toggle_setting(
                        ui,
                        theme,
                        lang.t("Captura de ángulo", "Angle snap"),
                        lang.t(
                            "Corrige líneas casi rectas.",
                            "Corrects nearly straight lines.",
                        ),
                        &mut perf.angle_snap,
                    );
                    toggle_setting(
                        ui,
                        theme,
                        lang.t("Sincronía de movimiento", "Motion sync"),
                        lang.t("Alinea la lectura del sensor.", "Aligns sensor readings."),
                        &mut perf.motion_sync,
                    );
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(lang.t("Distancia de despegue", "Lift-off distance"))
                            .size(11.0)
                            .color(theme.muted),
                    );
                    ui.horizontal(|ui| {
                        if choice_chip(ui, theme, "1 mm", perf.lift_mm == 1).clicked() {
                            perf.lift_mm = 1;
                        }
                        if choice_chip(ui, theme, "2 mm", perf.lift_mm == 2).clicked() {
                            perf.lift_mm = 2;
                        }
                    });
                });
        });

        if perf != original {
            self.ledger.profile_mut().performance = perf;
        }
        let _ = time;
    }

    fn screen_power(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let profile = self.ledger.profile().clone();
        let mut power = profile.power.clone();
        let original = power.clone();
        let polling = profile.performance.polling;
        let brightness = profile.light.brightness as f32;
        let estimated_hours = match polling {
            0 => 96.0,
            1 => 82.0,
            2 => 66.0,
            _ => 51.0,
        } - (brightness - 1.0) * 1.4;
        let estimated_hours = estimated_hours.max(6.0);

        let available = ui.available_width();
        let left_w = (available * 0.43).clamp(330.0, 450.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                Frame::NONE
                    .fill(mix(theme.card, theme.success, 0.045))
                    .stroke(Stroke::new(1.0, with_alpha(theme.success, 45)))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(20))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(lang.t("Batería", "Battery"))
                                    .size(13.5)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                pill(
                                    ui,
                                    theme,
                                    lang.t("VISTA PREVIA", "PREVIEW"),
                                    theme.warning,
                                );
                            });
                        });
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            theme::battery_ring(ui, theme, 82.0, 156.0);
                        });
                        ui.add_space(8.0);
                        ui.vertical_centered(|ui| {
                            ui.label(
                                RichText::new(lang.t("Autonomía estimada", "Estimated runtime"))
                                    .size(10.0)
                                    .color(theme.muted),
                            );
                            ui.label(
                                RichText::new(format!("{:.0} h", estimated_hours))
                                    .size(27.0)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.label(
                                RichText::new(lang.t(
                                    "Según sondeo, brillo y uso.",
                                    "Based on polling, brightness and use.",
                                ))
                                .size(9.5)
                                .color(theme.dim),
                            );
                        });
                        ui.add_space(10.0);
                        theme::divider(ui, theme);
                        ui.add_space(8.0);
                        info_row(ui, theme, lang.t("Estado", "Status"), lang.t("En uso", "In use"));
                        info_row(
                            ui,
                            theme,
                            lang.t("Carga", "Charge"),
                            lang.t("No disponible", "Not available"),
                        );
                        info_row(ui, theme, lang.t("Sondeo", "Polling"), &format!("{} Hz", POLLING_RATES[polling]));
                    });
            });
            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            section_heading(
                                ui,
                                theme,
                                lang.t("Ahorro inteligente", "Smart saving"),
                                Some(lang.t(
                                    "Apaga la luz y suspende el ratón cuando no lo uses.",
                                    "Turn off the light and sleep the mouse when idle.",
                                )),
                            );
                            ui.add_space(14.0);
                            timer_setting(
                                ui,
                                theme,
                                lang.t("Apagar la luz tras", "Turn light off after"),
                                &mut power.light_off_min,
                            );
                            ui.add_space(14.0);
                            timer_setting(
                                ui,
                                theme,
                                lang.t("Suspender tras", "Sleep after"),
                                &mut power.sleep_min,
                            );
                            ui.add_space(14.0);
                            toggle_setting(
                                ui,
                                theme,
                                lang.t("Despertar al mover", "Wake on movement"),
                                lang.t("Vuelve a la vida con un gesto.", "Comes back with a simple move."),
                                &mut power.move_wake,
                            );
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(lang.t(
                                    "La batería y la carga reales necesitan el dispositivo conectado.",
                                    "Real battery and charging need the device connected.",
                                ))
                                .size(9.5)
                                .color(theme.dim),
                            );
                        });

                    ui.add_space(14.0);
                    Frame::NONE
                        .fill(with_alpha(theme.accent, 12))
                        .stroke(Stroke::new(1.0, with_alpha(theme.accent, 38)))
                        .corner_radius(CornerRadius::same(16))
                        .inner_margin(Margin::same(16))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(
                                RichText::new(lang.t("Un buen equilibrio", "A good balance"))
                                    .size(11.0)
                                    .color(theme.accent),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(lang.t(
                                    "Con 500 Hz y brillo medio sueles ganar autonomía sin notar la diferencia.",
                                    "At 500 Hz and medium brightness you usually gain runtime without feeling it.",
                                ))
                                .size(10.5)
                                .color(theme.text),
                            );
                        });
                },
            );
        });

        if power != original {
            self.ledger.profile_mut().power = power;
        }
        let _ = time;
    }

    fn screen_macros(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let rows: Vec<(String, usize, u32)> = self
            .ledger
            .macros
            .iter()
            .map(|m| (m.name.clone(), m.actions.len(), m.duration_ms()))
            .collect();
        let mut select: Option<usize> = None;
        let mut add = false;
        let mut delete: Option<usize> = None;
        let selected = self
            .selected_macro
            .min(self.ledger.macros.len().saturating_sub(1));

        let available = ui.available_width();
        let list_w = 270.0;
        let right_w = (available - list_w - 14.0).max(500.0);
        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(list_w, 0.0), Layout::top_down(Align::Min), |ui| {
                Frame::NONE
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.stroke_soft))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(14))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(lang.t("Macros", "Macros"))
                                    .size(13.5)
                                    .color(theme.text)
                                    .strong(),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if icon_button(ui, theme, "+", lang.t("Nueva macro", "New macro"))
                                    .clicked()
                                {
                                    add = true;
                                }
                            });
                        });
                        ui.add_space(6.0);
                        for (i, (name, count, duration)) in rows.iter().enumerate() {
                            let (rect, response) = ui.allocate_exact_size(
                                vec2(ui.available_width(), 54.0),
                                Sense::click(),
                            );
                            let active = i == selected;
                            let hover = response.hovered();
                            ui.painter().rect_filled(
                                rect,
                                CornerRadius::same(10),
                                if active {
                                    with_alpha(theme.accent, 22)
                                } else if hover {
                                    with_alpha(theme.text, 6)
                                } else {
                                    Color32::TRANSPARENT
                                },
                            );
                            if active {
                                ui.painter().rect_filled(
                                    Rect::from_min_size(
                                        pos2(rect.left(), rect.center().y - 12.0),
                                        vec2(3.0, 24.0),
                                    ),
                                    CornerRadius::same(2),
                                    theme.accent,
                                );
                            }
                            ui.painter().text(
                                pos2(rect.left() + 12.0, rect.center().y - 8.0),
                                Align2::LEFT_CENTER,
                                name,
                                FontId::proportional(12.0),
                                if active { theme.text } else { theme.muted },
                            );
                            ui.painter().text(
                                pos2(rect.left() + 12.0, rect.center().y + 10.0),
                                Align2::LEFT_CENTER,
                                format!(
                                    "{} {} · {} ms",
                                    count,
                                    lang.t("acciones", "actions"),
                                    duration
                                ),
                                FontId::proportional(9.5),
                                theme.dim,
                            );
                            if response.clicked() {
                                select = Some(i);
                            }
                            if hover {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                            }
                        }
                        ui.add_space(6.0);
                        if outline_action(ui, theme, lang.t("Grabar teclas", "Record keys"))
                            .clicked()
                        {
                            if self.recording.is_some() {
                                self.recording = None;
                            } else {
                                self.recording = Some(MacroRecording {
                                    macro_index: selected,
                                    last_time: time,
                                });
                            }
                        }
                    });
            });
            ui.add_space(14.0);
            ui.allocate_ui_with_layout(vec2(right_w, 545.0), Layout::top_down(Align::Min), |ui| {
                if let Some(mac) = self.ledger.macros.get(selected).cloned() {
                    let mut m = mac;
                    let original = m.clone();
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(lang.t("Editor de macro", "Macro editor"))
                                            .size(13.5)
                                            .color(theme.text)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(lang.t(
                                            "Cada paso se ejecuta en orden.",
                                            "Each step runs in order.",
                                        ))
                                        .size(10.0)
                                        .color(theme.muted),
                                    );
                                });
                                let header_right = ui.available_width();
                                ui.allocate_ui_with_layout(
                                    vec2(header_right, 30.0),
                                    Layout::right_to_left(Align::Center),
                                    |ui| {
                                        if self.recording.is_some() {
                                            pill(
                                                ui,
                                                theme,
                                                lang.t("GRABANDO", "RECORDING"),
                                                theme.danger,
                                            );
                                        }
                                        if danger_button(ui, theme, lang.t("Eliminar", "Delete"))
                                            .clicked()
                                        {
                                            delete = Some(selected);
                                        }
                                    },
                                );
                            });
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Nombre", "Name"))
                                        .size(11.0)
                                        .color(theme.muted),
                                );
                                let name_w = (ui.available_width() - 12.0).max(160.0);
                                ui.add_sized(
                                    [name_w, 30.0],
                                    TextEdit::singleline(&mut m.name)
                                        .margin(Margin::symmetric(8, 6)),
                                );
                            });
                            ui.add_space(4.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Repetición", "Repeat"))
                                        .size(11.0)
                                        .color(theme.muted),
                                );
                                ComboBox::from_id_salt("macro_mode")
                                    .width(190.0)
                                    .selected_text(m.mode.label(lang))
                                    .show_ui(ui, |ui| {
                                        for mode in [
                                            MacroMode::Repeat,
                                            MacroMode::HoldRelease,
                                            MacroMode::UntilKey,
                                        ] {
                                            ui.selectable_value(
                                                &mut m.mode,
                                                mode.clone(),
                                                mode.label(lang),
                                            );
                                        }
                                    });
                                ui.add_space(8.0);
                                ui.label(
                                    RichText::new(lang.t("Veces", "Times"))
                                        .size(11.0)
                                        .color(theme.muted),
                                );
                                ui.add(
                                    egui::DragValue::new(&mut m.loops)
                                        .range(1..=99)
                                        .speed(1.0)
                                        .fixed_decimals(0),
                                );
                            });
                            if self.recording.is_some() {
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    let t = (time * 2.0).sin() as f32 * 0.5 + 0.5;
                                    let (dot, _) =
                                        ui.allocate_exact_size(vec2(9.0, 9.0), Sense::hover());
                                    ui.painter().circle_filled(
                                        dot.center(),
                                        3.5,
                                        with_alpha(theme.danger, (120.0 + t * 135.0) as u8),
                                    );
                                    ui.label(
                                        RichText::new(lang.t(
                                            "Grabando teclas… pulsa Esc para terminar.",
                                            "Recording keys… press Esc to stop.",
                                        ))
                                        .size(10.0)
                                        .color(theme.danger),
                                    );
                                });
                            }
                            ui.add_space(10.0);
                            theme::divider(ui, theme);
                            ui.add_space(8.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Secuencia", "Sequence"))
                                        .size(11.0)
                                        .color(theme.text)
                                        .strong(),
                                );
                                ui.label(
                                    RichText::new(format!(
                                        "{} {}",
                                        m.actions.len(),
                                        lang.t("pasos", "steps")
                                    ))
                                    .size(10.0)
                                    .color(theme.dim),
                                );
                                let clear_w = ui.available_width();
                                ui.allocate_ui_with_layout(
                                    vec2(clear_w, 28.0),
                                    Layout::right_to_left(Align::Center),
                                    |ui| {
                                        if ghost_button(ui, theme, lang.t("Vaciar", "Clear"))
                                            .clicked()
                                        {
                                            m.actions.clear();
                                        }
                                    },
                                );
                            });
                            ui.add_space(4.0);
                            let mut remove: Option<usize> = None;
                            let mut move_up: Option<usize> = None;
                            let mut move_down: Option<usize> = None;
                            ScrollArea::vertical()
                                .id_salt(("macro_actions", selected))
                                .max_height(260.0)
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.set_width(ui.available_width());
                                    if m.actions.is_empty() {
                                        ui.add_space(18.0);
                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                RichText::new(lang.t(
                                                    "Añade un paso o graba tus teclas.",
                                                    "Add a step or record your keys.",
                                                ))
                                                .size(11.0)
                                                .color(theme.dim),
                                            );
                                        });
                                        ui.add_space(18.0);
                                    }
                                    for (i, action) in m.actions.iter_mut().enumerate() {
                                        let result = macro_action_row(ui, theme, lang, i, action);
                                        if result.up {
                                            move_up = Some(i);
                                        }
                                        if result.down {
                                            move_down = Some(i);
                                        }
                                        if result.delete {
                                            remove = Some(i);
                                        }
                                    }
                                });
                            if let Some(i) = remove {
                                m.actions.remove(i);
                            } else if let Some(i) = move_up {
                                if i > 0 {
                                    m.actions.swap(i, i - 1);
                                }
                            } else if let Some(i) = move_down {
                                if i + 1 < m.actions.len() {
                                    m.actions.swap(i, i + 1);
                                }
                            }
                            ui.add_space(8.0);
                            ui.horizontal_wrapped(|ui| {
                                if outline_action(ui, theme, "+ Tecla").clicked() {
                                    m.actions.push(MacroAction::KeyDown("A".into()));
                                    m.actions.push(MacroAction::KeyUp("A".into()));
                                }
                                if outline_action(ui, theme, "+ Botón").clicked() {
                                    m.actions.push(MacroAction::MouseDown("Izq".into()));
                                    m.actions.push(MacroAction::MouseUp("Izq".into()));
                                }
                                if outline_action(ui, theme, "+ Retardo").clicked() {
                                    m.actions.push(MacroAction::Delay(10));
                                }
                                if outline_action(ui, theme, "+ Mover").clicked() {
                                    m.actions.push(MacroAction::Move(10, 0));
                                }
                            });
                        });
                    if m != original {
                        self.ledger.macros[selected] = m;
                    }
                }
            });
        });

        if add {
            let name = format!(
                "{} {}",
                lang.t("Macro", "Macro"),
                self.ledger.macros.len() + 1
            );
            self.ledger.macros.push(Macro {
                name,
                actions: Vec::new(),
                mode: MacroMode::Repeat,
                loops: 1,
            });
            self.selected_macro = self.ledger.macros.len() - 1;
        }
        if let Some(index) = select {
            self.selected_macro = index;
        }
        if let Some(index) = delete {
            self.delete_macro = Some(index);
        }
        let _ = available;
    }

    fn screen_profiles(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let profiles = self.ledger.profiles.clone();
        let mut activate: Option<usize> = None;
        let mut rename: Option<usize> = None;
        let mut duplicate: Option<usize> = None;
        let mut remove: Option<usize> = None;
        let mut add = false;

        ui.horizontal(|ui| {
            ui.label(
                RichText::new(lang.t(
                    "Tus configuraciones, separadas y listas para cambiar.",
                    "Your setups, separate and ready to switch.",
                ))
                .size(11.0)
                .color(theme.muted),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if primary_button(ui, theme, lang.t("Nuevo perfil", "New profile")).clicked() {
                    add = true;
                }
            });
        });
        ui.add_space(10.0);

        for row in 0..profiles.len().div_ceil(3) {
            ui.columns(3, |cols| {
                for col in 0..3 {
                    let index = row * 3 + col;
                    if let Some(profile) = profiles.get(index) {
                        let active = index == self.ledger.active;
                        profile_card(
                            &mut cols[col],
                            theme,
                            profile,
                            active,
                            lang,
                            index,
                            &mut activate,
                            &mut rename,
                            &mut duplicate,
                            &mut remove,
                        );
                    } else if index == profiles.len() {
                        new_profile_card(&mut cols[col], theme, lang, &mut add);
                    }
                }
            });
            ui.add_space(10.0);
        }

        ui.add_space(10.0);
        Frame::NONE
            .fill(theme.card)
            .stroke(Stroke::new(1.0, theme.stroke_soft))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(lang.t("Importar y exportar", "Import and export"))
                                .size(12.5)
                                .color(theme.text)
                                .strong(),
                        );
                        ui.label(
                            RichText::new(lang.t(
                                "Guarda un perfil en un archivo o compártelo con otro equipo.",
                                "Save a profile to a file or share it with another computer.",
                            ))
                            .size(10.0)
                            .color(theme.muted),
                        );
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if secondary_button(ui, theme, lang.t("Importar perfil", "Import profile"))
                            .clicked()
                        {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("X11 profile", &["json"])
                                .pick_file()
                            {
                                match std::fs::read_to_string(&path)
                                    .ok()
                                    .and_then(|text| profile_from_json(&text))
                                {
                                    Some(mut profile) => {
                                        let mut n = profile.name.clone();
                                        let mut i = 2;
                                        while self.ledger.profiles.iter().any(|p| p.name == n) {
                                            n = format!("{} {}", profile.name, i);
                                            i += 1;
                                        }
                                        profile.name = n;
                                        self.ledger.profiles.push(profile);
                                        self.ledger.active = self.ledger.profiles.len() - 1;
                                        self.notify(
                                            lang.t("Perfil importado.", "Profile imported."),
                                            ToastTone::Success,
                                            time,
                                        );
                                    }
                                    None => self.notify(
                                        lang.t(
                                            "No se pudo leer el perfil.",
                                            "Could not read the profile.",
                                        ),
                                        ToastTone::Warning,
                                        time,
                                    ),
                                }
                            }
                        }
                        if secondary_button(ui, theme, lang.t("Exportar activo", "Export active"))
                            .clicked()
                        {
                            let profile = self.ledger.profile().clone();
                            let default_name = format!(
                                "x11-{}.json",
                                profile.name.to_lowercase().replace(' ', "-")
                            );
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("X11 profile", &["json"])
                                .set_file_name(default_name)
                                .save_file()
                            {
                                match std::fs::write(&path, profile_to_json(&profile)) {
                                    Ok(()) => self.notify(
                                        lang.t("Perfil exportado.", "Profile exported."),
                                        ToastTone::Success,
                                        time,
                                    ),
                                    Err(_) => self.notify(
                                        lang.t(
                                            "No se pudo guardar el archivo.",
                                            "Could not save the file.",
                                        ),
                                        ToastTone::Warning,
                                        time,
                                    ),
                                }
                            }
                        }
                    });
                });
            });

        if add {
            let mut profile = self.ledger.profile().clone();
            let base = lang.t("Perfil", "Profile");
            let mut i = 2;
            let mut name = format!("{} {}", base, i);
            while self.ledger.profiles.iter().any(|p| p.name == name) {
                i += 1;
                name = format!("{} {}", base, i);
            }
            profile.name = name;
            self.ledger.profiles.push(profile);
            self.ledger.active = self.ledger.profiles.len() - 1;
        }
        if let Some(index) = activate {
            self.ledger.active = index;
            self.notify(
                format!(
                    "{}: {}",
                    lang.t("Perfil activo", "Active profile"),
                    self.ledger.profiles[index].name
                ),
                ToastTone::Success,
                time,
            );
        }
        if let Some(index) = rename {
            self.rename_profile = Some(RenameState {
                index,
                name: self.ledger.profiles[index].name.clone(),
            });
        }
        if let Some(index) = duplicate {
            let mut copy = self.ledger.profiles[index].clone();
            let base = copy.name.clone();
            let mut i = 2;
            let mut name = format!("{} {}", base, i);
            while self.ledger.profiles.iter().any(|p| p.name == name) {
                i += 1;
                name = format!("{} {}", base, i);
            }
            copy.name = name;
            self.ledger.profiles.push(copy);
            self.ledger.active = self.ledger.profiles.len() - 1;
        }
        if let Some(index) = remove {
            self.delete_profile = Some(index);
        }
    }

    fn screen_settings(&mut self, ui: &mut Ui, theme: &Theme, time: f64) {
        let lang = self.lang();
        let mut settings = self.ledger.settings.clone();
        let original = settings.clone();
        let available = ui.available_width();
        let left_w = (available * 0.52).max(430.0);

        ui.horizontal_top(|ui| {
            ui.allocate_ui_with_layout(vec2(left_w, 0.0), Layout::top_down(Align::Min), |ui| {
                Frame::NONE
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.stroke_soft))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        section_heading(
                            ui,
                            theme,
                            lang.t("Apariencia", "Appearance"),
                            Some(lang.t(
                                "Elige cómo se ve y se siente la app.",
                                "Choose how the app looks and feels.",
                            )),
                        );
                        ui.add_space(12.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(lang.t("Idioma", "Language"))
                                    .size(11.5)
                                    .color(theme.muted),
                            );
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                for l in [Lang::En, Lang::Es] {
                                    if choice_chip(ui, theme, l.name(), settings.lang == l).clicked() {
                                        settings.lang = l;
                                    }
                                }
                            });
                        });
                        ui.add_space(12.0);
                        ui.label(
                            RichText::new(lang.t("Color de acento", "Accent color"))
                                .size(11.5)
                                .color(theme.muted),
                        );
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            for accent in Accent::ALL {
                                let selected = settings.accent == accent;
                                let (w, _) = ui.allocate_exact_size(vec2(40.0, 40.0), Sense::hover());
                                let response = ui.interact(w, Id::new(("accent", accent)), Sense::click());
                                let c = Theme::new(accent).accent;
                                ui.painter().circle_filled(w.center(), 12.0, c);
                                if selected {
                                    ui.painter().circle_stroke(
                                        w.center(),
                                        16.0,
                                        Stroke::new(2.0, theme.text),
                                    );
                                }
                                if response.hovered() {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                                }
                                if response.clicked() {
                                    settings.accent = accent;
                                }
                            }
                        });
                        ui.add_space(12.0);
                        toggle_setting(
                            ui,
                            theme,
                            lang.t("Animaciones suaves", "Smooth animations"),
                            lang.t("Transiciones y brillos al interactuar.", "Transitions and glows when interacting."),
                            &mut settings.animations,
                        );
                        toggle_setting(
                            ui,
                            theme,
                            lang.t("Mostrar ayudas", "Show tooltips"),
                            lang.t("Pequeñas explicaciones al pasar el ratón.", "Small explanations on hover."),
                            &mut settings.show_tooltips,
                        );
                    });

                ui.add_space(14.0);
                Frame::NONE
                    .fill(theme.card)
                    .stroke(Stroke::new(1.0, theme.stroke_soft))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(18))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        section_heading(
                            ui,
                            theme,
                            lang.t("Comportamiento", "Behavior"),
                            None,
                        );
                        ui.add_space(10.0);
                        toggle_setting(
                            ui,
                            theme,
                            lang.t("Abrir al iniciar", "Open at startup"),
                            lang.t("Deja la app lista desde el arranque.", "Have the app ready from startup."),
                            &mut settings.startup,
                        );
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if secondary_button(ui, theme, lang.t("Copiar ruta de datos", "Copy data path")).clicked()
                            {
                                let path = Ledger::storage_path()
                                    .map(|p| p.display().to_string())
                                    .unwrap_or_default();
                                ui.ctx().copy_text(path);
                                self.notify(
                                    lang.t("Ruta copiada.", "Path copied."),
                                    ToastTone::Info,
                                    time,
                                );
                            }
                            if danger_button(ui, theme, lang.t("Restablecer todo", "Reset everything"))
                                .clicked()
                            {
                                self.reset_confirm = true;
                            }
                        });
                    });
            });

            ui.add_space(16.0);
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 0.0),
                Layout::top_down(Align::Min),
                |ui| {
                    Frame::NONE
                        .fill(mix(theme.card, theme.cyan, 0.04))
                        .stroke(Stroke::new(1.0, with_alpha(theme.cyan, 40)))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(lang.t("Dispositivo", "Device"))
                                        .size(13.5)
                                        .color(theme.text)
                                        .strong(),
                                );
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    if self.hardware.connected() {
                                        status_dot(
                                            ui,
                                            theme.success,
                                            lang.t("Conectado", "Connected"),
                                            theme,
                                        );
                                    } else {
                                        status_dot(
                                            ui,
                                            theme.warning,
                                            lang.t("Vista previa", "Preview"),
                                            theme,
                                        );
                                    }
                                });
                            });
                            ui.add_space(10.0);
                            info_row(ui, theme, lang.t("Modelo", "Model"), "X11 Wireless");
                            info_row(ui, theme, lang.t("Conexión", "Connection"), "2,4 GHz / USB-C");
                            info_row(ui, theme, lang.t("Batería", "Battery"), "82 %");
                            info_row(
                                ui,
                                theme,
                                lang.t("Lectura", "Reading"),
                                &format!("{} s", BATTERY_POLL_MS / 1_000),
                            );
                            info_row(
                                ui,
                                theme,
                                lang.t("Aplicar", "Apply"),
                                &format!("{} / {} ms", SLIDER_APPLY_MS, COMBO_APPLY_MS),
                            );
                            info_row(ui, theme, lang.t("Firmware", "Firmware"), "—");
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(if self.hardware.connected() {
                                    lang.t(
                                        "X11 conectado por HID. Los cambios se envían al ratón al pulsar Aplicar.",
                                        "X11 connected over HID. Changes are sent to the mouse when you press Apply.",
                                    )
                                } else {
                                    lang.t(
                                        "No se detecta el X11. Comprueba el receptor 2,4 GHz o el cable USB-C y vuelve a buscar.",
                                        "X11 not detected. Check the 2.4 GHz receiver or USB-C cable and search again.",
                                    )
                                })
                                .size(9.5)
                                .color(theme.dim),
                            );
                            ui.add_space(8.0);
                            if secondary_button(ui, theme, lang.t("Buscar dispositivo", "Find device")).clicked()
                            {
                                if self.hardware.connect() {
                                    let profile = self.ledger.profile().clone();
                                    let result = self.hardware.apply_profile(&profile);
                                    match result {
                                        Ok(()) => self.notify(
                                            lang.t(
                                                "X11 conectado. Configuración enviada.",
                                                "X11 connected. Settings sent.",
                                            ),
                                            ToastTone::Success,
                                            time,
                                        ),
                                        Err(err) => self.notify(
                                            format!(
                                                "{} ({err})",
                                                lang.t(
                                                    "X11 conectado, pero no se pudo aplicar.",
                                                    "X11 connected, but applying failed."
                                                )
                                            ),
                                            ToastTone::Warning,
                                            time,
                                        ),
                                    }
                                } else {
                                    self.notify(
                                        lang.t(
                                            "No se encontró el X11. Revisa la conexión.",
                                            "X11 not found. Check the connection.",
                                        ),
                                        ToastTone::Warning,
                                        time,
                                    );
                                }
                            }
                        });

                    ui.add_space(14.0);
                    Frame::NONE
                        .fill(theme.card)
                        .stroke(Stroke::new(1.0, theme.stroke_soft))
                        .corner_radius(CornerRadius::same(18))
                        .inner_margin(Margin::same(18))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.horizontal(|ui| {
                                let (mark, _) = ui.allocate_exact_size(vec2(44.0, 44.0), Sense::hover());
                                theme::app_mark(ui.painter(), mark, theme);
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("X11 Studio")
                                            .size(15.0)
                                            .color(theme.text)
                                            .strong(),
                                    );
                                    ui.label(
                                        RichText::new(format!(
                                            "{} {}",
                                            lang.t("Versión", "Version"),
                                            APP_VERSION
                                        ))
                                        .size(10.0)
                                        .color(theme.muted),
                                    );
                                });
                            });
                            ui.add_space(10.0);
                            ui.label(
                                RichText::new(lang.t(
                                    "Una interfaz nueva para el software de configuración del X11, reconstruida desde cero.",
                                    "A new interface for the X11 configuration software, rebuilt from scratch.",
                                ))
                                .size(10.0)
                                .color(theme.muted),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                RichText::new(lang.t(
                                    "Atajos: Ctrl+S aplicar · Ctrl+Z descartar · Esc cerrar",
                                    "Shortcuts: Ctrl+S apply · Ctrl+Z discard · Esc close",
                                ))
                                .size(9.0)
                                .color(theme.dim),
                            );
                        });
                },
            );
        });

        if settings != original {
            self.ledger.settings = settings;
            let _ = self.ledger.save();
        }
        let _ = available;
    }

    fn draw_modals(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        self.draw_command_picker(ctx, theme, time);
        self.draw_shortcut_editor(ctx, theme);
        self.draw_rename(ctx, theme, time);
        self.draw_delete_profile(ctx, theme, time);
        self.draw_delete_macro(ctx, theme, time);
        self.draw_reset(ctx, theme, time);
    }

    fn draw_command_picker(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        let Some(picker) = self.picker.as_ref() else {
            return;
        };
        let lang = self.lang();
        let button = picker.button;
        let names = button_names(lang);
        let macros = self.ledger.macros.clone();
        let current = self.ledger.profile().buttons[button].command.clone();
        let mut chosen: Option<Command> = None;
        let mut custom_shortcut = false;
        let mut open = true;
        let mut search = picker.search.clone();
        let mut category = picker.category;

        egui::Window::new(format!(
            "{} · {}",
            lang.t("Asignar acción", "Assign action"),
            names[button]
        ))
        .title_bar(false)
        .title_bar(false)
        .collapsible(false)
        .resizable(false)
        .default_width(560.0)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(
            Frame::NONE
                .fill(theme.panel)
                .stroke(Stroke::new(1.0, theme.stroke))
                .corner_radius(CornerRadius::same(18))
                .inner_margin(Margin::same(20)),
        )
        .show(ctx, |ui| {
            ui.set_width(540.0);
            modal_header(
                ui,
                theme,
                &format!(
                    "{} · {}",
                    lang.t("Asignar acción", "Assign action"),
                    names[button]
                ),
                lang.t(
                    "Elige una acción o busca por nombre.",
                    "Pick an action or search by name.",
                ),
                &mut open,
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                ui.add_sized(
                    [ui.available_width() - 112.0, 34.0],
                    TextEdit::singleline(&mut search)
                        .hint_text(lang.t("Buscar acción…", "Search actions…"))
                        .margin(Margin::symmetric(10, 7)),
                );
                if secondary_button(ui, theme, lang.t("Atajo…", "Shortcut…")).clicked() {
                    custom_shortcut = true;
                }
            });
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                if choice_chip(ui, theme, lang.t("Todas", "All"), category.is_none()).clicked() {
                    category = None;
                }
                for cat in CmdCategory::ALL {
                    if choice_chip(ui, theme, cat.label(lang), category == Some(cat)).clicked() {
                        category = Some(cat);
                    }
                }
                if choice_chip(
                    ui,
                    theme,
                    lang.t("Macros", "Macros"),
                    category == Some(CmdCategory::Macro),
                )
                .clicked()
                {
                    category = Some(CmdCategory::Macro);
                }
            });
            ui.add_space(6.0);
            theme::divider(ui, theme);
            ui.add_space(4.0);
            let needle = search.to_lowercase();
            let mut items = Command::all();
            for m in &macros {
                items.push(Command::Macro {
                    name: m.name.clone(),
                    mode: m.mode.clone(),
                    loops: m.loops,
                });
            }
            ScrollArea::vertical()
                .id_salt("command_picker_scroll")
                .max_height(390.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    for cmd in items {
                        if let Some(cat) = category {
                            if cmd.category() != cat {
                                continue;
                            }
                        }
                        let label = cmd.label(lang);
                        if !needle.is_empty() && !label.to_lowercase().contains(&needle) {
                            continue;
                        }
                        let selected = cmd == current;
                        let meta = format!("0x{:02X} · {}", cmd.code(), cmd.category().label(lang));
                        if command_row(ui, theme, &label, &meta, selected) {
                            chosen = Some(cmd);
                        }
                    }
                });
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        macros.len(),
                        lang.t("macros disponibles", "macros available")
                    ))
                    .size(9.0)
                    .color(theme.dim),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if ghost_button(ui, theme, lang.t("Cancelar", "Cancel")).clicked() {
                        open = false;
                    }
                });
            });
        });

        if let Some(picker) = self.picker.as_mut() {
            picker.search = search;
            picker.category = category;
        }
        if let Some(command) = chosen {
            let keeps_left = matches!(command, Command::LeftClick)
                || self
                    .ledger
                    .profile()
                    .buttons
                    .iter()
                    .enumerate()
                    .any(|(i, b)| i != button && matches!(b.command, Command::LeftClick));
            if keeps_left {
                self.ledger.profile_mut().buttons[button].command = command;
            } else {
                self.notify(
                    lang.t(
                        "Al menos un botón debe ser «Clic izquierdo».",
                        "At least one button must be \"Left click\".",
                    ),
                    ToastTone::Warning,
                    time,
                );
            }
            self.picker = None;
            return;
        }
        if custom_shortcut {
            self.shortcut_editor = Some(ShortcutDraft {
                button,
                ctrl: false,
                shift: false,
                alt: false,
                win: false,
                key: "A".into(),
            });
            self.picker = None;
            return;
        }
        if !open {
            self.picker = None;
        }
    }

    fn draw_shortcut_editor(&mut self, ctx: &egui::Context, theme: &Theme) {
        let lang = self.lang();
        let Some(draft) = self.shortcut_editor.as_mut() else {
            return;
        };
        let mut open = true;
        let mut save = false;
        egui::Window::new(lang.t("Atajo personalizado", "Custom shortcut"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, theme.stroke))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(20)),
            )
            .show(ctx, |ui| {
                ui.set_width(400.0);
                modal_header(
                    ui,
                    theme,
                    lang.t("Atajo personalizado", "Custom shortcut"),
                    lang.t(
                        "Combina modificadores con una tecla.",
                        "Combine modifiers with a key.",
                    ),
                    &mut open,
                );
                ui.add_space(4.0);
                ui.horizontal_wrapped(|ui| {
                    if choice_chip(ui, theme, "Ctrl", draft.ctrl).clicked() {
                        draft.ctrl = !draft.ctrl;
                    }
                    if choice_chip(ui, theme, "Shift", draft.shift).clicked() {
                        draft.shift = !draft.shift;
                    }
                    if choice_chip(ui, theme, "Alt", draft.alt).clicked() {
                        draft.alt = !draft.alt;
                    }
                    if choice_chip(ui, theme, "Win", draft.win).clicked() {
                        draft.win = !draft.win;
                    }
                });
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(lang.t("Tecla", "Key"))
                            .size(11.0)
                            .color(theme.muted),
                    );
                    ui.add_sized(
                        [160.0, 32.0],
                        TextEdit::singleline(&mut draft.key)
                            .hint_text("A, F5, Space…")
                            .margin(Margin::symmetric(8, 6)),
                    );
                });
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, theme, lang.t("Guardar atajo", "Save shortcut")).clicked()
                    {
                        save = true;
                    }
                    if ghost_button(ui, theme, lang.t("Cancelar", "Cancel")).clicked() {
                        open = false;
                    }
                });
            });
        if save {
            if draft.key.trim().is_empty() {
                self.shortcut_editor = None;
                return;
            }
            let command = Command::Shortcut {
                ctrl: draft.ctrl,
                shift: draft.shift,
                alt: draft.alt,
                win: draft.win,
                key: draft.key.trim().to_string(),
            };
            let button = draft.button;
            self.ledger.profile_mut().buttons[button].command = command;
            self.shortcut_editor = None;
        } else if !open {
            self.shortcut_editor = None;
        }
    }

    fn draw_rename(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        let lang = self.lang();
        let Some(state) = self.rename_profile.as_mut() else {
            return;
        };
        let mut open = true;
        let mut save = false;
        egui::Window::new(lang.t("Renombrar perfil", "Rename profile"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .default_width(380.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, theme.stroke))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(20)),
            )
            .show(ctx, |ui| {
                ui.set_width(360.0);
                modal_header(
                    ui,
                    theme,
                    lang.t("Renombrar perfil", "Rename profile"),
                    lang.t("Elige un nombre único.", "Choose a unique name."),
                    &mut open,
                );
                ui.add_space(4.0);
                ui.add(
                    TextEdit::singleline(&mut state.name)
                        .hint_text(lang.t("Nombre del perfil", "Profile name"))
                        .margin(Margin::symmetric(10, 8)),
                );
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if primary_button(ui, theme, lang.t("Guardar", "Save")).clicked() {
                        save = true;
                    }
                    if ghost_button(ui, theme, lang.t("Cancelar", "Cancel")).clicked() {
                        open = false;
                    }
                });
            });
        if save {
            let index = state.index;
            let name = state.name.trim().to_string();
            if name.is_empty() {
                self.notify(
                    lang.t("El nombre no puede estar vacío.", "Name cannot be empty."),
                    ToastTone::Warning,
                    time,
                );
            } else if self
                .ledger
                .profiles
                .iter()
                .enumerate()
                .any(|(i, p)| i != index && p.name.eq_ignore_ascii_case(&name))
            {
                self.notify(
                    lang.t("Ese nombre ya existe.", "That name already exists."),
                    ToastTone::Warning,
                    time,
                );
            } else {
                self.ledger.profiles[index].name = name;
                self.rename_profile = None;
            }
        } else if !open {
            self.rename_profile = None;
        }
    }

    fn draw_delete_profile(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        let Some(index) = self.delete_profile else {
            return;
        };
        let lang = self.lang();
        let name = self
            .ledger
            .profiles
            .get(index)
            .map(|p| p.name.clone())
            .unwrap_or_default();
        let mut open = true;
        let mut confirm = false;
        egui::Window::new(lang.t("Eliminar perfil", "Delete profile"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, theme.stroke))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(20)),
            )
            .show(ctx, |ui| {
                ui.set_width(380.0);
                modal_header(
                    ui,
                    theme,
                    lang.t("Eliminar perfil", "Delete profile"),
                    &format!(
                        "{} «{}»?",
                        lang.t("¿Eliminar el perfil", "Delete the profile"),
                        name
                    ),
                    &mut open,
                );
                ui.label(
                    RichText::new(lang.t(
                        "Esta acción no se puede deshacer.",
                        "This action cannot be undone.",
                    ))
                    .size(10.0)
                    .color(theme.muted),
                );
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if danger_button(ui, theme, lang.t("Eliminar", "Delete")).clicked() {
                        confirm = true;
                    }
                    if ghost_button(ui, theme, lang.t("Cancelar", "Cancel")).clicked() {
                        open = false;
                    }
                });
            });
        if confirm {
            if self.ledger.profiles.len() <= 1 {
                self.notify(
                    lang.t(
                        "Debe quedar al menos un perfil.",
                        "At least one profile must remain.",
                    ),
                    ToastTone::Warning,
                    time,
                );
            } else {
                self.ledger.profiles.remove(index);
                if self.ledger.active >= self.ledger.profiles.len() {
                    self.ledger.active = self.ledger.profiles.len() - 1;
                }
                self.notify(
                    lang.t("Perfil eliminado.", "Profile deleted."),
                    ToastTone::Info,
                    time,
                );
            }
            self.delete_profile = None;
        } else if !open {
            self.delete_profile = None;
        }
    }

    fn draw_delete_macro(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        let Some(index) = self.delete_macro else {
            return;
        };
        let lang = self.lang();
        let name = self
            .ledger
            .macros
            .get(index)
            .map(|m| m.name.clone())
            .unwrap_or_default();
        let mut open = true;
        let mut confirm = false;
        egui::Window::new(lang.t("Eliminar macro", "Delete macro"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .default_width(400.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, theme.stroke))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(20)),
            )
            .show(ctx, |ui| {
                ui.set_width(380.0);
                modal_header(
                    ui,
                    theme,
                    lang.t("Eliminar macro", "Delete macro"),
                    &format!(
                        "{} «{}»?",
                        lang.t("¿Eliminar la macro", "Delete the macro"),
                        name
                    ),
                    &mut open,
                );
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if danger_button(ui, theme, lang.t("Eliminar", "Delete")).clicked() {
                        confirm = true;
                    }
                    if ghost_button(ui, theme, lang.t("Cancelar", "Cancel")).clicked() {
                        open = false;
                    }
                });
            });
        if confirm {
            if self.ledger.macros.len() <= 1 {
                self.notify(
                    lang.t(
                        "Debe quedar al menos una macro.",
                        "At least one macro must remain.",
                    ),
                    ToastTone::Warning,
                    time,
                );
            } else {
                self.ledger.macros.remove(index);
                if self.selected_macro >= self.ledger.macros.len() {
                    self.selected_macro = self.ledger.macros.len() - 1;
                }
                self.notify(
                    lang.t("Macro eliminada.", "Macro deleted."),
                    ToastTone::Info,
                    time,
                );
            }
            self.delete_macro = None;
        } else if !open {
            self.delete_macro = None;
        }
    }

    fn draw_reset(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        if !self.reset_confirm {
            return;
        }
        let lang = self.lang();
        let mut open = true;
        let mut confirm = false;
        egui::Window::new(lang.t("Restablecer todo", "Reset everything"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
            .frame(
                Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, theme.stroke))
                    .corner_radius(CornerRadius::same(18))
                    .inner_margin(Margin::same(20)),
            )
            .show(ctx, |ui| {
                ui.set_width(400.0);
                modal_header(
                    ui,
                    theme,
                    lang.t("Restablecer todo", "Reset everything"),
                    lang.t(
                        "Se restaurarán los perfiles, los botones, el DPI, la luz y las macros.",
                        "Profiles, buttons, DPI, lighting and macros will be restored.",
                    ),
                    &mut open,
                );
                ui.label(
                    RichText::new(lang.t(
                        "Tus ajustes de apariencia se conservan.",
                        "Your appearance settings are kept.",
                    ))
                    .size(10.0)
                    .color(theme.muted),
                );
                ui.add_space(14.0);
                ui.horizontal(|ui| {
                    if danger_button(ui, theme, lang.t("Restablecer", "Reset")).clicked() {
                        confirm = true;
                    }
                    if ghost_button(ui, theme, lang.t("Cancelar", "Cancel")).clicked() {
                        open = false;
                    }
                });
            });
        if confirm {
            let settings = self.ledger.settings.clone();
            let mut fresh = Ledger::defaults(settings.lang);
            fresh.settings = settings;
            self.ledger = fresh;
            self.saved = self.ledger.clone();
            let _ = self.ledger.save();
            self.selected_button = 0;
            self.selected_dpi = 0;
            self.selected_macro = 0;
            self.notify(
                lang.t("Todo restablecido.", "Everything reset."),
                ToastTone::Success,
                time,
            );
            self.reset_confirm = false;
        } else if !open {
            self.reset_confirm = false;
        }
    }

    fn draw_toast(&mut self, ctx: &egui::Context, theme: &Theme, time: f64) {
        let Some(toast) = self.toast.as_ref() else {
            return;
        };
        let age = time - toast.born;
        if age > 3.8 {
            self.toast = None;
            return;
        }
        let fade = if age < 0.18 {
            (age / 0.18) as f32
        } else if age > 3.2 {
            ((3.8 - age) / 0.6).clamp(0.0, 1.0) as f32
        } else {
            1.0
        };
        let color = match toast.tone {
            ToastTone::Success => theme.success,
            ToastTone::Warning => theme.warning,
            ToastTone::Info => theme.cyan,
        };
        egui::Area::new(Id::new("toast_area"))
            .anchor(Align2::RIGHT_BOTTOM, vec2(-24.0, -46.0))
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                ui.set_opacity(fade);
                Frame::NONE
                    .fill(theme.panel)
                    .stroke(Stroke::new(1.0, with_alpha(color, 150)))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let (dot, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                            ui.painter().circle_filled(dot.center(), 3.5, color);
                            ui.label(RichText::new(&toast.text).size(11.5).color(theme.text));
                        });
                    });
            });
        ctx.request_repaint();
    }
}

impl Drop for X11App {
    fn drop(&mut self) {
        if !self.screenshot_mode {
            let _ = self.ledger.save();
        }
    }
}

impl eframe::App for X11App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.frames += 1;
        let ctx = ui.ctx().clone();
        let time = ctx.input(|i| i.time);
        if self.started == 0.0 {
            self.started = time;
        }
        let theme = Theme::new(self.ledger.settings.accent);
        if self.ledger.settings.accent != self.last_accent {
            theme::install_style(&ctx, &theme);
            self.last_accent = self.ledger.settings.accent;
        }
        self.handle_shortcuts(&ctx, time);
        self.handle_recording(&ctx, time);
        self.draw_main(ui, &theme, time);
        self.draw_modals(&ctx, &theme, time);
        self.draw_toast(&ctx, &theme, time);
        self.handle_screenshot(&ctx);
        if self.ledger.settings.animations || self.shot.is_some() {
            ctx.request_repaint();
        } else {
            ctx.request_repaint_after(std::time::Duration::from_millis(120));
        }
    }

    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        if !self.screenshot_mode {
            let _ = self.ledger.save();
        }
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        Theme::new(self.ledger.settings.accent)
            .bg
            .to_normalized_gamma_f32()
    }
}

// ---------------------------------------------------------------------------
// Local widgets and small paint helpers
// ---------------------------------------------------------------------------

pub fn fmt_count(value: u32) -> String {
    let s = value.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(' ');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn modal_header(ui: &mut Ui, theme: &Theme, title: &str, subtitle: &str, open: &mut bool) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(18.0).color(theme.text).strong());
            ui.label(RichText::new(subtitle).size(10.5).color(theme.muted));
        });
        let width = ui.available_width();
        ui.allocate_ui_with_layout(
            vec2(width, 30.0),
            Layout::right_to_left(Align::Center),
            |ui| {
                if icon_button(ui, theme, "×", "").clicked() {
                    *open = false;
                }
            },
        );
    });
    ui.add_space(10.0);
    theme::divider(ui, theme);
}

fn command_row(ui: &mut Ui, theme: &Theme, label: &str, category: &str, selected: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
    let hover = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(9),
        if selected {
            with_alpha(theme.accent, 16)
        } else if hover {
            theme.card_hover
        } else {
            Color32::TRANSPARENT
        },
    );
    if selected {
        ui.painter().rect_filled(
            Rect::from_min_size(pos2(rect.left(), rect.center().y - 10.0), vec2(3.0, 20.0)),
            CornerRadius::same(2),
            theme.accent,
        );
    }
    let mark = pos2(rect.left() + 20.0, rect.center().y);
    if selected {
        ui.painter()
            .circle_filled(mark, 9.0, with_alpha(theme.accent, 30));
        ui.painter().text(
            mark,
            Align2::CENTER_CENTER,
            "✓",
            FontId::proportional(10.0),
            theme.accent,
        );
    } else {
        ui.painter()
            .circle_stroke(mark, 8.0, Stroke::new(1.0, theme.stroke));
    }
    ui.painter().text(
        pos2(rect.left() + 38.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(12.0),
        if selected { theme.text } else { theme.muted },
    );
    ui.painter().text(
        pos2(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        category,
        FontId::proportional(9.0),
        theme.dim,
    );
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn hero_metric(ui: &mut Ui, theme: &Theme, label: &str, value: &str) {
    Frame::NONE
        .fill(with_alpha(theme.text, 10))
        .stroke(Stroke::new(1.0, with_alpha(theme.text, 18)))
        .corner_radius(CornerRadius::same(10))
        .inner_margin(Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(RichText::new(label).size(7.5).color(theme.dim));
                ui.label(RichText::new(value).size(12.0).color(theme.text));
            });
        });
}

fn quick_action(
    ui: &mut Ui,
    theme: &Theme,
    size: Vec2,
    icon: &str,
    title: &str,
    subtitle: &str,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hover = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(14),
        if hover { theme.card_hover } else { theme.card },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(14),
        Stroke::new(
            1.0,
            if hover {
                with_alpha(theme.accent, 110)
            } else {
                theme.stroke_soft
            },
        ),
        StrokeKind::Inside,
    );
    let circle = pos2(rect.left() + 24.0, rect.center().y);
    ui.painter()
        .circle_filled(circle, 14.0, with_alpha(theme.accent, 26));
    ui.painter().text(
        circle,
        Align2::CENTER_CENTER,
        icon,
        FontId::proportional(11.0),
        theme.accent,
    );
    ui.painter().text(
        pos2(rect.left() + 46.0, rect.center().y - 8.0),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(12.0),
        theme.text,
    );
    ui.painter().text(
        pos2(rect.left() + 46.0, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        subtitle,
        FontId::proportional(9.0),
        theme.dim,
    );
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn info_row(ui: &mut Ui, theme: &Theme, label: &str, value: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(10.5).color(theme.dim));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(value).size(10.5).color(theme.text));
        });
    });
}

fn performance_bar(
    ui: &mut Ui,
    theme: &Theme,
    label: &str,
    value: f32,
    right: &str,
    color: Color32,
) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(10.0).color(theme.muted));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(RichText::new(right).size(9.5).color(theme.text));
        });
    });
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 5.0), Sense::hover());
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(3),
        Color32::from_rgb(0x26, 0x30, 0x3D),
    );
    ui.painter().rect_filled(
        Rect::from_min_size(
            rect.min,
            vec2(rect.width() * value.clamp(0.0, 1.0), rect.height()),
        ),
        CornerRadius::same(3),
        color,
    );
}

fn button_row(
    ui: &mut Ui,
    theme: &Theme,
    index: usize,
    name: &str,
    hardware: u8,
    action: &str,
    category: &str,
    code: u8,
    selected: bool,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 58.0), Sense::click());
    let hover = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(12),
        if selected {
            with_alpha(theme.accent, 18)
        } else if hover {
            with_alpha(theme.text, 6)
        } else {
            Color32::TRANSPARENT
        },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(
            1.0,
            if selected {
                with_alpha(theme.accent, 120)
            } else if hover {
                theme.stroke
            } else {
                theme.stroke_soft
            },
        ),
        StrokeKind::Inside,
    );
    let badge = pos2(rect.left() + 26.0, rect.center().y);
    ui.painter().circle_filled(
        badge,
        14.0,
        if selected {
            theme.accent
        } else {
            with_alpha(theme.text, 10)
        },
    );
    ui.painter().text(
        badge,
        Align2::CENTER_CENTER,
        format!("{}", index + 1),
        FontId::proportional(11.0),
        if selected {
            theme.on_accent
        } else {
            theme.text
        },
    );
    ui.painter().text(
        pos2(rect.left() + 48.0, rect.center().y - 9.0),
        Align2::LEFT_CENTER,
        format!("{name}  ·  HID {hardware}"),
        FontId::proportional(9.0),
        theme.dim,
    );
    ui.painter().text(
        pos2(rect.left() + 48.0, rect.center().y + 9.0),
        Align2::LEFT_CENTER,
        action,
        FontId::proportional(12.0),
        theme.text,
    );
    ui.painter().text(
        pos2(rect.right() - 14.0, rect.center().y),
        Align2::RIGHT_CENTER,
        format!("0x{code:02X} · {category}"),
        FontId::proportional(9.0),
        theme.muted,
    );
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn choice_chip(ui: &mut Ui, theme: &Theme, text: &str, selected: bool) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(10.5).color(if selected {
            theme.on_accent
        } else {
            theme.muted
        }))
        .fill(if selected {
            theme.accent
        } else {
            with_alpha(theme.text, 6)
        })
        .stroke(Stroke::new(
            1.0,
            if selected {
                theme.accent
            } else {
                theme.stroke_soft
            },
        ))
        .corner_radius(CornerRadius::same(9))
        .min_size(vec2(0.0, 28.0)),
    )
}

fn outline_action(ui: &mut Ui, theme: &Theme, text: &str) -> egui::Response {
    ui.add(
        egui::Button::new(RichText::new(text).size(10.0).color(theme.muted))
            .fill(Color32::TRANSPARENT)
            .stroke(Stroke::new(1.0, theme.stroke))
            .corner_radius(CornerRadius::same(8))
            .min_size(vec2(0.0, 28.0)),
    )
}

fn toggle_setting(ui: &mut Ui, theme: &Theme, title: &str, subtitle: &str, value: &mut bool) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(11.5).color(theme.text));
            ui.label(RichText::new(subtitle).size(9.0).color(theme.dim));
        });
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            toggle(ui, theme, value);
        });
    });
}

fn timer_setting(ui: &mut Ui, theme: &Theme, label: &str, value: &mut u8) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(label).size(11.0).color(theme.muted));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(format!("{} min", value))
                    .size(11.0)
                    .color(theme.text),
            );
        });
    });
    let mut v = *value as u32;
    let _ = theme::slider_u32(ui, theme, &mut v, 1, 60, ui.available_width());
    *value = v.clamp(1, 60) as u8;
}

fn dpi_stage_card(ui: &mut Ui, theme: &Theme, lang: Lang, index: usize, stage: &mut DpiStage) {
    Frame::NONE
        .fill(if stage.enabled {
            theme.card
        } else {
            with_alpha(theme.card, 150)
        })
        .stroke(Stroke::new(
            1.0,
            if stage.enabled {
                with_alpha(
                    Color32::from_rgb(stage.color[0], stage.color[1], stage.color[2]),
                    80,
                )
            } else {
                theme.stroke_soft
            },
        ))
        .corner_radius(CornerRadius::same(15))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{} {}", lang.t("Etapa", "Stage"), index + 1))
                        .size(10.5)
                        .color(theme.muted),
                );
                let controls_w = ui.available_width();
                ui.allocate_ui_with_layout(
                    vec2(controls_w, 24.0),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        toggle(ui, theme, &mut stage.enabled);
                        ui.color_edit_button_srgb(&mut stage.color);
                    },
                );
            });
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(fmt_count(stage.dpi))
                        .size(24.0)
                        .color(if stage.enabled { theme.text } else { theme.dim })
                        .strong(),
                );
                ui.label(RichText::new("DPI").size(9.5).color(theme.dim));
                let controls_w = ui.available_width();
                ui.allocate_ui_with_layout(
                    vec2(controls_w, 24.0),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        if icon_button(ui, theme, "+", "+50 / +100").clicked() {
                            let pos = dpi_to_position(stage.dpi).saturating_add(1);
                            stage.dpi = position_to_dpi(pos);
                        }
                        if icon_button(ui, theme, "−", "−50 / −100").clicked() {
                            let pos = dpi_to_position(stage.dpi).saturating_sub(1).max(1);
                            stage.dpi = position_to_dpi(pos);
                        }
                    },
                );
            });
            ui.add_space(2.0);
            let mut pos = dpi_to_position(stage.dpi);
            let resp = theme::slider_u32(
                ui,
                theme,
                &mut pos,
                1,
                DPI_POSITION_MAX,
                ui.available_width(),
            );
            if resp.changed() {
                stage.dpi = position_to_dpi(pos);
            }
            ui.horizontal(|ui| {
                ui.label(RichText::new("50").size(8.5).color(theme.dim));
                let status_w = ui.available_width();
                ui.allocate_ui_with_layout(
                    vec2(status_w, 12.0),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        ui.label(RichText::new("22 000").size(8.5).color(theme.dim));
                        if !stage.enabled {
                            ui.label(
                                RichText::new(lang.t("Oculta", "Hidden"))
                                    .size(8.0)
                                    .color(theme.dim),
                            );
                        }
                    },
                );
            });
        });
}

fn apply_dpi_preset(stages: &mut [DpiStage], values: &[u32]) {
    for (i, stage) in stages.iter_mut().enumerate() {
        match values.get(i) {
            Some(v) => {
                stage.dpi = *v;
                stage.enabled = true;
            }
            // Presets use six stages; slots 7 and 8 stay hidden like the original.
            None => stage.enabled = false,
        }
    }
}

fn lighting_preview(light: &LightSettings, stages: &[DpiStage], time: f64) -> (Color32, f32) {
    let base = Color32::from_rgb(light.color[0], light.color[1], light.color[2]);
    let pulse = ((time * (light.speed as f64 * 0.55)).sin() * 0.5 + 0.5) as f32;
    match light.mode {
        0 => (base, 0.0),
        1 => (base, 0.8),
        2 => (base, 0.28 + pulse * 0.7),
        3 => (mix(base, Color32::WHITE, 0.35 + pulse * 0.35), 0.92),
        4 => {
            let rgb = hsv_to_rgb((time * 0.055) % 1.0, 0.85, 1.0);
            (
                Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
                0.35 + pulse * 0.65,
            )
        }
        5 => {
            let stage = stages
                .get((time as usize / 2) % stages.len().max(1))
                .or_else(|| stages.first());
            match stage {
                Some(s) => (Color32::from_rgb(s.color[0], s.color[1], s.color[2]), 0.82),
                None => (base, 0.8),
            }
        }
        6 => {
            let stage = stages
                .get((time as usize / 2) % stages.len().max(1))
                .or_else(|| stages.first());
            match stage {
                Some(s) => (
                    Color32::from_rgb(s.color[0], s.color[1], s.color[2]),
                    0.25 + pulse * 0.75,
                ),
                None => (base, 0.25 + pulse * 0.75),
            }
        }
        _ => {
            let rgb = hsv_to_rgb((time * 0.11) % 1.0, 0.82, 1.0);
            (Color32::from_rgb(rgb[0], rgb[1], rgb[2]), 0.92)
        }
    }
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> [u8; 3] {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    let (r, g, b) = match (i as i32).rem_euclid(6) {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    [(r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8]
}

fn mode_card(
    ui: &mut Ui,
    theme: &Theme,
    mode: usize,
    label: &str,
    selected: bool,
    time: f64,
) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(ui.available_width(), 48.0), Sense::click());
    let hover = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(11),
        if selected {
            with_alpha(theme.accent, 20)
        } else if hover {
            theme.card_hover
        } else {
            with_alpha(theme.text, 5)
        },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(11),
        Stroke::new(
            1.0,
            if selected {
                with_alpha(theme.accent, 150)
            } else {
                theme.stroke_soft
            },
        ),
        StrokeKind::Inside,
    );
    let color = match mode {
        0 => theme.dim,
        1 => theme.accent,
        2 => theme.cyan,
        3 => Color32::from_rgb(0xFF, 0xD1, 0x66),
        4 => Color32::from_rgb(0xA7, 0x8B, 0xFA),
        5 => theme.success,
        6 => Color32::from_rgb(0xFF, 0x9F, 0x43),
        _ => {
            let (c, _) = lighting_preview(
                &LightSettings {
                    mode: 7,
                    brightness: 8,
                    speed: 6,
                    color: [0xFF, 0x5C, 0x4D],
                },
                &[],
                time,
            );
            c
        }
    };
    ui.painter()
        .circle_filled(pos2(rect.left() + 18.0, rect.center().y), 4.0, color);
    ui.painter().text(
        pos2(rect.left() + 32.0, rect.center().y),
        Align2::LEFT_CENTER,
        label,
        FontId::proportional(10.5),
        if selected { theme.text } else { theme.muted },
    );
    if selected {
        ui.painter().text(
            pos2(rect.right() - 12.0, rect.center().y),
            Align2::RIGHT_CENTER,
            "●",
            FontId::proportional(7.0),
            theme.accent,
        );
    }
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn color_swatch(ui: &mut Ui, theme: &Theme, rgb: [u8; 3], selected: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(vec2(24.0, 24.0), Sense::click());
    let c = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
    ui.painter().rect_filled(rect, CornerRadius::same(7), c);
    if selected {
        ui.painter().rect_stroke(
            rect.expand(2.0),
            CornerRadius::same(8),
            Stroke::new(1.5, theme.text),
            StrokeKind::Outside,
        );
    } else if response.hovered() {
        ui.painter().rect_stroke(
            rect,
            CornerRadius::same(7),
            Stroke::new(1.0, theme.text),
            StrokeKind::Inside,
        );
    }
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response.clicked()
}

fn polling_card(
    ui: &mut Ui,
    theme: &Theme,
    lang: Lang,
    size: Vec2,
    index: usize,
    hz: u32,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let hover = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(13),
        if selected {
            with_alpha(theme.accent, 20)
        } else if hover {
            theme.card_hover
        } else {
            with_alpha(theme.text, 5)
        },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(13),
        Stroke::new(
            1.0,
            if selected {
                with_alpha(theme.accent, 150)
            } else {
                theme.stroke_soft
            },
        ),
        StrokeKind::Inside,
    );
    ui.painter().text(
        pos2(rect.center().x, rect.top() + 26.0),
        Align2::CENTER_CENTER,
        format!("{hz}"),
        FontId::proportional(23.0),
        if selected { theme.text } else { theme.muted },
    );
    ui.painter().text(
        pos2(rect.center().x, rect.top() + 42.0),
        Align2::CENTER_CENTER,
        format!("Hz · {} ms", POLLING_REPORT_CODES[index.min(3)]),
        FontId::proportional(9.0),
        theme.dim,
    );
    let label = match index {
        0 => lang.t("Ahorro de energía", "Power Saving"),
        1 => lang.t("Oficina", "Office"),
        2 => lang.t("Juego", "Gaming"),
        _ => "E-sports",
    };
    ui.painter().text(
        pos2(rect.center().x, rect.bottom() - 18.0),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(9.0),
        if selected { theme.accent } else { theme.dim },
    );
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

struct MacroRowResult {
    up: bool,
    down: bool,
    delete: bool,
}

fn macro_action_row(
    ui: &mut Ui,
    theme: &Theme,
    lang: Lang,
    index: usize,
    action: &mut MacroAction,
) -> MacroRowResult {
    let mut result = MacroRowResult {
        up: false,
        down: false,
        delete: false,
    };
    Frame::NONE
        .fill(with_alpha(theme.text, 5))
        .stroke(Stroke::new(1.0, theme.stroke_soft))
        .corner_radius(CornerRadius::same(9))
        .inner_margin(Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new(format!("{:02}", index + 1))
                        .size(9.0)
                        .color(theme.dim),
                );
                ui.add_sized(
                    [54.0, 22.0],
                    egui::Label::new(
                        RichText::new(action.kind(lang))
                            .size(9.0)
                            .color(theme.accent),
                    )
                    .truncate(),
                );
                match action {
                    MacroAction::KeyDown(key) | MacroAction::KeyUp(key) => {
                        ui.add_sized(
                            [74.0, 24.0],
                            TextEdit::singleline(key).margin(Margin::symmetric(6, 4)),
                        );
                    }
                    MacroAction::MouseDown(btn) | MacroAction::MouseUp(btn) => {
                        ComboBox::from_id_salt(("macro_mouse", index))
                            .width(92.0)
                            .selected_text(btn.clone())
                            .show_ui(ui, |ui| {
                                for b in ["Izq", "Der", "Medio", "Adelante", "Atrás"] {
                                    ui.selectable_value(btn, b.to_string(), b);
                                }
                            });
                    }
                    MacroAction::Delay(ms) => {
                        ui.add(
                            egui::DragValue::new(ms)
                                .range(1..=10_000)
                                .speed(5.0)
                                .suffix(" ms"),
                        );
                    }
                    MacroAction::Move(x, y) => {
                        ui.add(egui::DragValue::new(x).range(-1000..=1000).speed(1.0));
                        ui.label(RichText::new(",").size(9.0).color(theme.dim));
                        ui.add(egui::DragValue::new(y).range(-1000..=1000).speed(1.0));
                    }
                    MacroAction::Wheel(delta) => {
                        ui.add(egui::DragValue::new(delta).range(-10..=10).speed(1.0));
                    }
                }
                let controls_w = ui.available_width();
                ui.allocate_ui_with_layout(
                    vec2(controls_w, 24.0),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        if icon_button(ui, theme, "�", lang.t("Eliminar", "Delete")).clicked() {
                            result.delete = true;
                        }
                        if icon_button(ui, theme, "?", lang.t("Bajar", "Move down")).clicked() {
                            result.down = true;
                        }
                        if icon_button(ui, theme, "?", lang.t("Subir", "Move up")).clicked() {
                            result.up = true;
                        }
                    },
                );
            });
        });
    result
}

fn profile_card(
    ui: &mut Ui,
    theme: &Theme,
    profile: &Profile,
    active: bool,
    lang: Lang,
    index: usize,
    activate: &mut Option<usize>,
    rename: &mut Option<usize>,
    duplicate: &mut Option<usize>,
    remove: &mut Option<usize>,
) {
    Frame::NONE
        .fill(if active {
            mix(theme.card, theme.accent, 0.06)
        } else {
            theme.card
        })
        .stroke(Stroke::new(
            1.0,
            if active {
                with_alpha(theme.accent, 120)
            } else {
                theme.stroke_soft
            },
        ))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(&profile.name)
                            .size(16.0)
                            .color(theme.text)
                            .strong(),
                    );
                    ui.label(
                        RichText::new(format!(
                            "{} · {} Hz",
                            profile.light.mode_label(lang),
                            profile.performance.polling_hz()
                        ))
                        .size(9.5)
                        .color(theme.muted),
                    );
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    if active {
                        pill(ui, theme, lang.t("ACTIVO", "ACTIVE"), theme.success);
                    }
                });
            });
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                for stage in profile.dpi.iter().filter(|s| s.enabled).take(6) {
                    let (dot, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    ui.painter().circle_filled(
                        dot.center(),
                        3.0,
                        Color32::from_rgb(stage.color[0], stage.color[1], stage.color[2]),
                    );
                }
                ui.label(
                    RichText::new(format!(
                        "{} {}",
                        profile.dpi.iter().filter(|s| s.enabled).count(),
                        lang.t("etapas", "stages")
                    ))
                    .size(9.0)
                    .color(theme.dim),
                );
            });
            ui.add_space(12.0);
            if !active {
                if primary_button(ui, theme, lang.t("Activar", "Activate")).clicked() {
                    *activate = Some(index);
                }
            } else if secondary_button(ui, theme, lang.t("En uso", "In use")).clicked() {
                *activate = Some(index);
            }
            ui.add_space(4.0);
            ui.horizontal_wrapped(|ui| {
                if ghost_button(ui, theme, lang.t("Renombrar", "Rename")).clicked() {
                    *rename = Some(index);
                }
                if ghost_button(ui, theme, lang.t("Duplicar", "Duplicate")).clicked() {
                    *duplicate = Some(index);
                }
                if ghost_button(ui, theme, lang.t("Eliminar", "Delete")).clicked() {
                    *remove = Some(index);
                }
            });
        });
}

fn new_profile_card(ui: &mut Ui, theme: &Theme, lang: Lang, add: &mut bool) {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), 214.0), Sense::click());
    let hover = response.hovered();
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(16),
        if hover { theme.card_hover } else { theme.card },
    );
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(16),
        Stroke::new(
            1.0,
            if hover {
                with_alpha(theme.accent, 120)
            } else {
                theme.stroke
            },
        ),
        StrokeKind::Inside,
    );
    ui.painter().text(
        rect.center() - vec2(0.0, 14.0),
        Align2::CENTER_CENTER,
        "+",
        FontId::proportional(30.0),
        theme.accent,
    );
    ui.painter().text(
        rect.center() + vec2(0.0, 20.0),
        Align2::CENTER_CENTER,
        lang.t("Nuevo perfil", "New profile"),
        FontId::proportional(11.0),
        theme.muted,
    );
    if hover {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    if response.clicked() {
        *add = true;
    }
}
