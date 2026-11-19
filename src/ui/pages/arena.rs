use crate::{
    config::UserConfig,
    domain::{ChessMatchState, Color, PlayerController},
    engine::{
        bot::{
            lookup_persona_book_move, select_bot_dialogue, BotId, BotPersona, DialogueTrigger,
        },
        eval::evaluate_for_white,
        search::{search_best_move, TranspositionTable},
    },
    ui::{
        board_view::render_board_grid_ext,
        eval_bar::EvalBarState,
        piece_renderer::draw_bot_avatar,
        theme::UiTokens,
    },
};
use egui::{Color32, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};
use shakmaty::Position;
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

pub struct BotArenaState {
    pub white_bot: BotId,
    pub black_bot: BotId,
    pub match_state: ChessMatchState,
    pub eval_bar: EvalBarState,
    pub white_dialogue: String,
    pub black_dialogue: String,
    pub is_running: bool,
    pub speed_mult: f32,
    pub last_step_at: Instant,
    tt: TranspositionTable,
}

impl Default for BotArenaState {
    fn default() -> Self {
        let mut s = Self {
            white_bot: BotId::Magnus,
            black_bot: BotId::Hikaru,
            match_state: ChessMatchState::new_standard(Color::White),
            eval_bar: EvalBarState::default(),
            white_dialogue: select_bot_dialogue(BotId::Magnus, DialogueTrigger::MatchStart),
            black_dialogue: select_bot_dialogue(BotId::Hikaru, DialogueTrigger::MatchStart),
            is_running: false,
            speed_mult: 1.0,
            last_step_at: Instant::now(),
            tt: TranspositionTable::new(),
        };
        s.match_state.white_controller = PlayerController::Bot;
        s.match_state.black_controller = PlayerController::Bot;
        s
    }
}

impl BotArenaState {
    pub fn reset_match(&mut self) {
        let mut st = ChessMatchState::new_standard(Color::White);
        st.white_controller = PlayerController::Bot;
        st.black_controller = PlayerController::Bot;
        self.match_state = st;
        self.eval_bar = EvalBarState::default();
        self.white_dialogue = select_bot_dialogue(self.white_bot, DialogueTrigger::MatchStart);
        self.black_dialogue = select_bot_dialogue(self.black_bot, DialogueTrigger::MatchStart);
        self.last_step_at = Instant::now();
        self.tt.clear();
    }

    pub fn step_one_bot_move(&mut self) {
        if self.match_state.outcome.is_game_over() {
            self.is_running = false;
            return;
        }
        let turn = self.match_state.current_pos.turn();
        let active_id = match turn {
            Color::White => self.white_bot,
            Color::Black => self.black_bot,
        };
        let mut persona = BotPersona::by_id(active_id);
        // Fast snappy exhibition depth so UI never stalls
        persona.max_depth = persona.max_depth.min(3);
        persona.time_budget_ms = 90;

        let ply = self.match_state.history.len();
        let chosen_move = lookup_persona_book_move(&self.match_state.current_pos, &persona, ply)
            .or_else(|| {
                let abort = AtomicBool::new(false);
                search_best_move(
                    &self.match_state.current_pos,
                    &persona,
                    &self.match_state.position_hashes,
                    &mut self.tt,
                    &abort,
                    |_, _, _, _| {},
                )
                .map(|res| res.best_move)
            });

        if let Some(mv) = chosen_move {
            let is_cap = mv.is_capture();
            let mut next_pos = self.match_state.current_pos.clone();
            next_pos.play_unchecked(&mv);
            let eval_w = evaluate_for_white(&next_pos, &persona.weights);
            if self.match_state.apply_move(mv, eval_w, 120).is_ok() {
                self.eval_bar.set_eval_cp(eval_w);
                let trigger = if self.match_state.outcome.is_game_over() {
                    DialogueTrigger::GameOverWin
                } else if self.match_state.current_pos.is_check() {
                    DialogueTrigger::BotGivesCheck
                } else if is_cap {
                    DialogueTrigger::BotCapturesPiece
                } else {
                    DialogueTrigger::OpeningPhase
                };
                let line = select_bot_dialogue(active_id, trigger);
                match turn {
                    Color::White => self.white_dialogue = line,
                    Color::Black => self.black_dialogue = line,
                }
            }
        }
        self.last_step_at = Instant::now();
    }
}

