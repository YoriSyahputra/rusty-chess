use crate::config::UserConfig;
use egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Default)]
pub struct TopBarActions {
    pub open_new_game_modal: bool,
    pub open_fen_pgn_modal: bool,
    pub open_appearance_modal: bool,
    pub flip_board_toggled: bool,
    pub toggle_zen_mode: bool,
    pub open_command_palette: bool,
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
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.add_space(4.0);

        // 1. Board & Pieces Visual Preview Launcher
        if ui
            .button(RichText::new("◈ Board & Pieces").size(12.0))
            .on_hover_text("Preview & choose Board Themes and Chess.com Piece Sets")
            .clicked()
        {
            actions.open_appearance_modal = true;
        }

        // 2. Live Instant Analysis Toggle (Feature: Real-Time Blunder / Brilliant Indicator)
        let live_on = config.live_analysis_enabled;
        let live_label = if live_on {
            "⚡ Live Eval: ON"
        } else {
            "⚡ Live Eval: OFF"
        };
        if ui
            .add(
                egui::Button::new(RichText::new(live_label).size(12.0).color(if live_on {
                    Color32::from_rgb(0x95, 0xD8, 0x62)
                } else {
                    Color32::from_rgb(180, 178, 174)
                }))
                .rounding(Rounding::same(8.0_f32)),
            )
            .on_hover_text("Toggle instant live move classification (!! / ★ / ?! / ? / ??) as you play")
            .clicked()
        {
            config.live_analysis_enabled = !config.live_analysis_enabled;
        }

        // 3. X-Ray Heatmap Toggle
        let xray_on = config.show_xray_heatmap;
        if ui
            .button(
                RichText::new(if xray_on { "👁 X-Ray: ON" } else { "👁 X-Ray" })
                    .size(12.0)
                    .color(if xray_on {
                        Color32::from_rgb(0x38, 0xC8, 0xFA)
                    } else {
                        Color32::from_rgb(225, 224, 220)
                    }),
            )
            .on_hover_text("Toggle X-Ray Square Control Heatmap (Shortcut: X)")
            .clicked()
        {
            config.show_xray_heatmap = !config.show_xray_heatmap;
        }

        // 4. Sound Toggle
        let sound_txt = if config.sound_enabled {
            "🔊 Sound"
        } else {
            "🔇 Muted"
        };
        if ui
            .button(RichText::new(sound_txt).size(12.0))
            .on_hover_text("Toggle procedural chess sound effects")
            .clicked()
        {
            config.sound_enabled = !config.sound_enabled;
        }

        // 5. Zen Mode
        if ui
            .button(RichText::new("☾ Zen").size(12.0))
            .on_hover_text("Distraction-free focus mode (Shortcut: Z)")
            .clicked()
        {
            actions.toggle_zen_mode = true;
        }

        // 6. FEN / PGN
        if ui
            .button(RichText::new("☰ FEN/PGN").size(12.0))
            .on_hover_text("Import / Export FEN & PGN")
            .clicked()
        {
            actions.open_fen_pgn_modal = true;
        }

        // 7. Command Palette
        if ui
            .button(RichText::new("⌘ Ctrl+K").size(12.0))
            .on_hover_text("Open Spotlight Command Palette & Keyboard Move Entry")
            .clicked()
        {
            actions.open_command_palette = true;
        }

        // Right-aligned single compact telemetry pill — guaranteed zero overlap at 1180px+
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);

            let (status_text, status_color, dot_color) = if is_thinking {
                (
                    format!("D{thinking_depth}/{thinking_max_depth} • {nps_kns:.0} kN/s"),
                    Color32::from_rgb(255, 210, 95),
                    Color32::from_rgb(255, 185, 40),
                )
            } else {
                (
                    format!("Idle • {fps:.0} FPS • {rss_mb:.0} MB"),
                    Color32::from_rgb(165, 215, 175),
                    Color32::from_rgb(129, 182, 76),
                )
            };

            egui::Frame::none()
                .fill(Color32::from_rgb(0x28, 0x26, 0x23))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x45, 0x42, 0x3D)))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::symmetric(8.0, 3.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(7.0), Sense::hover());
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.5, dot_color);
                        ui.label(
                            RichText::new(status_text)
                                .monospace()
                                .size(11.0)
                                .color(status_color),
                        );
                    });
                });
        });
    });

    actions
}
