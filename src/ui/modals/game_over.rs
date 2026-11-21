use crate::{
    domain::GameOutcome,
    engine::bot::BotPersona,
    ui::{
        piece_renderer::draw_bot_avatar,
        theme::{chess_cta_button, tactile_surface_button, UiTokens},
    },
};
use egui::{Align2, Color32, Context, RichText, Sense, Vec2, Window};
use shakmaty::Color;

#[derive(Debug, Default)]
pub struct GameOverModalActions {
    pub open_game_review: bool,
    pub play_rematch: bool,
    pub select_new_bot: bool,
}

/// Menampilkan Game Over Modal bergaya Chess.com dengan tombol rata penuh & 3D bevel CTA.
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
        ("½ Match Drawn", tokens.accent_gold)
    } else {
        ("Defeat", Color32::from_rgb(0xE0, 0x4F, 0x44))
    };

    let mut should_close = false;

    Window::new("Match Complete")
        .open(open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(360.0, 295.0))
        .show(ctx, |ui| {
            let content_w = ui.available_width();
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

                ui.add_space(10.0);

                let (av_rect, _) = ui.allocate_exact_size(Vec2::splat(58.0), Sense::hover());
                draw_bot_avatar(
                    ui.painter(),
                    av_rect,
                    persona.id,
                    persona.accent_rgb,
                    human_won,
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!("vs {} ({})", persona.name, persona.elo))
                        .strong()
                        .size(14.5)
                        .color(tokens.text_primary),
                );

                ui.add_space(4.0);

                let crown_display: String = (0..3)
                    .map(|i| if i < awarded_crowns { "👑 " } else { "☆ " })
                    .collect();
                ui.label(
                    RichText::new(crown_display.trim_end())
                        .size(20.0)
                        .color(tokens.accent_gold),
                );
            });

            ui.add_space(12.0);

            // Primary CTA: Full-width Chess.com green 3D button
            if chess_cta_button(
                ui,
                Vec2::new(content_w, 42.0),
                "📊  Game Review",
                tokens.accent_primary,
            ) {
                actions.open_game_review = true;
                should_close = true;
            }

            ui.add_space(8.0);

            // Secondary Row: Strictly aligned 50/50 smooth tactile columns matching exact CTA width
            ui.columns(2, |cols| {
                let col_w = cols[0].available_width();
                if tactile_surface_button(
                    &mut cols[0],
                    Vec2::new(col_w, 36.0),
                    "🔄 Rematch",
                    true,
                    &tokens,
                )
                .clicked()
                {
                    actions.play_rematch = true;
                    should_close = true;
                }

                let col_w2 = cols[1].available_width();
                if tactile_surface_button(
                    &mut cols[1],
                    Vec2::new(col_w2, 36.0),
                    "♞ New Bot",
                    true,
                    &tokens,
                )
                .clicked()
                {
                    actions.select_new_bot = true;
                    should_close = true;
                }
            });
        });

    if should_close {
        *open = false;
    }

    actions
}
