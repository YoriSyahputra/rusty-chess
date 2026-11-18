use crate::{
    domain::{format_fen, format_san, format_uci, ChessMatchState, Color, Move},
    engine::{
        bot::BotPersona,
        eval::{evaluate_for_white, material::piece_value, EvalWeights},
        search::{search_best_move, TranspositionTable, MATE_THRESHOLD},
    },
};
use shakmaty::Position;
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MoveClassification {
    Brilliant,
    Great,
    Best,
    Excellent,
    Good,
    Book,
    Forced,
    Inaccuracy,
    Miss,
    Mistake,
    Blunder,
}

impl MoveClassification {
    pub fn symbol(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "!!",
            MoveClassification::Great => "!",
            MoveClassification::Best => "★",
            MoveClassification::Excellent => "✓",
            MoveClassification::Good => "○",
            MoveClassification::Book => "📖",
            MoveClassification::Forced => "□",
            MoveClassification::Inaccuracy => "?!",
            MoveClassification::Miss => "✗",
            MoveClassification::Mistake => "?",
            MoveClassification::Blunder => "??",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "Brilliant",
            MoveClassification::Great => "Great",
            MoveClassification::Best => "Best",
            MoveClassification::Excellent => "Excellent",
            MoveClassification::Good => "Good",
            MoveClassification::Book => "Book",
            MoveClassification::Forced => "Forced",
            MoveClassification::Inaccuracy => "Inaccuracy",
            MoveClassification::Miss => "Miss",
            MoveClassification::Mistake => "Mistake",
            MoveClassification::Blunder => "Blunder",
        }
    }

    pub fn rgb(&self) -> [u8; 3] {
        match self {
            MoveClassification::Brilliant => [0x1B, 0xAC, 0xA6],
            MoveClassification::Great => [0x5C, 0x8B, 0xB0],
            MoveClassification::Best => [0x81, 0xB6, 0x4C],
            MoveClassification::Excellent => [0x95, 0xC9, 0x5D],
            MoveClassification::Good => [0x96, 0xAF, 0x8B],
            MoveClassification::Book => [0xA8, 0x88, 0x65],
            MoveClassification::Forced => [0x96, 0x94, 0x90],
            MoveClassification::Inaccuracy => [0xF7, 0xC6, 0x31],
            MoveClassification::Miss => [0xFF, 0x77, 0x69],
            MoveClassification::Mistake => [0xE6, 0x8A, 0x2E],
            MoveClassification::Blunder => [0xCA, 0x34, 0x31],
        }
    }

    pub fn is_mistake_or_worse(&self) -> bool {
        matches!(
            self,
            MoveClassification::Inaccuracy
                | MoveClassification::Miss
                | MoveClassification::Mistake
                | MoveClassification::Blunder
        )
    }
}

#[derive(Debug, Clone)]
pub struct PlyReviewEntry {
    pub ply: usize,
    pub side: Color,
    pub played_mv: Move,
    pub played_san: String,
    pub best_mv: Move,
    pub best_san: String,
    pub best_uci: String,
    pub eval_before_white_cp: i32,
    pub eval_after_white_cp: i32,
    pub win_prob_white: f32,
    pub delta_w_loss: f32,
    pub classification: MoveClassification,
    pub fen_before: String,
}

#[derive(Debug, Clone, Default)]
pub struct GameReviewReport {
    pub entries: Vec<PlyReviewEntry>,
    pub white_accuracy: f32,
    pub black_accuracy: f32,
    pub white_est_elo: u32,
    pub black_est_elo: u32,
}

/// Menghitung probabilitas menang Putih dari skor centipawn sesuai rumus NEW_DESIGN.md Section 8:
/// `W(cp) = 1.0 / (1.0 + 10^(-cp / 400.0))`
#[inline]
pub fn win_probability_from_cp(cp: i32) -> f32 {
    if cp > MATE_THRESHOLD {
        return 1.0;
    }
    if cp < -MATE_THRESHOLD {
        return 0.0;
    }
    let exp = (-cp as f32 / 400.0).clamp(-12.0, 12.0);
    1.0 / (1.0 + 10.0_f32.powf(exp))
}

