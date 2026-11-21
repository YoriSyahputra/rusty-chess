use egui::{
    epaint::Shadow, Align2, Color32, Context, FontId, Margin, Pos2, Rect, Rounding, Sense, Stroke,
    Style, Ui, Vec2, Visuals,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BoardTheme {
    ClassicGreen,
    WalnutWood,
    Emerald,
    TournamentBlue,
    IcySea,
    Newspaper,
    DarkSlate,
}

#[derive(Debug, Clone, Copy)]
pub struct UiTokens {
    pub dark_mode: bool,
    pub bg_canvas: Color32,
    pub bg_sidebar: Color32,
    pub bg_surface: Color32,
    pub bg_surface_elevated: Color32,
    pub border_subtle: Color32,
    pub accent_primary: Color32,
    pub accent_primary_hover: Color32,
    pub accent_gold: Color32,
    pub accent_info: Color32,
    pub text_primary: Color32,
    pub text_secondary: Color32,
}

impl UiTokens {
    pub fn for_mode(dark_mode: bool) -> Self {
        if dark_mode {
            Self {
                dark_mode: true,
                bg_canvas: Color32::from_rgb(0x1F, 0x1D, 0x1A),
                bg_sidebar: Color32::from_rgb(0x18, 0x16, 0x14),
                bg_surface: Color32::from_rgb(0x2B, 0x29, 0x26),
                bg_surface_elevated: Color32::from_rgb(0x37, 0x34, 0x30),
                border_subtle: Color32::from_rgba_unmultiplied(255, 255, 255, 18),
                accent_primary: Color32::from_rgb(0x81, 0xB6, 0x4C),
                accent_primary_hover: Color32::from_rgb(0x98, 0xCE, 0x60),
                accent_gold: Color32::from_rgb(0xFF, 0xB0, 0x20),
                accent_info: Color32::from_rgb(0x29, 0xB6, 0xF6),
                text_primary: Color32::from_rgb(0xF6, 0xF5, 0xF2),
                text_secondary: Color32::from_rgb(0xAE, 0xAA, 0xA3),
            }
        } else {
            Self {
                dark_mode: false,
                bg_canvas: Color32::from_rgb(0xF2, 0xF1, 0xED),
                bg_sidebar: Color32::from_rgb(0xE8, 0xE6, 0xE1),
                bg_surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                bg_surface_elevated: Color32::from_rgb(0xF4, 0xF3, 0xEF),
                border_subtle: Color32::from_rgba_unmultiplied(30, 28, 24, 22),
                accent_primary: Color32::from_rgb(0x81, 0xB6, 0x4C),
                accent_primary_hover: Color32::from_rgb(0x95, 0xC9, 0x5D),
                accent_gold: Color32::from_rgb(0xE0, 0x94, 0x00),
                accent_info: Color32::from_rgb(0x00, 0x88, 0xBD),
                text_primary: Color32::from_rgb(0x22, 0x20, 0x1E),
                text_secondary: Color32::from_rgb(0x68, 0x64, 0x5E),
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoardPalette {
    pub light_square: Color32,
    pub dark_square: Color32,
    pub border_bg: Color32,
    pub selected_square: Color32,
    pub last_move_from: Color32,
    pub last_move_to: Color32,
    pub premove_square: Color32,
    pub legal_move_dot: Color32,
    pub legal_capture_ring: Color32,
    pub check_glow_inner: Color32,
    pub check_glow_outer: Color32,
    pub arrow_red: Color32,
    pub arrow_yellow: Color32,
    pub arrow_blue: Color32,
    pub arrow_green: Color32,
    pub arrow_tactical: Color32,
    pub arrow_hint: Color32,
    pub marked_square: Color32,
}

impl BoardTheme {
    pub fn all() -> &'static [BoardTheme] {
        &[
            BoardTheme::ClassicGreen,
            BoardTheme::WalnutWood,
            BoardTheme::Emerald,
            BoardTheme::TournamentBlue,
            BoardTheme::IcySea,
            BoardTheme::Newspaper,
            BoardTheme::DarkSlate,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BoardTheme::ClassicGreen => "Classic Green",
            BoardTheme::WalnutWood => "Walnut Wood",
            BoardTheme::Emerald => "Emerald",
            BoardTheme::TournamentBlue => "Tournament Blue",
            BoardTheme::IcySea => "Icy Sea",
            BoardTheme::Newspaper => "Newspaper",
            BoardTheme::DarkSlate => "Dark Slate",
        }
    }

    pub fn palette(&self) -> BoardPalette {
        let (light_square, dark_square, border_bg) = match self {
            BoardTheme::ClassicGreen => (
                Color32::from_rgb(0xEE, 0xEE, 0xD2),
                Color32::from_rgb(0x76, 0x96, 0x56),
                Color32::from_rgb(0x25, 0x23, 0x20),
            ),
            BoardTheme::WalnutWood => (
                Color32::from_rgb(0xF0, 0xD9, 0xB5),
                Color32::from_rgb(0xB5, 0x88, 0x63),
                Color32::from_rgb(0x2F, 0x25, 0x1D),
            ),
            BoardTheme::Emerald => (
                Color32::from_rgb(0xAD, 0xBD, 0x8F),
                Color32::from_rgb(0x6F, 0x8F, 0x72),
                Color32::from_rgb(0x20, 0x29, 0x22),
            ),
            BoardTheme::TournamentBlue => (
                Color32::from_rgb(0xDE, 0xE3, 0xE6),
                Color32::from_rgb(0x8C, 0xA2, 0xAD),
                Color32::from_rgb(0x1E, 0x26, 0x2D),
            ),
            BoardTheme::IcySea => (
                Color32::from_rgb(0xE8, 0xED, 0xF2),
                Color32::from_rgb(0x5B, 0x87, 0xA6),
                Color32::from_rgb(0x1A, 0x25, 0x30),
            ),
            BoardTheme::Newspaper => (
                Color32::from_rgb(0xF3, 0xF3, 0xF3),
                Color32::from_rgb(0x9E, 0x9E, 0x9E),
                Color32::from_rgb(0x24, 0x24, 0x24),
            ),
            BoardTheme::DarkSlate => (
                Color32::from_rgb(0x5E, 0x63, 0x6B),
                Color32::from_rgb(0x3D, 0x41, 0x48),
                Color32::from_rgb(0x1D, 0x1F, 0x23),
            ),
        };

        BoardPalette {
            light_square,
            dark_square,
            border_bg,
            selected_square: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0x33, 145),
            last_move_from: Color32::from_rgba_unmultiplied(0xF6, 0xF6, 0x69, 120),
            last_move_to: Color32::from_rgba_unmultiplied(0xF6, 0xF6, 0x69, 150),
            premove_square: Color32::from_rgba_unmultiplied(0xE0, 0x4F, 0x44, 140),
            legal_move_dot: Color32::from_rgba_unmultiplied(20, 22, 18, 78),
            legal_capture_ring: Color32::from_rgba_unmultiplied(20, 22, 18, 102),
            check_glow_inner: Color32::from_rgba_unmultiplied(235, 38, 38, 215),
            check_glow_outer: Color32::from_rgba_unmultiplied(200, 20, 20, 105),
            arrow_red: Color32::from_rgba_unmultiplied(0xE0, 0x4F, 0x44, 210),
            arrow_yellow: Color32::from_rgba_unmultiplied(0xFF, 0xAA, 0x00, 215),
            arrow_blue: Color32::from_rgba_unmultiplied(0x00, 0x9F, 0xD9, 215),
            arrow_green: Color32::from_rgba_unmultiplied(0x81, 0xB6, 0x4C, 220),
            arrow_tactical: Color32::from_rgba_unmultiplied(0xFF, 0xAA, 0x00, 210),
            arrow_hint: Color32::from_rgba_unmultiplied(0x81, 0xB6, 0x4C, 225),
            marked_square: Color32::from_rgba_unmultiplied(0xE0, 0x4F, 0x44, 150),
        }
    }
}

pub fn apply_global_style(ctx: &Context) {
    apply_global_style_with_mode(ctx, true);
}

pub fn apply_global_style_with_mode(ctx: &Context, dark_mode: bool) {
    let tokens = UiTokens::for_mode(dark_mode);
    let mut style: Style = (*ctx.style()).clone();
    let mut visuals = if dark_mode {
        Visuals::dark()
    } else {
        Visuals::light()
    };

    visuals.panel_fill = tokens.bg_canvas;
    visuals.window_fill = tokens.bg_surface;
    visuals.faint_bg_color = tokens.bg_surface_elevated;
    visuals.extreme_bg_color = tokens.bg_sidebar;

    visuals.widgets.noninteractive.bg_fill = tokens.bg_surface;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, tokens.border_subtle);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);
    visuals.widgets.noninteractive.rounding = Rounding::same(10.0_f32);

    visuals.widgets.inactive.bg_fill = tokens.bg_surface_elevated;
    visuals.widgets.inactive.bg_stroke = Stroke::NONE;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);
    visuals.widgets.inactive.rounding = Rounding::same(9.0_f32);

    visuals.widgets.hovered.bg_fill = if dark_mode {
        Color32::from_rgb(0x49, 0x46, 0x41)
    } else {
        Color32::from_rgb(0xE6, 0xE4, 0xDE)
    };
    visuals.widgets.hovered.bg_stroke = Stroke::new(
        1.0_f32,
        Color32::from_rgba_unmultiplied(255, 255, 255, if dark_mode { 34 } else { 0 }),
    );
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);
    visuals.widgets.hovered.rounding = Rounding::same(9.0_f32);
    visuals.widgets.hovered.expansion = 1.0;

    visuals.widgets.active.bg_fill = tokens.accent_primary;
    visuals.widgets.active.bg_stroke = Stroke::NONE;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(9.0_f32);

    visuals.window_rounding = Rounding::same(14.0_f32);
    visuals.window_stroke = Stroke::new(1.0_f32, tokens.border_subtle);
    visuals.window_shadow = Shadow {
        offset: Vec2::new(0.0, 10.0),
        blur: 26.0,
        spread: 2.0,
        color: Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 135 } else { 55 }),
    };
    visuals.popup_shadow = Shadow {
        offset: Vec2::new(0.0, 6.0),
        blur: 16.0,
        spread: 0.0,
        color: Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 110 } else { 42 }),
    };

    style.animation_time = 0.14;
    style.spacing.button_padding = Vec2::new(11.0, 6.0);
    style.spacing.item_spacing = Vec2::new(8.0, 6.0);
    style.visuals = visuals;
    ctx.set_style(style);
}

