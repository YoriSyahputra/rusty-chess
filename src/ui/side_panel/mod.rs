pub mod bot_card;
pub mod bot_selector;
pub mod captured_tray;
pub mod move_table;

use crate::{domain::ChessMatchState, engine::bot::BotPersona};
use bot_card::render_bot_card;
pub use bot_selector::{show_bot_selector_modal, MatchSetupSelection};
use captured_tray::render_captured_tray;
use egui::{Color32, RichText, Ui};
use move_table::render_move_table;

#[derive(Debug, Default)]
pub struct SidePanelActions {
    pub open_bot_selector: bool,
    pub undo_takeback: bool,
    pub request_hint: bool,
    pub resign_game: bool,
}

/// Merender SidePanel kanan selebar 360px (Bagian 1 s.d. Bagian 4 sesuai SPEC.md Section 7.1.2).
pub fn render_side_panel(
    ui: &mut Ui,
    match_state: &mut ChessMatchState,
    persona: &BotPersona,
    bot_dialogue: &str,
    is_thinking: bool,
) -> SidePanelActions {
    let mut actions = SidePanelActions::default();

    ui.add_space(4.0);

    // Bagian 1 — Bot Profile & Dialogue Box
    render_bot_card(ui, persona, bot_dialogue, is_thinking);

    ui.add_space(6.0);

    // Bagian 2 — Captured Pieces & Player Info
    render_captured_tray(ui, match_state, persona.name);

    ui.add_space(6.0);

    // Bagian 4 — Action Bar (diukur terlebih dahulu atau dirender di bagian bawah)
    let action_bar_height = 82.0;
    let remaining_for_table = (ui.available_height() - action_bar_height).max(120.0);

    // Bagian 3 — Algebraic Move History Table
    render_move_table(ui, match_state, remaining_for_table);

    ui.add_space(6.0);

    // Bagian 4 — Action Bar (New Match / Ganti Bot, Undo / Takeback, Hint, Resign)
    let full_w = ui.available_width();
    if ui
        .add_sized(
            [full_w, 32.0],
            egui::Button::new(
                RichText::new("⚔ New Match / Ganti Bot")
                    .strong()
                    .size(13.5)
                    .color(Color32::WHITE),
            )
            .fill(Color32::from_rgb(52, 108, 185)),
        )
        .clicked()
    {
        actions.open_bot_selector = true;
    }

    ui.add_space(4.0);

    ui.horizontal(|ui| {
        let btn_w = (ui.available_width() - 16.0) / 3.0;
        let can_undo = !match_state.history.is_empty();
        if ui
            .add_enabled(
                can_undo,
                egui::Button::new(RichText::new("↩ Undo / Takeback").size(12.0))
                    .min_size([btn_w, 28.0].into()),
            )
            .on_hover_text("Batalkan 1 pasang langkah terakhir")
            .clicked()
        {
            actions.undo_takeback = true;
        }

        let can_hint = !match_state.outcome.is_game_over()
            && match_state.is_viewing_latest()
            && !is_thinking;
        if ui
            .add_enabled(
                can_hint,
                egui::Button::new(RichText::new("💡 Hint").size(12.0))
                    .min_size([btn_w, 28.0].into()),
            )
            .on_hover_text("Tampilkan panah hijau saran langkah terbaik")
            .clicked()
        {
            actions.request_hint = true;
        }

        let can_resign = !match_state.outcome.is_game_over();
        if ui
            .add_enabled(
                can_resign,
                egui::Button::new(RichText::new("🏳 Resign").size(12.0))
                    .min_size([btn_w, 28.0].into()),
            )
            .on_hover_text("Menyerah pada partai ini")
            .clicked()
        {
            actions.resign_game = true;
        }
    });

    actions
}
