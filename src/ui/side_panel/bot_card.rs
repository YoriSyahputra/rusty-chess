use crate::{engine::bot::BotPersona, ui::piece_renderer::draw_bot_avatar};
use egui::{Color32, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2};

/// Merender Bagian 1 SidePanel: Avatar Bot, Nama, ELO Badge, Stat Bars, dan Speech Bubble Reaktif.
pub fn render_bot_card(
    ui: &mut Ui,
    persona: &BotPersona,
    dialogue_text: &str,
    is_thinking: bool,
) {
    let accent = Color32::from_rgb(
        persona.accent_rgb[0],
        persona.accent_rgb[1],
        persona.accent_rgb[2],
    );

    egui::Frame::none()
        .fill(Color32::from_rgb(29, 33, 42))
        .stroke(Stroke::new(1.2_f32, Color32::from_rgb(52, 59, 75)))
        .rounding(Rounding::same(8.0_f32))
        .inner_margin(egui::Margin::same(10.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                // Avatar vektor berwarna unik dengan glow saat berpikir
                let (avatar_rect, _) =
                    ui.allocate_exact_size(Vec2::splat(60.0), Sense::hover());
                draw_bot_avatar(
                    ui.painter(),
                    avatar_rect,
                    persona.id,
                    persona.accent_rgb,
                    is_thinking,
                );

                ui.add_space(6.0);

                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(persona.name)
                                .strong()
                                .size(15.5)
                                .color(Color32::WHITE),
                        );

                        egui::Frame::none()
                            .fill(Color32::from_rgba_unmultiplied(
                                persona.accent_rgb[0],
                                persona.accent_rgb[1],
                                persona.accent_rgb[2],
                                45,
                            ))
                            .stroke(Stroke::new(1.0_f32, accent))
                            .rounding(Rounding::same(4.0_f32))
                            .inner_margin(egui::Margin::symmetric(6.0, 1.5))
                            .show(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("ELO {}", persona.elo))
                                        .strong()
                                        .size(11.0)
                                        .color(accent),
                                );
                            });
                    });

                    ui.label(
                        RichText::new(persona.title)
                            .italics()
                            .size(12.0)
                            .color(Color32::from_rgb(175, 185, 205)),
                    );

                    ui.add_space(4.0);

                    // Bar karakteristik singkat: Speed / Tactical / Positional
                    render_mini_stat_bar(ui, "SPD", persona.speed_stat, accent);
                    render_mini_stat_bar(ui, "TAC", persona.tactical_stat, accent);
                    render_mini_stat_bar(ui, "POS", persona.positional_stat, accent);
                });
            });

            ui.add_space(8.0);

            // Speech Bubble (Balon Dialog Kontekstual)
            egui::Frame::none()
                .fill(Color32::from_rgb(20, 23, 30))
                .stroke(Stroke::new(1.0_f32, accent.gamma_multiply(0.55)))
                .rounding(Rounding::same(7.0_f32))
                .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(format!("“{dialogue_text}”"))
                            .size(12.5)
                            .color(Color32::from_rgb(230, 235, 245)),
                    );
                });
        });
}

fn render_mini_stat_bar(ui: &mut Ui, label: &str, value: f32, color: Color32) {
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(label)
                .monospace()
                .size(10.0)
                .color(Color32::from_rgb(155, 165, 182)),
        );
        let bar_w = (ui.available_width() - 8.0).clamp(60.0, 180.0);
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, 6.0), Sense::hover());
        let painter = ui.painter();
        painter.rect_filled(
            rect,
            Rounding::same(3.0_f32),
            Color32::from_rgb(42, 47, 58),
        );
        let fill_w = rect.width() * value.clamp(0.05, 1.0);
        let fill_rect = Rect::from_min_max(
            rect.min,
            Pos2::new(rect.min.x + fill_w, rect.max.y),
        );
        painter.rect_filled(fill_rect, Rounding::same(3.0_f32), color);
    });
}
