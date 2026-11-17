use crate::engine::eval::material::piece_value;
use shakmaty::{Chess, Color, Move, Position, Role};

pub const MAX_PLY: usize = 64;

pub const SCORE_TT_MOVE: i32 = 1_000_000;
pub const SCORE_PROMOTION: i32 = 900_000;
pub const SCORE_CAPTURE_BASE: i32 = 100_000;
pub const SCORE_KILLER_1: i32 = 80_000;
pub const SCORE_KILLER_2: i32 = 75_000;
pub const MAX_HISTORY_SCORE: i32 = 70_000;

pub struct SearchHeuristics {
    pub killers: [[Option<Move>; 2]; MAX_PLY],
    pub history: [[[i32; 64]; 64]; 2],
}

impl SearchHeuristics {
    pub fn new() -> Self {
        Self {
            killers: std::array::from_fn(|_| [None, None]),
            history: [[[0; 64]; 64]; 2],
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.killers {
            *slot = [None, None];
        }
        self.history = [[[0; 64]; 64]; 2];
    }

    #[inline]
    pub fn record_killer(&mut self, ply: usize, mv: Move) {
        if ply >= MAX_PLY || mv.is_capture() || mv.is_promotion() {
            return;
        }
        if self.killers[ply][0].as_ref() != Some(&mv) {
            self.killers[ply][1] = self.killers[ply][0].take();
            self.killers[ply][0] = Some(mv);
        }
    }

    #[inline]
    pub fn record_history(&mut self, color: Color, mv: &Move, depth: u8) {
        if mv.is_capture() || mv.is_promotion() {
            return;
        }
        if let Some(from) = mv.from() {
            let c_idx = match color {
                Color::White => 0,
                Color::Black => 1,
            };
            let bonus = (depth as i32) * (depth as i32);
            let entry = &mut self.history[c_idx][from as usize][mv.to() as usize];
            *entry = (*entry + bonus).min(MAX_HISTORY_SCORE);
        }
    }

    #[inline]
    pub fn score_move(
        &self,
        pos: &Chess,
        mv: &Move,
        tt_move: Option<&Move>,
        ply: usize,
    ) -> i32 {
        // 1. Hash Move (TT Move)
        if tt_move == Some(mv) {
            return SCORE_TT_MOVE;
        }

        // 2. Promotions
        if let Some(promo_role) = mv.promotion() {
            return SCORE_PROMOTION + piece_value(promo_role);
        }

        // 3. Captures (MVV-LVA: 100_000 + victim_val * 10 - attacker_val)
        if mv.is_capture() {
            let victim_role = match mv {
                Move::Normal { capture, .. } => capture.unwrap_or(Role::Pawn),
                Move::EnPassant { .. } => Role::Pawn,
                _ => Role::Pawn,
            };
            let attacker_role = mv.role();
            let victim_val = piece_value(victim_role);
            let attacker_val = piece_value(attacker_role);
            return SCORE_CAPTURE_BASE + (victim_val * 10) - (attacker_val / 10);
        }

        // 4. Killer Moves
        if ply < MAX_PLY {
            if self.killers[ply][0].as_ref() == Some(mv) {
                return SCORE_KILLER_1;
            }
            if self.killers[ply][1].as_ref() == Some(mv) {
                return SCORE_KILLER_2;
            }
        }

        // 5. History Heuristic (0..70_000)
        if let Some(from) = mv.from() {
            let c_idx = match pos.turn() {
                Color::White => 0,
                Color::Black => 1,
            };
            return self.history[c_idx][from as usize][mv.to() as usize];
        }

        // 6. Quiet Moves
        0
    }

    pub fn order_moves(
        &self,
        pos: &Chess,
        moves: &mut [Move],
        tt_move: Option<&Move>,
        ply: usize,
    ) {
        moves.sort_by_cached_key(|mv| -self.score_move(pos, mv, tt_move, ply));
    }
}

impl Default for SearchHeuristics {
    fn default() -> Self {
        Self::new()
    }
}
