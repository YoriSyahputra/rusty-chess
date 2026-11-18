use egui::{Color32, Context, Rounding, Stroke, Style, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BoardTheme {
    ClassicGreen,
    WalnutWood,
    TournamentBlue,
    IcySea,
    DarkSlate,
}

#[derive(Debug, Clone, Copy)]
pub struct UiTokens {
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
                bg_canvas: Color32::from_rgb(0x26, 0x24, 0x21),
                bg_sidebar: Color32::from_rgb(0x21, 0x1F, 0x1C),
                bg_surface: Color32::from_rgb(0x30, 0x2E, 0x2B),
                bg_surface_elevated: Color32::from_rgb(0x3B, 0x38, 0x34),
                border_subtle: Color32::from_rgb(0x47, 0x44, 0x3F),
                accent_primary: Color32::from_rgb(0x81, 0xB6, 0x4C),
                accent_primary_hover: Color32::from_rgb(0x95, 0xC9, 0x5D),
                accent_gold: Color32::from_rgb(0xFF, 0xAA, 0x00),
                accent_info: Color32::from_rgb(0x00, 0x9F, 0xD9),
                text_primary: Color32::from_rgb(0xF5, 0xF5, 0xF5),
                text_secondary: Color32::from_rgb(0xB0, 0xAC, 0xA6),
            }
        } else {
            Self {
                bg_canvas: Color32::from_rgb(0xF1, 0xF1, 0xEE),
                bg_sidebar: Color32::from_rgb(0xE6, 0xE6, 0xE2),
                bg_surface: Color32::from_rgb(0xFF, 0xFF, 0xFF),
                bg_surface_elevated: Color32::from_rgb(0xF6, 0xF6, 0xF3),
                border_subtle: Color32::from_rgb(0xD6, 0xD4, 0xCE),
                accent_primary: Color32::from_rgb(0x81, 0xB6, 0x4C),
                accent_primary_hover: Color32::from_rgb(0x95, 0xC9, 0x5D),
                accent_gold: Color32::from_rgb(0xE0, 0x94, 0x00),
                accent_info: Color32::from_rgb(0x00, 0x88, 0xBD),
                text_primary: Color32::from_rgb(0x22, 0x20, 0x1E),
                text_secondary: Color32::from_rgb(0x66, 0x62, 0x5C),
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
            BoardTheme::TournamentBlue,
            BoardTheme::IcySea,
            BoardTheme::DarkSlate,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BoardTheme::ClassicGreen => "Classic Green",
            BoardTheme::WalnutWood => "Walnut / Wood",
            BoardTheme::TournamentBlue => "Tournament Blue",
            BoardTheme::IcySea => "Icy Sea",
            BoardTheme::DarkSlate => "Dark Slate",
        }
    }

    pub fn palette(&self) -> BoardPalette {
        let (light_square, dark_square, border_bg) = match self {
            BoardTheme::ClassicGreen => (
                Color32::from_rgb(0xEB, 0xEC, 0xD0),
                Color32::from_rgb(0x73, 0x95, 0x52),
                Color32::from_rgb(0x21, 0x1F, 0x1C),
            ),
            BoardTheme::WalnutWood => (
                Color32::from_rgb(0xF0, 0xD9, 0xB5),
                Color32::from_rgb(0xB5, 0x88, 0x63),
                Color32::from_rgb(0x2A, 0x22, 0x1B),
            ),
            BoardTheme::TournamentBlue => (
                Color32::from_rgb(0xDE, 0xE3, 0xE6),
                Color32::from_rgb(0x8C, 0xA2, 0xAD),
                Color32::from_rgb(0x1C, 0x23, 0x29),
            ),
            BoardTheme::IcySea => (
                Color32::from_rgb(0xE8, 0xED, 0xF2),
                Color32::from_rgb(0x5B, 0x87, 0xA6),
                Color32::from_rgb(0x18, 0x22, 0x2C),
            ),
            BoardTheme::DarkSlate => (
                Color32::from_rgb(0x58, 0x5C, 0x62),
                Color32::from_rgb(0x3B, 0x3E, 0x44),
                Color32::from_rgb(0x1B, 0x1C, 0x1F),
            ),
        };

        BoardPalette {
            light_square,
            dark_square,
            border_bg,
            selected_square: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0x33, 145),
            last_move_from: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0x33, 115),
            last_move_to: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0x33, 138),
            premove_square: Color32::from_rgba_unmultiplied(0xE0, 0x4F, 0x44, 140),
            legal_move_dot: Color32::from_rgba_unmultiplied(20, 22, 18, 85),
            legal_capture_ring: Color32::from_rgba_unmultiplied(20, 22, 18, 110),
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
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0_f32);

    visuals.widgets.inactive.bg_fill = tokens.bg_surface_elevated;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, tokens.border_subtle);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);
    visuals.widgets.inactive.rounding = Rounding::same(6.0_f32);

    visuals.widgets.hovered.bg_fill = if dark_mode {
        Color32::from_rgb(0x48, 0x45, 0x40)
    } else {
        Color32::from_rgb(0xE8, 0xE6, 0xE0)
    };
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.2_f32, tokens.accent_primary);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, tokens.text_primary);
    visuals.widgets.hovered.rounding = Rounding::same(6.0_f32);

    visuals.widgets.active.bg_fill = tokens.accent_primary;
    visuals.widgets.active.bg_stroke = Stroke::new(1.5_f32, tokens.accent_primary_hover);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(6.0_f32);

    style.visuals = visuals;
    ctx.set_style(style);
}