#[inline]
pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let inv = 1.0 - t;
    Color32::from_rgba_unmultiplied(
        (a.r() as f32 * inv + b.r() as f32 * t).round() as u8,
        (a.g() as f32 * inv + b.g() as f32 * t).round() as u8,
        (a.b() as f32 * inv + b.b() as f32 * t).round() as u8,
        (a.a() as f32 * inv + b.a() as f32 * t).round() as u8,
    )
}

#[inline]
pub fn adjust_color(c: Color32, delta: i16) -> Color32 {
    Color32::from_rgba_unmultiplied(
        (c.r() as i16 + delta).clamp(0, 255) as u8,
        (c.g() as i16 + delta).clamp(0, 255) as u8,
        (c.b() as i16 + delta).clamp(0, 255) as u8,
        c.a(),
    )
}

/// Kartu elevasi modern dengan sudut membulat lembut dan bayangan ambient (anti kaku).
pub fn soft_card_frame(tokens: &UiTokens, margin: Margin) -> egui::Frame {
    egui::Frame::none()
        .fill(tokens.bg_surface)
        .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
        .rounding(Rounding::same(12.0_f32))
        .inner_margin(margin)
        .shadow(Shadow {
            offset: Vec2::new(0.0, 3.0),
            blur: 12.0,
            spread: 0.0,
            color: Color32::from_rgba_unmultiplied(
                0,
                0,
                0,
                if tokens.dark_mode { 62 } else { 22 },
            ),
        })
}

