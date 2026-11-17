use shakmaty::{attacks, Bitboard, Board, Color, Square};

pub const MOBILITY_UNIT_CP: i32 = 4;
pub const CENTER_ATTACK_BONUS_CP: i32 = 6;

const CENTER_SQUARES: [Square; 4] = [Square::D4, Square::E4, Square::D5, Square::E5];

/// Menghitung skor mobilitas perwira dan bonus kontrol pusat untuk satu sisi.
pub fn side_mobility_and_center(board: &Board, color: Color) -> i32 {
    let us = board.by_color(color);
    let occupied = board.occupied();
    let targets = !us;

    let mut mobility_squares = 0i32;
    let mut center_attacks = 0i32;

    let center_bb = Bitboard::from_square(Square::D4)
        | Bitboard::from_square(Square::E4)
        | Bitboard::from_square(Square::D5)
        | Bitboard::from_square(Square::E5);

    // Serangan pion ke pusat
    for sq in us & board.pawns() {
        let atk = attacks::pawn_attacks(color, sq);
        center_attacks += (atk & center_bb).count() as i32;
    }

    // Knights
    for sq in us & board.knights() {
        let atk = attacks::knight_attacks(sq) & targets;
        mobility_squares += atk.count() as i32;
        center_attacks += (atk & center_bb).count() as i32;
    }

    // Bishops
    for sq in us & board.bishops() {
        let atk = attacks::bishop_attacks(sq, occupied) & targets;
        mobility_squares += atk.count() as i32;
        center_attacks += (atk & center_bb).count() as i32;
    }

    // Rooks
    for sq in us & board.rooks() {
        let atk = attacks::rook_attacks(sq, occupied) & targets;
        mobility_squares += atk.count() as i32;
        center_attacks += (atk & center_bb).count() as i32;
    }

    // Queens
    for sq in us & board.queens() {
        let atk = attacks::queen_attacks(sq, occupied) & targets;
        mobility_squares += atk.count() as i32;
        center_attacks += (atk & center_bb).count() as i32;
    }

    let _ = CENTER_SQUARES;
    mobility_squares * MOBILITY_UNIT_CP + center_attacks * CENTER_ATTACK_BONUS_CP
}

/// Evaluasi mobilitas & kontrol pusat (Perspektif Putih: White - Black).
pub fn evaluate_mobility(board: &Board) -> i32 {
    side_mobility_and_center(board, Color::White) - side_mobility_and_center(board, Color::Black)
}
