use shakmaty::{attacks, Bitboard, Board, Color, File, Rank};

pub const UNCASTLED_EXPOSED_PENALTY_CP: i32 = 20;
pub const KING_ZONE_ATTACK_BONUS_CP: i32 = 12;

/// Mengevaluasi keamanan Raja milik `color` dan agresivitas serangan `color` terhadap Raja lawan.
pub fn side_king_safety_and_attack(board: &Board, color: Color) -> i32 {
    let mut score = 0i32;
    let us = board.by_color(color);
    let them = board.by_color(color.other());
    let occupied = board.occupied();

    let our_pawns = us & board.pawns();
    let enemy_queens = them & board.queens();

    // 1. Evaluasi keamanan Raja sendiri (Pawn shield & uncastled penalty saat Ratu lawan masih ada)
    if let Some(our_king_sq) = board.king_of(color) {
        if !enemy_queens.is_empty() {
            let king_file = our_king_sq.file();
            let home_rank = match color {
                Color::White => Rank::First,
                Color::Black => Rank::Eighth,
            };
            let shield_rank = match color {
                Color::White => Rank::Second,
                Color::Black => Rank::Seventh,
            };

            // Jika Raja masih di kolom tengah (D atau E) dan pion pelindung di depannya sudah maju/hilang
            if matches!(king_file, File::D | File::E) && our_king_sq.rank() == home_rank {
                let d_shield = Bitboard::from_file(File::D) & Bitboard::from_rank(shield_rank);
                let e_shield = Bitboard::from_file(File::E) & Bitboard::from_rank(shield_rank);
                if (our_pawns & d_shield).is_empty() || (our_pawns & e_shield).is_empty() {
                    score -= UNCASTLED_EXPOSED_PENALTY_CP;
                }
            }

            // Jika Raja sudah rokade ke sayap (A/B/C atau F/G/H), cek pawn shield di lajur sekitar Raja
            if matches!(king_file, File::A | File::B | File::C | File::F | File::G | File::H)
                && our_king_sq.rank() == home_rank
            {
                let file_idx = king_file as i32;
                for df in -1..=1 {
                    let f = (file_idx + df).clamp(0, 7) as u32;
                    let f_bb = Bitboard::from_file(File::new(f));
                    let shield_sq_bb = f_bb & Bitboard::from_rank(shield_rank);
                    if !(our_pawns & shield_sq_bb).is_empty() {
                        score += 6;
                    } else if (our_pawns & f_bb).is_empty() {
                        // Open/semi-open file di depan Raja
                        score -= 10;
                    }
                }
            }
        }
    }

    // 2. Evaluasi serangan perwira kita ke zona 3x3 di sekitar Raja lawan (+12 cp per penyerang)
    if let Some(enemy_king_sq) = board.king_of(color.other()) {
        let king_zone =
            attacks::king_attacks(enemy_king_sq) | Bitboard::from_square(enemy_king_sq);
        let mut attackers_count = 0i32;

        for sq in us & board.knights() {
            if !(attacks::knight_attacks(sq) & king_zone).is_empty() {
                attackers_count += 1;
            }
        }
        for sq in us & board.bishops() {
            if !(attacks::bishop_attacks(sq, occupied) & king_zone).is_empty() {
                attackers_count += 1;
            }
        }
        for sq in us & board.rooks() {
            if !(attacks::rook_attacks(sq, occupied) & king_zone).is_empty() {
                attackers_count += 1;
            }
        }
        for sq in us & board.queens() {
            if !(attacks::queen_attacks(sq, occupied) & king_zone).is_empty() {
                attackers_count += 1;
            }
        }

        score += attackers_count * KING_ZONE_ATTACK_BONUS_CP;
    }

    score
}

/// Evaluasi keamanan & serangan Raja (Perspektif Putih: White - Black).
pub fn evaluate_king_safety(board: &Board) -> i32 {
    side_king_safety_and_attack(board, Color::White)
        - side_king_safety_and_attack(board, Color::Black)
}