/// Tombol CTA khas Chess.com dengan animasi hover lift, glow halus, specular sheen, dan 3D lip taktil.
pub fn chess_cta_button(ui: &mut Ui, size: Vec2, label: &str, base_color: Color32) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    let pressed = resp.is_pointer_button_down_on();

    let hover_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("cta_hover"), hovered, 0.13);
    let press_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("cta_press"), pressed, 0.07);

    let y_shift = -1.4 * hover_t * (1.0 - press_t) + 1.8 * press_t;
    let shifted = rect.translate(Vec2::new(0.0, y_shift));

    let lip_color = adjust_color(base_color, -38);
    let face_hover = adjust_color(base_color, 18);
    let face_base = lerp_color(base_color, face_hover, hover_t);
    let face_color = lerp_color(face_base, adjust_color(base_color, -10), press_t);

    let painter = ui.painter();
    let rounding = Rounding::same(9.0_f32);

    // Soft ambient glow underneath on hover
    if hover_t > 0.01 {
        let glow_col = Color32::from_rgba_unmultiplied(
            base_color.r(),
            base_color.g(),
            base_color.b(),
            (hover_t * 48.0 * (1.0 - press_t * 0.5)) as u8,
        );
        painter.rect_filled(
            shifted.expand2(Vec2::new(2.0, 3.0)).translate(Vec2::new(0.0, 2.0)),
            Rounding::same(11.0_f32),
            glow_col,
        );
    }

    // 3D bottom bevel shadow
    painter.rect_filled(shifted, rounding, lip_color);

    let lip_h = 3.5 * (1.0 - press_t * 0.72);
    let face_rect = Rect::from_min_max(
        shifted.min,
        Pos2::new(shifted.max.x, (shifted.max.y - lip_h).max(shifted.min.y + 4.0)),
    );
    painter.rect_filled(face_rect, rounding, face_color);

    // Subtle top specular catch-light
    let sheen_alpha = (28.0 + hover_t * 22.0) as u8;
    painter.line_segment(
        [
            Pos2::new(face_rect.min.x + 8.0, face_rect.min.y + 1.0),
            Pos2::new(face_rect.max.x - 8.0, face_rect.min.y + 1.0),
        ],
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(255, 255, 255, sheen_alpha)),
    );

    let font_size = (size.y * 0.39).clamp(11.5, 15.5);
    painter.text(
        face_rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(font_size),
        Color32::WHITE,
    );

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    resp.clicked()
}

