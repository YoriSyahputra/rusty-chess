use shakmaty::{
    fen::Fen,
    san::{San, SanPlus},
    uci::UciMove,
    zobrist::{Zobrist64, ZobristHash},
    CastlingMode, Chess, EnPassantMode, Move,
};

pub const INITIAL_FEN: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// Mem-parsing string FEN menjadi objek posisi `shakmaty::Chess`.
pub fn parse_fen(fen_str: &str) -> Result<Chess, String> {
    let trimmed = fen_str.trim();
    let fen: Fen = trimmed
        .parse()
        .map_err(|e| format!("Invalid FEN syntax: {e}"))?;
    fen.into_position(CastlingMode::Standard)
        .map_err(|e| format!("Illegal chess position in FEN: {e}"))
}

/// Menghasilkan string FEN standar dari posisi `shakmaty::Chess`.
pub fn format_fen(pos: &Chess) -> String {
    Fen::from_position(pos.clone(), EnPassantMode::Legal).to_string()
}

/// Menghasilkan string Standard Algebraic Notation (SAN) lengkap dengan tanda `+` atau `#`.
pub fn format_san(pos: &Chess, mv: &Move) -> String {
    SanPlus::from_move(pos.clone(), mv).to_string()
}

/// Mem-parsing string SAN (misal `"e4"`, `"Nf3"`, `"O-O"`) menjadi `shakmaty::Move` legal.
pub fn parse_san(pos: &Chess, san_str: &str) -> Result<Move, String> {
    let clean = san_str.trim().trim_end_matches(['+', '#', '!', '?']);
    let san: San = clean
        .parse()
        .map_err(|e| format!("Invalid SAN notation '{san_str}': {e}"))?;
    san.to_move(pos)
        .map_err(|e| format!("Illegal SAN move '{san_str}' in position: {e}"))
}

/// Menghasilkan notasi UCI (misal `"e2e4"`, `"e7e8q"`).
pub fn format_uci(mv: &Move) -> String {
    UciMove::from_standard(mv).to_string()
}

/// Mem-parsing string UCI menjadi langkah legal.
pub fn parse_uci(pos: &Chess, uci_str: &str) -> Result<Move, String> {
    let uci: UciMove = uci_str
        .trim()
        .parse()
        .map_err(|e| format!("Invalid UCI move '{uci_str}': {e}"))?;
    uci.to_move(pos)
        .map_err(|e| format!("Illegal UCI move '{uci_str}': {e}"))
}

/// Menghitung 64-bit Zobrist hash dari posisi saat ini untuk deteksi Threefold Repetition & TT.
pub fn position_zobrist_hash(pos: &Chess) -> u64 {
    pos.zobrist_hash::<Zobrist64>(EnPassantMode::Legal).0
}
