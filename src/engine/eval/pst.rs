use shakmaty::{Bitboard, Board, Color, File, Rank, Role, Square};

// Seluruh tabel PST ditulis secara visual dari Baris ke-8 (indeks 0..8) hingga Baris ke-1 (indeks 56..64)
// dari perspektif Putih. Karena `Square::A1 == 0` dan `Square::A8 == 56` di `shakmaty`,
// indeks tabel untuk Putih adalah `(sq as usize) ^ 56`, dan untuk Hitam adalah `sq as usize`.
// Dengan demikian `white_idx ^ 56 == black_idx`, menjamin simetri vertikal 100%.

#[rustfmt::skip]
pub const PAWN_MG_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     80,  85,  85,  90,  90,  85,  85,  80,
     50,  55,  60,  65,  65,  60,  55,  50,
     20,  20,  25,  30,  30,  25,  20,  20,
      5,   5,  15,  25,  25,  15,   5,   5,
      0,   0,   5,   5,   5,   5,   0,   0,
      5,  10,  10, -15, -15,  10,  10,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub const PAWN_EG_PST: [i32; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
     90,  90,  90,  95,  95,  90,  90,  90,
     60,  60,  65,  70,  70,  65,  60,  60,
     35,  35,  40,  45,  45,  40,  35,  35,
     15,  15,  20,  25,  25,  20,  15,  15,
      5,   5,  10,  10,  10,  10,   5,   5,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub const KNIGHT_MG_PST: [i32; 64] = [
    -50, -40, -30, -30, -30, -30, -40, -50,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -30,   5,  15,  20,  20,  15,   5, -30,
    -30,   5,  20,  25,  25,  20,   5, -30,
    -30,   5,  15,  22,  22,  15,   5, -30,
    -30,   5,  15,  15,  15,  15,   5, -30,
    -40, -20,   0,   5,   5,   0, -20, -40,
    -50, -40, -30, -30, -30, -30, -40, -50,
];

#[rustfmt::skip]
pub const KNIGHT_EG_PST: [i32; 64] = [
    -45, -35, -25, -25, -25, -25, -35, -45,
    -35, -15,   0,   5,   5,   0, -15, -35,
    -25,   5,  15,  18,  18,  15,   5, -25,
    -25,   5,  18,  25,  25,  18,   5, -25,
    -25,   5,  18,  25,  25,  18,   5, -25,
    -25,   5,  15,  18,  18,  15,   5, -25,
    -35, -15,   0,   5,   5,   0, -15, -35,
    -45, -35, -25, -25, -25, -25, -35, -45,
];

#[rustfmt::skip]
pub const BISHOP_MG_PST: [i32; 64] = [
    -20, -10, -10, -10, -10, -10, -10, -20,
    -10,   5,   0,   0,   0,   0,   5, -10,
    -10,  10,  10,  15,  15,  10,  10, -10,
    -10,   5,  15,  20,  20,  15,   5, -10,
    -10,   5,  12,  20,  20,  12,   5, -10,
    -10,  12,  12,  12,  12,  12,  12, -10,
    -10,  15,   0,   5,   5,   0,  15, -10,
    -20, -10, -10, -10, -10, -10, -10, -20,
];

#[rustfmt::skip]
pub const BISHOP_EG_PST: [i32; 64] = [
    -15, -10, -10, -10, -10, -10, -10, -15,
    -10,   5,   5,   5,   5,   5,   5, -10,
    -10,   5,  12,  15,  15,  12,   5, -10,
    -10,   5,  15,  20,  20,  15,   5, -10,
    -10,   5,  15,  20,  20,  15,   5, -10,
    -10,   5,  12,  15,  15,  12,   5, -10,
    -10,   5,   5,   5,   5,   5,   5, -10,
    -15, -10, -10, -10, -10, -10, -10, -15,
];

#[rustfmt::skip]
pub const ROOK_MG_PST: [i32; 64] = [
      5,   5,   5,  10,  10,   5,   5,   5,
     25,  25,  25,  25,  25,  25,  25,  25,
     -5,   0,   0,   5,   5,   0,   0,  -5,
     -5,   0,   0,   5,   5,   0,   0,  -5,
     -5,   0,   0,   5,   5,   0,   0,  -5,
     -5,   0,   0,   5,   5,   0,   0,  -5,
     -5,   0,   0,   5,   5,   0,   0,  -5,
      0,   0,   5,  12,  12,   5,   0,   0,
];

#[rustfmt::skip]
pub const ROOK_EG_PST: [i32; 64] = [
     10,  10,  10,  15,  15,  10,  10,  10,
     25,  25,  25,  25,  25,  25,  25,  25,
      5,   5,   5,  10,  10,   5,   5,   5,
      0,   0,   5,   5,   5,   5,   0,   0,
      0,   0,   0,   5,   5,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   0,   0,   0,   0,   0,   0,
      0,   0,   5,   5,   5,   5,   0,   0,
];

#[rustfmt::skip]
pub const QUEEN_MG_PST: [i32; 64] = [
    -20, -10, -10,  -5,  -5, -10, -10, -20,
    -10,   0,   0,   0,   0,   0,   0, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
     -5,   0,   5,  10,  10,   5,   0,  -5,
     -5,   0,   5,  10,  10,   5,   0,  -5,
    -10,   5,   5,   5,   5,   5,   0, -10,
    -10,   0,   5,   0,   0,   0,   0, -10,
    -20, -10, -10,  -5,  -5, -10, -10, -20,
];

#[rustfmt::skip]
pub const QUEEN_EG_PST: [i32; 64] = [
    -15, -10, -10,  -5,  -5, -10, -10, -15,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -10,   5,  10,  12,  12,  10,   5, -10,
     -5,   5,  12,  18,  18,  12,   5,  -5,
     -5,   5,  12,  18,  18,  12,   5,  -5,
    -10,   5,  10,  12,  12,  10,   5, -10,
    -10,   0,   5,   5,   5,   5,   0, -10,
    -15, -10, -10,  -5,  -5, -10, -10, -15,
];

#[rustfmt::skip]
pub const KING_MG_PST: [i32; 64] = [
    -40, -45, -45, -50, -50, -45, -45, -40,
    -40, -45, -45, -50, -50, -45, -45, -40,
    -40, -45, -45, -50, -50, -45, -45, -40,
    -40, -45, -45, -50, -50, -45, -45, -40,
    -35, -40, -40, -45, -45, -40, -40, -35,
    -20, -25, -25, -35, -35, -25, -25, -20,
     15,  15,  -5, -20, -20,  -5,  15,  15,
     20,  30,  25, -15, -10, -10,  30,  20,
];

#[rustfmt::skip]
pub const KING_EG_PST: [i32; 64] = [
    -30, -20, -15, -10, -10, -15, -20, -30,
    -20,  -5,   5,  10,  10,   5,  -5, -20,
    -15,   5,  20,  25,  25,  20,   5, -15,
    -10,  10,  25,  30,  30,  25,  10, -10,
    -10,  10,  25,  30,  30,  25,  10, -10,
    -15,   5,  20,  25,  25,  20,   5, -15,
    -20,  -5,   5,  10,  10,   5,  -5, -20,
    -30, -20, -15, -10, -10, -15, -20, -30,
];

#[inline]
pub fn pst_table_index(sq: Square, color: Color) -> usize {
    let raw = sq as usize;
    match color {
        Color::White => raw ^ 56,
        Color::Black => raw,
    }
}

/// Menghitung `(mg_score, eg_score)` PST dari seluruh bidak untuk satu sisi.
pub fn side_pst_scores(board: &Board, color: Color) -> (i32, i32) {
    let mut mg = 0i32;
    let mut eg = 0i32;
    let us = board.by_color(color);

    for sq in us {
        if let Some(role) = board.role_at(sq) {
            let idx = pst_table_index(sq, color);
            match role {
                Role::Pawn => {
                    mg += PAWN_MG_PST[idx];
                    eg += PAWN_EG_PST[idx];
                }
                Role::Knight => {
                    mg += KNIGHT_MG_PST[idx];
                    eg += KNIGHT_EG_PST[idx];
                }
                Role::Bishop => {
                    mg += BISHOP_MG_PST[idx];
                    eg += BISHOP_EG_PST[idx];
                }
                Role::Rook => {
                    mg += ROOK_MG_PST[idx];
                    eg += ROOK_EG_PST[idx];
                }
                Role::Queen => {
                    mg += QUEEN_MG_PST[idx];
                    eg += QUEEN_EG_PST[idx];
                }
                Role::King => {
                    mg += KING_MG_PST[idx];
                    eg += KING_EG_PST[idx];
                }
            }
        }
    }
    (mg, eg)
}

/// Evaluasi Tapered PST `(mg_score * phase + eg_score * (24 - phase)) / 24`
/// dengan pemisahan komponen Midgame dan Endgame.
pub fn evaluate_tapered_pst(board: &Board, phase: i32, endgame_scale: f32) -> i32 {
    let (w_mg, w_eg) = side_pst_scores(board, Color::White);
    let (b_mg, b_eg) = side_pst_scores(board, Color::Black);

    let mg_diff = w_mg - b_mg;
    let eg_diff = ((w_eg - b_eg) as f32 * endgame_scale).round() as i32;

    (mg_diff * phase + eg_diff * (24 - phase)) / 24
}

/// Evaluasi struktur pion:
/// - Passed Pawn: +25 cp s.d. +120 cp tergantung kedekatan promosi
/// - Doubled Pawn: -15 cp
/// - Isolated Pawn: -15 cp
pub fn evaluate_pawn_structure(board: &Board) -> i32 {
    side_pawn_structure(board, Color::White) - side_pawn_structure(board, Color::Black)
}

fn side_pawn_structure(board: &Board, color: Color) -> i32 {
    const PASSED_PAWN_BONUS_BY_RANK: [i32; 8] = [0, 10, 25, 40, 65, 95, 120, 0];
    const DOUBLED_PAWN_PENALTY: i32 = 15;
    const ISOLATED_PAWN_PENALTY: i32 = 15;

    let our_pawns = board.by_color(color) & board.pawns();
    let enemy_pawns = board.by_color(color.other()) & board.pawns();

    let mut score = 0i32;

    for file_idx in 0..8usize {
        let file = File::new(file_idx as u32);
        let file_bb = Bitboard::from_file(file);
        let pawns_on_file = our_pawns & file_bb;
        let count = pawns_on_file.count() as i32;

        if count > 1 {
            score -= (count - 1) * DOUBLED_PAWN_PENALTY;
        }

        if count > 0 {
            let mut adj_mask = Bitboard(0);
            if file_idx > 0 {
                adj_mask |= Bitboard::from_file(File::new((file_idx - 1) as u32));
            }
            if file_idx < 7 {
                adj_mask |= Bitboard::from_file(File::new((file_idx + 1) as u32));
            }

            if (our_pawns & adj_mask).is_empty() {
                score -= count * ISOLATED_PAWN_PENALTY;
            }
        }
    }

    for sq in our_pawns {
        let file_idx = sq.file() as usize;
        let rank_idx = sq.rank() as usize;

        let mut front_files = Bitboard::from_file(sq.file());
        if file_idx > 0 {
            front_files |= Bitboard::from_file(File::new((file_idx - 1) as u32));
        }
        if file_idx < 7 {
            front_files |= Bitboard::from_file(File::new((file_idx + 1) as u32));
        }

        let mut forward_ranks = Bitboard(0);
        match color {
            Color::White => {
                for r in (rank_idx + 1)..8 {
                    forward_ranks |= Bitboard::from_rank(Rank::new(r as u32));
                }
            }
            Color::Black => {
                for r in 0..rank_idx {
                    forward_ranks |= Bitboard::from_rank(Rank::new(r as u32));
                }
            }
        }

        let front_span = front_files & forward_ranks;
        if (enemy_pawns & front_span).is_empty() {
            let relative_rank = match color {
                Color::White => rank_idx,
                Color::Black => 7 - rank_idx,
            };
            score += PASSED_PAWN_BONUS_BY_RANK[relative_rank];
        }
    }

    score
}
