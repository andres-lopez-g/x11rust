use crate::model::Accent;
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontData, FontDefinitions, FontFamily, FontId, Margin,
    Pos2, Rect, Response, RichText, Sense, Shape, Stroke, StrokeKind, TextStyle, Ui, pos2, vec2,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub accent: Color32,
    pub accent2: Color32,
    pub accent_soft: Color32,
    pub on_accent: Color32,
    pub bg: Color32,
    pub bg_top: Color32,
    pub sidebar: Color32,
    pub panel: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub input: Color32,
    pub stroke: Color32,
    pub stroke_soft: Color32,
    pub text: Color32,
    pub muted: Color32,
    pub dim: Color32,
    pub success: Color32,
    pub warning: Color32,
    pub danger: Color32,
    pub cyan: Color32,
}

impl Theme {
    pub fn new(accent: Accent) -> Self {
        let (r, g, b) = accent.rgb();
        let (r2, g2, b2) = match accent {
            Accent::Coral => (0xFF, 0x8A, 0x3D),
            Accent::Cyan => (0x6E, 0xE7, 0xF5),
            Accent::Violet => (0xD8, 0xB4, 0xFE),
            Accent::Mint => (0xA7, 0xF3, 0xD0),
        };
        let accent = Color32::from_rgb(r, g, b);
        Self {
            accent,
            accent2: Color32::from_rgb(r2, g2, b2),
            accent_soft: Color32::from_rgba_unmultiplied(r, g, b, 34),
            on_accent: Color32::from_rgb(0x10, 0x12, 0x16),
            bg: Color32::from_rgb(0x0A, 0x0D, 0x12),
            bg_top: Color32::from_rgb(0x0F, 0x14, 0x1C),
            sidebar: Color32::from_rgb(0x0D, 0x11, 0x18),
            panel: Color32::from_rgb(0x14, 0x1A, 0x23),
            card: Color32::from_rgb(0x17, 0x1E, 0x28),
            card_hover: Color32::from_rgb(0x1C, 0x25, 0x31),
            input: Color32::from_rgb(0x0F, 0x14, 0x1C),
            stroke: Color32::from_rgb(0x27, 0x31, 0x3F),
            stroke_soft: Color32::from_rgb(0x1E, 0x27, 0x33),
            text: Color32::from_rgb(0xF1, 0xF4, 0xF8),
            muted: Color32::from_rgb(0x94, 0xA2, 0xB2),
            dim: Color32::from_rgb(0x5E, 0x6A, 0x78),
            success: Color32::from_rgb(0x52, 0xE5, 0xA3),
            warning: Color32::from_rgb(0xFF, 0xC8, 0x57),
            danger: Color32::from_rgb(0xFF, 0x5D, 0x6C),
            cyan: Color32::from_rgb(0x3D, 0xD6, 0xF5),
        }
    }

    pub fn card_frame(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.card)
            .stroke(Stroke::new(1.0, self.stroke_soft))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::same(18))
    }

    pub fn soft_frame(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.panel)
            .stroke(Stroke::new(1.0, self.stroke_soft))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(14))
    }

    pub fn input_frame(&self) -> egui::Frame {
        egui::Frame::NONE
            .fill(self.input)
            .stroke(Stroke::new(1.0, self.stroke))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(Margin::symmetric(12, 9))
    }

    pub fn heading(&self, text: impl Into<String>, size: f32) -> RichText {
        RichText::new(text).size(size).color(self.text)
    }

    pub fn body(&self, text: impl Into<String>) -> RichText {
        RichText::new(text).size(13.5).color(self.text)
    }

    pub fn small(&self, text: impl Into<String>) -> RichText {
        RichText::new(text).size(11.5).color(self.muted)
    }

    pub fn label(&self, text: impl Into<String>) -> RichText {
        RichText::new(text).size(12.5).color(self.muted)
    }

    pub fn accent_text(&self, text: impl Into<String>) -> RichText {
        RichText::new(text).size(12.5).color(self.accent)
    }

    pub fn mix(&self, other: Color32, t: f32) -> Color32 {
        mix(self.text, other, t)
    }
}

pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    Color32::from_rgba_unmultiplied(
        (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8,
        (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8,
        (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8,
        (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8,
    )
}

pub fn with_alpha(color: Color32, alpha: u8) -> Color32 {
    Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), alpha)
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "Inter".to_owned(),
        Arc::new(FontData::from_static(include_bytes!("../assets/Inter.ttf"))),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "Inter".to_owned());
    ctx.set_fonts(fonts);
}

pub fn install_style(ctx: &egui::Context, theme: &Theme) {
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.visuals = egui::Visuals::dark();
    style.visuals.panel_fill = theme.bg;
    style.visuals.window_fill = theme.panel;
    style.visuals.extreme_bg_color = theme.input;
    style.visuals.faint_bg_color = theme.panel;
    style.visuals.code_bg_color = theme.input;
    style.visuals.selection.bg_fill = with_alpha(theme.accent, 90);
    style.visuals.selection.stroke = Stroke::new(1.0, theme.accent);
    style.visuals.hyperlink_color = theme.cyan;
    style.visuals.warn_fg_color = theme.warning;
    style.visuals.window_stroke = Stroke::new(1.0, theme.stroke);
    style.visuals.window_corner_radius = CornerRadius::same(18);
    style.visuals.menu_corner_radius = CornerRadius::same(14);
    style.visuals.popup_shadow = egui::epaint::Shadow {
        offset: [0, 10],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(120),
    };
    style.visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 16],
        blur: 40,
        spread: 0,
        color: Color32::from_black_alpha(150),
    };
    style.visuals.widgets.noninteractive.bg_fill = theme.panel;
    style.visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, theme.stroke_soft);
    style.visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, theme.text);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(10);
    style.visuals.widgets.inactive.bg_fill = theme.card;
    style.visuals.widgets.inactive.weak_bg_fill = theme.card;
    style.visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, theme.stroke_soft);
    style.visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, theme.text);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(10);
    style.visuals.widgets.hovered.bg_fill = theme.card_hover;
    style.visuals.widgets.hovered.weak_bg_fill = theme.card_hover;
    style.visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, with_alpha(theme.accent, 160));
    style.visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, theme.text);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(10);
    style.visuals.widgets.active.bg_fill = with_alpha(theme.accent, 40);
    style.visuals.widgets.active.weak_bg_fill = with_alpha(theme.accent, 40);
    style.visuals.widgets.active.bg_stroke = Stroke::new(1.2, theme.accent);
    style.visuals.widgets.active.fg_stroke = Stroke::new(1.0, theme.text);
    style.visuals.widgets.active.corner_radius = CornerRadius::same(10);
    style.visuals.widgets.open.bg_fill = theme.card_hover;
    style.visuals.widgets.open.weak_bg_fill = theme.card_hover;
    style.visuals.widgets.open.bg_stroke = Stroke::new(1.0, theme.stroke);
    style.visuals.widgets.open.fg_stroke = Stroke::new(1.0, theme.text);
    style.visuals.widgets.open.corner_radius = CornerRadius::same(10);
    style.visuals.override_text_color = None;
    style.spacing.item_spacing = vec2(10.0, 10.0);
    style.spacing.window_margin = Margin::same(18);
    style.spacing.button_padding = vec2(14.0, 8.0);
    style.spacing.menu_margin = Margin::same(8);
    style.spacing.interact_size = vec2(34.0, 32.0);
    style.spacing.slider_width = 190.0;
    style.spacing.slider_rail_height = 5.0;
    style.spacing.combo_width = 150.0;
    style.spacing.text_edit_width = 220.0;
    style.spacing.icon_width = 16.0;
    style.spacing.icon_width_inner = 10.0;
    style.spacing.icon_spacing = 8.0;
    style.spacing.scroll = egui::style::ScrollStyle::solid();
    style
        .text_styles
        .insert(TextStyle::Heading, FontId::proportional(27.0));
    style
        .text_styles
        .insert(TextStyle::Body, FontId::proportional(13.5));
    style
        .text_styles
        .insert(TextStyle::Monospace, FontId::monospace(12.5));
    style
        .text_styles
        .insert(TextStyle::Button, FontId::proportional(13.0));
    style
        .text_styles
        .insert(TextStyle::Small, FontId::proportional(11.0));
    ctx.set_global_style(style);
}