/// Tombol pill interaktif dengan transisi warna & hover lift lembut (untuk Top Bar & Filter Pills).
pub fn tactile_pill_button(
    ui: &mut Ui,
    id_salt: &str,
    label: &str,
    is_active: bool,
    accent: Color32,
    dark_mode: bool,
) -> egui::Response {
    let font_id = FontId::proportional(12.0);
    let galley = ui
        .painter()
        .layout_no_wrap(label.to_string(), font_id.clone(), Color32::WHITE);
    let size = Vec2::new((galley.size().x + 20.0).max(52.0), 27.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let hovered = resp.hovered();
    let pressed = resp.is_pointer_button_down_on();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with(id_salt).with("h"), hovered, 0.13);
    let active_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with(id_salt).with("a"), is_active, 0.16);
    let press_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with(id_salt).with("p"), pressed, 0.07);

    let y_shift = -0.9 * hover_t * (1.0 - press_t) + 1.0 * press_t;
    let draw_rect = rect.translate(Vec2::new(0.0, y_shift));

    let idle_bg = if dark_mode {
        Color32::from_rgb(0x31, 0x2F, 0x2B)
    } else {
        Color32::from_rgb(0xEA, 0xE8, 0xE3)
    };
    let hover_bg = if dark_mode {
        Color32::from_rgb(0x41, 0x3E, 0x39)
    } else {
        Color32::from_rgb(0xDF, 0xDD, 0xD7)
    };
    let active_bg = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 44);

    let base_bg = lerp_color(idle_bg, hover_bg, hover_t);
    let fill = lerp_color(base_bg, active_bg, active_t);

    let idle_text = if dark_mode {
        Color32::from_rgb(0xE2, 0xDF, 0xDA)
    } else {
        Color32::from_rgb(0x2A, 0x28, 0x25)
    };
    let text_col = lerp_color(
        lerp_color(idle_text, Color32::WHITE, if dark_mode { hover_t * 0.8 } else { 0.0 }),
        accent,
        active_t,
    );

    let border_col = lerp_color(
        Color32::from_rgba_unmultiplied(255, 255, 255, (14.0 + hover_t * 24.0) as u8),
        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 165),
        active_t,
    );

    let painter = ui.painter();
    painter.rect(
        draw_rect,
        Rounding::same(13.5_f32),
        fill,
        Stroke::new(1.0_f32, border_col),
    );
    painter.text(
        draw_rect.center(),
        Align2::CENTER_CENTER,
        label,
        font_id,
        text_col,
    );

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    resp
}

