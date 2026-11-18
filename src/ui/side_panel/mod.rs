pub mod bot_card;
pub mod bot_selector;
pub mod captured_tray;
pub mod move_table;

use crate::{
    config::{AssistanceMode, NotationStyle},
    domain::ChessMatchState,
    engine::bot::BotPersona,
};
use bot_card::render_bot_card;
pub use bot_selector::{show_bot_selector_modal, MatchSetupSelection};
pub use captured_tray::render_player_strip;
use egui::{Color32, RichText, Ui};
use move_table::render_move_table;

#[derive(Debug, Default)]
pub struct SidePanelActions {
    pub open_bot_selector: bool,
    pub undo_takeback: bool,
    pub request_hint: bool,
    pub resign_game: bool,
    pub flip_board: bool,
    pub open_review: bool,
}

/// Merender Bounded Right Control Panel (`360 px` Fixed Width) sesuai NEW_DESIGN.md Section 5.2:
/// - Pinned Header (~112 px): Bot Speech Bubble + Match Mode Badge
/// - Dynamic Center Region (`ScrollArea::vertical().max_height(remaining_h)`): Move Table
/// - Pinned Footer (~92 px): Top Stepper Row (`|<<`, `<`, `>`, `>>|`, `Flip`) + Bottom Action Row (`Hint`, `Takeback`, `Resign`, `New Match`)
#[allow(clippy::too_many_arguments)]
pub fn render_side_panel(
    ui: &mut Ui,
    match_state: &mut ChessMatchState,
    persona: &BotPersona,
    bot_dialogue: &str,
    is_thinking: bool,
    assistance_mode: AssistanceMode,
    hints_remaining: Option<u8>,
    takebacks_remaining: Option<u8>,
    earned_crowns: u8,
    notation_style: NotationStyle,
    show_move_feedback: bool,
) -> SidePanelActions {
    let mut actions = SidePanelActions::default();

    ui.add_space(4.0);

    // 1. Pinned Header (~112 px): Reactive Bot Speech Bubble + Mode Badge
    render_bot_card(
        ui,
        persona,
        bot_dialogue,
        is_thinking,
        assistance_mode,
        hints_remaining,
        takebacks_remaining,
        earned_crowns,
    );

    ui.add_space(6.0);

    // Reserve exact height for Pinned Footer (96 px) so ScrollArea never overflows vertically!
    let pinned_footer_reserve = if match_state.outcome.is_game_over() {
        132.0
    } else {
        94.0
    };
    let remaining_h = (ui.available_height() - pinned_footer_reserve).max(90.0);

    // 2. Dynamic Center Region: Move Table constrained to remaining_h
    render_move_table(
        ui,
        match_state,
        remaining_h,
        notation_style,
        show_move_feedback,
    );

    ui.add_space(6.0);

    // 3. Pinned Footer Top Row (32 px): History Stepper (`|<<`, `<`, `>`, `>>|`, `Flip`)
    let viewed_ply = match_state.viewed_ply;
    let total_plies = match_state.history.len();

    ui.horizontal(|ui| {
        let btn_w = (ui.available_width() - 32.0) / 5.0;
        let btn_size = [btn_w.max(38.0), 28.0];

        if ui
            .add_enabled(
                viewed_ply > 0,
                egui::Button::new(RichText::new("|<<").monospace().strong()).min_size(btn_size.into()),
            )
            .on_hover_text("First Move (Home)")
            .clicked()
        {
            match_state.step_history_first();
        }

        if ui
            .add_enabled(
                viewed_ply > 0,
                egui::Button::new(RichText::new("<").monospace().strong()).min_size(btn_size.into()),
            )
            .on_hover_text("Previous Move (Left Arrow)")
            .clicked()
        {
            match_state.step_history_prev();
        }

        if ui
            .add_enabled(
                viewed_ply < total_plies,
                egui::Button::new(RichText::new(">").monospace().strong()).min_size(btn_size.into()),
            )
            .on_hover_text("Next Move (Right Arrow)")
            .clicked()
        {
            match_state.step_history_next();
        }

        if ui
            .add_enabled(
                viewed_ply < total_plies,
                egui::Button::new(RichText::new(">>|").monospace().strong()).min_size(btn_size.into()),
            )
            .on_hover_text("Latest Position (End)")
            .clicked()
        {
            match_state.step_history_last();
        }

        if ui
            .add_sized(btn_size, egui::Button::new(RichText::new("⇅ Flip").size(12.0)))
            .on_hover_text("Flip Board Orientation (F)")
            .clicked()
        {
            actions.flip_board = true;
        }
    });

    ui.add_space(4.0);

    // 4. Pinned Footer Bottom Row (40 px): Action Buttons (`Hint`, `Takeback`, `Resign`, `New Match`)
    let can_use_hint = !match_state.outcome.is_game_over()
        && match_state.is_viewing_latest()
        && !is_thinking
        && assistance_mode != AssistanceMode::Challenge
        && hints_remaining.unwrap_or(99) > 0;

    let can_use_takeback = !match_state.history.is_empty()
        && assistance_mode != AssistanceMode::Challenge
        && takebacks_remaining.unwrap_or(99) > 0;

    let can_resign = !match_state.outcome.is_game_over();

    ui.horizontal(|ui| {
        let btn_w = (ui.available_width() - 24.0) / 4.0;
        let btn_size = [btn_w.max(54.0), 34.0];

        if ui
            .add_enabled(
                can_use_hint,
                egui::Button::new(RichText::new("💡 Hint").size(12.0)).min_size(btn_size.into()),
            )
            .on_hover_text("Show recommended best move arrow")
            .clicked()
        {
            actions.request_hint = true;
        }

        if ui
            .add_enabled(
                can_use_takeback,
                egui::Button::new(RichText::new("↩ Undo").size(12.0)).min_size(btn_size.into()),
            )
            .on_hover_text("Take back last move pair")
            .clicked()
        {
            actions.undo_takeback = true;
        }

        if ui
            .add_enabled(
                can_resign,
                egui::Button::new(RichText::new("🏳 Resign").size(12.0)).min_size(btn_size.into()),
            )
            .on_hover_text("Resign current match")
            .clicked()
        {
            actions.resign_game = true;
        }

        if ui
            .add_sized(
                btn_size,
                egui::Button::new(
                    RichText::new("⚔ New")
                        .strong()
                        .size(12.0)
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0x81, 0xB6, 0x4C)),
            )
            .on_hover_text("Start a new match or switch opponent")
            .clicked()
        {
            actions.open_bot_selector = true;
        }
    });

    if match_state.outcome.is_game_over() {
        ui.add_space(4.0);
        let full_w = ui.available_width();
        if ui
            .add_sized(
                [full_w, 30.0],
                egui::Button::new(
                    RichText::new("📊 Game Review")
                        .strong()
                        .size(13.0)
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(0x81, 0xB6, 0x4C)),
            )
            .clicked()
        {
            actions.open_review = true;
        }
    }

    actions
}
