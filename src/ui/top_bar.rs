use crate::{
    config::UserConfig,
    ui::{piece_renderer::draw_chess_piece, theme::BoardTheme},
};
use egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Color, Role};

#[derive(Debug, Default)]
pub struct TopBarActions {
    pub open_new_game_modal: bool,
    pub open_fen_pgn_modal: bool,
    pub flip_board_toggled: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn render_top_bar(
    ui: &mut Ui,
    config: &mut UserConfig,
    is_thinking: bool,
    thinking_depth: u8,
    thinking_max_depth: u8,
    nps_kns: f32,
    fps: f32,
    rss_mb: f32,
) -> TopBarActions {
    let mut actions = TopBarActions::default();

    ui.horizontal_centered(|ui| {
        ui.add_space(6.0);

        // Kiri: Logo vektor ikon kuda + judul bold "RUSTY CHESS" + badge versi v0.1.0
        let (logo_rect, _) = ui.allocate_exact_size(Vec2::splat(28.0), Sense::hover());
        draw_chess_piece(
            ui.painter(),
            logo_rect.center(),
            26.0,
            Color::White,
            Role::Knight,
            false,
        );

        ui.label(
            RichText::new("RUSTY CHESS")
                .strong()
                .size(16.5)
                .color(Color32::from_rgb(240, 244, 252)),
        );

        egui::Frame::none()
            .fill(Color32::from_rgb(45, 54, 72))
            .rounding(Rounding::same(4.0_f32))
            .inner_margin(egui::Margin::symmetric(6.0, 2.0))
            .show(ui, |ui| {
                ui.label(
                    RichText::new("v0.2.0")
                        .size(11.0)
                        .color(Color32::from_rgb(155, 195, 255)),
                );
            });

        ui.separator();

        // Tengah: Pemilih cepat Board Theme, Flip Board, Sound Toggle, FEN/PGN Modal
        ui.label(RichText::new("Theme:").size(12.5).color(Color32::LIGHT_GRAY));
        egui::ComboBox::from_id_salt("board_theme_combo")
            .selected_text(config.board_theme.name())
            .width(135.0)
            .show_ui(ui, |ui| {
                for &theme in BoardTheme::all() {
                    ui.selectable_value(&mut config.board_theme, theme, theme.name());
                }
            });

        if ui
            .button(RichText::new("⇅ Flip Board").size(12.5))
            .on_hover_text("Putar orientasi papan (Shortcut: F)")
            .clicked()
        {
            config.board_flipped = !config.board_flipped;
            actions.flip_board_toggled = true;
        }

        let sound_label = if config.sound_enabled {
            "🔊 Sound: ON"
        } else {
            "🔇 Sound: OFF"
        };
        if ui.button(RichText::new(sound_label).size(12.5)).clicked() {
            config.sound_enabled = !config.sound_enabled;
        }

        let xray_label = if config.show_xray_heatmap {
            "👁 X-Ray: ON"
        } else {
            "👁 X-Ray"
        };
        if ui
            .button(RichText::new(xray_label).size(12.5).color(
                if config.show_xray_heatmap {
                    Color32::from_rgb(0x00, 0x9F, 0xD9)
                } else {
                    Color32::from_rgb(230, 234, 242)
                },
            ))
            .on_hover_text("Toggle X-Ray Square Control & Hanging Piece Heatmap (Shortcut: X)")
            .clicked()
        {
            config.show_xray_heatmap = !config.show_xray_heatmap;
        }

        if ui
            .button(RichText::new("📋 FEN / PGN").size(12.5))
            .on_hover_text("Ekspor atau impor posisi FEN & riwayat PGN")
            .clicked()
        {
            actions.open_fen_pgn_modal = true;
        }

        if ui
            .button(RichText::new("⚔ New Game").size(12.5))
            .clicked()
        {
            actions.open_new_game_modal = true;
        }

        // Kanan: Indikator status Engine, kN/s, RAM, dan FPS counter
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(8.0);

            egui::Frame::none()
                .fill(Color32::from_rgb(30, 34, 43))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(54, 62, 78)))
                .rounding(Rounding::same(5.0_f32))
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("{fps:.0} FPS | {rss_mb:.1} MB"))
                            .monospace()
                            .size(11.5)
                            .color(Color32::from_rgb(165, 178, 198)),
                    );
                });

            let (status_text, status_color, dot_color) = if is_thinking {
                (
                    format!(
                        "Thinking Depth {thinking_depth}/{thinking_max_depth} ({nps_kns:.1} kN/s)"
                    ),
                    Color32::from_rgb(255, 210, 95),
                    Color32::from_rgb(255, 185, 40),
                )
            } else {
                (
                    format!("Engine Idle ({nps_kns:.1} kN/s)"),
                    Color32::from_rgb(145, 225, 165),
                    Color32::from_rgb(65, 210, 115),
                )
            };

            egui::Frame::none()
                .fill(Color32::from_rgb(28, 33, 42))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(54, 62, 78)))
                .rounding(Rounding::same(5.0_f32))
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 4.0, dot_color);
                        ui.label(
                            RichText::new(status_text)
                                .monospace()
                                .size(11.5)
                                .color(status_color),
                        );
                    });
                });
        });
    });

    actions
}
