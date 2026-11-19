use crate::{
    config::NotationStyle,
    domain::ChessMatchState,
    engine::review::LiveMoveEval,
};
use egui::{Color32, RichText, Rounding, ScrollArea, Stroke, Ui};

/// Mengonversi SAN standar ke Figurine Algebraic Notation (misal `"Nf3"` -> `"♘f3"`).
pub fn format_san_with_style(san: &str, style: NotationStyle, is_white: bool) -> String {
    if style == NotationStyle::Text || san.is_empty() {
        return san.to_string();
    }
    let mut chars = san.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return String::new(),
    };
    let rest: String = chars.collect();
    let sym = match (first, is_white) {
        ('K', true) => "♔",
        ('Q', true) => "♕",
        ('R', true) => "♖",
        ('B', true) => "♗",
        ('N', true) => "♘",
        ('K', false) => "♚",
        ('Q', false) => "♛",
        ('R', false) => "♜",
        ('B', false) => "♝",
        ('N', false) => "♞",
        _ => return san.to_string(),
    };
    format!("{sym}{rest}")
}

/// Merender Dynamic Center Region (`ScrollArea::vertical().max_height(max_height)`)
/// hanya untuk daftar langkah (tanpa tombol di dalamnya) sehingga tidak pernah meluap!
pub fn render_move_table(
    ui: &mut Ui,
    match_state: &mut ChessMatchState,
    max_height: f32,
    notation_style: NotationStyle,
    show_move_feedback: bool,
    live_evals: &[LiveMoveEval],
) {
    let clamped_h = max_height.max(80.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(0x26, 0x24, 0x21))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x42, 0x3F, 0x3B)))
        .rounding(Rounding::same(6.0_f32))
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            ui.set_height(clamped_h);

            // Header kolom (#No | White | Black)
            ui.horizontal(|ui| {
                ui.add_sized(
                    [40.0, 18.0],
                    egui::Label::new(
                        RichText::new("#No")
                            .strong()
                            .size(11.5)
                            .color(Color32::from_rgb(0xB0, 0xAC, 0xA6)),
                    ),
                );
                ui.add_sized(
                    [132.0, 18.0],
                    egui::Label::new(
                        RichText::new("White")
                            .strong()
                            .size(11.5)
                            .color(Color32::from_rgb(0xB0, 0xAC, 0xA6)),
                    ),
                );
                ui.add_sized(
                    [132.0, 18.0],
                    egui::Label::new(
                        RichText::new("Black")
                            .strong()
                            .size(11.5)
                            .color(Color32::from_rgb(0xB0, 0xAC, 0xA6)),
                    ),
                );
            });

            ui.separator();

            let scroll_h = (clamped_h - 32.0).max(52.0);
            let mut clicked_ply: Option<usize> = None;
            let viewed_ply = match_state.viewed_ply;

            ScrollArea::vertical()
                .id_salt("algebraic_move_history_scroll")
                .max_height(scroll_h)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if match_state.history.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(18.0);
                            ui.label(
                                RichText::new("Make a move on the board to begin")
                                    .italics()
                                    .size(12.0)
                                    .color(Color32::from_rgb(0xB0, 0xAC, 0xA6)),
                            );
                        });
                        return;
                    }

                    for (pair_idx, chunk) in match_state.history.chunks(2).enumerate() {
                        let move_num = pair_idx + 1;
                        let row_bg = if pair_idx % 2 == 0 {
                            Color32::TRANSPARENT
                        } else {
                            Color32::from_rgba_unmultiplied(255, 255, 255, 6)
                        };

                        egui::Frame::none().fill(row_bg).show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.add_sized(
                                    [40.0, 20.0],
                                    egui::Label::new(
                                        RichText::new(format!("{move_num}."))
                                            .monospace()
                                            .size(12.0)
                                            .color(Color32::from_rgb(0xB0, 0xAC, 0xA6)),
                                    ),
                                );

                                // White move
                                if let Some(w_rec) = chunk.first() {
                                    let live = live_evals.iter().find(|e| e.ply == w_rec.ply);
                                    let (badge, col_opt) = if show_move_feedback {
                                        if let Some(le) = live {
                                            let rgb = le.classification.rgb();
                                            (
                                                format!(" {}", le.classification.symbol()),
                                                Some(Color32::from_rgb(rgb[0], rgb[1], rgb[2])),
                                            )
                                        } else {
                                            let prev_cp = if w_rec.ply >= 2 {
                                                match_state
                                                    .history
                                                    .get(w_rec.ply - 2)
                                                    .map(|r| r.eval_cp)
                                                    .unwrap_or(0)
                                            } else {
                                                0
                                            };
                                            (
                                                inline_feedback_badge(w_rec.eval_cp - prev_cp)
                                                    .to_string(),
                                                None,
                                            )
                                        }
                                    } else {
                                        (String::new(), None)
                                    };
                                    let display_san = format!(
                                        "{}{badge}",
                                        format_san_with_style(&w_rec.san, notation_style, true)
                                    );
                                    let is_selected = viewed_ply == w_rec.ply;
                                    let mut rt = RichText::new(display_san).monospace().size(12.5);
                                    if let Some(c) = col_opt {
                                        if !is_selected {
                                            rt = rt.color(c);
                                        }
                                    }
                                    let btn = egui::SelectableLabel::new(is_selected, rt);
                                    if ui.add_sized([132.0, 20.0], btn).clicked() {
                                        clicked_ply = Some(w_rec.ply);
                                    }
                                }

                                // Black move
                                if let Some(b_rec) = chunk.get(1) {
                                    let live = live_evals.iter().find(|e| e.ply == b_rec.ply);
                                    let (badge, col_opt) = if show_move_feedback {
                                        if let Some(le) = live {
                                            let rgb = le.classification.rgb();
                                            (
                                                format!(" {}", le.classification.symbol()),
                                                Some(Color32::from_rgb(rgb[0], rgb[1], rgb[2])),
                                            )
                                        } else {
                                            let prev_cp = match_state
                                                .history
                                                .get(b_rec.ply - 2)
                                                .map(|r| r.eval_cp)
                                                .unwrap_or(0);
                                            (
                                                inline_feedback_badge(prev_cp - b_rec.eval_cp)
                                                    .to_string(),
                                                None,
                                            )
                                        }
                                    } else {
                                        (String::new(), None)
                                    };
                                    let display_san = format!(
                                        "{}{badge}",
                                        format_san_with_style(&b_rec.san, notation_style, false)
                                    );
                                    let is_selected = viewed_ply == b_rec.ply;
                                    let mut rt = RichText::new(display_san).monospace().size(12.5);
                                    if let Some(c) = col_opt {
                                        if !is_selected {
                                            rt = rt.color(c);
                                        }
                                    }
                                    let btn = egui::SelectableLabel::new(is_selected, rt);
                                    if ui.add_sized([132.0, 20.0], btn).clicked() {
                                        clicked_ply = Some(b_rec.ply);
                                    }
                                }
                            });
                        });
                    }
                });

            if let Some(ply) = clicked_ply {
                match_state.jump_to_ply(ply);
            }
        });
}

fn inline_feedback_badge(delta_for_mover_cp: i32) -> &'static str {
    if delta_for_mover_cp <= -220 {
        " ??"
    } else if delta_for_mover_cp <= -110 {
        " ?"
    } else if delta_for_mover_cp <= -60 {
        " ?!"
    } else if delta_for_mover_cp >= 140 {
        " !"
    } else {
        ""
    }
}
