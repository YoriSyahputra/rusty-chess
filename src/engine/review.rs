use crate::{
    domain::{format_fen, format_san, format_uci, ChessMatchState, Color, Move, Square},
    engine::{
        bot::BotPersona,
        eval::{evaluate_for_white, material::piece_value, EvalWeights},
        search::{search_best_move, TranspositionTable, MATE_THRESHOLD},
    },
};
use shakmaty::{Chess, Position};
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
            MoveClassification::Book => "B",
            MoveClassification::Forced => "»",
            MoveClassification::Inaccuracy => "?!",
            MoveClassification::Miss => "✕",
            MoveClassification::Mistake => "?",
            MoveClassification::Blunder => "??",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            MoveClassification::Brilliant => "Brilliant",
            MoveClassification::Great => "Great",
            MoveClassification::Best => "Best Move",
            MoveClassification::Excellent => "Excellent",
            MoveClassification::Good => "Good",
            MoveClassification::Book => "Book Move",
            MoveClassification::Forced => "Forced",
            MoveClassification::Inaccuracy => "Inaccuracy",
            MoveClassification::Miss => "Missed Win",
            MoveClassification::Mistake => "Mistake",
            MoveClassification::Blunder => "Blunder",
        }
    }

    pub fn rgb(&self) -> [u8; 3] {
        match self {
            MoveClassification::Brilliant => [0x1B, 0xAC, 0xA6],
            MoveClassification::Great => [0x5C, 0x8B, 0xB0],
            MoveClassification::Best => [0x81, 0xB6, 0x4C],
            MoveClassification::Excellent => [0x5B, 0x9B, 0xD5],
            MoveClassification::Good => [0x92, 0x98, 0x94],
            MoveClassification::Book => [0xA8, 0x88, 0x65],
            MoveClassification::Forced => [0x96, 0x94, 0x90],
            MoveClassification::Inaccuracy => [0xF7, 0xC6, 0x31],
            MoveClassification::Miss => [0xE0, 0x4F, 0x44],
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

#[derive(Debug, Clone)]
pub struct LiveMoveEval {
    pub ply: usize,
    pub side: Color,
    pub played_san: String,
    pub played_to: Square,
    pub best_mv: Move,
    pub best_san: String,
    pub eval_after_white_cp: i32,
    pub classification: MoveClassification,
    pub coach_note: String,
}

#[derive(Debug, Clone, Copy)]
pub struct BreakdownRow {
    pub classification: MoveClassification,
    pub label: &'static str,
    pub white_count: u32,
    pub black_count: u32,
}

#[derive(Debug, Clone, Default)]
pub struct GameReviewReport {
    pub entries: Vec<PlyReviewEntry>,
    pub white_accuracy: f32,
    pub black_accuracy: f32,
    pub white_est_elo: u32,
    pub black_est_elo: u32,
}

impl GameReviewReport {
    /// Menghasilkan 9 baris klasifikasi gaya Chess.com (Brilliant -> Missed Win) untuk tabel laporan.
    pub fn chesscom_breakdown(&self) -> [BreakdownRow; 9] {
        let mut counts = [(0u32, 0u32); 9];
        for e in &self.entries {
            let idx = match e.classification {
                MoveClassification::Brilliant => 0,
                MoveClassification::Best | MoveClassification::Great | MoveClassification::Forced => 1,
                MoveClassification::Excellent => 2,
                MoveClassification::Good => 3,
                MoveClassification::Book => 4,
                MoveClassification::Inaccuracy => 5,
                MoveClassification::Mistake => 6,
                MoveClassification::Blunder => 7,
                MoveClassification::Miss => 8,
            };
            match e.side {
                Color::White => counts[idx].0 += 1,
                Color::Black => counts[idx].1 += 1,
            }
        }

        [
            BreakdownRow {
                classification: MoveClassification::Brilliant,
                label: "Brilliant",
                white_count: counts[0].0,
                black_count: counts[0].1,
            },
            BreakdownRow {
                classification: MoveClassification::Best,
                label: "Best Move",
                white_count: counts[1].0,
                black_count: counts[1].1,
            },
            BreakdownRow {
                classification: MoveClassification::Excellent,
                label: "Excellent",
                white_count: counts[2].0,
                black_count: counts[2].1,
            },
            BreakdownRow {
                classification: MoveClassification::Good,
                label: "Good",
                white_count: counts[3].0,
                black_count: counts[3].1,
            },
            BreakdownRow {
                classification: MoveClassification::Book,
                label: "Book Move",
                white_count: counts[4].0,
                black_count: counts[4].1,
            },
            BreakdownRow {
                classification: MoveClassification::Inaccuracy,
                label: "Inaccuracy",
                white_count: counts[5].0,
                black_count: counts[5].1,
            },
            BreakdownRow {
                classification: MoveClassification::Mistake,
                label: "Mistake",
                white_count: counts[6].0,
                black_count: counts[6].1,
            },
            BreakdownRow {
                classification: MoveClassification::Blunder,
                label: "Blunder",
                white_count: counts[7].0,
                black_count: counts[7].1,
            },
            BreakdownRow {
                classification: MoveClassification::Miss,
                label: "Missed Win",
                white_count: counts[8].0,
                black_count: counts[8].1,
            },
        ]
    }

    /// Menentukan label mood pertandingan (CLEAN / DOMINANT / SHARP / ROUGH) dan ringkasan coach.
    pub fn game_mood_verdict(&self, human_won: Option<bool>) -> (&'static str, &'static str) {
        let total_blunders = self
            .entries
            .iter()
            .filter(|e| {
                matches!(
                    e.classification,
                    MoveClassification::Blunder | MoveClassification::Miss
                )
            })
            .count();
        let avg_acc = (self.white_accuracy + self.black_accuracy) * 0.5;

        let mood = if total_blunders >= 4 {
            "ROUGH"
        } else if avg_acc >= 88.0 && total_blunders == 0 {
            "DOMINANT"
        } else if total_blunders <= 1 {
            "CLEAN"
        } else {
            "SHARP"
        };

        let summary = match (human_won, mood) {
            (Some(true), "DOMINANT" | "CLEAN") => "Clinical victory. You controlled the board from start to finish.",
            (Some(true), _) => "Nice win. You punished key mistakes and kept the pressure on.",
            (Some(false), "ROUGH") => "A wild tactical fight! Drill the blunders in your Blunder Deck.",
            (Some(false), _) => "A tough battle. Inspect the turning point with the coach above.",
            (None, _) => "Hard-fought draw with balanced chances for both sides.",
        };

        (mood, summary)
    }
}

/// Evaluasi langkah secara real-time (instant live analysis, <30ms) langsung setelah langkah dimainkan!
/// Menggunakan `root_candidates` dari search dangkal sehingga blunder menggantung bidak langsung terdeteksi.
pub fn evaluate_move_live(
    pos_before: &Chess,
    played_mv: &Move,
    ply: usize,
    is_book_opening: bool,
) -> LiveMoveEval {
    let side = pos_before.turn();
    let legal_count = pos_before.legal_moves().len();
    let played_san = format_san(pos_before, played_mv);
    let weights = EvalWeights::default();

    let mut analyzer = BotPersona::magnus();
    analyzer.max_depth = 2;
    analyzer.time_budget_ms = 75;
    analyzer.softmax_temp_cp = 0.0;
    analyzer.noise_cp = 0;

    let mut tt = TranspositionTable::new();
    let abort = AtomicBool::new(false);

    let search_res = search_best_move(pos_before, &analyzer, &[], &mut tt, &abort, |_, _, _, _| {});

    let mut next_pos = pos_before.clone();
    next_pos.play_unchecked(played_mv);

    let (best_mv, best_eval_white, second_best_gap_w) = match search_res {
        Some(res) => {
            let gap = if res.root_candidates.len() >= 2 {
                let w0 = win_probability_from_cp(res.root_candidates[0].1);
                let w1 = win_probability_from_cp(res.root_candidates[1].1);
                (w0 - w1).max(0.0)
            } else {
                0.0
            };
            (res.best_move, res.eval_cp_white, gap)
        }
        None => {
            let ev = evaluate_for_white(&next_pos, &weights);
            (played_mv.clone(), ev, 0.0)
        }
    };

    // Hitung skor taktis persis dari `next_pos` dengan search cepat (memanfaatkan TT hangat)
    // sehingga langkah menggantung bidak langsung terdeteksi sebagai Blunder (??)!
    let played_eval_white = if next_pos.is_checkmate() {
        match side {
            Color::White => MATE_THRESHOLD + 500,
            Color::Black => -(MATE_THRESHOLD + 500),
        }
    } else if next_pos.is_stalemate() || next_pos.is_insufficient_material() {
        0
    } else if played_mv == &best_mv {
        best_eval_white
    } else {
        let mut reply_analyzer = analyzer.clone();
        reply_analyzer.max_depth = 2;
        reply_analyzer.time_budget_ms = 35;
        search_best_move(&next_pos, &reply_analyzer, &[], &mut tt, &abort, |_, _, _, _| {})
            .map(|r| r.eval_cp_white)
            .unwrap_or_else(|| evaluate_for_white(&next_pos, &weights))
    };

    let best_san = format_san(pos_before, &best_mv);
    let w_before_white = win_probability_from_cp(best_eval_white);
    let w_after_white = win_probability_from_cp(played_eval_white);

    let (w_before_mover, w_after_mover, cp_loss_mover) = match side {
        Color::White => (
            w_before_white,
            w_after_white,
            (best_eval_white - played_eval_white).max(0),
        ),
        Color::Black => (
            1.0 - w_before_white,
            1.0 - w_after_white,
            (played_eval_white - best_eval_white).max(0),
        ),
    };

    let delta_w_loss = (w_before_mover - w_after_mover).clamp(0.0, 1.0);
    let is_sacrifice = piece_value(played_mv.role()) >= 320
        && next_pos
            .board()
            .attacks_to(
                played_mv.to(),
                side.other(),
                next_pos.board().occupied(),
            )
            .any();

    let missed_mate = (side == Color::White
        && best_eval_white > MATE_THRESHOLD
        && played_eval_white <= MATE_THRESHOLD)
        || (side == Color::Black
            && best_eval_white < -MATE_THRESHOLD
            && played_eval_white >= -MATE_THRESHOLD);

    let mut classification = classify_move(
        if is_book_opening { ply } else { 99 },
        legal_count,
        played_mv,
        &best_mv,
        cp_loss_mover,
        w_before_mover,
        w_after_mover,
        delta_w_loss,
        second_best_gap_w,
        is_sacrifice,
        missed_mate,
    );

    if is_book_opening && ply <= 12 && delta_w_loss <= 0.04 {
        classification = MoveClassification::Book;
    }

    let coach_note = match classification {
        MoveClassification::Brilliant => format!("{played_san}!! Brilliant tactical sacrifice!"),
        MoveClassification::Great => format!("{played_san}! Great only-move find!"),
        MoveClassification::Best => format!("{played_san} is the Best Move!"),
        MoveClassification::Excellent => format!("{played_san} is an Excellent continuation."),
        MoveClassification::Good => format!("{played_san} is a solid Good move."),
        MoveClassification::Book => format!("{played_san} is a standard Book Move."),
        MoveClassification::Forced => format!("{played_san} was the only legal move."),
        MoveClassification::Inaccuracy => {
            format!("Inaccuracy ({played_san}?!). Stronger was {best_san}.")
        }
        MoveClassification::Mistake => {
            format!("Mistake ({played_san}?). Better was {best_san}.")
        }
        MoveClassification::Blunder => {
            format!("Blunder ({played_san}??)! Best move was {best_san}.")
        }
        MoveClassification::Miss => {
            format!("Missed Win ({played_san}✕)! {best_san} was decisive.")
        }
    };

    LiveMoveEval {
        ply,
        side,
        played_san,
        played_to: played_mv.to(),
        best_mv,
        best_san,
        eval_after_white_cp: played_eval_white,
        classification,
        coach_note,
    }
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

        // Evaluasi posisi setelah langkah yang dimainkan dengan depth-2 search cepat (TT hangat)
        let mut next_pos = pos.clone();
        next_pos.play_unchecked(&rec.mv);
        let after_eval_white = if next_pos.is_checkmate() {
            match side {
                Color::White => MATE_THRESHOLD + 500,
                Color::Black => -(MATE_THRESHOLD + 500),
            }
        } else if next_pos.is_stalemate() || next_pos.is_insufficient_material() {
            0
        } else if rec.mv == best_mv {
            best_eval_white
        } else {
            let mut reply_analyzer = analyzer.clone();
            reply_analyzer.max_depth = 2;
            reply_analyzer.time_budget_ms = 30;
            search_best_move(&next_pos, &reply_analyzer, &[], &mut tt, &abort, |_, _, _, _| {})
                .map(|r| r.eval_cp_white)
                .unwrap_or_else(|| evaluate_for_white(&next_pos, &weights))
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
