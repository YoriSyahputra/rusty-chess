use crate::{
    config::UserConfig,
    ui::theme::{tactile_pill_button, UiTokens},
};
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
    let tokens = UiTokens::for_mode(config.dark_mode);

    ui.horizontal_centered(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.add_space(4.0);

        // 1. Board & Pieces Visual Preview Launcher
        if tactile_pill_button(
            ui,
            "tb_appearance",
            "◈ Board & Pieces",
            false,
            tokens.accent_primary,
            config.dark_mode,
        )
        .on_hover_text("Preview & choose Board Themes and Chess.com Piece Sets")
        .clicked()
        {
            actions.open_appearance_modal = true;
        }

        // 2. Live Instant Analysis Toggle
        let live_on = config.live_analysis_enabled;
        if tactile_pill_button(
            ui,
            "tb_live_eval",
            if live_on { "⚡ Live Eval" } else { "⚡ Live Eval: Off" },
            live_on,
            tokens.accent_primary,
            config.dark_mode,
        )
        .on_hover_text("Toggle instant live move classification (!! / ★ / ?! / ? / ??) as you play")
        .clicked()
        {
            config.live_analysis_enabled = !config.live_analysis_enabled;
        }

        // 3. X-Ray Heatmap Toggle
        let xray_on = config.show_xray_heatmap;
        if tactile_pill_button(
            ui,
            "tb_xray",
            "👁 X-Ray",
            xray_on,
            tokens.accent_info,
            config.dark_mode,
        )
        .on_hover_text("Toggle X-Ray Square Control Heatmap (Shortcut: X)")
        .clicked()
        {
            config.show_xray_heatmap = !config.show_xray_heatmap;
        }

        // 4. Sound Toggle
        if tactile_pill_button(
            ui,
            "tb_sound",
            if config.sound_enabled { "🔊 Sound" } else { "🔇 Muted" },
            config.sound_enabled,
            tokens.accent_primary,
            config.dark_mode,
        )
        .on_hover_text("Toggle procedural chess sound effects")
        .clicked()
        {
            config.sound_enabled = !config.sound_enabled;
        }

        // 5. Zen Mode
        if tactile_pill_button(
            ui,
            "tb_zen",
            "☾ Zen",
            false,
            tokens.accent_gold,
            config.dark_mode,
        )
        .on_hover_text("Distraction-free focus mode (Shortcut: Z)")
        .clicked()
        {
            actions.toggle_zen_mode = true;
        }

        // 6. FEN / PGN
        if tactile_pill_button(
            ui,
            "tb_fen",
            "☰ FEN/PGN",
            false,
            tokens.accent_info,
            config.dark_mode,
        )
        .on_hover_text("Import / Export FEN & PGN")
        .clicked()
        {
            actions.open_fen_pgn_modal = true;
        }

        // 7. Command Palette
        if tactile_pill_button(
            ui,
            "tb_cmd",
            "⌘ Ctrl+K",
            false,
            tokens.accent_gold,
            config.dark_mode,
        )
        .on_hover_text("Open Spotlight Command Palette & Keyboard Move Entry")
        .clicked()
        {
            actions.open_command_palette = true;
        }

        // Right-aligned single sleek telemetry pill
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(6.0);

            let (status_text, status_color, dot_color) = if is_thinking {
                (
                    format!("D{thinking_depth}/{thinking_max_depth} • {nps_kns:.0} kN/s"),
                    Color32::from_rgb(255, 215, 105),
                    Color32::from_rgb(255, 185, 45),
                )
            } else {
                (
                    format!("Idle • {fps:.0} FPS • {rss_mb:.0} MB"),
                    tokens.text_secondary,
                    tokens.accent_primary,
                )
            };

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(13.0_f32))
                .inner_margin(egui::Margin::symmetric(10.0, 4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let (dot_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(8.0), Sense::hover());
                        ui.painter().circle_filled(
                            dot_rect.center(),
                            5.0,
                            dot_color.gamma_multiply(0.28),
                        );
                        ui.painter()
                            .circle_filled(dot_rect.center(), 3.2, dot_color);
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
