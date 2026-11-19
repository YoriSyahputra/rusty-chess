use rand::Rng;

const KNIGHT_PAIRS: [(usize, usize); 10] = [
    (0, 1),
    (0, 2),
    (0, 3),
    (0, 4),
    (1, 2),
    (1, 3),
    (1, 4),
    (2, 3),
    (2, 4),
    (3, 4),
];

/// Menghasilkan susunan baris belakang Chess960 (Fischer Random) berdasarkan nomor Scharnagl `0..=959`.
/// Posisi standar klasik adalah ID `518` (`"RNBQKBNR"`).
pub fn scharnagl_back_rank(id: u16) -> String {
    let n = (id % 960) as usize;
    let mut rank = [' '; 8];

    // 1. Light-squared bishop (files b, d, f, h => 1, 3, 5, 7)
    let b1 = n % 4;
    let n2 = n / 4;
    rank[b1 * 2 + 1] = 'B';

    // 2. Dark-squared bishop (files a, c, e, g => 0, 2, 4, 6)
    let b2 = n2 % 4;
    let n3 = n2 / 4;
    rank[b2 * 2] = 'B';

    // 3. Queen on one of the 6 remaining empty squares
    let q = n3 % 6;
    let n4 = n3 / 6;
    let mut empty: Vec<usize> = (0..8).filter(|&i| rank[i] == ' ').collect();
    rank[empty.remove(q)] = 'Q';

    // 4. Two Knights on one of the 10 pairs among the 5 remaining empty squares
    let (k1, k2) = KNIGHT_PAIRS[n4 % 10];
    rank[empty[k2]] = 'N';
    rank[empty[k1]] = 'N';
    empty.remove(k2);
    empty.remove(k1);

    // 5. Remaining 3 empty squares in left-to-right order: Rook, King, Rook
    rank[empty[0]] = 'R';
    rank[empty[1]] = 'K';
    rank[empty[2]] = 'R';

    rank.iter().collect()
}

/// Menghasilkan string X-FEN lengkap untuk posisi awal Chess960 dengan ID Scharnagl `0..=959`.
pub fn chess960_fen(id: u16) -> String {
    let white_back = scharnagl_back_rank(id);
    let black_back = white_back.to_ascii_lowercase();
    format!("{black_back}/pppppppp/8/8/8/8/PPPPPPPP/{white_back} w KQkq - 0 1")
}

/// Menghasilkan ID acak Chess960 (`0..=959`) beserta FEN awalnya.
pub fn random_chess960_fen() -> (u16, String) {
    let id: u16 = rand::thread_rng().gen_range(0..960);
    (id, chess960_fen(id))
}
