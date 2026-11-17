use crate::{
    domain::TimeControl,
    engine::bot::{BotId, BotPersona},
    ui::piece_renderer::draw_bot_avatar,
};
use egui::{Color32, Context, RichText, Rounding, Sense, Stroke, Vec2, Window};
use shakmaty::Color;

#[derive(Debug, Clone, Copy)]
pub struct MatchSetupSelection {
    pub bot_id: BotId,
    pub human_color: Color,
    pub time_control: TimeControl,
}

/// Menampilkan Modal Pemilih Lawan Bot, Warna Bidak (Putih/Hitam/Acak), dan Time Control.
pub fn show_bot_selector_modal(
    ctx: &Context,
    open: &mut bool,
    selected_bot: &mut BotId,
    selected_color: &mut Color,
    selected_tc: &mut TimeControl,
) -> Option<MatchSetupSelection> {
    if !*open {
        return None;
    }

    let mut confirmed: Option<MatchSetupSelection> = None;
    let mut should_close = false;

    Window::new("⚔ Select Opponent & Match Setup")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(560.0, 470.0))
        .show(ctx, |ui| {
            ui.label(
                RichText::new("Choose an AI Grandmaster or Club Persona:")
                    .strong()
                    .size(13.5),
            );
            ui.add_space(6.0);

            egui::ScrollArea::vertical()
                .max_height(285.0)
                .show(ui, |ui| {
                    for persona in BotPersona::all_roster() {
                        let is_selected = *selected_bot == persona.id;
                        let accent = Color32::from_rgb(
                            persona.accent_rgb[0],
                            persona.accent_rgb[1],
                            persona.accent_rgb[2],
                        );
                        let bg = if is_selected {
                            Color32::from_rgb(44, 54, 74)
                        } else {
                            Color32::from_rgb(28, 32, 40)
                        };
                        let border = if is_selected {
                            Stroke::new(2.0_f32, accent)
                        } else {
                            Stroke::new(1.0_f32, Color32::from_rgb(52, 58, 72))
                        };

                        let resp = egui::Frame::none()
                            .fill(bg)
                            .stroke(border)
                            .rounding(Rounding::same(8.0_f32))
                            .inner_margin(egui::Margin::same(8.0))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    let (av_rect, _) =
                                        ui.allocate_exact_size(Vec2::splat(44.0), Sense::hover());
                                    draw_bot_avatar(
                                        ui.painter(),
                                        av_rect,
                                        persona.id,
                                        persona.accent_rgb,
                                        false,
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
                                            ui.label(
                                                RichText::new(format!("({} ELO)", persona.elo))
                                                    .strong()
                                                    .size(12.5)
                                                    .color(accent),
                                            );
                                            ui.label(
                                                RichText::new(format!("• {}", persona.title))
                                                    .italics()
                                                    .size(12.0)
                                                    .color(Color32::LIGHT_GRAY),
                                            );
                                        });
                                        ui.label(
                                            RichText::new(persona.bio)
                                                .size(11.5)
                                                .color(Color32::from_rgb(185, 192, 208)),
                                        );
                                    });
                                });
                            })
                            .response
                            .interact(Sense::click());

                        if resp.clicked() {
                            *selected_bot = persona.id;
                        }
                        ui.add_space(4.0);
                    }
                });

            ui.separator();

            // Pilihan warna bidak & Time Control
            ui.horizontal(|ui| {
                ui.label(RichText::new("Play as:").strong().size(13.0));
                ui.selectable_value(selected_color, Color::White, "⬜ White");
                ui.selectable_value(selected_color, Color::Black, "⬛ Black");

                ui.add_space(16.0);
                ui.label(RichText::new("Time Control:").strong().size(13.0));
                egui::ComboBox::from_id_salt("modal_tc_selector")
                    .selected_text(selected_tc.label())
                    .show_ui(ui, |ui| {
                        for &tc in TimeControl::all_presets() {
                            ui.selectable_value(selected_tc, tc, tc.label());
                        }
                    });
            });

            ui.add_space(10.0);

            ui.horizontal(|ui| {
                if ui
                    .add_sized(
                        [180.0, 34.0],
                        egui::Button::new(
                            RichText::new("▶ Start Match")
                                .strong()
                                .size(14.0)
                                .color(Color32::WHITE),
                        )
                        .fill(Color32::from_rgb(46, 134, 86)),
                    )
                    .clicked()
                {
                    confirmed = Some(MatchSetupSelection {
                        bot_id: *selected_bot,
                        human_color: *selected_color,
                        time_control: *selected_tc,
                    });
                    should_close = true;
                }

                if ui
                    .add_sized([100.0, 34.0], egui::Button::new("Cancel"))
                    .clicked()
                {
                    should_close = true;
                }
            });
        });

    if should_close {
        *open = false;
    }

    confirmed
}
