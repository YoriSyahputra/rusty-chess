use shakmaty::{Board, Color, Role};

pub const PAWN_VALUE: i32 = 100;
pub const KNIGHT_VALUE: i32 = 320;
pub const BISHOP_VALUE: i32 = 330;
pub const ROOK_VALUE: i32 = 500;
pub const QUEEN_VALUE: i32 = 900;
pub const KING_VALUE: i32 = 20_000;
pub const BISHOP_PAIR_BONUS: i32 = 35;

const LIGHT_SQUARES_MASK: u64 = 0x55AA_55AA_55AA_55AA;
const DARK_SQUARES_MASK: u64 = 0xAA55_AA55_AA55_AA55;

#[inline]
pub fn piece_value(role: Role) -> i32 {
    match role {
        Role::Pawn => PAWN_VALUE,
        Role::Knight => KNIGHT_VALUE,
        Role::Bishop => BISHOP_VALUE,
        Role::Rook => ROOK_VALUE,
        Role::Queen => QUEEN_VALUE,
        Role::King => KING_VALUE,
    }
}

/// Menghitung skor material satu sisi beserta Bishop Pair Bonus (+35 cp).
pub fn side_material_score(board: &Board, color: Color) -> i32 {
    let us = board.by_color(color);
    let pawns = (us & board.pawns()).count() as i32;
    let knights = (us & board.knights()).count() as i32;
    let bishops_bb = us & board.bishops();
    let bishops = bishops_bb.count() as i32;
    let rooks = (us & board.rooks()).count() as i32;
    let queens = (us & board.queens()).count() as i32;

    let mut score = pawns * PAWN_VALUE
        + knights * KNIGHT_VALUE
        + bishops * BISHOP_VALUE
        + rooks * ROOK_VALUE
        + queens * QUEEN_VALUE;

    // Bishop Pair Bonus: +35 cp jika masih memiliki gajah petak terang dan gajah petak gelap
    let raw_bishops = bishops_bb.0;
    if (raw_bishops & LIGHT_SQUARES_MASK) != 0 && (raw_bishops & DARK_SQUARES_MASK) != 0 {
        score += BISHOP_PAIR_BONUS;
    }

    score
}

/// Menghitung selisih material (Perspektif Putih: White - Black).
pub fn evaluate_material(board: &Board) -> i32 {
    side_material_score(board, Color::White) - side_material_score(board, Color::Black)
}

/// Menghitung `game_phase` dalam rentang `0..=24` (24 = pembukaan penuh, 0 = endgame pion murni).
pub fn compute_game_phase(board: &Board) -> i32 {
    let knights = board.knights().count() as i32;
    let bishops = board.bishops().count() as i32;
    let rooks = board.rooks().count() as i32;
    let queens = board.queens().count() as i32;

    let phase = knights + bishops + rooks * 2 + queens * 4;
    phase.clamp(0, 24)
}
