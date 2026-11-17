use crate::domain::{
    clock::{ChessClock, TimeControl},
    notation::{format_fen, format_san, parse_fen, position_zobrist_hash, INITIAL_FEN},
    types::{
        role_point_value, role_sort_priority, GameOutcome, MoveRecord, PlayerController,
        TacticalArrow,
    },
};
use shakmaty::{Chess, Color, Move, Position, Role, Square};

#[derive(Debug, Clone)]
pub struct ChessMatchState {
    pub initial_pos: Chess,
    pub initial_fen: String,
    pub current_pos: Chess,
    pub viewed_ply: usize,
    pub history: Vec<MoveRecord>,
    pub position_hashes: Vec<u64>,
    pub white_controller: PlayerController,
    pub black_controller: PlayerController,
    pub human_color: Color,
    pub last_move: Option<(Square, Square)>,
    pub captured_by_white: Vec<Role>,
    pub captured_by_black: Vec<Role>,
    pub material_balance: i32,
    pub outcome: GameOutcome,
    pub clock: ChessClock,
    pub arrows: Vec<TacticalArrow>,
    pub marked_squares: Vec<Square>,
}

impl ChessMatchState {
    pub fn new_standard(human_color: Color) -> Self {
        let initial_pos = Chess::default();
        let initial_fen = INITIAL_FEN.to_string();
        let initial_hash = position_zobrist_hash(&initial_pos);
        let (white_controller, black_controller) = match human_color {
            Color::White => (PlayerController::Human, PlayerController::Bot),
            Color::Black => (PlayerController::Bot, PlayerController::Human),
        };
        let mut clock = ChessClock::new(TimeControl::Rapid10_0);
        clock.start(Color::White);

        let mut state = Self {
            initial_pos: initial_pos.clone(),
            initial_fen,
            current_pos: initial_pos,
            viewed_ply: 0,
            history: Vec::with_capacity(128),
            position_hashes: vec![initial_hash],
            white_controller,
            black_controller,
            human_color,
            last_move: None,
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            material_balance: 0,
            outcome: GameOutcome::Ongoing,
            clock,
            arrows: Vec::new(),
            marked_squares: Vec::new(),
        };
        state.recalculate_material_and_captures();
        state.outcome = state.evaluate_outcome();
        state
    }

    pub fn from_fen(fen_str: &str, human_color: Color) -> Result<Self, String> {
        let initial_pos = parse_fen(fen_str)?;
        let initial_fen = format_fen(&initial_pos);
        let initial_hash = position_zobrist_hash(&initial_pos);
        let (white_controller, black_controller) = match human_color {
            Color::White => (PlayerController::Human, PlayerController::Bot),
            Color::Black => (PlayerController::Bot, PlayerController::Human),
        };
        let mut clock = ChessClock::new(TimeControl::Rapid10_0);
        clock.start(initial_pos.turn());

        let mut state = Self {
            initial_pos: initial_pos.clone(),
            initial_fen,
            current_pos: initial_pos,
            viewed_ply: 0,
            history: Vec::with_capacity(128),
            position_hashes: vec![initial_hash],
            white_controller,
            black_controller,
            human_color,
            last_move: None,
            captured_by_white: Vec::new(),
            captured_by_black: Vec::new(),
            material_balance: 0,
            outcome: GameOutcome::Ongoing,
            clock,
            arrows: Vec::new(),
            marked_squares: Vec::new(),
        };
        state.recalculate_material_and_captures();
        state.outcome = state.evaluate_outcome();
        Ok(state)
    }

    pub fn set_time_control(&mut self, tc: TimeControl) {
        self.clock = ChessClock::new(tc);
        if self.outcome == GameOutcome::Ongoing {
            self.clock.start(self.current_pos.turn());
        }
    }

