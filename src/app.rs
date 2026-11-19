use crate::{
    audio::{AudioEngine, SoundEffect},
    config::{
        AnimationProfile, AssistanceMode, MoveInputMethod, NotationStyle, PieceSetStyle,
        PlayColorChoice, UserConfig,
    },
    domain::{
        detect_opening_from_records, format_fen, AnnotationColor, ChessMatchState, Color,
        GameOutcome, Move, PlayerController, Role, TacticalArrow, TimeControl,
    },
    engine::{
        bot::{
            detect_dialogue_trigger, select_bot_dialogue, BotId, BotPersona, DialogueTrigger,
        },
        eval::{evaluate_for_white, material::compute_game_phase},
        review::analyze_match_review,
        worker::{EngineEvent, EngineWorker},
    },
    persistence::{SavedMatchSummary, UserProfile},
    ui::{
        board_view::BoardInteractionState,
        eval_bar::EvalBarState,
        modals::{
            show_command_palette, show_game_over_modal, CommandPaletteState, PaletteCommand,
        },
        pages::{
            render_analytics_page, render_bots_lobby, render_home_page, render_play_hub,
            render_puzzles_page, render_review_page, BotsLobbyState, PuzzlesTrainerState,
            ReviewViewState,
        },
        side_panel::{render_player_strip, render_side_panel, show_bot_selector_modal},
        sidebar::{render_left_sidebar, ActiveRoute},
        theme::{apply_global_style_with_mode, BoardTheme, UiTokens},
        top_bar::render_top_bar,
    },
};
use egui::{Color32, Context, Key, RichText, Vec2, Window};
use rand::Rng;
use shakmaty::Position;
use std::time::{Duration, Instant};

pub struct RustyChessApp {
    pub active_route: ActiveRoute,
    pub has_active_match: bool,
    pub profile: UserProfile,
    pub config: UserConfig,
    pub lobby_state: BotsLobbyState,
    pub puzzles_state: PuzzlesTrainerState,
    pub review_state: ReviewViewState,
    pub match_state: ChessMatchState,
    pub bot_persona: BotPersona,
    pub bot_dialogue: String,
    pub engine_worker: EngineWorker,
    pub board_ui: BoardInteractionState,
    pub eval_bar: EvalBarState,
    pub audio: AudioEngine,

    // Assistance & Game Over tracking per match
    pub hints_used: u8,
    pub takebacks_used: u8,
    pub match_result_recorded: bool,
    pub show_game_over: bool,
    pub last_awarded_crowns: u8,

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

    // Modals & UI modes state
    pub zen_mode: bool,
    pub command_palette: CommandPaletteState,
    pub show_bot_selector: bool,
    pub show_fen_pgn_modal: bool,
    pub show_settings_modal: bool,
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
        let profile = UserProfile::load();
        let config = profile.config.clone();
        apply_global_style_with_mode(&cc.egui_ctx, config.dark_mode);

        let bot_persona = BotPersona::magnus();
        let bot_dialogue = select_bot_dialogue(bot_persona.id, DialogueTrigger::MatchStart);
        let mut match_state = ChessMatchState::new_standard(Color::White);
        match_state.set_time_control(config.time_control);
        let engine_worker = EngineWorker::spawn();
        let audio = AudioEngine::new(config.sound_enabled);

        let mut lobby_state = BotsLobbyState::default();
        lobby_state.custom_engine_elo = profile.custom_engine_elo.clamp(250, 3200);