/// Menghitung Rusty Chess Local Accuracy % (0.0..=100.0) dari rata-rata persentase kehilangan poin harapan:
/// `Accuracy = clamp(100.0 * exp(-0.065 * mean_delta_w_pct), 0.0, 100.0)`
#[inline]
pub fn accuracy_from_mean_loss_pct(mean_delta_w_pct: f32) -> f32 {
    let acc = 100.0 * (-0.065 * mean_delta_w_pct.max(0.0)).exp();
    acc.clamp(0.0, 100.0)
}

/// Menganalisis seluruh langkah pada `ChessMatchState` untuk menghasilkan `GameReviewReport`.
pub fn analyze_match_review(match_state: &ChessMatchState, review_depth: u8) -> GameReviewReport {
    if match_state.history.is_empty() {
        return GameReviewReport {
            entries: Vec::new(),
            white_accuracy: 100.0,
            black_accuracy: 100.0,
            white_est_elo: 1500,
            black_est_elo: 1500,
        };
    }

    let mut tt = TranspositionTable::new();
    let abort = AtomicBool::new(false);
    let mut analyzer = BotPersona::magnus();
    analyzer.max_depth = review_depth.clamp(2, 5);
    analyzer.time_budget_ms = 90;
    analyzer.softmax_temp_cp = 0.0;
    analyzer.noise_cp = 0;

    let weights = EvalWeights::default();
    let mut pos = match_state.initial_pos.clone();
    let mut prev_eval_white = evaluate_for_white(&pos, &weights);

    let mut entries = Vec::with_capacity(match_state.history.len());
    let mut white_losses_pct = Vec::new();
    let mut black_losses_pct = Vec::new();

    for rec in &match_state.history {
        let side = pos.turn();
        let legal_moves = pos.legal_moves();
        let legal_count = legal_moves.len();
        let fen_before = format_fen(&pos);

        // Cari langkah terbaik mesin dari posisi sebelum langkah dimainkan
        let search_res =
            search_best_move(&pos, &analyzer, &[], &mut tt, &abort, |_, _, _, _| {});

        let (best_mv, best_eval_white, second_best_gap_w) = match search_res {
            Some(res) => {
                let gap = if res.root_candidates.len() >= 2 {
                    let s0 = res.root_candidates[0].1;
                    let s1 = res.root_candidates[1].1;
                    let w0 = win_probability_from_cp(s0);
                    let w1 = win_probability_from_cp(s1);
                    (w0 - w1).max(0.0)
                } else {
                    0.0
                };
                (res.best_move, res.eval_cp_white, gap)
            }
            None => (rec.mv.clone(), prev_eval_white, 0.0),
        };

        let best_san = format_san(&pos, &best_mv);
        let best_uci = format_uci(&best_mv);

        // Evaluasi posisi setelah langkah yang dimainkan
        let mut next_pos = pos.clone();
        next_pos.play_unchecked(&rec.mv);
        let after_eval_white = if next_pos.is_checkmate() {
            match side {
                Color::White => MATE_THRESHOLD + 500,
                Color::Black => -(MATE_THRESHOLD + 500),
            }
        } else {
            evaluate_for_white(&next_pos, &weights)
        };

        let w_before_white = win_probability_from_cp(best_eval_white);
        let w_after_white = win_probability_from_cp(after_eval_white);

        let (w_before_mover, w_after_mover, cp_loss_mover) = match side {
            Color::White => (
                w_before_white,
                w_after_white,
                (best_eval_white - after_eval_white).max(0),
            ),
            Color::Black => (
                1.0 - w_before_white,
                1.0 - w_after_white,
                (after_eval_white - best_eval_white).max(0),
            ),
        };

        let delta_w_loss = (w_before_mover - w_after_mover).clamp(0.0, 1.0);

        // Deteksi pengorbanan material untuk klasifikasi Brilliant (!!)
        let is_sacrifice = piece_value(rec.mv.role()) >= 320
            && next_pos.board().attacks_to(
                rec.mv.to(),
                side.other(),
                next_pos.board().occupied(),
            ).any();

        let missed_mate = (side == Color::White && best_eval_white > MATE_THRESHOLD && after_eval_white <= MATE_THRESHOLD)
            || (side == Color::Black && best_eval_white < -MATE_THRESHOLD && after_eval_white >= -MATE_THRESHOLD);

        let classification = classify_move(
            rec.ply,
            legal_count,
            &rec.mv,
            &best_mv,
            cp_loss_mover,
            w_before_mover,
            w_after_mover,
            delta_w_loss,
            second_best_gap_w,
            is_sacrifice,
            missed_mate,
        );

        match side {
            Color::White => white_losses_pct.push(delta_w_loss * 100.0),
            Color::Black => black_losses_pct.push(delta_w_loss * 100.0),
        }

        entries.push(PlyReviewEntry {
            ply: rec.ply,
            side,
            played_mv: rec.mv.clone(),
            played_san: rec.san.clone(),
            best_mv,
            best_san,
            best_uci,
            eval_before_white_cp: prev_eval_white,
            eval_after_white_cp: after_eval_white,
            win_prob_white: w_after_white,
            delta_w_loss,
            classification,
            fen_before,
        });

        pos = next_pos;
        prev_eval_white = after_eval_white;
    }

    let mean_w = if white_losses_pct.is_empty() {
        0.0
    } else {
        white_losses_pct.iter().sum::<f32>() / (white_losses_pct.len() as f32)
    };
    let mean_b = if black_losses_pct.is_empty() {
        0.0
    } else {
        black_losses_pct.iter().sum::<f32>() / (black_losses_pct.len() as f32)
    };

    let white_accuracy = accuracy_from_mean_loss_pct(mean_w);
    let black_accuracy = accuracy_from_mean_loss_pct(mean_b);

    let white_est_elo = (600.0 + white_accuracy * 21.0).round().clamp(250.0, 3100.0) as u32;
    let black_est_elo = (600.0 + black_accuracy * 21.0).round().clamp(250.0, 3100.0) as u32;

    GameReviewReport {
        entries,
        white_accuracy,
        black_accuracy,
        white_est_elo,
        black_est_elo,
    }
}

