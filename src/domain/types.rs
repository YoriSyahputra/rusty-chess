pub use shakmaty::{Color, File, Move, Position, Rank, Role, Square};

pub type Side = Color;
pub type PieceKind = Role;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerController {
    Human,
    Bot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AnnotationColor {
    Red,
    Yellow,
    Blue,
    Green,
}

impl Default for AnnotationColor {
    fn default() -> Self {
        AnnotationColor::Red
    }
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub ply: usize,                   // Nomor setengah langkah (dimulai dari 1)
    pub mv: Move,                     // Objek Move dari shakmaty
    pub san: String,                  // Notasi standar misal "Nf3", "exd5", "O-O", "Qh7#"
    pub fen_after: String,            // String FEN setelah langkah dieksekusi
    pub eval_cp: i32,                 // Evaluasi posisi dalam centipawn (perspektif Putih)
    pub captured_piece: Option<Role>, // Bidak yang dimakan pada langkah ini (jika ada)
    pub time_spent_ms: u64,           // Waktu berpikir untuk langkah ini
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Ongoing,
    Checkmate { winner: Color },
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule,
    Resignation { winner: Color },
    Timeout { winner: Color },
}

impl GameOutcome {
    pub fn is_game_over(&self) -> bool {
        !matches!(self, GameOutcome::Ongoing)
    }

    pub fn winner(&self) -> Option<Color> {
        match *self {
            GameOutcome::Checkmate { winner }
            | GameOutcome::Resignation { winner }
            | GameOutcome::Timeout { winner } => Some(winner),
            _ => None,
        }
    }

    pub fn pgn_result_str(&self) -> &'static str {
        match *self {
            GameOutcome::Ongoing => "*",
            GameOutcome::Checkmate { winner }
            | GameOutcome::Resignation { winner }
            | GameOutcome::Timeout { winner } => match winner {
                Color::White => "1-0",
                Color::Black => "0-1",
            },
            GameOutcome::Stalemate
            | GameOutcome::InsufficientMaterial
            | GameOutcome::ThreefoldRepetition
            | GameOutcome::FiftyMoveRule => "1/2-1/2",
        }
    }

    pub fn description(&self) -> String {
        match *self {
            GameOutcome::Ongoing => "Game in progress".to_string(),
            GameOutcome::Checkmate { winner } => {
                let side = if winner == Color::White { "White" } else { "Black" };
                format!("Checkmate! {} wins", side)
            }
            GameOutcome::Stalemate => "Draw by Stalemate".to_string(),
            GameOutcome::InsufficientMaterial => "Draw by Insufficient Material".to_string(),
            GameOutcome::ThreefoldRepetition => "Draw by Threefold Repetition".to_string(),
            GameOutcome::FiftyMoveRule => "Draw by 50-Move Rule".to_string(),
            GameOutcome::Resignation { winner } => {
                let side = if winner == Color::White { "White" } else { "Black" };
                format!("{} wins by Resignation", side)
            }
            GameOutcome::Timeout { winner } => {
                let side = if winner == Color::White { "White" } else { "Black" };
                format!("{} wins on Time", side)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacticalArrow {
    pub from: Square,
    pub to: Square,
    pub color: AnnotationColor,
}

impl TacticalArrow {
    pub fn new(from: Square, to: Square, color: AnnotationColor) -> Self {
        Self { from, to, color }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MarkedSquare {
    pub square: Square,
    pub color: AnnotationColor,
}

/// Poin material standar untuk perhitungan selisih bidak (Q=9, R=5, B=3, N=3, P=1)
pub fn role_point_value(role: Role) -> i32 {
    match role {
        Role::Queen => 9,
        Role::Rook => 5,
        Role::Bishop => 3,
        Role::Knight => 3,
        Role::Pawn => 1,
        Role::King => 0,
    }
}

/// Urutan pengurutan bidak tertangkap dari nilai tertinggi (Queen -> Pawn)
pub fn role_sort_priority(role: Role) -> u8 {
    match role {
        Role::Queen => 0,
        Role::Rook => 1,
        Role::Bishop => 2,
        Role::Knight => 3,
        Role::Pawn => 4,
        Role::King => 5,
    }
}
