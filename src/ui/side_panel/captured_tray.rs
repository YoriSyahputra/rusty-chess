use crate::{domain::ChessMatchState, ui::piece_renderer::draw_chess_piece};
use egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Color, Position, Role};

/// Merender Bagian 2 SidePanel: Deretan bidak yang dimakan oleh Bot dan Pemain,
/// selisih poin material (`+3`), serta indikator jam catur masing-masing sisi.
pub fn render_captured_tray(ui: &mut Ui, match_state: &ChessMatchState, bot_name: &str) {
    let human_color = match_state.human_color;
    let bot_color = human_color.other();

    let (bot_captured, human_captured) = match human_color {
        Color::White => (
            &match_state.captured_by_black,
            &match_state.captured_by_white,
        ),
        Color::Black => (
            &match_state.captured_by_white,
            &match_state.captured_by_black,
        ),
    };

    // Selisih poin dari perspektif masing-masing sisi
    let white_adv = match_state.material_balance;
    let bot_adv = match bot_color {
        Color::White => white_adv,
        Color::Black => -white_adv,
    };
    let human_adv = -bot_adv;

    egui::Frame::none()
        .fill(Color32::from_rgb(27, 31, 39))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(48, 54, 68)))
        .rounding(Rounding::same(7.0_f32))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            // Baris Bot
            render_player_capture_row(
                ui,
                bot_name,
                bot_color,
                bot_captured,
                bot_adv,
                &match_state.clock.format_time(bot_color),
                match_state.current_pos.turn() == bot_color
                    && !match_state.outcome.is_game_over(),
            );

            ui.separator();

            // Baris Pemain (Human)
            render_player_capture_row(
                ui,
                "You (Player)",
                human_color,
                human_captured,
                human_adv,
                &match_state.clock.format_time(human_color),
                match_state.current_pos.turn() == human_color
                    && !match_state.outcome.is_game_over(),
            );
        });
}

fn render_player_capture_row(
    ui: &mut Ui,
    label: &str,
    side: Color,
    captured_pieces: &[Role],
    advantage_pts: i32,
    clock_str: &str,
    is_active_turn: bool,
) {
    let captured_color = side.other();

    ui.horizontal(|ui| {
        let side_badge = match side {
            Color::White => "⬜",
            Color::Black => "⬛",
        };
        ui.label(RichText::new(side_badge).size(11.5));
        ui.label(
            RichText::new(label)
                .strong()
                .size(12.5)
                .color(if is_active_turn {
                    Color32::from_rgb(255, 225, 115)
                } else {
                    Color32::from_rgb(215, 220, 232)
                }),
        );

        if advantage_pts > 0 {
            egui::Frame::none()
                .fill(Color32::from_rgb(42, 78, 54))
                .rounding(Rounding::same(4.0_f32))
                .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("+{advantage_pts}"))
                            .strong()
                            .size(11.0)
                            .color(Color32::from_rgb(130, 240, 165)),
                    );
                });
        }

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let clock_bg = if is_active_turn {
                Color32::from_rgb(52, 68, 95)
            } else {
                Color32::from_rgb(35, 39, 48)
            };
            egui::Frame::none()
                .fill(clock_bg)
                .rounding(Rounding::same(4.0_f32))
                .inner_margin(egui::Margin::symmetric(6.0, 2.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(clock_str)
                            .monospace()
                            .strong()
                            .size(12.0)
                            .color(Color32::WHITE),
                    );
                });
        });
    });

    // Deretan mini ikon bidak yang berhasil dimakan
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 1.5;
        if captured_pieces.is_empty() {
            ui.label(
                RichText::new("No captures yet")
                    .italics()
                    .size(11.0)
                    .color(Color32::from_rgb(115, 122, 138)),
            );
        } else {
            for &role in captured_pieces {
                let (icon_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(18.0), Sense::hover());
                draw_chess_piece(
                    ui.painter(),
                    icon_rect.center(),
                    17.5,
                    captured_color,
                    role,
                    false,
                );
            }
        }
    });
}
