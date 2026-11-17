pub mod king_safety;
pub mod material;
pub mod mobility;
pub mod pst;

use king_safety::evaluate_king_safety;
use material::{compute_game_phase, evaluate_material};
use mobility::evaluate_mobility;
use pst::{evaluate_pawn_structure, evaluate_tapered_pst};
use shakmaty::{Chess, Color, Position};

#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct EvalWeights {
    pub material_scale: f32,    // Default 1.0
    pub positional_scale: f32,  // Skala PST & Pawn Structure
    pub mobility_scale: f32,    // Skala agresivitas perwira & kontrol ruang
    pub king_attack_scale: f32, // Skala keberanian menyerang Raja lawan
    pub endgame_scale: f32,     // Ketajaman evaluasi saat masuk fase Endgame
    pub contempt_cp: i32,       // Penalti terhadap hasil seri (Magnus benci remis!)
}

impl Default for EvalWeights {
    fn default() -> Self {
        Self {
            material_scale: 1.0,
            positional_scale: 1.0,
            mobility_scale: 1.0,
            king_attack_scale: 1.0,
            endgame_scale: 1.0,
            contempt_cp: 0,
        }
    }
}

/// Menghitung evaluasi posisi dalam satuan centipawn (cp) dari perspektif Putih (`+` = Putih unggul).
pub fn evaluate_for_white(pos: &Chess, weights: &EvalWeights) -> i32 {
    let board = pos.board();
    let phase = compute_game_phase(board);

    let material_cp = (evaluate_material(board) as f32 * weights.material_scale).round() as i32;

    let pst_cp = evaluate_tapered_pst(board, phase, weights.endgame_scale);
    let pawn_struct_cp =
        (evaluate_pawn_structure(board) as f32 * weights.endgame_scale).round() as i32;
    let positional_cp =
        ((pst_cp + pawn_struct_cp) as f32 * weights.positional_scale).round() as i32;

    let mobility_cp = (evaluate_mobility(board) as f32 * weights.mobility_scale).round() as i32;
    let king_safety_cp =
        (evaluate_king_safety(board) as f32 * weights.king_attack_scale).round() as i32;

    material_cp + positional_cp + mobility_cp + king_safety_cp
}

/// Menghitung evaluasi posisi dalam satuan centipawn (cp) dari perspektif sisi yang sedang jalan (Negamax).
pub fn evaluate(pos: &Chess, weights: &EvalWeights) -> i32 {
    let white_score = evaluate_for_white(pos, weights);
    match pos.turn() {
        Color::White => white_score,
        Color::Black => -white_score,
    }
}
