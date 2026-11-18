use crate::theme::{Theme, mix, paint_glow, with_alpha};
use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Pos2, Rect, Response, Sense, Stroke, StrokeKind,
    Ui, Vec2, pos2, vec2,
};

pub struct MouseArt<'a> {
    pub theme: &'a Theme,
    pub glow: Option<(Color32, f32)>,
    pub selected: Option<usize>,
    pub numbered: bool,
    pub caption: Option<String>,
}

pub struct MouseArtResult {
    pub response: Response,
    pub hovered: Option<usize>,
    pub clicked: Option<usize>,
    pub body: Rect,
}

pub fn mouse_art(ui: &mut Ui, desired: Vec2, art: MouseArt<'_>) -> MouseArtResult {
    let (rect, response) = ui.allocate_exact_size(desired, Sense::click());
    let painter = ui.painter();
    let body_w = (rect.height() * 0.53).min(rect.width() * 0.62).max(120.0);
    let body_h = (body_w * 1.78).min(rect.height() * 0.92);
    let center = rect.center() + vec2(0.0, -4.0);
    let body = Rect::from_center_size(center, vec2(body_w, body_h));

    // Floor shadow and ambient glow.
    painter.add(egui::Shape::ellipse_filled(
        pos2(center.x, body.bottom() + body_h * 0.035),
        vec2(body_w * 0.54, body_h * 0.045),
        Color32::from_black_alpha(110),
    ));
    if let Some((color, strength)) = art.glow {
        paint_glow(
            painter,
            pos2(center.x, body.top() + body_h * 0.305),
            body_w * 0.55,
            color,
            0.30 * strength,
        );
        paint_glow(
            painter,
            pos2(center.x, body.top() + body_h * 0.20),
            body_w * 0.42,
            color,
            0.15 * strength,
        );
    }

    // Main shell.
    let shell = CornerRadius {
        nw: 34,
        ne: 34,
        sw: 82,
        se: 82,
    };
    painter.rect_filled(body, shell, Color32::from_rgb(0xC7, 0xD1, 0xDD));
    painter.rect_filled(
        body.shrink(1.5),
        CornerRadius {
            nw: 33,
            ne: 33,
            sw: 80,
            se: 80,
        },
        Color32::from_rgb(0xF1, 0xF5, 0xF9),
    );
    painter.rect_stroke(
        body,
        shell,
        Stroke::new(1.2, Color32::from_rgb(0x9F, 0xAC, 0xBB)),
        StrokeKind::Inside,
    );

    let clipped = painter.with_clip_rect(body.shrink(1.0));
    let split_y = body.top() + body_h * 0.35;
    let left_button = Rect::from_min_max(
        pos2(body.left() + 1.0, body.top() + 1.0),
        pos2(center.x - 0.7, split_y),
    );
    let right_button = Rect::from_min_max(
        pos2(center.x + 0.7, body.top() + 1.0),
        pos2(body.right() - 1.0, split_y),
    );
    clipped.rect_filled(
        left_button,
        CornerRadius {
            nw: 32,
            ne: 8,
            sw: 0,
            se: 0,
        },
        Color32::from_rgb(0xF8, 0xFA, 0xFC),
    );
    clipped.rect_filled(
        right_button,
        CornerRadius {
            nw: 8,
            ne: 32,
            sw: 0,
            se: 0,
        },
        Color32::from_rgb(0xF8, 0xFA, 0xFC),
    );
    clipped.line_segment(
        [
            pos2(center.x, body.top() + 2.0),
            pos2(center.x, split_y + 6.0),
        ],
        Stroke::new(1.3, Color32::from_rgb(0xB4, 0xC0, 0xCC)),
    );
    clipped.line_segment(
        [
            pos2(body.left() + 3.0, split_y),
            pos2(body.right() - 3.0, split_y),
        ],
        Stroke::new(1.1, Color32::from_rgb(0xC2, 0xCD, 0xD8)),
    );

    // Scroll wheel.
    let wheel = Rect::from_center_size(
        pos2(center.x, body.top() + body_h * 0.175),
        vec2(body_w * 0.115, body_h * 0.135),
    );
    clipped.rect_filled(
        wheel,
        CornerRadius::same(8),
        Color32::from_rgb(0x1B, 0x22, 0x2C),
    );
    clipped.rect_stroke(
        wheel,
        CornerRadius::same(8),
        Stroke::new(1.0, Color32::from_rgb(0x45, 0x52, 0x62)),
        StrokeKind::Inside,
    );
    for i in 0..4 {
        let y = wheel.top() + wheel.height() * (0.22 + i as f32 * 0.17);
        clipped.line_segment(
            [pos2(wheel.left() + 3.0, y), pos2(wheel.right() - 3.0, y)],
            Stroke::new(1.0, Color32::from_rgb(0x5A, 0x68, 0x78)),
        );
    }
    clipped.circle_filled(
        pos2(wheel.center().x, wheel.top() + 4.0),
        1.3,
        Color32::from_rgb(0x86, 0x96, 0xA8),
    );

    // Side buttons.
    let side_x = body.left() + 1.0;
    for (i, y_frac) in [0.425, 0.535].iter().enumerate() {
        let r = Rect::from_center_size(
            pos2(side_x - 2.0, body.top() + body_h * y_frac),
            vec2(body_w * 0.045, body_h * 0.075),
        );
        painter.rect_filled(
            r,
            CornerRadius {
                nw: 5,
                ne: 2,
                sw: 5,
                se: 2,
            },
            if i == 0 {
                Color32::from_rgb(0xD5, 0xDF, 0xE9)
            } else {
                Color32::from_rgb(0xC9, 0xD4, 0xDF)
            },
        );
        painter.rect_stroke(
            r,
            CornerRadius::same(4),
            Stroke::new(1.0, Color32::from_rgb(0xA8, 0xB5, 0xC3)),
            StrokeKind::Inside,
        );
    }

    // Quiet product mark on the shell.
    painter.text(
        pos2(center.x, body.top() + body_h * 0.635),
        Align2::CENTER_CENTER,
        "X11",
        FontId::proportional(body_w * 0.095),
        Color32::from_rgb(0xB3, 0xBF, 0xCC),
    );

    // Bottom light strip.
    let led = Rect::from_center_size(
        pos2(center.x, body.top() + body_h * 0.305),
        vec2(body_w * 0.28, body_h * 0.010),
    );
    let led_color = art
        .glow
        .map(|g| g.0)
        .unwrap_or(Color32::from_rgb(0x3D, 0xD6, 0xF5));
    painter.rect_filled(led, CornerRadius::same(3), with_alpha(led_color, 150));
    if let Some((color, strength)) = art.glow {
        paint_glow(painter, led.center(), body_w * 0.20, color, 0.30 * strength);
    }

    let hotspot = |index: usize| -> Pos2 {
        match index {
            0 => pos2(body.center().x - body_w * 0.20, body.top() + body_h * 0.20),
            1 => pos2(body.center().x + body_w * 0.20, body.top() + body_h * 0.20),
            2 => pos2(body.center().x, body.top() + body_h * 0.13),
            3 => pos2(body.left() - 2.0, body.top() + body_h * 0.425),
            _ => pos2(body.left() - 2.0, body.top() + body_h * 0.535),
        }
    };

    let pointer = ui.ctx().pointer_hover_pos();
    let mut hovered = None;
    if art.numbered {
        let mut best = 28.0;
        for i in 0..5 {
            if let Some(p) = pointer {
                let d = p.distance(hotspot(i));
                if d < best {
                    best = d;
                    hovered = Some(i);
                }
            }
        }
    }

    if art.numbered {
        for i in 0..5 {
            let center = hotspot(i);
            let selected = art.selected == Some(i);
            let hot = hovered == Some(i);
            let radius = if hot { 15.0 } else { 13.5 };
            if selected || hot {
                paint_glow(painter, center, radius * 2.2, art.theme.accent, 0.45);
            }
            painter.circle_filled(
                center,
                radius,
                if selected {
                    art.theme.accent
                } else if hot {
                    mix(Color32::from_rgb(0x0E, 0x12, 0x18), art.theme.accent, 0.32)
                } else {
                    Color32::from_rgba_unmultiplied(0x0E, 0x12, 0x18, 235)
                },
            );
            painter.circle_stroke(
                center,
                radius,
                Stroke::new(
                    1.4,
                    if selected || hot {
                        with_alpha(art.theme.accent, 230)
                    } else {
                        with_alpha(art.theme.text, 80)
                    },
                ),
            );
            painter.text(
                center,
                Align2::CENTER_CENTER,
                format!("{}", i + 1),
                FontId::proportional(if size_small(body) { 10.5 } else { 12.0 }),
                if selected {
                    art.theme.on_accent
                } else {
                    art.theme.text
                },
            );
        }
    }

    if let Some(caption) = &art.caption {
        painter.text(
            pos2(rect.center().x, rect.bottom() - 4.0),
            Align2::CENTER_BOTTOM,
            caption,
            FontId::proportional(11.0),
            art.theme.muted,
        );
    }

    if hovered.is_some() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }
    let clicked = if response.clicked() { hovered } else { None };
    MouseArtResult {
        response,
        hovered,
        clicked,
        body,
    }
}

fn size_small(body: Rect) -> bool {
    body.width() < 130.0
}
