use crate::engine::{
    eval::{evaluate, material::piece_value, EvalWeights},
    search::tt::MATE_SCORE,
};
use shakmaty::{Chess, Move, Position, Role};

pub const MAX_QUIESCENCE_DEPTH: u8 = 6;

/// Quiescence search untuk mencegah Horizon Effect pada akhir pencarian Negamax (`depth == 0`).
/// Batas kedalaman (`max_q_depth`) diskalakan menurut ELO bot (0 untuk ELO <= 400, 2 untuk 450-1200, 6 untuk 1300+).
pub fn quiescence_search(
    pos: &Chess,
    mut alpha: i32,
    beta: i32,
    q_depth: u8,
    max_q_depth: u8,
    ply: i32,
    weights: &EvalWeights,
    nodes: &mut u64,
) -> i32 {
    *nodes += 1;

    if pos.is_checkmate() {
        return -MATE_SCORE + ply;
    }
    if pos.is_stalemate() || pos.is_insufficient_material() {
        return 0;
    }

    let stand_pat = evaluate(pos, weights);
    if stand_pat >= beta {
        return beta;
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    if q_depth >= max_q_depth {
        return alpha;
    }

    let mut tactical_moves: Vec<Move> = pos
        .legal_moves()
        .into_iter()
        .filter(|mv| mv.is_capture() || mv.promotion() == Some(Role::Queen))
        .collect();

    if tactical_moves.is_empty() {
        return alpha;
    }

    tactical_moves.sort_by_cached_key(|mv| {
        let promo_bonus = mv.promotion().map(piece_value).unwrap_or(0);
        let victim_val = match mv {
            Move::Normal { capture, .. } => capture.map(piece_value).unwrap_or(0),
            Move::EnPassant { .. } => 100,
            _ => 0,
        };
        let attacker_val = piece_value(mv.role());
        -(promo_bonus + victim_val * 10 - attacker_val)
    });

    for mv in tactical_moves {
        let mut child = pos.clone();
        child.play_unchecked(&mv);

        let score = -quiescence_search(
            &child,
            -beta,
            -alpha,
            q_depth + 1,
            max_q_depth,
            ply + 1,
            weights,
            nodes,
        );

        if score >= beta {
            return beta;
        }
        if score > alpha {
            alpha = score;
        }
    }

    alpha
}
