use crate::{
    domain::ChessMatchState,
    engine::bot::BotPersona,
    ui::piece_renderer::{draw_bot_avatar, draw_chess_piece},
};
use egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Color, Position, Role};

/// Merender Player Strip (`width = strip_width`, `height = 44.0 px`) di atas atau bawah papan catur
/// sesuai spesifikasi NEW_DESIGN.md Section 5.1.
#[allow(clippy::too_many_arguments)]
pub fn render_player_strip(
    ui: &mut Ui,
    strip_width: f32,
    match_state: &ChessMatchState,
    persona: &BotPersona,
    is_bot_strip: bool,
    is_thinking: bool,
    dark_mode: bool,
) {
    let human_color = match_state.human_color;
    let bot_color = human_color.other();
    let strip_side = if is_bot_strip { bot_color } else { human_color };

    let captured_pieces = match strip_side {
        Color::White => &match_state.captured_by_white,
        Color::Black => &match_state.captured_by_black,
    };

    let white_adv = match_state.material_balance;
    let side_adv = match strip_side {
        Color::White => white_adv,
        Color::Black => -white_adv,
    };

    let is_active_turn =
        match_state.current_pos.turn() == strip_side && !match_state.outcome.is_game_over();
    let clock_str = match_state.clock.format_time(strip_side);

    let bg_fill = if dark_mode {
        Color32::from_rgb(0x30, 0x2E, 0x2B)
    } else {
        Color32::from_rgb(0xFF, 0xFF, 0xFF)
    };
    let border_color = if is_active_turn {
        Color32::from_rgb(0x81, 0xB6, 0x4C)
    } else if dark_mode {
        Color32::from_rgb(0x42, 0x3F, 0x3B)
    } else {
        Color32::from_rgb(0xD6, 0xD4, 0xCE)
    };

    let text_primary = if dark_mode {
        Color32::from_rgb(0xF5, 0xF5, 0xF5)
    } else {
        Color32::from_rgb(0x22, 0x20, 0x1E)
    };

    ui.allocate_ui(Vec2::new(strip_width, 44.0), |ui| {
        egui::Frame::none()
            .fill(bg_fill)
            .stroke(Stroke::new(if is_active_turn { 1.6_f32 } else { 1.0_f32 }, border_color))
            .rounding(Rounding::same(6.0_f32))
            .inner_margin(egui::Margin::symmetric(10.0, 5.0))
            .show(ui, |ui| {
                ui.set_width((strip_width - 22.0).max(160.0));
                ui.set_height(32.0);

                ui.horizontal_centered(|ui| {
                    // 1. Avatar Badge (32x32 px) dengan glowing turn border
                    let (avatar_rect, _) =
                        ui.allocate_exact_size(Vec2::splat(32.0), Sense::hover());
                    if is_bot_strip {
                        draw_bot_avatar(
                            ui.painter(),
                            avatar_rect,
                            persona.id,
                            persona.accent_rgb,
                            is_thinking || is_active_turn,
                        );
                    } else {
                        let p_painter = ui.painter();
                        let center = avatar_rect.center();
                        let accent = if is_active_turn {
                            Color32::from_rgb(0x81, 0xB6, 0x4C)
                        } else {
                            Color32::from_rgb(120, 130, 145)
                        };
                        p_painter.circle_filled(center, 15.0, Color32::from_rgb(38, 42, 48));
                        p_painter.circle_stroke(center, 15.0, Stroke::new(2.0_f32, accent));
                        draw_chess_piece(
                            p_painter,
                            center,
                            20.0,
                            human_color,
                            Role::King,
                            false,
                        );
                    }

                    ui.add_space(6.0);

                    // 2. Player / Bot Name + ELO Pill
                    if is_bot_strip {
                        ui.label(
                            RichText::new(persona.name)
                                .strong()
                                .size(13.5)
                                .color(text_primary),
                        );

                        let elo_display = if persona.code == "mittens" {
                            "1".to_string()
                        } else {
                            format!("{}", persona.elo)
                        };

                        egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(0xFF, 0xAA, 0x00, 38))
                            .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0xFF, 0xAA, 0x00)))
                            .rounding(Rounding::same(4.0_f32))
                            .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("({elo_display})"))
                                        .strong()
                                        .size(11.0)
                                        .color(Color32::from_rgb(0xFF, 0xAA, 0x00)),
                                );
                            });

                        // 3. Thinking indicator dots
                        if is_thinking {
                            ui.add_space(4.0);
                            egui::Frame::none()
                                .fill(Color32::from_rgba_unmultiplied(0x81, 0xB6, 0x4C, 42))
                                .rounding(Rounding::same(4.0_f32))
                                .inner_margin(egui::Margin::symmetric(6.0, 1.5))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new("Thinking...")
                                            .italics()
                                            .size(11.0)
                                            .color(Color32::from_rgb(0x95, 0xC9, 0x5D)),
                                    );
                                });
                        }
                    } else {
                        ui.label(
                            RichText::new("You")
                                .strong()
                                .size(13.5)
                                .color(text_primary),
                        );
                    }

                    ui.add_space(8.0);

                    // 4. Captured Piece Silhouettes + Material Differential (+3)
                    let captured_piece_color = strip_side.other();
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = 1.0;
                        for &role in captured_pieces.iter().take(12) {
                            let (icon_rect, _) =
                                ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            draw_chess_piece(
                                ui.painter(),
                                icon_rect.center(),
                                15.5,
                                captured_piece_color,
                                role,
                                false,
                            );
                        }
                        if side_adv > 0 {
                            ui.add_space(3.0);
                            ui.label(
                                RichText::new(format!("+{side_adv}"))
                                    .strong()
                                    .size(11.5)
                                    .color(Color32::from_rgb(0x81, 0xB6, 0x4C)),
                            );
                        }
                    });

                    // 5. High-Contrast Digital Clock Display (Right-Aligned)
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let clock_bg = if is_active_turn {
                            if strip_side == Color::White {
                                Color32::from_rgb(238, 240, 235)
                            } else {
                                Color32::from_rgb(0x21, 0x1F, 0x1C)
                            }
                        } else if dark_mode {
                            Color32::from_rgb(0x26, 0x24, 0x21)
                        } else {
                            Color32::from_rgb(0xE6, 0xE6, 0xE2)
                        };

                        let clock_fg = if is_active_turn && strip_side == Color::White {
                            Color32::from_rgb(0x21, 0x1F, 0x1C)
                        } else if is_active_turn {
                            Color32::WHITE
                        } else if dark_mode {
                            Color32::from_rgb(0xB0, 0xAC, 0xA6)
                        } else {
                            Color32::from_rgb(0x66, 0x62, 0x5C)
                        };

                        egui::Frame::none()
                            .fill(clock_bg)
                            .stroke(Stroke::new(
                                1.0_f32,
                                if is_active_turn {
                                    Color32::from_rgb(0x81, 0xB6, 0x4C)
                                } else {
                                    Color32::from_rgb(75, 72, 68)
                                },
                            ))
                            .rounding(Rounding::same(5.0_f32))
                            .inner_margin(egui::Margin::symmetric(10.0, 3.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(&clock_str)
                                        .monospace()
                                        .strong()
                                        .size(13.5)
                                        .color(clock_fg),
                                );
                            });
                    });
                });
            });
    });
}
