use egui::{Color32, Context, Rounding, Stroke, Style, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum BoardTheme {
    ClassicGreen,
    LichessWood,
    TournamentBlue,
    CyberpunkDark,
}

#[derive(Debug, Clone, Copy)]
pub struct BoardPalette {
    pub light_square: Color32,
    pub dark_square: Color32,
    pub border_bg: Color32,
    pub selected_square: Color32,
    pub last_move_from: Color32,
    pub last_move_to: Color32,
    pub legal_move_dot: Color32,
    pub legal_capture_ring: Color32,
    pub check_glow_inner: Color32,
    pub check_glow_outer: Color32,
    pub arrow_tactical: Color32,
    pub arrow_hint: Color32,
    pub marked_square: Color32,
}

impl BoardTheme {
    pub fn all() -> &'static [BoardTheme] {
        &[
            BoardTheme::ClassicGreen,
            BoardTheme::LichessWood,
            BoardTheme::TournamentBlue,
            BoardTheme::CyberpunkDark,
        ]
    }

    pub fn name(&self) -> &'static str {
        match self {
            BoardTheme::ClassicGreen => "Classic Green",
            BoardTheme::LichessWood => "Lichess Wood",
            BoardTheme::TournamentBlue => "Tournament Blue",
            BoardTheme::CyberpunkDark => "Cyberpunk Dark",
        }
    }

    pub fn palette(&self) -> BoardPalette {
        match self {
            BoardTheme::ClassicGreen => BoardPalette {
                light_square: Color32::from_rgb(235, 236, 208),
                dark_square: Color32::from_rgb(115, 149, 82),
                border_bg: Color32::from_rgb(31, 35, 30),
                selected_square: Color32::from_rgba_unmultiplied(186, 202, 43, 185),
                last_move_from: Color32::from_rgba_unmultiplied(245, 246, 130, 140),
                last_move_to: Color32::from_rgba_unmultiplied(245, 246, 130, 175),
                legal_move_dot: Color32::from_rgba_unmultiplied(20, 28, 18, 90),
                legal_capture_ring: Color32::from_rgba_unmultiplied(20, 28, 18, 110),
                check_glow_inner: Color32::from_rgba_unmultiplied(235, 45, 45, 210),
                check_glow_outer: Color32::from_rgba_unmultiplied(200, 20, 20, 100),
                arrow_tactical: Color32::from_rgba_unmultiplied(255, 165, 20, 205),
                arrow_hint: Color32::from_rgba_unmultiplied(46, 213, 115, 220),
                marked_square: Color32::from_rgba_unmultiplied(235, 65, 65, 150),
            },
            BoardTheme::LichessWood => BoardPalette {
                light_square: Color32::from_rgb(240, 217, 181),
                dark_square: Color32::from_rgb(181, 136, 99),
                border_bg: Color32::from_rgb(42, 34, 27),
                selected_square: Color32::from_rgba_unmultiplied(130, 151, 105, 185),
                last_move_from: Color32::from_rgba_unmultiplied(205, 210, 106, 140),
                last_move_to: Color32::from_rgba_unmultiplied(205, 210, 106, 175),
                legal_move_dot: Color32::from_rgba_unmultiplied(32, 85, 30, 105),
                legal_capture_ring: Color32::from_rgba_unmultiplied(32, 85, 30, 130),
                check_glow_inner: Color32::from_rgba_unmultiplied(235, 45, 45, 210),
                check_glow_outer: Color32::from_rgba_unmultiplied(200, 20, 20, 100),
                arrow_tactical: Color32::from_rgba_unmultiplied(245, 155, 30, 205),
                arrow_hint: Color32::from_rgba_unmultiplied(46, 213, 115, 220),
                marked_square: Color32::from_rgba_unmultiplied(235, 65, 65, 150),
            },
            BoardTheme::TournamentBlue => BoardPalette {
                light_square: Color32::from_rgb(222, 227, 230),
                dark_square: Color32::from_rgb(100, 146, 176),
                border_bg: Color32::from_rgb(24, 33, 42),
                selected_square: Color32::from_rgba_unmultiplied(110, 205, 165, 180),
                last_move_from: Color32::from_rgba_unmultiplied(155, 199, 215, 155),
                last_move_to: Color32::from_rgba_unmultiplied(155, 199, 215, 190),
                legal_move_dot: Color32::from_rgba_unmultiplied(18, 35, 50, 95),
                legal_capture_ring: Color32::from_rgba_unmultiplied(18, 35, 50, 120),
                check_glow_inner: Color32::from_rgba_unmultiplied(235, 45, 45, 210),
                check_glow_outer: Color32::from_rgba_unmultiplied(200, 20, 20, 100),
                arrow_tactical: Color32::from_rgba_unmultiplied(255, 165, 20, 205),
                arrow_hint: Color32::from_rgba_unmultiplied(46, 213, 115, 220),
                marked_square: Color32::from_rgba_unmultiplied(235, 65, 65, 150),
            },
            BoardTheme::CyberpunkDark => BoardPalette {
                light_square: Color32::from_rgb(58, 64, 88),
                dark_square: Color32::from_rgb(31, 35, 52),
                border_bg: Color32::from_rgb(17, 19, 29),
                selected_square: Color32::from_rgba_unmultiplied(0, 225, 255, 115),
                last_move_from: Color32::from_rgba_unmultiplied(175, 85, 255, 110),
                last_move_to: Color32::from_rgba_unmultiplied(175, 85, 255, 145),
                legal_move_dot: Color32::from_rgba_unmultiplied(0, 240, 255, 130),
                legal_capture_ring: Color32::from_rgba_unmultiplied(255, 65, 165, 160),
                check_glow_inner: Color32::from_rgba_unmultiplied(255, 35, 85, 220),
                check_glow_outer: Color32::from_rgba_unmultiplied(200, 15, 60, 110),
                arrow_tactical: Color32::from_rgba_unmultiplied(255, 185, 0, 215),
                arrow_hint: Color32::from_rgba_unmultiplied(0, 245, 160, 225),
                marked_square: Color32::from_rgba_unmultiplied(255, 45, 95, 155),
            },
        }
    }
}

pub fn apply_global_style(ctx: &Context) {
    let mut style: Style = (*ctx.style()).clone();
    let mut visuals = Visuals::dark();

    visuals.panel_fill = Color32::from_rgb(22, 24, 30);
    visuals.window_fill = Color32::from_rgb(28, 31, 38);
    visuals.faint_bg_color = Color32::from_rgb(30, 34, 42);
    visuals.extreme_bg_color = Color32::from_rgb(16, 18, 23);

    visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(28, 31, 38);
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(48, 53, 66));
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(222, 226, 235));
    visuals.widgets.noninteractive.rounding = Rounding::same(6.0);

    visuals.widgets.inactive.bg_fill = Color32::from_rgb(38, 43, 54);
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Color32::from_rgb(58, 65, 82));
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Color32::from_rgb(232, 236, 244));
    visuals.widgets.inactive.rounding = Rounding::same(6.0);

    visuals.widgets.hovered.bg_fill = Color32::from_rgb(52, 60, 76);
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.2, Color32::from_rgb(100, 155, 240));
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.rounding = Rounding::same(6.0);

    visuals.widgets.active.bg_fill = Color32::from_rgb(65, 115, 195);
    visuals.widgets.active.bg_stroke = Stroke::new(1.5, Color32::from_rgb(135, 185, 255));
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.rounding = Rounding::same(6.0);

    style.visuals = visuals;
    ctx.set_style(style);
}
