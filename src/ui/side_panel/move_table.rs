use crate::domain::ChessMatchState;
use egui::{Color32, RichText, Rounding, ScrollArea, Stroke, Ui};

/// Merender Bagian 3 SidePanel: Tabel Riwayat Langkah 3 Kolom (`#No`, `White Move`, `Black Move`)
/// dengan klik untuk History Inspection Mode serta 4 tombol navigasi (`|<<`, `<`, `>`, `>>|`).
pub fn render_move_table(ui: &mut Ui, match_state: &mut ChessMatchState, available_height: f32) {
    let table_h = (available_height - 38.0).max(90.0);

    egui::Frame::none()
        .fill(Color32::from_rgb(22, 25, 32))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(46, 52, 66)))
        .rounding(Rounding::same(7.0_f32))
        .inner_margin(egui::Margin::same(6.0))
        .show(ui, |ui| {
            // Header kolom
            ui.horizontal(|ui| {
                ui.add_sized(
                    [42.0, 18.0],
                    egui::Label::new(
                        RichText::new("#No")
                            .strong()
                            .size(11.5)
                            .color(Color32::from_rgb(145, 155, 175)),
                    ),
                );
                ui.add_sized(
                    [125.0, 18.0],
                    egui::Label::new(
                        RichText::new("White Move")
                            .strong()
                            .size(11.5)
                            .color(Color32::from_rgb(145, 155, 175)),
                    ),
                );
                ui.add_sized(
                    [125.0, 18.0],
                    egui::Label::new(
                        RichText::new("Black Move")
                            .strong()
                            .size(11.5)
                            .color(Color32::from_rgb(145, 155, 175)),
                    ),
                );
            });

            ui.separator();

            let mut clicked_ply: Option<usize> = None;
            let viewed_ply = match_state.viewed_ply;
            let total_plies = match_state.history.len();

            ScrollArea::vertical()
                .id_salt("algebraic_move_history_scroll")
                .max_height(table_h)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if match_state.history.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(24.0);
                            ui.label(
                                RichText::new("1. Make your first move on the board")
                                    .italics()
                                    .size(12.0)
                                    .color(Color32::from_rgb(120, 128, 145)),
                            );
                        });
                        return;
                    }

                    for (pair_idx, chunk) in match_state.history.chunks(2).enumerate() {
                        let move_num = pair_idx + 1;
                        ui.horizontal(|ui| {
                            ui.add_sized(
                                [42.0, 20.0],
                                egui::Label::new(
                                    RichText::new(format!("{move_num}."))
                                        .monospace()
                                        .size(12.0)
                                        .color(Color32::from_rgb(135, 145, 165)),
                                ),
                            );

                            // White move button
                            if let Some(w_rec) = chunk.first() {
                                let is_selected = viewed_ply == w_rec.ply;
                                let btn = egui::SelectableLabel::new(
                                    is_selected,
                                    RichText::new(&w_rec.san).monospace().size(12.5),
                                );
                                if ui.add_sized([125.0, 20.0], btn).clicked() {
                                    clicked_ply = Some(w_rec.ply);
                                }
                            }

                            // Black move button
                            if let Some(b_rec) = chunk.get(1) {
                                let is_selected = viewed_ply == b_rec.ply;
                                let btn = egui::SelectableLabel::new(
                                    is_selected,
                                    RichText::new(&b_rec.san).monospace().size(12.5),
                                );
                                if ui.add_sized([125.0, 20.0], btn).clicked() {
                                    clicked_ply = Some(b_rec.ply);
                                }
                            }
                        });
                    }
                });

            if let Some(ply) = clicked_ply {
                match_state.jump_to_ply(ply);
            }

            ui.separator();

            // 4 Tombol Navigasi History: |<< (Awal), < (Mundur), > (Maju), >>| (Posisi Terkini)
            ui.horizontal(|ui| {
                let btn_w = (ui.available_width() - 24.0) / 4.0;
                if ui
                    .add_enabled(
                        viewed_ply > 0,
                        egui::Button::new(RichText::new("|<<").monospace().strong())
                            .min_size([btn_w, 24.0].into()),
                    )
                    .on_hover_text("Posisi Awal (Home)")
                    .clicked()
                {
                    match_state.step_history_first();
                }

                if ui
                    .add_enabled(
                        viewed_ply > 0,
                        egui::Button::new(RichText::new("<").monospace().strong())
                            .min_size([btn_w, 24.0].into()),
                    )
                    .on_hover_text("Langkah Sebelumnya (Left Arrow)")
                    .clicked()
                {
                    match_state.step_history_prev();
                }

                if ui
                    .add_enabled(
                        viewed_ply < total_plies,
                        egui::Button::new(RichText::new(">").monospace().strong())
                            .min_size([btn_w, 24.0].into()),
                    )
                    .on_hover_text("Langkah Berikutnya (Right Arrow)")
                    .clicked()
                {
                    match_state.step_history_next();
                }

                if ui
                    .add_enabled(
                        viewed_ply < total_plies,
                        egui::Button::new(RichText::new(">>|").monospace().strong())
                            .min_size([btn_w, 24.0].into()),
                    )
                    .on_hover_text("Posisi Terkini (End)")
                    .clicked()
                {
                    match_state.step_history_last();
                }
            });
        });
}