/// Merender Bot vs. Bot Exhibition Arena (Feature #22).
pub fn render_bot_arena_page(ui: &mut Ui, arena: &mut BotArenaState, config: &UserConfig) {
    let tokens = UiTokens::for_mode(config.dark_mode);

    // Auto-step timer when running
    if arena.is_running && !arena.match_state.outcome.is_game_over() {
        let interval_ms = (850.0 / arena.speed_mult.clamp(0.4, 4.0)) as u64;
        if arena.last_step_at.elapsed() >= Duration::from_millis(interval_ms) {
            arena.step_one_bot_move();
        }
        ui.ctx()
            .request_repaint_after(Duration::from_millis(80));
    }

    let white_p = BotPersona::by_id(arena.white_bot);
    let black_p = BotPersona::by_id(arena.black_bot);

    egui::SidePanel::right("bot_arena_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new("⚔ Bot vs. Bot Exhibition Arena")
                    .strong()
                    .size(17.5)
                    .color(tokens.text_primary),
            );
            ui.add_space(6.0);

            // Fighter Pickers
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⬜ White:").strong().size(12.5));
                        egui::ComboBox::from_id_salt("arena_white_bot")
                            .selected_text(format!("{} ({})", white_p.name, white_p.elo))
                            .show_ui(ui, |ui| {
                                for &id in BotId::all_lobby_personalities() {
                                    let p = BotPersona::by_id(id);
                                    if ui
                                        .selectable_value(
                                            &mut arena.white_bot,
                                            id,
                                            format!("{} ({})", p.name, p.elo),
                                        )
                                        .clicked()
                                    {
                                        arena.reset_match();
                                    }
                                }
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⬛ Black:").strong().size(12.5));
                        egui::ComboBox::from_id_salt("arena_black_bot")
                            .selected_text(format!("{} ({})", black_p.name, black_p.elo))
                            .show_ui(ui, |ui| {
                                for &id in BotId::all_lobby_personalities() {
                                    let p = BotPersona::by_id(id);
                                    if ui
                                        .selectable_value(
                                            &mut arena.black_bot,
                                            id,
                                            format!("{} ({})", p.name, p.elo),
                                        )
                                        .clicked()
                                    {
                                        arena.reset_match();
                                    }
                                }
                            });
                    });

                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Speed:");
                        ui.selectable_value(&mut arena.speed_mult, 0.5, "0.5×");
                        ui.selectable_value(&mut arena.speed_mult, 1.0, "1×");
                        ui.selectable_value(&mut arena.speed_mult, 3.0, "3×");
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        let play_lbl = if arena.is_running {
                            "⏸ Pause"
                        } else {
                            "▶ Start Exhibition"
                        };
                        if ui
                            .add_sized(
                                [155.0, 34.0],
                                egui::Button::new(
                                    RichText::new(play_lbl).strong().color(Color32::WHITE),
                                )
                                .fill(tokens.accent_primary),
                            )
                            .clicked()
                        {
                            arena.is_running = !arena.is_running;
                            arena.last_step_at = Instant::now();
                        }

                        if ui
                            .add_sized([82.0, 34.0], egui::Button::new("⏭ Step"))
                            .clicked()
                        {
                            arena.step_one_bot_move();
                        }

                        if ui
                            .add_sized([82.0, 34.0], egui::Button::new("🔄 Reset"))
                            .clicked()
                        {
                            arena.is_running = false;
                            arena.reset_match();
                        }
                    });
                });

            ui.add_space(8.0);

            // Dual Trash-Talk Speech Bubbles
            render_arena_fighter_bubble(ui, &black_p, "Black", &arena.black_dialogue, &tokens);
            ui.add_space(6.0);
            render_arena_fighter_bubble(ui, &white_p, "White", &arena.white_dialogue, &tokens);

            ui.add_space(8.0);

            if arena.match_state.outcome.is_game_over() {
                ui.label(
                    RichText::new(format!("🏆 {}", arena.match_state.outcome.description()))
                        .strong()
                        .size(14.0)
                        .color(tokens.accent_gold),
                );
            }

            // Live PGN Move Strip
            let rem_h = (ui.available_height() - 12.0).max(80.0);
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .id_salt("arena_moves_scroll")
                        .max_height(rem_h)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true)
                        .show(ui, |ui| {
                            let mut text = String::new();
                            for (i, rec) in arena.match_state.history.iter().enumerate() {
                                if i % 2 == 0 {
                                    text.push_str(&format!("{}. {} ", (i / 2) + 1, rec.san));
                                } else {
                                    text.push_str(&format!("{}   ", rec.san));
                                }
                            }
                            if text.is_empty() {
                                text = "Click Start Exhibition to watch the bots battle!".to_string();
                            }
                            ui.label(
                                RichText::new(text)
                                    .monospace()
                                    .size(12.0)
                                    .color(tokens.text_primary),
                            );
                        });
                });
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let avail = ui.available_size();
        let eval_w = 36.0;
        let board_size = (avail.x - eval_w - 24.0).min(avail.y - 48.0).max(320.0);
        let pad_x = ((avail.x - (board_size + eval_w)) * 0.5).max(8.0);
        let pad_y = ((avail.y - board_size) * 0.5).max(8.0);

        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            arena.eval_bar.ui(ui, board_size, false);
            ui.add_space(8.0);
            let (b_rect, _) =
                ui.allocate_exact_size(Vec2::splat(board_size), Sense::hover());
            render_board_grid_ext(
                ui.painter(),
                b_rect,
                &arena.match_state.current_pos,
                &config.board_theme.palette(),
                false,
                config.show_coordinates,
                None,
                arena.match_state.last_move,
                None,
                &[],
                &[],
                &[],
                None,
                None,
                None,
                &[],
                config.piece_set,
                None,
            );
        });
    });
}

fn render_arena_fighter_bubble(
    ui: &mut Ui,
    persona: &BotPersona,
    side_label: &str,
    quote: &str,
    tokens: &UiTokens,
) {
    let accent = Color32::from_rgb(
        persona.accent_rgb[0],
        persona.accent_rgb[1],
        persona.accent_rgb[2],
    );
    egui::Frame::none()
        .fill(tokens.bg_surface)
        .stroke(Stroke::new(1.0_f32, accent.gamma_multiply(0.6)))
        .rounding(Rounding::same(8.0_f32))
        .inner_margin(egui::Margin::same(8.0))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.horizontal(|ui| {
                let (av_rect, _) = ui.allocate_exact_size(Vec2::splat(34.0), Sense::hover());
                draw_bot_avatar(ui.painter(), av_rect, persona.id, persona.accent_rgb, false);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(format!("{} ({}) • {side_label}", persona.name, persona.elo))
                            .strong()
                            .size(12.0)
                            .color(accent),
                    );
                    ui.label(
                        RichText::new(format!("“{quote}”"))
                            .size(11.5)
                            .color(tokens.text_primary),
                    );
                });
            });
        });
}