    pub fn apply_move(
        &mut self,
        mv: Move,
        eval_cp: i32,
        elapsed_ms: u64,
    ) -> Result<GameOutcome, String> {
        if self.outcome.is_game_over() {
            return Err("Cannot apply move: game is already over".to_string());
        }
        if !self.current_pos.is_legal(&mv) {
            return Err(format!("Illegal move: {mv:?}"));
        }

        let san = format_san(&self.current_pos, &mv);
        let captured_piece = match &mv {
            Move::Normal { capture, .. } => *capture,
            Move::EnPassant { .. } => Some(Role::Pawn),
            _ => None,
        };

        let from_sq = mv.from().unwrap_or_else(|| mv.to());
        let to_sq = mv.to();

        self.current_pos.play_unchecked(&mv);
        let fen_after = format_fen(&self.current_pos);
        let new_hash = position_zobrist_hash(&self.current_pos);
        self.position_hashes.push(new_hash);

        let ply = self.history.len() + 1;
        self.history.push(MoveRecord {
            ply,
            mv,
            san,
            fen_after,
            eval_cp,
            captured_piece,
            time_spent_ms: elapsed_ms,
        });

        self.viewed_ply = self.history.len();
        self.last_move = Some((from_sq, to_sq));
        self.arrows.clear();
        self.marked_squares.clear();

        let _ = self.clock.switch_turn(self.current_pos.turn());
        self.recalculate_material_and_captures();
        self.outcome = self.evaluate_outcome();

        if self.outcome.is_game_over() {
            self.clock.stop();
        }

        Ok(self.outcome)
    }

    /// Membatalkan langkah Bot dan langkah Human terakhir (Takeback) serta menyinkronkan ulang `current_pos`.
    pub fn undo_move_pair(&mut self) {
        if self.history.is_empty() {
            return;
        }

        // Jika giliran Human dan ada >= 2 langkah, batalkan sepasang (Bot + Human).
        // Jika sedang giliran Bot (misal saat bot berpikir) atau baru 1 langkah, batalkan 1 langkah.
        let steps_to_undo = if self.controller_for(self.current_pos.turn()) == PlayerController::Human
            && self.history.len() >= 2
        {
            2
        } else {
            1
        };

        for _ in 0..steps_to_undo {
            let _ = self.history.pop();
            if self.position_hashes.len() > 1 {
                let _ = self.position_hashes.pop();
            }
        }

        let mut pos = self.initial_pos.clone();
        for rec in &self.history {
            pos.play_unchecked(&rec.mv);
        }
        self.current_pos = pos;
        self.viewed_ply = self.history.len();
        self.last_move = self.history.last().map(|r| {
            let from = r.mv.from().unwrap_or_else(|| r.mv.to());
            (from, r.mv.to())
        });
        self.arrows.clear();
        self.marked_squares.clear();

        self.recalculate_material_and_captures();
        self.outcome = self.evaluate_outcome();
        if !self.outcome.is_game_over() {
            self.clock.start(self.current_pos.turn());
        }
    }

    /// Menghasilkan teks PGN lengkap dengan header dan daftar langkah.
    pub fn export_pgn(&self, bot_name: &str) -> String {
        let (white_name, black_name) = match self.human_color {
            Color::White => ("Player (Human)", bot_name),
            Color::Black => (bot_name, "Player (Human)"),
        };
        let result_str = self.outcome.pgn_result_str();

        let mut pgn = String::with_capacity(512);
        pgn.push_str("[Event \"RustyChess Match\"]\n");
        pgn.push_str("[Site \"Linux Native Desktop\"]\n");
        pgn.push_str(&format!("[White \"{white_name}\"]\n"));
        pgn.push_str(&format!("[Black \"{black_name}\"]\n"));
        pgn.push_str(&format!("[Result \"{result_str}\"]\n"));

        if self.initial_fen != INITIAL_FEN {
            pgn.push_str("[SetUp \"1\"]\n");
            pgn.push_str(&format!("[FEN \"{}\"]\n", self.initial_fen));
        }
        pgn.push('\n');

        let mut move_text = String::new();
        for (idx, record) in self.history.iter().enumerate() {
            if idx % 2 == 0 {
                let move_num = (idx / 2) + 1;
                if !move_text.is_empty() {
                    move_text.push(' ');
                }
                move_text.push_str(&format!("{move_num}. {}", record.san));
            } else {
                move_text.push_str(&format!(" {}", record.san));
            }
        }

        if !move_text.is_empty() {
            move_text.push(' ');
        }
        move_text.push_str(result_str);
        pgn.push_str(&move_text);
        pgn.push('\n');
        pgn
    }