pub fn primary_button(ui: &mut Ui, theme: &Theme, text: impl Into<String>) -> Response {
    let text = text.into();
    let button = egui::Button::new(
        RichText::new(text)
            .size(13.0)
            .color(theme.on_accent)
            .strong(),
    )
    .fill(theme.accent)
    .stroke(Stroke::NONE)
    .corner_radius(CornerRadius::same(10))
    .min_size(vec2(0.0, 36.0));
    ui.add(button)
}

pub fn secondary_button(ui: &mut Ui, theme: &Theme, text: impl Into<String>) -> Response {
    let button = egui::Button::new(RichText::new(text).size(13.0).color(theme.text))
        .fill(theme.card)
        .stroke(Stroke::new(1.0, theme.stroke))
        .corner_radius(CornerRadius::same(10))
        .min_size(vec2(0.0, 34.0));
    ui.add(button)
}

pub fn ghost_button(ui: &mut Ui, theme: &Theme, text: impl Into<String>) -> Response {
    let button = egui::Button::new(RichText::new(text).size(12.5).color(theme.muted))
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
        .corner_radius(CornerRadius::same(9))
        .min_size(vec2(0.0, 30.0));
    ui.add(button)
}

pub fn danger_button(ui: &mut Ui, theme: &Theme, text: impl Into<String>) -> Response {
    let button = egui::Button::new(RichText::new(text).size(12.5).color(theme.danger))
        .fill(with_alpha(theme.danger, 18))
        .stroke(Stroke::new(1.0, with_alpha(theme.danger, 110)))
        .corner_radius(CornerRadius::same(9))
        .min_size(vec2(0.0, 32.0));
    ui.add(button)
}

pub fn icon_button(ui: &mut Ui, theme: &Theme, icon: &str, tip: &str) -> Response {
    let response = ui.add(
        egui::Button::new(RichText::new(icon).size(13.0).color(theme.muted))
            .fill(theme.card)
            .stroke(Stroke::new(1.0, theme.stroke_soft))
            .corner_radius(CornerRadius::same(9))
            .min_size(vec2(30.0, 30.0)),
    );
    response.on_hover_text(tip)
}

pub fn pill(ui: &mut Ui, _theme: &Theme, text: impl Into<String>, color: Color32) {
    let text = text.into();
    let galley = ui
        .painter()
        .layout_no_wrap(text.clone(), FontId::proportional(11.0), color);
    let size = vec2(galley.size().x + 18.0, 24.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    ui.painter()
        .rect_filled(rect, CornerRadius::same(12), with_alpha(color, 22));
    ui.painter().rect_stroke(
        rect,
        CornerRadius::same(12),
        Stroke::new(1.0, with_alpha(color, 70)),
        StrokeKind::Inside,
    );
    ui.painter()
        .galley(rect.center() - galley.size() * 0.5, galley, color);
}

pub fn status_dot(ui: &mut Ui, color: Color32, label: impl Into<String>, theme: &Theme) {
    let label = label.into();
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
        ui.painter().circle_filled(rect.center(), 4.0, color);
        ui.painter()
            .circle_stroke(rect.center(), 5.0, Stroke::new(1.0, with_alpha(color, 80)));
        ui.label(RichText::new(label).size(11.5).color(theme.muted));
    });
}

