use crate::{
    config::AssistanceMode,
    engine::bot::BotPersona,
    ui::piece_renderer::{draw_bot_avatar_with_mood, BotMood},
};
use egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};

/// Merender Pinned Header (~112 px) pada SidePanel kanan:
/// Avatar Bot dengan mikro-ekspresi, Nama, ELO Badge, Active Assistance Mode Pill, dan Speech Bubble Reaktif.
#[allow(clippy::too_many_arguments)]
pub fn render_bot_card(
    ui: &mut Ui,
    persona: &BotPersona,
    dialogue_text: &str,
    is_thinking: bool,
    bot_mood: BotMood,
    assistance_mode: AssistanceMode,
    hints_remaining: Option<u8>,
    takebacks_remaining: Option<u8>,
    earned_crowns: u8,
) {
    let accent = Color32::from_rgb(
        persona.accent_rgb[0],
        persona.accent_rgb[1],
        persona.accent_rgb[2],
    );

    egui::Frame::none()
        .fill(Color32::from_rgb(0x30, 0x2E, 0x2B))
        .stroke(Stroke::new(1.0_f32, Color32::from_rgb(0x47, 0x44, 0x3F)))
        .rounding(Rounding::same(8.0_f32))
        .inner_margin(egui::Margin::symmetric(10.0, 8.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                let (avatar_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(46.0), Sense::hover());
                draw_bot_avatar_with_mood(
                    ui.painter(),
                    avatar_rect,
                    persona.id,
                    persona.accent_rgb,
                    is_thinking,
                    bot_mood,
                );

                ui.add_space(6.0);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(persona.name)
                                .strong()
                                .size(14.5)
                                .color(Color32::WHITE),
                        );

                        let elo_str = if persona.code == "mittens" {
                            "ELO 1".to_string()
                        } else {
                            format!("ELO {}", persona.elo)
                        };

                        egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(
                                persona.accent_rgb[0],
                                persona.accent_rgb[1],
                                persona.accent_rgb[2],
                                40,
                            ))
                            .stroke(Stroke::new(1.0_f32, accent))
                            .rounding(Rounding::same(4.0_f32))
                            .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(elo_str)
                                        .strong()
                                        .size(10.5)
                                        .color(accent),
                                );
                            });

                        // Crown indicator (0..3)
                        let mut crowns_str = String::new();
                        for i in 0..3 {
                            if i < earned_crowns {
                                crowns_str.push('👑');
                            } else {
                                crowns_str.push('☆');
                            }
                        }
                        ui.label(
                            RichText::new(crowns_str)
                                .size(11.0)
                                .color(Color32::from_rgb(0xFF, 0xAA, 0x00)),
                        );
                    });

                    let mode_badge = match (assistance_mode, hints_remaining, takebacks_remaining) {
                        (AssistanceMode::Challenge, _, _) => {
                            "Challenge • 3 Crowns Max".to_string()
                        }
                        (AssistanceMode::Friendly, Some(h), Some(t)) => {
                            format!("Friendly (2★) • Hints: {h}/3, Undo: {t}/3")
                        }
                        (AssistanceMode::Assisted, _, _) => {
                            "Assisted • 1 Crown Max".to_string()
                        }
                        (AssistanceMode::Custom, _, _) => "Custom Mode".to_string(),
                        _ => assistance_mode.label().to_string(),
                    };

                    ui.label(
                        RichText::new(mode_badge)
                            .size(11.0)
                            .color(Color32::from_rgb(0x81, 0xB6, 0x4C)),
                    );
                });
            });

            ui.add_space(6.0);

            // Speech Bubble dengan border aksen
            egui::Frame::none()
                .fill(Color32::from_rgb(0x26, 0x24, 0x21))
                .stroke(Stroke::new(1.0_f32, accent.gamma_multiply(0.55)))
                .rounding(Rounding::same(6.0_f32))
                .inner_margin(egui::Margin::symmetric(9.0, 6.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(format!("💬 “{dialogue_text}”"))
                            .size(12.0)
                            .color(Color32::from_rgb(0xF5, 0xF5, 0xF5)),
                    );
                });
        });
}