/// Tombol sekunder lembut dengan animasi hover & press squish (pengganti tombol kotak kaku).
pub fn tactile_surface_button(
    ui: &mut Ui,
    size: Vec2,
    label: &str,
    enabled: bool,
    tokens: &UiTokens,
) -> egui::Response {
    let sense = if enabled { Sense::click() } else { Sense::hover() };
    let (rect, resp) = ui.allocate_exact_size(size, sense);

    let hovered = enabled && resp.hovered();
    let pressed = enabled && resp.is_pointer_button_down_on();
    let hover_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("sh"), hovered, 0.12);
    let press_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("sp"), pressed, 0.06);

    let y_shift = -1.0 * hover_t * (1.0 - press_t) + 1.2 * press_t;
    let draw_rect = rect.translate(Vec2::new(0.0, y_shift));

    let idle_bg = tokens.bg_surface_elevated;
    let hover_bg = if tokens.dark_mode {
        Color32::from_rgb(0x49, 0x46, 0x41)
    } else {
        Color32::from_rgb(0xE5, 0xE3, 0xDD)
    };
    let fill = if !enabled {
        idle_bg.gamma_multiply(0.55)
    } else {
        lerp_color(lerp_color(idle_bg, hover_bg, hover_t), adjust_color(idle_bg, -8), press_t)
    };

    let fg = if !enabled {
        tokens.text_secondary.gamma_multiply(0.45)
    } else {
        lerp_color(tokens.text_primary, Color32::WHITE, if tokens.dark_mode { hover_t * 0.6 } else { 0.0 })
    };

    let stroke_col = if !enabled {
        Color32::TRANSPARENT
    } else {
        Color32::from_rgba_unmultiplied(
            255,
            255,
            255,
            (12.0 + hover_t * 32.0) as u8,
        )
    };

    let painter = ui.painter();
    painter.rect(
        draw_rect,
        Rounding::same(9.0_f32),
        fill,
        Stroke::new(1.0_f32, stroke_col),
    );

    let font_size = (size.y * 0.37).clamp(11.0, 13.5);
    painter.text(
        draw_rect.center(),
        Align2::CENTER_CENTER,
        label,
        FontId::proportional(font_size),
        fg,
    );

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    resp
}