#[allow(clippy::too_many_arguments)]
fn classify_move(
    ply: usize,
    legal_count: usize,
    played_mv: &Move,
    best_mv: &Move,
    cp_loss: i32,
    w_before: f32,
    w_after: f32,
    delta_w: f32,
    second_best_gap_w: f32,
    is_sacrifice: bool,
    missed_mate: bool,
) -> MoveClassification {
    if legal_count <= 1 {
        return MoveClassification::Forced;
    }
    if ply <= 16 && delta_w <= 0.03 {
        return MoveClassification::Book;
    }
    if is_sacrifice && w_before < 0.85 && delta_w <= 0.02 && w_after >= 0.45 && played_mv == best_mv
    {
        return MoveClassification::Brilliant;
    }
    if second_best_gap_w >= 0.18 && delta_w <= 0.02 {
        return MoveClassification::Great;
    }
    if played_mv == best_mv || cp_loss <= 10 {
        return MoveClassification::Best;
    }
    if delta_w <= 0.02 {
        return MoveClassification::Excellent;
    }
    if delta_w < 0.05 {
        return MoveClassification::Good;
    }
    if missed_mate || (w_before >= 0.72 && delta_w >= 0.18 && w_after >= 0.42) {
        return MoveClassification::Miss;
    }
    if delta_w < 0.10 {
        return MoveClassification::Inaccuracy;
    }
    if delta_w < 0.20 {
        return MoveClassification::Mistake;
    }
    MoveClassification::Blunder
}