pub fn divider(ui: &mut Ui, theme: &Theme) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 1.0), Sense::hover());
    ui.painter().rect_filled(rect, 0, theme.stroke_soft);
}

pub fn section_heading(ui: &mut Ui, theme: &Theme, title: &str, subtitle: Option<&str>) {
    ui.label(RichText::new(title).size(16.0).color(theme.text).strong());
    if let Some(subtitle) = subtitle {
        ui.label(RichText::new(subtitle).size(11.5).color(theme.muted));
    }
}

pub fn toggle(ui: &mut Ui, theme: &Theme, on: &mut bool) -> Response {
    let desired = vec2(40.0, 23.0);
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    if response.clicked() {
        *on = !*on;
    }
    let t = if ui.style().animation_time > 0.0 {
        ui.ctx().animate_bool(response.id, *on)
    } else if *on {
        1.0
    } else {
        0.0
    };
    let bg = mix(
        Color32::from_rgb(0x2A, 0x33, 0x40),
        theme.accent,
        if *on { 1.0 } else { 0.0 },
    );
    let _ = t;
    ui.painter().rect_filled(
        rect,
        CornerRadius::same(12),
        if *on {
            bg
        } else {
            Color32::from_rgb(0x2A, 0x33, 0x40)
        },
    );
    let knob_x = if *on {
        rect.right() - 11.0
    } else {
        rect.left() + 11.0
    };
    ui.painter().circle_filled(
        pos2(knob_x, rect.center().y),
        8.0,
        if *on {
            Color32::from_rgb(0x10, 0x12, 0x16)
        } else {
            theme.muted
        },
    );
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn slider_u32(
    ui: &mut Ui,
    theme: &Theme,
    value: &mut u32,
    min: u32,
    max: u32,
    width: f32,
) -> Response {
    let desired = vec2(width, 22.0);
    let (rect, mut response) = ui.allocate_exact_size(desired, Sense::click_and_drag());
    if response.dragged() || response.clicked() {
        if let Some(pos) = ui.ctx().pointer_interact_pos() {
            let t = ((pos.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
            let new_value = (min as f32 + t * (max - min) as f32).round() as u32;
            if new_value != *value {
                *value = new_value;
                response.mark_changed();
            }
        }
    }
    let t = if max > min {
        (*value).clamp(min, max).saturating_sub(min) as f32 / (max - min) as f32
    } else {
        0.0
    };
    let center_y = rect.center().y;
    ui.painter().rect_filled(
        Rect::from_min_max(
            pos2(rect.left(), center_y - 3.0),
            pos2(rect.right(), center_y + 3.0),
        ),
        CornerRadius::same(3),
        Color32::from_rgb(0x28, 0x31, 0x3D),
    );
    let fill_right = rect.left() + rect.width() * t;
    if fill_right > rect.left() + 0.5 {
        ui.painter().rect_filled(
            Rect::from_min_max(
                pos2(rect.left(), center_y - 3.0),
                pos2(fill_right, center_y + 3.0),
            ),
            CornerRadius::same(3),
            theme.accent,
        );
    }
    let knob = pos2(
        fill_right.clamp(rect.left() + 7.0, rect.right() - 7.0),
        center_y,
    );
    ui.painter().circle_filled(knob, 7.0, theme.text);
    ui.painter()
        .circle_stroke(knob, 8.0, Stroke::new(2.0, with_alpha(theme.accent, 180)));
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    response
}

pub fn battery_ring(ui: &mut Ui, theme: &Theme, percent: f32, size: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let center = rect.center();
    let radius = size * 0.42;
    ui.painter().circle_stroke(
        center,
        radius,
        Stroke::new(7.0, Color32::from_rgb(0x26, 0x30, 0x3D)),
    );
    let color = if percent > 35.0 {
        theme.success
    } else {
        theme.warning
    };
    let start = -std::f32::consts::FRAC_PI_2;
    let end = start + std::f32::consts::TAU * (percent / 100.0).clamp(0.0, 1.0);
    let mut points = Vec::with_capacity(64);
    for i in 0..=48 {
        let a = start + (end - start) * i as f32 / 48.0;
        points.push(center + vec2(a.cos(), a.sin()) * radius);
    }
    if points.len() > 1 {
        ui.painter()
            .add(Shape::line(points, Stroke::new(7.0, color)));
    }
    ui.painter().text(
        center,
        Align2::CENTER_CENTER,
        format!("{:.0}%", percent),
        FontId::proportional(size * 0.24),
        theme.text,
    );
}

pub fn paint_glow(painter: &egui::Painter, center: Pos2, radius: f32, color: Color32, alpha: f32) {
    // Many low-alpha discs approximate a soft radial gradient without a texture.
    for i in 0..24 {
        let t = i as f32 / 23.0;
        let r = radius * (1.0 - t * 0.88);
        let a = (1.0 - t).powf(2.3) * alpha * 255.0 * 0.10;
        painter.circle_filled(
            center,
            r,
            Color32::from_rgba_unmultiplied(color.r(), color.g(), color.b(), a as u8),
        );
    }
}

pub fn background_gradient(ui: &mut Ui, theme: &Theme) {
    let rect = ui.max_rect();
    let mut mesh = egui::Mesh::default();
    mesh.colored_vertex(rect.left_top(), theme.bg_top);
    mesh.colored_vertex(rect.right_top(), theme.bg_top);
    mesh.colored_vertex(rect.right_bottom(), theme.bg);
    mesh.colored_vertex(rect.left_bottom(), theme.bg);
    mesh.add_triangle(0, 1, 2);
    mesh.add_triangle(0, 2, 3);
    ui.painter().add(Shape::mesh(mesh));
}

pub fn paint_nav_icon(painter: &egui::Painter, rect: Rect, tab: crate::app::Tab, color: Color32) {
    use crate::app::Tab;
    let c = rect.center();
    let s = rect.width().min(rect.height());
    let stroke = Stroke::new(1.6, color);
    match tab {
        Tab::Overview => {
            for (dx, dy) in [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)] {
                let r = Rect::from_center_size(
                    c + vec2(dx * s * 0.20, dy * s * 0.20),
                    vec2(s * 0.22, s * 0.22),
                );
                painter.rect_stroke(r, CornerRadius::same(2), stroke, StrokeKind::Inside);
            }
        }
        Tab::Buttons => {
            let body = Rect::from_center_size(c + vec2(0.0, 1.0), vec2(s * 0.44, s * 0.68));
            painter.rect_stroke(body, CornerRadius::same(8), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    pos2(c.x, body.top() + 1.0),
                    pos2(c.x, body.top() + s * 0.28),
                ],
                Stroke::new(1.1, color),
            );
            painter.circle_stroke(
                pos2(c.x, body.top() + s * 0.18),
                s * 0.045,
                Stroke::new(1.2, color),
            );
        }
        Tab::Dpi => {
            painter.circle_stroke(c, s * 0.30, stroke);
            painter.circle_stroke(c, s * 0.13, Stroke::new(1.2, color));
            for a in [0.0f32, 0.5, 1.0, 1.5] {
                let dir = vec2(a.cos(), a.sin());
                painter.line_segment(
                    [c + dir * s * 0.34, c + dir * s * 0.46],
                    Stroke::new(1.5, color),
                );
            }
        }
        Tab::Lighting => {
            painter.circle_stroke(c + vec2(0.0, -1.0), s * 0.20, stroke);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = vec2(a.cos(), a.sin());
                painter.line_segment(
                    [c + dir * s * 0.31, c + dir * s * 0.43],
                    Stroke::new(1.5, color),
                );
            }
        }
        Tab::Performance => {
            let pts = vec![
                pos2(rect.left() + 1.0, c.y + s * 0.05),
                pos2(rect.left() + s * 0.22, c.y + s * 0.05),
                pos2(rect.left() + s * 0.34, c.y - s * 0.24),
                pos2(rect.left() + s * 0.46, c.y + s * 0.20),
                pos2(rect.left() + s * 0.58, c.y - s * 0.10),
                pos2(rect.right() - 2.0, c.y - s * 0.10),
            ];
            painter.add(Shape::line(pts, stroke));
        }
        Tab::Power => {
            let body = Rect::from_center_size(c, vec2(s * 0.62, s * 0.36));
            painter.rect_stroke(body, CornerRadius::same(3), stroke, StrokeKind::Inside);
            painter.line_segment(
                [
                    pos2(body.right(), c.y - s * 0.10),
                    pos2(body.right() + s * 0.10, c.y - s * 0.10),
                ],
                Stroke::new(2.0, color),
            );
            let fill = Rect::from_min_max(
                body.min + vec2(3.0, 3.0),
                pos2(body.left() + body.width() * 0.62, body.bottom() - 3.0),
            );
            painter.rect_filled(fill, CornerRadius::same(2), color);
        }
        Tab::Macros => {
            let body = Rect::from_center_size(c, vec2(s * 0.78, s * 0.48));
            painter.rect_stroke(body, CornerRadius::same(3), stroke, StrokeKind::Inside);
            for row in 0..2 {
                for col in 0..3 {
                    let p = pos2(
                        body.left() + s * 0.18 + col as f32 * s * 0.20,
                        body.top() + s * 0.16 + row as f32 * s * 0.16,
                    );
                    painter.rect_filled(
                        Rect::from_center_size(p, vec2(s * 0.10, s * 0.07)),
                        CornerRadius::same(1),
                        color,
                    );
                }
            }
        }
        Tab::Profiles => {
            for i in 0..3 {
                let offset = i as f32 * s * 0.10;
                let r = Rect::from_center_size(
                    c + vec2(-offset * 0.4, offset * 0.7 - s * 0.08),
                    vec2(s * 0.58, s * 0.30),
                );
                painter.rect_stroke(r, CornerRadius::same(3), stroke, StrokeKind::Inside);
            }
        }
        Tab::Settings => {
            painter.circle_stroke(c, s * 0.17, Stroke::new(1.5, color));
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::TAU / 8.0;
                let dir = vec2(a.cos(), a.sin());
                painter.line_segment(
                    [c + dir * s * 0.26, c + dir * s * 0.38],
                    Stroke::new(1.8, color),
                );
            }
        }
    }
}

pub fn app_mark(painter: &egui::Painter, rect: Rect, theme: &Theme) {
    painter.rect_filled(
        rect,
        CornerRadius::same(11),
        Color32::from_rgb(0x18, 0x20, 0x2A),
    );
    painter.rect_stroke(
        rect,
        CornerRadius::same(11),
        Stroke::new(1.0, with_alpha(theme.accent, 90)),
        StrokeKind::Inside,
    );
    let body = Rect::from_center_size(
        rect.center() + vec2(0.0, 1.0),
        vec2(rect.width() * 0.36, rect.height() * 0.54),
    );
    painter.rect_stroke(
        body,
        CornerRadius::same(6),
        Stroke::new(1.6, theme.text),
        StrokeKind::Inside,
    );
    painter.line_segment(
        [
            pos2(body.center().x, body.top() + 1.0),
            pos2(body.center().x, body.top() + body.height() * 0.34),
        ],
        Stroke::new(1.0, theme.text),
    );
    painter.circle_filled(
        pos2(body.center().x, body.top() + body.height() * 0.18),
        2.0,
        theme.accent,
    );
    painter.circle_filled(
        pos2(body.center().x, body.bottom() - 5.0),
        1.7,
        theme.accent,
    );
}
