use crate::{
    audio::{AudioEngine, SoundEffect},
    config::UserConfig,
    domain::{
        format_fen, ChessMatchState, Color, GameOutcome, Move, Role, TacticalArrow, TimeControl,
    },
    engine::{
        bot::{
            detect_dialogue_trigger, select_bot_dialogue, BotId, BotPersona, DialogueTrigger,
        },
        eval::{evaluate_for_white, material::compute_game_phase},
        worker::{EngineEvent, EngineWorker},
    },
    ui::{
        board_view::BoardInteractionState,
        eval_bar::EvalBarState,
        side_panel::{ render_side_panel, show_bot_selector_modal },
        theme::apply_global_style,
        top_bar::render_top_bar,
    },
};
use egui::{Color32, Context, Key, RichText, Rounding, Stroke, Vec2, Window};
use shakmaty::Position;
use std::time::{Duration, Instant};

pub struct RustyChessApp {
    pub config: UserConfig,
    pub match_state: ChessMatchState,
    pub bot_persona: BotPersona,
    pub bot_dialogue: String,
    pub engine_worker: EngineWorker,
    pub board_ui: BoardInteractionState,
    pub eval_bar: EvalBarState,
    pub audio: AudioEngine,

    // Engine tracking state
    pub current_request_id: u64,
    pub is_engine_thinking: bool,
    pub is_hint_search: bool,
    pub thinking_depth: u8,
    pub thinking_max_depth: u8,
    pub thinking_nodes: u64,
    pub last_nps_kns: f32,
    pub search_started_at: Option<Instant>,
    pub human_turn_started_at: Instant,
    pub last_eval_cp_white: i32,
    pub entered_endgame_announced: bool,

    // Modals state
    pub show_bot_selector: bool,
    pub show_fen_pgn_modal: bool,
    pub custom_fen_input: String,
    pub fen_error_msg: Option<String>,
    pub modal_bot_id: BotId,
    pub modal_human_color: Color,
    pub modal_time_control: TimeControl,

    // Diagnostics (FPS & RAM RSS)
    pub fps_smoothed: f32,
    pub rss_mb: f32,
    pub last_rss_poll: Instant,
}