        Self {
            active_route: ActiveRoute::Home,
            has_active_match: false,
            modal_bot_id: bot_persona.id,
            modal_human_color: Color::White,
            modal_time_control: config.time_control,
            profile,
            config,
            lobby_state,
            puzzles_state: PuzzlesTrainerState::default(),
            review_state: ReviewViewState::default(),
            match_state,
            bot_persona,
            bot_dialogue,
            engine_worker,
            board_ui: BoardInteractionState::default(),
            eval_bar: EvalBarState::default(),
            audio,
            hints_used: 0,
            takebacks_used: 0,
            match_result_recorded: false,
            show_game_over: false,
            last_awarded_crowns: 0,
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
            zen_mode: false,
            command_palette: CommandPaletteState::default(),
            show_bot_selector: false,
            show_fen_pgn_modal: false,
            show_settings_modal: false,
            custom_fen_input: String::new(),
            fen_error_msg: None,
            fps_smoothed: 60.0,
            rss_mb: read_process_rss_mb(),
            last_rss_poll: Instant::now(),
        }
    }

    pub fn start_new_match(&mut self, bot_id: BotId, human_color: Color, tc: TimeControl) {
        self.abort_engine();
        self.bot_persona = match bot_id {
            BotId::AdaptiveJimmy => BotPersona::adaptive_jimmy(self.profile.adaptive_jimmy_elo),
            BotId::EngineCustom => BotPersona::engine_custom(self.lobby_state.custom_engine_elo),
            other => BotPersona::by_id(other),
        };
        self.config.time_control = tc;
        self.config.board_flipped = human_color == Color::Black;
        self.profile.config = self.config.clone();
        self.profile.save();

        let mut state = ChessMatchState::new_standard(human_color);
        state.set_time_control(tc);
        self.match_state = state;
        self.board_ui.reset();
        self.eval_bar = EvalBarState::default();
        self.review_state = ReviewViewState::default();
        self.last_eval_cp_white = 0;
        self.entered_endgame_announced = false;
        self.hints_used = 0;
        self.takebacks_used = 0;
        self.match_result_recorded = false;
        self.show_game_over = false;
        self.last_awarded_crowns = 0;
        self.has_active_match = true;
        self.active_route = ActiveRoute::Match;
        self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, DialogueTrigger::MatchStart);
        self.human_turn_started_at = Instant::now();
        self.audio.play(SoundEffect::GameStart);

        if self.match_state.is_bot_turn() {
            self.dispatch_bot_search();
        }
    }

    pub fn start_pass_and_play_match(&mut self) {
        self.abort_engine();
        let mut state = ChessMatchState::new_standard(Color::White);
        state.white_controller = PlayerController::Human;
        state.black_controller = PlayerController::Human;
        state.set_time_control(self.config.time_control);
        self.match_state = state;
        self.board_ui.reset();
        self.eval_bar = EvalBarState::default();
        self.review_state = ReviewViewState::default();
        self.last_eval_cp_white = 0;
        self.entered_endgame_announced = false;
        self.hints_used = 0;
        self.takebacks_used = 0;
        self.match_result_recorded = false;
        self.show_game_over = false;
        self.last_awarded_crowns = 0;
        self.has_active_match = true;
        self.active_route = ActiveRoute::Match;
        self.bot_dialogue = "Pass & Play (2-Player Local Mode) active.".to_string();
        self.human_turn_started_at = Instant::now();
        self.audio.play(SoundEffect::GameStart);
    }

    pub fn start_from_custom_fen(&mut self, fen_str: &str) -> Result<(), String> {
        let mut state = ChessMatchState::from_fen(fen_str, self.match_state.human_color)?;
        state.set_time_control(self.config.time_control);

        self.abort_engine();
        let initial_eval = evaluate_for_white(&state.current_pos, &self.bot_persona.weights);
        self.match_state = state;
        self.board_ui.reset();
        self.eval_bar.set_eval_cp(initial_eval);
        self.review_state = ReviewViewState::default();
        self.last_eval_cp_white = initial_eval;
        self.entered_endgame_announced = false;
        self.hints_used = 0;
        self.takebacks_used = 0;
        self.match_result_recorded = false;
        self.show_game_over = false;
        self.last_awarded_crowns = 0;
        self.has_active_match = true;
        self.active_route = ActiveRoute::Match;
        self.bot_dialogue = select_bot_dialogue(self.bot_persona.id, DialogueTrigger::MatchStart);
        self.human_turn_started_at = Instant::now();
        self.audio.play(SoundEffect::GameStart);

        if self.match_state.is_bot_turn() {
            self.dispatch_bot_search();
        }
        Ok(())
    }

    pub fn hints_remaining(&self) -> Option<u8> {
        match self.config.assistance_mode {
            AssistanceMode::Challenge => Some(0),
            AssistanceMode::Friendly => Some(3u8.saturating_sub(self.hints_used)),
            AssistanceMode::Assisted | AssistanceMode::Custom => None,
        }
    }

    pub fn takebacks_remaining(&self) -> Option<u8> {
        match self.config.assistance_mode {
            AssistanceMode::Challenge => Some(0),
            AssistanceMode::Friendly => Some(3u8.saturating_sub(self.takebacks_used)),
            AssistanceMode::Assisted | AssistanceMode::Custom => None,
        }
    }

    pub fn effective_show_eval_bar(&self) -> bool {
        match self.config.assistance_mode {
            AssistanceMode::Challenge | AssistanceMode::Friendly => false,
            AssistanceMode::Assisted => true,
            AssistanceMode::Custom => self.config.custom_assistance.eval_bar,
        }
    }

    pub fn effective_show_move_feedback(&self) -> bool {
        match self.config.assistance_mode {
            AssistanceMode::Challenge => false,
            AssistanceMode::Friendly | AssistanceMode::Assisted => true,
            AssistanceMode::Custom => self.config.custom_assistance.move_feedback,
        }
    }

    pub fn calculate_earned_crowns_on_win(&self) -> u8 {
        match self.config.assistance_mode {
            AssistanceMode::Challenge => 3,
            AssistanceMode::Friendly => 2,
            AssistanceMode::Assisted => 1,
            AssistanceMode::Custom => {
                if self.hints_used == 0
                    && self.takebacks_used == 0
                    && !self.config.custom_assistance.eval_bar
                {
                    3
                } else if self.hints_used <= 3 && self.takebacks_used <= 3 {
                    2
                } else {
                    1
                }
            }
        }
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

        self.engine_worker.request_move_with_pacing(
            self.current_request_id,
            self.match_state.current_pos.clone(),
            self.bot_persona.clone(),
            self.match_state.position_hashes.clone(),
            self.match_state.history.len(),
            self.config.time_control.clock_factor(),
            true,
        );
    }

    fn dispatch_hint_search(&mut self) {
        if self.is_engine_thinking || self.match_state.outcome.is_game_over() {
            return;
        }
        if let Some(0) = self.hints_remaining() {
            return;
        }
        self.hints_used = self.hints_used.saturating_add(1);
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

        self.engine_worker.request_move_with_pacing(
            self.current_request_id,
            self.match_state.current_pos.clone(),
            hint_persona,
            self.match_state.position_hashes.clone(),
            self.match_state.history.len(),
            1.0,
            false,
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
                            self.board_ui.hint_arrow = Some(TacticalArrow::new(
                                from,
                                best_move.to(),
                                AnnotationColor::Green,
                            ));
                        }
                        self.eval_bar.set_eval_cp(eval_cp);
                    } else {
                        self.execute_move_and_update(best_move, Some(eval_cp), elapsed_ms, true);
                        if !self.match_state.outcome.is_game_over() {
                            if let Some(premove_mv) =
                                self.board_ui.try_consume_premove(&self.match_state)
                            {
                                self.execute_move_and_update(premove_mv, None, 0, false);
                            }
                        }
                    }
                }
            }
        }
    }

    fn record_match_completion_if_needed(&mut self) {
        if self.match_result_recorded || !self.match_state.outcome.is_game_over() {
            return;
        }
        self.match_result_recorded = true;
        self.board_ui
            .anim_layer
            .update_result_glyphs(&self.match_state.current_pos, self.match_state.outcome);

        let human_color = self.match_state.human_color;
        let won = self.match_state.outcome.winner().map(|w| w == human_color);
        let crowns = if won == Some(true) {
            self.calculate_earned_crowns_on_win()
        } else {
            0
        };
        self.last_awarded_crowns = crowns;
        self.show_game_over = true;

        if self.bot_persona.id == BotId::AdaptiveJimmy {
            match won {
                Some(true) => {
                    self.profile.adaptive_jimmy_elo =
                        (self.profile.adaptive_jimmy_elo + 100).min(2200);
                }
                Some(false) => {
                    self.profile.adaptive_jimmy_elo =
                        self.profile.adaptive_jimmy_elo.saturating_sub(75).max(400);
                }
                None => {}
            }
        }

        let report = analyze_match_review(&self.match_state, 3);
        let (player_acc, bot_acc) = match human_color {
            Color::White => (report.white_accuracy, report.black_accuracy),
            Color::Black => (report.black_accuracy, report.white_accuracy),
        };

        // Harvest human Blunders (??) and Misses (✗) into Personal Mistake Vault (Feature #5)
        for entry in &report.entries {
            if entry.side == human_color
                && matches!(
                    entry.classification,
                    crate::engine::review::MoveClassification::Blunder
                        | crate::engine::review::MoveClassification::Miss
                )
            {
                self.profile.add_mistake_entry(
                    entry.fen_before.clone(),
                    entry.side == Color::White,
                    entry.played_san.clone(),
                    entry.best_san.clone(),
                    entry.best_uci.clone(),
                    entry.classification.label().to_string(),
                    self.bot_persona.name.to_string(),
                );
            }
        }

        // Detect and record Opening ECO stats (Feature #1)
        let opening_label = if let Some(op) = detect_opening_from_records(&self.match_state.history)
        {
            self.profile.record_opening_result(op.eco, op.name, won);
            format!("{} • {}", op.eco, op.name)
        } else {
            "Unclassified Opening".to_string()
        };

        self.review_state.analyzed_plies_count = self.match_state.history.len();
        self.review_state.selected_ply = report.entries.len();
        self.review_state.report = Some(report);

        let summary = SavedMatchSummary {
            opponent_name: self.bot_persona.name.to_string(),
            opponent_elo: self.bot_persona.elo,
            result_label: self.match_state.outcome.description(),
            opening_name: opening_label,
            player_accuracy: player_acc,
            bot_accuracy: bot_acc,
            moves_count: (self.match_state.history.len() + 1) / 2,
            pgn: self.match_state.export_pgn(self.bot_persona.name),
        };
        self.profile
            .record_match_result(won, self.bot_persona.code, crowns, summary);
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
        let pos_before = self.match_state.current_pos.clone();

        let mut next_pos = pos_before.clone();
        next_pos.play_unchecked(&mv);
        let new_eval_white = precomputed_eval_white
            .unwrap_or_else(|| evaluate_for_white(&next_pos, &self.bot_persona.weights));

        if self
            .match_state
            .apply_move(mv.clone(), new_eval_white, elapsed_ms)
            .is_err()
        {
            self.audio.play(SoundEffect::Illegal);
            return;
        }

        self.board_ui.anim_layer.trigger_move_animation(
            &pos_before,
            &self.match_state.current_pos,
            &mv,
            self.match_state.outcome,
            self.config.animation_profile,
        );

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

        if self.match_state.outcome.is_game_over() {
            self.audio.play(SoundEffect::GameOver);
            self.record_match_completion_if_needed();
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

            if !was_bot_move && self.bot_persona.id == BotId::AdaptiveJimmy {
                let eval_delta_for_bot = new_bot_eval - prev_bot_eval;
                if eval_delta_for_bot <= 25 {
                    self.profile.adaptive_jimmy_elo =
                        (self.profile.adaptive_jimmy_elo + 25).min(2200);
                } else if eval_delta_for_bot >= 140 {
                    self.profile.adaptive_jimmy_elo =
                        self.profile.adaptive_jimmy_elo.saturating_sub(45).max(400);
                }
                self.bot_persona = BotPersona::adaptive_jimmy(self.profile.adaptive_jimmy_elo);
            }
        }

        if !was_bot_move && self.match_state.is_bot_turn() {
            self.dispatch_bot_search();
        } else {
            self.human_turn_started_at = Instant::now();
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &Context) {
        // Ctrl+K toggles Command Palette from anywhere
        let ctrl_k = ctx.input(|i| (i.modifiers.ctrl || i.modifiers.command) && i.key_pressed(Key::K));
        if ctrl_k {
            self.command_palette.toggle();
            return;
        }

        if self.command_palette.open
            || self.show_fen_pgn_modal
            || self.show_bot_selector
            || self.show_settings_modal
            || self.show_game_over
        {
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
            if i.key_pressed(Key::X) {
                self.config.show_xray_heatmap = !self.config.show_xray_heatmap;
            }
            if i.key_pressed(Key::Z) && self.active_route == ActiveRoute::Match {
                self.zen_mode = !self.zen_mode;
            }
        });
    }

    fn show_settings_modal_window(&mut self, ctx: &Context) {
        if !self.show_settings_modal {
            return;
        }
        let mut open = self.show_settings_modal;
        let mut style_changed = false;
        let mut reset_requested = false;

        Window::new("⚙ Rusty Chess Settings")
            .open(&mut open)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
            .fixed_size(Vec2::new(460.0, 390.0))
            .show(ctx, |ui| {
                ui.label(RichText::new("Board & Visual Appearance").strong().size(14.5));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("UI Theme:");
                    if ui
                        .selectable_value(&mut self.config.dark_mode, true, "🌙 Dark UI")
                        .clicked()
                    {
                        style_changed = true;
                    }
                    if ui
                        .selectable_value(&mut self.config.dark_mode, false, "☀ Light UI")
                        .clicked()
                    {
                        style_changed = true;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Board Theme:");
                    egui::ComboBox::from_id_salt("settings_board_theme")
                        .selected_text(self.config.board_theme.name())
                        .show_ui(ui, |ui| {
                            for &t in BoardTheme::all() {
                                ui.selectable_value(&mut self.config.board_theme, t, t.name());
                            }
                        });
                });

                ui.horizontal(|ui| {
                    ui.label("Piece Set:");
                    egui::ComboBox::from_id_salt("settings_piece_set")
                        .selected_text(self.config.piece_set.label())
                        .show_ui(ui, |ui| {
                            for &ps in PieceSetStyle::all() {
                                ui.selectable_value(&mut self.config.piece_set, ps, ps.label());
                            }
                        });
                });

                ui.checkbox(
                    &mut self.config.textured_board,
                    "Procedural Textured Board Surface (Wood / Linen / Slate / Ice)",
                );

                ui.horizontal(|ui| {
                    ui.label("Animation Speed:");
                    egui::ComboBox::from_id_salt("settings_anim_speed")
                        .selected_text(self.config.animation_profile.label())
                        .show_ui(ui, |ui| {
                            for &p in AnimationProfile::all() {
                                ui.selectable_value(
                                    &mut self.config.animation_profile,
                                    p,
                                    p.label(),
                                );
                            }
                        });
                });

                ui.add_space(6.0);
                ui.separator();
                ui.label(RichText::new("Gameplay & Notation").strong().size(14.5));
                ui.add_space(4.0);

                ui.horizontal(|ui| {
                    ui.label("Move Method:");
                    for &m in MoveInputMethod::all() {
                        ui.selectable_value(&mut self.config.move_input_method, m, m.label());
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Move Notation:");
                    ui.selectable_value(
                        &mut self.config.notation_style,
                        NotationStyle::Figurine,
                        "♘ Figurine (♘f3)",
                    );
                    ui.selectable_value(
                        &mut self.config.notation_style,
                        NotationStyle::Text,
                        "Text (Nf3)",
                    );
                });

                ui.checkbox(
                    &mut self.config.enable_premoves,
                    "Enable Premoves during Bot Turn",
                );
                ui.checkbox(
                    &mut self.config.show_coordinates,
                    "Show Board Coordinates (A–H / 1–8)",
                );
                ui.checkbox(
                    &mut self.config.show_xray_heatmap,
                    "X-Ray Square Control & Hanging Piece Heatmap (X)",
                );
                ui.checkbox(&mut self.config.sound_enabled, "Enable Sound Effects");

                ui.add_space(8.0);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui
                        .button(
                            RichText::new("🗑 Reset Crown & Match Progress")
                                .color(Color32::from_rgb(235, 85, 85)),
                        )
                        .clicked()
                    {
                        reset_requested = true;
                    }
                });
            });

        self.show_settings_modal = open;
        if style_changed {
            apply_global_style_with_mode(ctx, self.config.dark_mode);
        }
        if reset_requested {
            self.profile.reset_progress();
        }
        self.profile.config = self.config.clone();
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

        let dt = ctx.input(|i| i.unstable_dt).clamp(0.001, 0.25);
        let instant_fps = (1.0 / dt).clamp(1.0, 240.0);
        self.fps_smoothed = self.fps_smoothed * 0.90 + instant_fps * 0.10;

        if self.last_rss_poll.elapsed() >= Duration::from_secs(2) {
            self.rss_mb = read_process_rss_mb();
            self.last_rss_poll = Instant::now();
        }

        let clock_actively_ticking = self.active_route == ActiveRoute::Match
            && !self.match_state.clock.is_unlimited
            && self.match_state.outcome == GameOutcome::Ongoing
            && !self.match_state.history.is_empty();

        if clock_actively_ticking {
            if let Some(winner) = self.match_state.clock.tick() {
                self.abort_engine();
                self.match_state.flag_timeout(winner);
                self.record_match_completion_if_needed();
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

        self.poll_engine_events();

        if self.active_route == ActiveRoute::Match
            && self.match_state.is_bot_turn()
            && !self.is_engine_thinking
        {
            self.dispatch_bot_search();
        }

        self.handle_keyboard_shortcuts(ctx);

        // Persistent Left Navigation Sidebar (hidden during Zen Mode in Match)
        let in_zen_match = self.zen_mode && self.active_route == ActiveRoute::Match;
        let win_w = ctx.screen_rect().width();
        let compact_sidebar = self.config.sidebar_collapsed.unwrap_or(win_w < 1100.0);
        let sidebar_w = if compact_sidebar { 60.0 } else { 168.0 };
        let tokens = UiTokens::for_mode(self.config.dark_mode);

        if !in_zen_match {
            egui::SidePanel::left("left_navigation_sidebar")
                .exact_width(sidebar_w)
                .resizable(false)
                .frame(
                    egui::Frame::none()
                        .fill(tokens.bg_sidebar)
                        .stroke(egui::Stroke::new(1.0_f32, tokens.border_subtle)),
                )
                .show(ctx, |ui| {
                    let nav = render_left_sidebar(
                        ui,
                        self.active_route,
                        &mut self.config,
                        self.has_active_match,
                        compact_sidebar,
                    );
                    if let Some(next_route) = nav.navigate_to {
                        self.active_route = next_route;
                    }
                    if nav.toggled_theme_mode {
                        apply_global_style_with_mode(ctx, self.config.dark_mode);
                        self.profile.config = self.config.clone();
                        self.profile.save();
                    }
                    if nav.open_settings_modal {
                        self.show_settings_modal = true;
                    }
                });
        }

        match self.active_route {
            ActiveRoute::Home => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let home_actions = render_home_page(
                        ui,
                        &self.profile,
                        self.config.board_theme,
                        self.config.dark_mode,
                    );
                    if let Some(r) = home_actions.navigate_to {
                        self.active_route = r;
                    }
                    if let Some(bot_id) = home_actions.quick_start_bot {
                        self.lobby_state.selected_bot = bot_id;
                        self.start_new_match(bot_id, Color::White, self.config.time_control);
                    }
                    if home_actions.start_coach_match {
                        self.config.assistance_mode = AssistanceMode::Friendly;
                        self.start_new_match(
                            BotId::AdaptiveJimmy,
                            Color::White,
                            self.config.time_control,
                        );
                    }
                });
            }

            ActiveRoute::PlayHub => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let hub_actions = render_play_hub(
                        ui,
                        &self.match_state,
                        &self.bot_persona,
                        self.config.board_theme,
                        self.config.dark_mode,
                    );
                    if let Some(r) = hub_actions.navigate_to {
                        self.active_route = r;
                    }
                    if hub_actions.start_coach_mode {
                        self.config.assistance_mode = AssistanceMode::Assisted;
                        self.start_new_match(
                            BotId::AdaptiveJimmy,
                            Color::White,
                            self.config.time_control,
                        );
                    }
                    if hub_actions.start_pass_and_play {
                        self.start_pass_and_play_match();
                    }
                    if hub_actions.open_custom_fen_modal {
                        self.custom_fen_input =
                            format_fen(&self.match_state.displayed_position());
                        self.fen_error_msg = None;
                        self.show_fen_pgn_modal = true;
                    }
                });
            }

            ActiveRoute::PlayBots => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let lobby_actions = render_bots_lobby(
                        ui,
                        &mut self.lobby_state,
                        &mut self.profile,
                        &mut self.config,
                        &self.match_state,
                    );
                    if lobby_actions.start_match_requested {
                        let human_color = match self.lobby_state.play_color_choice {
                            PlayColorChoice::White => Color::White,
                            PlayColorChoice::Black => Color::Black,
                            PlayColorChoice::Random => {
                                if rand::thread_rng().gen_bool(0.5) {
                                    Color::White
                                } else {
                                    Color::Black
                                }
                            }
                        };
                        self.start_new_match(
                            self.lobby_state.selected_bot,
                            human_color,
                            self.config.time_control,
                        );
                    }
                });
            }

            ActiveRoute::Puzzles => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_puzzles_page(
                        ui,
                        &mut self.puzzles_state,
                        &mut self.profile,
                        &self.config,
                    );
                });
            }

            ActiveRoute::Analytics => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    let analytics_actions =
                        render_analytics_page(ui, &self.profile, self.config.dark_mode);
                    if analytics_actions.open_blunder_deck {
                        self.puzzles_state.open_blunder_deck(&self.profile);
                    }
                    if let Some(r) = analytics_actions.navigate_to {
                        self.active_route = r;
                    }
                });
            }

            ActiveRoute::Review => {
                egui::CentralPanel::default().show(ctx, |ui| {
                    render_review_page(
                        ui,
                        &mut self.review_state,
                        &mut self.match_state,
                        self.bot_persona.name,
                        &self.config,
                    );
                });
            }

            ActiveRoute::Match => {
                if !in_zen_match {
                    egui::TopBottomPanel::top("top_bar")
                        .exact_height(40.0)
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
                                self.active_route = ActiveRoute::PlayBots;
                            }
                            if top_actions.open_fen_pgn_modal {
                                self.custom_fen_input =
                                    format_fen(&self.match_state.displayed_position());
                                self.fen_error_msg = None;
                                self.show_fen_pgn_modal = true;
                            }
                            if top_actions.toggle_zen_mode {
                                self.zen_mode = !self.zen_mode;
                            }
                            if top_actions.open_command_palette {
                                self.command_palette.toggle();
                            }
                        });

                    let hints_rem = self.hints_remaining();
                    let takebacks_rem = self.takebacks_remaining();
                    let earned_crowns = self.profile.crowns_for(self.bot_persona.code);
                    let show_feedback = self.effective_show_move_feedback();

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
                                self.config.assistance_mode,
                                hints_rem,
                                takebacks_rem,
                                earned_crowns,
                                self.config.notation_style,
                                show_feedback,
                            );

                            if side_actions.open_bot_selector {
                                self.active_route = ActiveRoute::PlayBots;
                            }
                            if side_actions.open_review {
                                self.active_route = ActiveRoute::Review;
                            }
                            if side_actions.flip_board {
                                self.config.board_flipped = !self.config.board_flipped;
                            }
                            if side_actions.undo_takeback {
                                if self.takebacks_remaining().unwrap_or(99) > 0 {
                                    self.takebacks_used = self.takebacks_used.saturating_add(1);
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
                            }
                            if side_actions.request_hint {
                                self.dispatch_hint_search();
                            }
                            if side_actions.resign_game {
                                self.abort_engine();
                                let human_color = self.match_state.human_color;
                                self.match_state.resign(human_color);
                                self.record_match_completion_if_needed();
                                self.bot_dialogue = select_bot_dialogue(
                                    self.bot_persona.id,
                                    DialogueTrigger::GameOverWin,
                                );
                                self.audio.play(SoundEffect::GameOver);
                            }
                        });
                }

                let show_eval_bar = self.effective_show_eval_bar();
                egui::CentralPanel::default().show(ctx, |ui| {
                    if in_zen_match {
                        ui.horizontal(|ui| {
                            if ui.small_button("✕ Exit Zen Mode (Z)").clicked() {
                                self.zen_mode = false;
                            }
                            if ui.small_button("⌘ Ctrl+K").clicked() {
                                self.command_palette.toggle();
                            }
                        });
                    }
                    let avail = ui.available_size();
                    let eval_w = if show_eval_bar { 28.0 + 8.0 } else { 0.0 };
                    let board_size = (avail.x - eval_w - 16.0)
                        .min(avail.y - 104.0)
                        .max(320.0);

                    let total_group_w = board_size + eval_w;
                    let pad_x = ((avail.x - total_group_w) * 0.5).max(8.0);
                    let pad_y = ((avail.y - (board_size + 96.0)) * 0.5).max(4.0);

                    let top_is_bot = (self.match_state.human_color == Color::White
                        && !self.config.board_flipped)
                        || (self.match_state.human_color == Color::Black
                            && self.config.board_flipped);

                    ui.add_space(pad_y);

                    ui.horizontal(|ui| {
                        ui.add_space(pad_x + eval_w);
                        render_player_strip(
                            ui,
                            board_size,
                            &self.match_state,
                            &self.bot_persona,
                            top_is_bot,
                            self.is_engine_thinking && top_is_bot,
                            self.config.dark_mode,
                        );
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.add_space(pad_x);

                        if show_eval_bar {
                            self.eval_bar
                                .ui(ui, board_size, self.config.board_flipped);
                            ui.add_space(8.0);
                        }

                        if let Some(human_mv) = self.board_ui.ui_with_config(
                            ui,
                            board_size,
                            &mut self.match_state,
                            &self.config,
                        ) {
                            let spent_ms = self.human_turn_started_at.elapsed().as_millis() as u64;
                            self.execute_move_and_update(human_mv, None, spent_ms, false);
                        }
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.add_space(pad_x + eval_w);
                        render_player_strip(
                            ui,
                            board_size,
                            &self.match_state,
                            &self.bot_persona,
                            !top_is_bot,
                            self.is_engine_thinking && !top_is_bot,
                            self.config.dark_mode,
                        );
                    });
                });
            }
        }

        // Modals
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

        let go_actions = show_game_over_modal(
            ctx,
            &mut self.show_game_over,
            self.match_state.outcome,
            self.match_state.human_color,
            &self.bot_persona,
            self.last_awarded_crowns,
            self.config.dark_mode,
        );
        if go_actions.open_game_review {
            self.active_route = ActiveRoute::Review;
        }
        if go_actions.play_rematch {
            let next_color = self.match_state.human_color.other();
            self.start_new_match(self.bot_persona.id, next_color, self.config.time_control);
        }
        if go_actions.select_new_bot {
            self.active_route = ActiveRoute::PlayBots;
        }

        self.show_fen_pgn_export_modal(ctx);
        self.show_settings_modal_window(ctx);

        // Spotlight Command Palette (Ctrl+K) & Keyboard SAN Move Entry (Feature #10)
        if let Some(cmd) = show_command_palette(
            ctx,
            &mut self.command_palette,
            &self.match_state,
            self.active_route == ActiveRoute::Match,
            self.config.dark_mode,
        ) {
            match cmd {
                PaletteCommand::Navigate(route) => self.active_route = route,
                PaletteCommand::ChallengeBot(bot_id) => {
                    self.lobby_state.selected_bot = bot_id;
                    self.start_new_match(bot_id, Color::White, self.config.time_control);
                }
                PaletteCommand::SetBoardTheme(theme) => {
                    self.config.board_theme = theme;
                    self.profile.config = self.config.clone();
                    self.profile.save();
                }
                PaletteCommand::SetPieceSet(ps) => {
                    self.config.piece_set = ps;
                    self.profile.config = self.config.clone();
                    self.profile.save();
                }
                PaletteCommand::ToggleXRay => {
                    self.config.show_xray_heatmap = !self.config.show_xray_heatmap;
                }
                PaletteCommand::ToggleZenMode => {
                    self.zen_mode = !self.zen_mode;
                }
                PaletteCommand::ToggleDarkLight => {
                    self.config.dark_mode = !self.config.dark_mode;
                    apply_global_style_with_mode(ctx, self.config.dark_mode);
                    self.profile.config = self.config.clone();
                    self.profile.save();
                }
                PaletteCommand::FlipBoard => {
                    self.config.board_flipped = !self.config.board_flipped;
                }
                PaletteCommand::OpenFenPgn => {
                    self.custom_fen_input = format_fen(&self.match_state.displayed_position());
                    self.fen_error_msg = None;
                    self.show_fen_pgn_modal = true;
                }
                PaletteCommand::OpenSettings => {
                    self.show_settings_modal = true;
                }
                PaletteCommand::OpenBlunderDeck => {
                    self.puzzles_state.open_blunder_deck(&self.profile);
                    self.active_route = ActiveRoute::Puzzles;
                }
                PaletteCommand::PlaySanMove(mv, _san) => {
                    if self.active_route == ActiveRoute::Match && self.match_state.is_viewing_latest()
                    {
                        let spent = self.human_turn_started_at.elapsed().as_millis() as u64;
                        self.execute_move_and_update(mv, None, spent, false);
                    }
                }
            }
        }

        // Strict Reactive Repaint Mode (NEW_DESIGN.md Section 6.4)
        let puzzle_anim_active = !self.puzzles_state.board_ui.anim_layer.is_idle()
            || self.puzzles_state.board_ui.dragging_from.is_some();

        if !self.board_ui.anim_layer.is_idle()
            || puzzle_anim_active
            || self.eval_bar.is_lerping()
            || self.board_ui.dragging_from.is_some()
        {
            ctx.request_repaint();
        } else if self.is_engine_thinking || clock_actively_ticking {
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