    pub fn evaluate_outcome(&self) -> GameOutcome {
        if self.current_pos.is_checkmate() {
            return GameOutcome::Checkmate {
                winner: self.current_pos.turn().other(),
            };
        }
        if self.current_pos.is_stalemate() {
            return GameOutcome::Stalemate;
        }
        if self.current_pos.is_insufficient_material() {
            return GameOutcome::InsufficientMaterial;
        }
        if self.current_pos.halfmoves() >= 100 {
            return GameOutcome::FiftyMoveRule;
        }
        if let Some(&current_hash) = self.position_hashes.last() {
            let count = self
                .position_hashes
                .iter()
                .filter(|&&h| h == current_hash)
                .count();
            if count >= 3 {
                return GameOutcome::ThreefoldRepetition;
            }
        }
        GameOutcome::Ongoing
    }

    fn recalculate_material_and_captures(&mut self) {
        self.captured_by_white.clear();
        self.captured_by_black.clear();

        let mut mover = self.initial_pos.turn();
        for rec in &self.history {
            if let Some(captured) = rec.captured_piece {
                match mover {
                    Color::White => self.captured_by_white.push(captured),
                    Color::Black => self.captured_by_black.push(captured),
                }
            }
            mover = mover.other();
        }

        self.captured_by_white
            .sort_by_key(|&role| role_sort_priority(role));
        self.captured_by_black
            .sort_by_key(|&role| role_sort_priority(role));

        let board = self.current_pos.board();
        let mut white_pts = 0i32;
        let mut black_pts = 0i32;
        for sq in Square::ALL {
            if let Some(piece) = board.piece_at(sq) {
                let pts = role_point_value(piece.role);
                match piece.color {
                    Color::White => white_pts += pts,
                    Color::Black => black_pts += pts,
                }
            }
        }
        self.material_balance = white_pts - black_pts;
    }

    /// Mengembalikan posisi yang sedang ditampilkan sesuai `viewed_ply` (untuk History Inspection Mode).
    pub fn displayed_position(&self) -> Chess {
        if self.viewed_ply >= self.history.len() {
            return self.current_pos.clone();
        }
        let mut pos = self.initial_pos.clone();
        for rec in self.history.iter().take(self.viewed_ply) {
            pos.play_unchecked(&rec.mv);
        }
        pos
    }

    /// Mengembalikan highlight langkah terakhir pada posisi `viewed_ply`.
    pub fn displayed_last_move(&self) -> Option<(Square, Square)> {
        if self.viewed_ply == 0 {
            return None;
        }
        self.history
            .get(self.viewed_ply.saturating_sub(1))
            .map(|rec| {
                let from = rec.mv.from().unwrap_or_else(|| rec.mv.to());
                (from, rec.mv.to())
            })
    }

    pub fn is_viewing_latest(&self) -> bool {
        self.viewed_ply == self.history.len()
    }

    pub fn step_history_first(&mut self) {
        self.viewed_ply = 0;
    }

    pub fn step_history_prev(&mut self) {
        self.viewed_ply = self.viewed_ply.saturating_sub(1);
    }

    pub fn step_history_next(&mut self) {
        if self.viewed_ply < self.history.len() {
            self.viewed_ply += 1;
        }
    }

    pub fn step_history_last(&mut self) {
        self.viewed_ply = self.history.len();
    }

    pub fn jump_to_ply(&mut self, ply: usize) {
        self.viewed_ply = ply.min(self.history.len());
    }

    pub fn controller_for(&self, color: Color) -> PlayerController {
        match color {
            Color::White => self.white_controller,
            Color::Black => self.black_controller,
        }
    }

    pub fn is_bot_turn(&self) -> bool {
        self.outcome == GameOutcome::Ongoing
            && self.controller_for(self.current_pos.turn()) == PlayerController::Bot
    }

    pub fn legal_moves_from(&self, sq: Square) -> Vec<Move> {
        if !self.is_viewing_latest() || self.outcome.is_game_over() {
            return Vec::new();
        }
        self.current_pos
            .legal_moves()
            .into_iter()
            .filter(|m| m.from() == Some(sq))
            .collect()
    }

    pub fn resign(&mut self, resigning_color: Color) {
        if !self.outcome.is_game_over() {
            self.outcome = GameOutcome::Resignation {
                winner: resigning_color.other(),
            };
            self.clock.stop();
        }
    }

    pub fn flag_timeout(&mut self, winner: Color) {
        if !self.outcome.is_game_over() {
            self.outcome = GameOutcome::Timeout { winner };
            self.clock.stop();
        }
    }
}