impl RustyChessApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        apply_global_style(&cc.egui_ctx);

        let config = UserConfig::default();
        let bot_persona = BotPersona::magnus();
        let bot_dialogue = select_bot_dialogue(bot_persona.id, DialogueTrigger::MatchStart);
        let match_state = ChessMatchState::new_standard(Color::White);
        let engine_worker = EngineWorker::spawn();
        let audio = AudioEngine::new(config.sound_enabled);

        Self {
            modal_bot_id: bot_persona.id,
            modal_human_color: Color::White,
            modal_time_control: config.time_control,
            config,
            match_state,
            bot_persona,
            bot_dialogue,
            engine_worker,
            board_ui: BoardInteractionState::default(),
            eval_bar: EvalBarState::default(),
            audio,
            current_request_id: 1,
            is_engine_thinking: false,
            is_hint_search: false,
            thinking_depth: 0,
            thinking_max_depth: 6,
            thinking_nodes: 0,
            last_nps_kns: 0.0,
            search_started_at: None,
            human_turn_started_at: Instant::now(),
            last_eval_cp_white: 0,
            entered_endgame_announced: false,
            show_bot_selector: false,
            show_fen_pgn_modal: false,
            custom_fen_input: String::new(),
            fen_error_msg: None,
            fps_smoothed: 60.0,
            rss_mb: read_process_rss_mb(),
            last_rss_poll: Instant::now(),
        }
    }

    pub fn start_new_match(&mut self, bot_id: BotId, human_color: Color, tc: TimeControl) {
        self.abort_engine();
        self.bot_persona = BotPersona::by_id(bot_id);
        self.config.time_control = tc;
        self.config.board_flipped = human_color == Color::Black;

        let mut state = ChessMatchState::new_standard(human_color);
        state.set_time_control(tc);
        self.match_state = state;
        self.board_ui.reset();
        self.eval_bar = EvalBarState::default();
        self.last_eval_cp_white = 0;
        self.entered_endgame_announced = false;
        self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, DialogueTrigger::MatchStart);
        self.human_turn_started_at = Instant::now();
        self.audio.play(SoundEffect::GameStart);

        if self.match_state.is_bot_turn() {
            self.dispatch_bot_search();
        }
    }

    pub fn start_from_custom_fen(&mut self, fen_str: &str) -> Result<(), String> {
        let mut state = ChessMatchState::from_fen(fen_str, self.match_state.human_color)?;
        state.set_time_control(self.config.time_control);

        self.abort_engine();
        let initial_eval = evaluate_for_white(&state.current_pos, &self.bot_persona.weights);
        self.match_state = state;
        self.board_ui.reset();
        self.eval_bar.set_eval_cp(initial_eval);
        self.last_eval_cp_white = initial_eval;
        self.entered_endgame_announced = false;
        self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, DialogueTrigger::MatchStart);
        self.human_turn_started_at = Instant::now();
        self.audio.play(SoundEffect::GameStart);

        if self.match_state.is_bot_turn() {
            self.dispatch_bot_search();
        }
        Ok(())
    }

    fn abort_engine(&mut self) {
        self.engine_worker.abort_current_search();
        self.current_request_id = self.current_request_id.wrapping_add(1);
        self.is_engine_thinking = false;
        self.is_hint_search = false;
    }

    fn dispatch_bot_search(&mut self) {
        self.current_request_id = self.current_request_id.wrapping_add(1);
        self.is_engine_thinking = true;
        self.is_hint_search = false;
        self.thinking_depth = 1;
        self.thinking_max_depth = self.bot_persona.max_depth;
        self.thinking_nodes = 0;
        self.search_started_at = Some(Instant::now());

        self.engine_worker.request_move(
            self.current_request_id,
            self.match_state.current_pos.clone(),
            self.bot_persona.clone(),
            self.match_state.position_hashes.clone(),
        );
    }

    fn dispatch_hint_search(&mut self) {
        if self.is_engine_thinking || self.match_state.outcome.is_game_over() {
            return;
        }
        self.current_request_id = self.current_request_id.wrapping_add(1);
        self.is_engine_thinking = true;
        self.is_hint_search = true;

        let mut hint_persona = BotPersona::magnus();
        hint_persona.max_depth = 4;
        hint_persona.time_budget_ms = 350;

        self.thinking_depth = 1;
        self.thinking_max_depth = hint_persona.max_depth;
        self.thinking_nodes = 0;
        self.search_started_at = Some(Instant::now());

        self.engine_worker.request_move(
            self.current_request_id,
            self.match_state.current_pos.clone(),
            hint_persona,
            self.match_state.position_hashes.clone(),
        );
    }

    fn poll_engine_events(&mut self) {
        while let Some(event) = self.engine_worker.try_recv_event() {
            match event {
                EngineEvent::ThinkingProgress {
                    request_id,
                    depth,
                    max_depth,
                    eval_cp,
                    nodes,
                } => {
                    if request_id != self.current_request_id {
                        continue;
                    }
                    self.thinking_depth = depth;
                    self.thinking_max_depth = max_depth;
                    self.thinking_nodes = nodes;
                    self.eval_bar.set_eval_cp(eval_cp);

                    if let Some(started) = self.search_started_at {
                        let secs = started.elapsed().as_secs_f32().max(0.001);
                        self.last_nps_kns = (nodes as f32 / secs) / 1000.0;
                    }
                }
                EngineEvent::MoveReady {
                    request_id,
                    best_move,
                    eval_cp,
                    depth_reached,
                    nodes_searched,
                    elapsed_ms,
                } => {
                    if request_id != self.current_request_id {
                        continue;
                    }
                    self.is_engine_thinking = false;
                    self.thinking_depth = depth_reached;
                    self.thinking_nodes = nodes_searched;
                    let secs = (elapsed_ms as f32 / 1000.0).max(0.001);
                    self.last_nps_kns = (nodes_searched as f32 / secs) / 1000.0;

                    if self.is_hint_search {
                        self.is_hint_search = false;
                        if let Some(from) = best_move.from() {
                            self.board_ui.hint_arrow = Some(TacticalArrow {
                                from,
                                to: best_move.to(),
                            });
                        }
                        self.eval_bar.set_eval_cp(eval_cp);
                    } else {
                        self.execute_move_and_update(best_move, Some(eval_cp), elapsed_ms, true);
                    }
                }
            }
        }
    }

    fn execute_move_and_update(
        &mut self,
        mv: Move,
        precomputed_eval_white: Option<i32>,
        elapsed_ms: u64,
        was_bot_move: bool,
    ) {
        let is_capture = mv.is_capture();
        let is_castle = mv.is_castle();
        let is_promo = mv.is_promotion();
        let captured_non_pawn = match &mv {
            Move::Normal {
                capture: Some(role),
                ..
            } => *role != Role::Pawn,
            _ => false,
        };

        let prev_eval_white = self.last_eval_cp_white;

        // Hitung evaluasi posisi setelah langkah jika belum disediakan engine
        let mut next_pos = self.match_state.current_pos.clone();
        next_pos.play_unchecked(&mv);
        let new_eval_white = precomputed_eval_white
            .unwrap_or_else(|| evaluate_for_white(&next_pos, &self.bot_persona.weights));

        if self
            .match_state
            .apply_move(mv, new_eval_white, elapsed_ms)
            .is_err()
        {
            self.audio.play(SoundEffect::Illegal);
            return;
        }

        self.eval_bar.set_eval_cp(new_eval_white);
        self.last_eval_cp_white = new_eval_white;

        let gave_check = self.match_state.current_pos.is_check();
        let phase = compute_game_phase(self.match_state.current_pos.board());
        let entered_endgame_now = if !self.entered_endgame_announced && phase <= 8 {
            self.entered_endgame_announced = true;
            true
        } else {
            false
        };

        // Mainkan efek suara kontekstual
        if self.match_state.outcome.is_game_over() {
            self.audio.play(SoundEffect::GameOver);
        } else if gave_check {
            self.audio.play(SoundEffect::Check);
        } else if is_promo {
            self.audio.play(SoundEffect::Promote);
        } else if is_castle {
            self.audio.play(SoundEffect::Castle);
        } else if is_capture {
            self.audio.play(SoundEffect::Capture);
        } else {
            self.audio.play(SoundEffect::MovePiece);
        }

        // Update dialog bot berdasarkan outcome atau kondisi papan
        let bot_color = self.match_state.human_color.other();
        if self.match_state.outcome.is_game_over() {
            let trigger = match self.match_state.outcome.winner() {
                Some(w) if w == bot_color => DialogueTrigger::GameOverWin,
                Some(_) => DialogueTrigger::GameOverLoss,
                None => DialogueTrigger::GameOverDraw,
            };
            self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, trigger);
        } else {
            let prev_bot_eval = match bot_color {
                Color::White => prev_eval_white,
                Color::Black => -prev_eval_white,
            };
            let new_bot_eval = match bot_color {
                Color::White => new_eval_white,
                Color::Black => -new_eval_white,
            };

            if let Some(trigger) = detect_dialogue_trigger(
                self.match_state.history.len(),
                prev_bot_eval,
                new_bot_eval,
                was_bot_move,
                captured_non_pawn,
                gave_check,
                phase,
                entered_endgame_now,
            ) {
                self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, trigger);
            }
        }

        if !was_bot_move && self.match_state.is_bot_turn() {
            self.dispatch_bot_search();
        } else {
            self.human_turn_started_at = Instant::now();
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        if self.show_fen_pgn_modal || self.show_bot_selector {
            return;
        }
        ctx.input(|i| {
            if i.key_pressed(Key::ArrowLeft) {
                self.match_state.step_history_prev();
            }
            if i.key_pressed(Key::ArrowRight) {
                self.match_state.step_history_next();
            }
            if i.key_pressed(Key::Home) {
                self.match_state.step_history_first();
            }
            if i.key_pressed(Key::End) {
                self.match_state.step_history_last();
            }
            if i.key_pressed(Key::F) {
                self.config.board_flipped = !self.config.board_flipped;
            }
        });
    }

    fn show_fen_pgn_export_modal(&mut self, ctx: &Context) {
        if !self.show_fen_pgn_modal {
            return;
        }

        let current_fen = format_fen(&self.match_state.displayed_position());
        let pgn_text = self.match_state.export_pgn(self.bot_persona.name);
        let mut open = self.show_fen_pgn_modal;
        let mut load_requested = false;

        Window::new("📋 FEN & PGN Inspector / Export")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .default_size(Vec2::new(540.0, 390.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Current Position FEN:").strong().size(13.0));
                ui.horizontal(|ui| {
                    let mut fen_ro = current_fen.clone();
                    ui.add(
                        egui::TextEdit::singleline(&mut fen_ro)
                            .desired_width(410.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if ui.button("Copy FEN").clicked() {
                        ui.ctx().copy_text(current_fen.clone());
                    }
                });

                ui.add_space(8.0);
                ui.separator();

                ui.label(
                    RichText::new("Load Custom Position from FEN:")
                        .strong()
                        .size(13.0),
                );
                ui.horizontal(|ui| {
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_fen_input)
                            .hint_text("Paste FEN string here...")
                            .desired_width(390.0)
                            .font(egui::TextStyle::Monospace),
                    );
                    if ui.button("Load FEN").clicked() {
                        load_requested = true;
                    }
                });

                if let Some(err) = &self.fen_error_msg {
                    ui.label(
                        RichText::new(err)
                            .size(11.5)
                            .color(Color32::from_rgb(255, 95, 95)),
                    );
                }

                ui.add_space(8.0);
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label(RichText::new("Match PGN Record:").strong().size(13.0));
                    if ui.button("Copy PGN").clicked() {
                        ui.ctx().copy_text(pgn_text.clone());
                    }
                });

                let mut pgn_ro = pgn_text;
                egui::ScrollArea::vertical()
                    .max_height(150.0)
                    .show(ui, |ui| {
                        ui.add(
                            egui::TextEdit::multiline(&mut pgn_ro)
                                .desired_width(f32::INFINITY)
                                .desired_rows(7)
                                .font(egui::TextStyle::Monospace),
                        );
                    });
            });

        self.show_fen_pgn_modal = open;

        if load_requested {
            let input = self.custom_fen_input.trim().to_string();
            match self.start_from_custom_fen(&input) {
                Ok(()) => {
                    self.fen_error_msg = None;
                    self.show_fen_pgn_modal = false;
                }
                Err(e) => {
                    self.fen_error_msg = Some(e);
                }
            }
        }
    }
}

