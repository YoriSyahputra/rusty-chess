use crate::{
    domain::GameOutcome,
    engine::bot::BotPersona,
    ui::{piece_renderer::draw_bot_avatar, theme::UiTokens},
};
use egui::{Align2, Color32, Context, RichText, Rounding, Sense, Stroke, Vec2, Window};
use shakmaty::Color;

#[derive(Debug, Default)]
pub struct GameOverModalActions {
    pub open_game_review: bool,
    pub play_rematch: bool,
    pub select_new_bot: bool,
}

/// Menampilkan Game Over Modal sesuai NEW_DESIGN.md Section 9 (`src/ui/modals/game_over.rs`)
/// lengkap dengan perolehan mahkota (0..3 Crowns), pemicu Game Review, Rematch, dan New Bot.
#[allow(clippy::too_many_arguments)]
pub fn show_game_over_modal(
    ctx: &Context,
    open: &mut bool,
    outcome: GameOutcome,
    human_color: Color,
    persona: &BotPersona,
    awarded_crowns: u8,
    dark_mode: bool,
) -> GameOverModalActions {
    let mut actions = GameOverModalActions::default();
    if !*open || !outcome.is_game_over() {
        return actions;
    }

    let tokens = UiTokens::for_mode(dark_mode);
    let human_won = outcome.winner() == Some(human_color);
    let is_draw = outcome.winner().is_none();

    let (headline, headline_color) = if human_won {
        ("🏆 Victory!", tokens.accent_primary)
    } else if is_draw {
        ("🤝 Match Drawn", tokens.accent_gold)
    } else {
        ("Defeat", Color32::from_rgb(0xE0, 0x4F, 0x44))
    };

    let mut should_close = false;

    Window::new("Match Complete")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(370.0, 290.0))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(4.0);
                ui.label(
                    RichText::new(headline)
                        .strong()
                        .size(24.0)
                        .color(headline_color),
                );
                ui.label(
                    RichText::new(outcome.description())
                        .size(13.0)
                        .color(tokens.text_secondary),
                );

                ui.add_space(8.0);

                let (av_rect, _) = ui.allocate_exact_size(Vec2::splat(56.0), Sense::hover());
                draw_bot_avatar(
                    ui.painter(),
                    av_rect,
                    persona.id,
                    persona.accent_rgb,
                    human_won,
                );
                ui.label(
                    RichText::new(format!("vs {} ({})", persona.name, persona.elo))
                        .strong()
                        .size(14.0)
                        .color(tokens.text_primary),
                );

                ui.add_space(6.0);

                // Crown display (0..3)
                let crown_display: String = (0..3)
                    .map(|i| if i < awarded_crowns { "👑 " } else { "☆ " })
                    .collect();
                ui.label(
                    RichText::new(crown_display)
                        .size(20.0)
                        .color(tokens.accent_gold),
                );

                ui.add_space(10.0);

                // Primary CTA: Game Review (#81B64C)
                if ui
                    .add_sized(
                        [310.0, 40.0],
                        egui::Button::new(
                            RichText::new("📊  Game Review")
                                .strong()
                                .size(15.5)
                                .color(Color32::WHITE),
                        )
                        .fill(tokens.accent_primary)
                        .rounding(Rounding::same(7.0_f32)),
                    )
                    .clicked()
                {
                    actions.open_game_review = true;
                    should_close = true;
                }

                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    let half_w = (310.0 - 8.0) * 0.5;
                    if ui
                        .add_sized(
                            [half_w, 34.0],
                            egui::Button::new(
                                RichText::new("🔄 Rematch").strong().size(13.0),
                            )
                            .stroke(Stroke::new(1.0_f32, tokens.border_subtle)),
                        )
                        .clicked()
                    {
                        actions.play_rematch = true;
                        should_close = true;
                    }

                    if ui
                        .add_sized(
                            [half_w, 34.0],
                            egui::Button::new(
                                RichText::new("🤖 New Bot").strong().size(13.0),
                            )
                            .stroke(Stroke::new(1.0_f32, tokens.border_subtle)),
                        )
                        .clicked()
                    {
                        actions.select_new_bot = true;
                        should_close = true;
                    }
                });
            });
        });

    if should_close {
        *open = false;
    }

    actions
}
