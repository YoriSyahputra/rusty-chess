use crate::{
    domain::ChessMatchState,
    engine::bot::BotPersona,
    ui::{
        board_view::render_board_grid,
        side_panel::render_player_strip,
        sidebar::ActiveRoute,
        theme::{chess_cta_button, soft_card_frame, BoardTheme, UiTokens},
    },
};
use egui::{Color32, RichText, Sense, Ui, Vec2};

#[derive(Debug, Default)]
pub struct PlayHubActions {
    pub navigate_to: Option<ActiveRoute>,
    pub start_coach_mode: bool,
    pub start_chess960: bool,
    pub start_pass_and_play: bool,
    pub open_custom_fen_modal: bool,
}

/// Merender Play Hub (`Route::PlayHub`) dengan papan pratinjau di kiri dan kartu pemilih mode di kanan.
pub fn render_play_hub(
    ui: &mut Ui,
    match_state: &ChessMatchState,
    persona: &BotPersona,
    board_theme: BoardTheme,
    dark_mode: bool,
) -> PlayHubActions {
    let mut actions = PlayHubActions::default();
    let tokens = UiTokens::for_mode(dark_mode);

    egui::SidePanel::right("play_hub_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new("♟ Choose Game Mode")
                    .strong()
                    .size(20.0)
                    .color(tokens.text_primary),
            );
            ui.label(
                RichText::new("Select how you want to play on your desktop today:")
                    .size(12.5)
                    .color(tokens.text_secondary),
            );
            ui.add_space(10.0);

            egui::ScrollArea::vertical()
                .id_salt("play_hub_modes_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if render_mode_selector_card(
                        ui,
                        "🤖  Play Computer Bots",
                        "Challenge 18 distinct AI personalities from Beginner to Grandmaster, earn up to 54 Crowns, or dial in a custom ELO engine.",
                        "Select Bot Opponent",
                        tokens.accent_primary,
                        &tokens,
                    ) {
                        actions.navigate_to = Some(ActiveRoute::PlayBots);
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "🎓  Play with Coach (Assisted)",
                        "Practice in Friendly/Assisted mode with real-time move evaluations, threat arrows, best-move hints, and takebacks.",
                        "Start Guided Match",
                        tokens.accent_info,
                        &tokens,
                    ) {
                        actions.start_coach_mode = true;
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "⚔  Bot vs. Bot Exhibition Arena",
                        "Pit any two AI personalities (like Mittens vs GM Magnus!) against each other with live eval bar and trash-talk.",
                        "Open Bot Arena",
                        Color32::from_rgb(225, 95, 70),
                        &tokens,
                    ) {
                        actions.navigate_to = Some(ActiveRoute::Arena);
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "♔  Endgame Mastery Drills",
                        "Train Lucena, Philidor, Opposition, and Bishop+Knight checkmate directly against GM Magnus.",
                        "Open Endgame Drills",
                        Color32::from_rgb(65, 165, 145),
                        &tokens,
                    ) {
                        actions.navigate_to = Some(ActiveRoute::Endgames);
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "🎲  Play Chess960 (Fischer Random)",
                        "Battle any bot from one of 960 randomized Scharnagl back-rank setups with full Chess960 castling rules.",
                        "Start Random Chess960",
                        Color32::from_rgb(195, 115, 225),
                        &tokens,
                    ) {
                        actions.start_chess960 = true;
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "🎯  Coordinate Speed Vision Trainer",
                        "Train 30-second blind square recognition (a1–h8) to level up your board visualization speed.",
                        "Launch Vision Trainer",
                        Color32::from_rgb(235, 155, 55),
                        &tokens,
                    ) {
                        actions.navigate_to = Some(ActiveRoute::Vision);
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "👥  Pass & Play (Local 2-Player)",
                        "Play over-the-board with a friend on the same screen with full chess clock support and post-game review.",
                        "Start 2-Player Board",
                        tokens.accent_gold,
                        &tokens,
                    ) {
                        actions.start_pass_and_play = true;
                    }

                    ui.add_space(8.0);

                    if render_mode_selector_card(
                        ui,
                        "📋  Custom Position (FEN)",
                        "Paste any FEN string to practice specific endgames, opening tabiyas, or tactical studies against any bot.",
                        "Load FEN Position",
                        Color32::from_rgb(155, 110, 235),
                        &tokens,
                    ) {
                        actions.open_custom_fen_modal = true;
                    }
                    ui.add_space(8.0);
                });
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let avail = ui.available_size();
        let board_size = (avail.x - 24.0).min(avail.y - 104.0).max(320.0);
        let pad_x = ((avail.x - board_size) * 0.5).max(8.0);
        let pad_y = ((avail.y - (board_size + 96.0)) * 0.5).max(4.0);

        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            render_player_strip(
                ui,
                board_size,
                match_state,
                persona,
                true,
                false,
                dark_mode,
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            let (b_rect, _) =
                ui.allocate_exact_size(Vec2::splat(board_size), Sense::hover());
            render_board_grid(
                ui.painter(),
                b_rect,
                &match_state.displayed_position(),
                &board_theme.palette(),
                false,
                true,
                None,
                match_state.displayed_last_move(),
                &[],
                &[],
                &[],
                None,
                None,
                None,
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            render_player_strip(
                ui,
                board_size,
                match_state,
                persona,
                false,
                false,
                dark_mode,
            );
        });
    });

    actions
}

fn render_mode_selector_card(
    ui: &mut Ui,
    title: &str,
    subtitle: &str,
    btn_text: &str,
    accent: Color32,
    tokens: &UiTokens,
) -> bool {
    let mut clicked = false;
    soft_card_frame(tokens, egui::Margin::same(12.0))
        .show(ui, |ui| {
            let w = ui.available_width();
            ui.set_min_width(w);
            ui.label(
                RichText::new(title)
                    .strong()
                    .size(15.0)
                    .color(tokens.text_primary),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new(subtitle)
                    .size(11.5)
                    .color(tokens.text_secondary),
            );
            ui.add_space(10.0);
            if chess_cta_button(ui, Vec2::new(w, 34.0), btn_text, accent) {
                clicked = true;
            }
        });
    clicked
}