impl eframe::App for RustyChessApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.audio.enabled = self.config.sound_enabled;

        // Update FPS & Memory RSS metrics
        let dt = ctx.input(|i| i.unstable_dt).clamp(0.001, 0.25);
        let instant_fps = (1.0 / dt).clamp(1.0, 240.0);
        self.fps_smoothed = self.fps_smoothed * 0.90 + instant_fps * 0.10;

        if self.last_rss_poll.elapsed() >= Duration::from_secs(1) {
            self.rss_mb = read_process_rss_mb();
            self.last_rss_poll = Instant::now();
        }

        // Tick chess clock
        if self.match_state.outcome == GameOutcome::Ongoing && !self.match_state.history.is_empty()
        {
            if let Some(winner) = self.match_state.clock.tick() {
                self.abort_engine();
                self.match_state.flag_timeout(winner);
                let bot_color = self.match_state.human_color.other();
                let trigger = if winner == bot_color {
                    DialogueTrigger::GameOverWin
                } else {
                    DialogueTrigger::GameOverLoss
                };
                self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, trigger);
                self.audio.play(SoundEffect::GameOver);
            }
        }

        // Poll background AI engine worker events
        self.poll_engine_events();

        // Jika giliran bot tetapi belum ada pencarian aktif, jalankan pencarian
        if self.match_state.is_bot_turn() && !self.is_engine_thinking {
            self.dispatch_bot_search();
        }

        self.handle_keyboard_shortcuts(ctx);

        // 1. Top Bar Panel
        egui::TopBottomPanel::top("top_bar")
            .exact_height(42.0)
            .show(ctx, |ui| {
                let top_actions = render_top_bar(
                    ui,
                    &mut self.config,
                    self.is_engine_thinking,
                    self.thinking_depth,
                    self.thinking_max_depth,
                    self.last_nps_kns,
                    self.fps_smoothed,
                    self.rss_mb,
                );

                if top_actions.open_new_game_modal {
                    self.modal_bot_id = self.bot_persona.id;
                    self.modal_human_color = self.match_state.human_color;
                    self.modal_time_control = self.config.time_control;
                    self.show_bot_selector = true;
                }
                if top_actions.open_fen_pgn_modal {
                    self.custom_fen_input =
                        format_fen(&self.match_state.displayed_position());
                    self.fen_error_msg = None;
                    self.show_fen_pgn_modal = true;
                }
            });

        // 2. Right Control SidePanel (Lebar: 360px)
        egui::SidePanel::right("control_panel")
            .exact_width(360.0)
            .resizable(false)
            .show(ctx, |ui| {
                let side_actions = render_side_panel(
                    ui,
                    &mut self.match_state,
                    &self.bot_persona,
                    &self.bot_dialogue,
                    self.is_engine_thinking,
                );

                if side_actions.open_bot_selector {
                    self.modal_bot_id = self.bot_persona.id;
                    self.modal_human_color = self.match_state.human_color;
                    self.modal_time_control = self.config.time_control;
                    self.show_bot_selector = true;
                }
                if side_actions.undo_takeback {
                    self.abort_engine();
                    self.match_state.undo_move_pair();
                    self.board_ui.reset();
                    let eval_cp = evaluate_for_white(
                        &self.match_state.current_pos,
                        &self.bot_persona.weights,
                    );
                    self.eval_bar.set_eval_cp(eval_cp);
                    self.last_eval_cp_white = eval_cp;
                }
                if side_actions.request_hint {
                    self.dispatch_hint_search();
                }
                if side_actions.resign_game {
                    self.abort_engine();
                    let human_color = self.match_state.human_color;
                    self.match_state.resign(human_color);
                    self.bot_dialogue =
                        select_bot_dialogue(self.bot_persona.id, DialogueTrigger::GameOverWin);
                    self.audio.play(SoundEffect::GameOver);
                }
            });

        // 3. Central Panel (Evaluation Bar + 8x8 Interactive Chessboard)
        egui::CentralPanel::default().show(ctx, |ui| {
            let avail = ui.available_size();
            let banner_reserve = if self.match_state.outcome.is_game_over()
                || !self.match_state.is_viewing_latest()
            {
                40.0
            } else {
                8.0
            };

            let eval_bar_space = if self.config.show_eval_bar { 36.0 } else { 0.0 };
            let max_w = (avail.x - eval_bar_space - 24.0).max(240.0);
            let max_h = (avail.y - banner_reserve - 16.0).max(240.0);
            let board_size = max_w.min(max_h);

            let total_group_w = board_size + eval_bar_space;
            let pad_x = ((avail.x - total_group_w) * 0.5).max(6.0);
            let pad_y = ((avail.y - board_size - banner_reserve) * 0.5).max(6.0);

            ui.add_space(pad_y);

            // Banner status jika Game Over atau sedang di History Inspection Mode
            if self.match_state.outcome.is_game_over() {
                ui.vertical_centered(|ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(44, 62, 92))
                        .stroke(Stroke::new(1.5_f32, Color32::from_rgb(110, 165, 245)))
                        .rounding(Rounding::same(6.0_f32))
                        .inner_margin(egui::Margin::symmetric(14.0, 4.0))
                        .show(ui, |ui| {
                            ui.label(
                                RichText::new(self.match_state.outcome.description())
                                    .strong()
                                    .size(14.0)
                                    .color(Color32::WHITE),
                            );
                        });
                });
                ui.add_space(6.0);
            } else if !self.match_state.is_viewing_latest() {
                ui.vertical_centered(|ui| {
                    egui::Frame::none()
                        .fill(Color32::from_rgb(75, 58, 28))
                        .stroke(Stroke::new(1.2_f32, Color32::from_rgb(235, 185, 65)))
                        .rounding(Rounding::same(6.0_f32))
                        .inner_margin(egui::Margin::symmetric(12.0, 4.0))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "👁 Viewing Move History (Ply {}/{})",
                                        self.match_state.viewed_ply,
                                        self.match_state.history.len()
                                    ))
                                    .strong()
                                    .size(12.5)
                                    .color(Color32::from_rgb(255, 225, 130)),
                                );
                                if ui.small_button("Resume Live >>|").clicked() {
                                    self.match_state.step_history_last();
                                }
                            });
                        });
                });
                ui.add_space(6.0);
            }

            ui.horizontal(|ui| {
                ui.add_space(pad_x);

                if self.config.show_eval_bar {
                    self.eval_bar
                        .ui(ui, board_size, self.config.board_flipped);
                    ui.add_space(8.0);
                }

                if let Some(human_mv) = self.board_ui.ui(
                    ui,
                    board_size,
                    &mut self.match_state,
                    self.config.board_theme,
                    self.config.board_flipped,
                    self.config.show_coordinates,
                ) {
                    let spent_ms = self.human_turn_started_at.elapsed().as_millis() as u64;
                    self.execute_move_and_update(human_mv, None, spent_ms, false);
                }
            });
        });

        // 4. Modals
        if let Some(selection) = show_bot_selector_modal(
            ctx,
            &mut self.show_bot_selector,
            &mut self.modal_bot_id,
            &mut self.modal_human_color,
            &mut self.modal_time_control,
        ) {
            self.start_new_match(
                selection.bot_id,
                selection.human_color,
                selection.time_control,
            );
        }

        self.show_fen_pgn_export_modal(ctx);

        // Reactive wakeup saat engine sedang berpikir atau jam catur berjalan
        if self.is_engine_thinking {
            ctx.request_repaint_after(Duration::from_millis(25));
        } else if !self.match_state.clock.is_unlimited
            && !self.match_state.outcome.is_game_over()
            && !self.match_state.history.is_empty()
        {
            ctx.request_repaint_after(Duration::from_millis(100));
        }
    }
}

/// Membaca penggunaan RAM RSS proses saat ini dari `/proc/self/statm` (Linux).
fn read_process_rss_mb() -> f32 {
    if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
        if let Some(rss_pages_str) = statm.split_whitespace().nth(1) {
            if let Ok(pages) = rss_pages_str.parse::<u64>() {
                let bytes = pages * 4096;
                return (bytes as f32) / (1024.0 * 1024.0);
            }
        }
    }
    18.5
}
