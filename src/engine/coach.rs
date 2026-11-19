use crate::{
    domain::{format_fen, format_san},
    engine::{
        bot::BotPersona,
        eval::material::piece_value,
        review::MoveClassification,
        search::{search_best_move, TranspositionTable},
    },
};
use shakmaty::{Chess, Move, Position, Role, Square};
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone)]
pub struct CoachExplanation {
    pub headline: String,
    pub detail: String,
    pub refutation_sans: Vec<String>,
    pub refutation_fens: Vec<String>,
}

/// Menghasilkan penjelasan bahasa alami ("Why Was That Bad / Good?") berbasis analisis bitboard
/// serta baris refutasi 3-ply untuk diputar ulang di papan (Feature #8).
pub fn explain_move(
    pos_before: &Chess,
    played_mv: &Move,
    best_mv: &Move,
    classification: MoveClassification,
) -> CoachExplanation {
    let mover = pos_before.turn();
    let played_san = format_san(pos_before, played_mv);
    let best_san = format_san(pos_before, best_mv);

    let mut pos_after = pos_before.clone();
    pos_after.play_unchecked(played_mv);

    let board_after = pos_after.board();
    let occupied_after = board_after.occupied();

    // 1. Check immediate checkmate or check
    if pos_after.is_checkmate() {
        return CoachExplanation {
            headline: format!("{played_san} delivers Checkmate!"),
            detail: "Flawless finishing blow — the opponent's King has no legal escape squares."
                .to_string(),
            refutation_sans: Vec::new(),
            refutation_fens: Vec::new(),
        };
    }

    // 2. Check if played move hung the moved piece or another piece
    let to_sq = played_mv.to();
    let enemy_atk_to = board_after
        .attacks_to(to_sq, mover.other(), occupied_after)
        .count();
    let own_def_to = board_after
        .attacks_to(to_sq, mover, occupied_after)
        .count();

    let moved_role = played_mv.promotion().unwrap_or_else(|| played_mv.role());
    let hung_moved_piece = moved_role != Role::Pawn
        && enemy_atk_to > own_def_to
        && classification.is_mistake_or_worse();

    // Check if any other friendly major/minor piece was left hanging
    let mut other_hanging: Option<(Role, Square)> = None;
    if classification.is_mistake_or_worse() {
        for sq in board_after.by_color(mover) {
            if sq == to_sq {
                continue;
            }
            if let Some(role) = board_after.role_at(sq) {
                if role != Role::King && role != Role::Pawn {
                    let e_atk = board_after
                        .attacks_to(sq, mover.other(), occupied_after)
                        .count();
                    let o_def = board_after.attacks_to(sq, mover, occupied_after).count();
                    if e_atk > o_def {
                        if other_hanging
                            .map(|(r, _)| piece_value(role) > piece_value(r))
                            .unwrap_or(true)
                        {
                            other_hanging = Some((role, sq));
                        }
                    }
                }
            }
        }
    }

    // 3. Build 3-ply follow-up refutation line when the move is inaccurate/mistake/blunder
    let mut refutation_sans = Vec::new();
    let mut refutation_fens = Vec::new();

    if classification.is_mistake_or_worse() {
        let mut tt = TranspositionTable::new();
        let abort = AtomicBool::new(false);
        let mut quick_bot = BotPersona::magnus();
        quick_bot.max_depth = 2;
        quick_bot.time_budget_ms = 60;

        let mut sim_pos = pos_after.clone();
        refutation_fens.push(format_fen(&sim_pos));

        for _ in 0..3 {
            if sim_pos.is_game_over() {
                break;
            }
            if let Some(res) =
                search_best_move(&sim_pos, &quick_bot, &[], &mut tt, &abort, |_, _, _, _| {})
            {
                let san = format_san(&sim_pos, &res.best_move);
                sim_pos.play_unchecked(&res.best_move);
                refutation_sans.push(san);
                refutation_fens.push(format_fen(&sim_pos));
            } else {
                break;
            }
        }
    }

    // 4. Construct natural-language coaching text
    let (headline, detail) = if hung_moved_piece {
        (
            format!("{played_san} hangs your {:?} on {to_sq}!", moved_role),
            format!(
                "Moving to {to_sq} leaves the {:?} undefended against enemy attackers. {best_san} kept your pieces safe.",
                moved_role
            ),
        )
    } else if let Some((h_role, h_sq)) = other_hanging {
        (
            format!("{played_san} leaves your {:?} on {h_sq} unprotected!", h_role),
            format!(
                "Your opponent can immediately punish this tactic. {best_san} was much stronger here."
            ),
        )
    } else if classification.is_mistake_or_worse() {
        let missed_capture = best_mv.is_capture();
        if missed_capture {
            (
                format!("{played_san} missed a strong capture ({best_san})"),
                format!(
                    "Playing {best_san} would have won crucial material or seized the initiative."
                ),
            )
        } else if let Some(first_ref) = refutation_sans.first() {
            (
                format!("{played_san} allows a strong reply ({first_ref})"),
                format!(
                    "This concedes key squares and lets the opponent take over with {first_ref}. Prefer {best_san}."
                ),
            )
        } else {
            (
                format!("{played_san} weakens your position"),
                format!("Prefer {best_san} to maintain stronger piece coordination and safety."),
            )
        }
    } else if played_mv.is_castle() {
        (
            format!("{played_san} safeguards your King!"),
            "Castling tucks your King behind a pawn shield and connects your Rooks for the middlegame."
                .to_string(),
        )
    } else if matches!(to_sq, Square::D4 | Square::E4 | Square::D5 | Square::E5) {
        (
            format!("{played_san} fights for the center!"),
            format!(
                "Controlling {to_sq} expands your space and maximizes piece mobility."
            ),
        )
    } else if pos_after.is_check() {
        (
            format!("{played_san} seizes initiative with Check!"),
            "Forcing the opponent's King to react disrupts their defensive coordination."
                .to_string(),
        )
    } else if played_mv.is_capture() {
        (
            format!("{played_san} executes a clean tactical exchange"),
            "Accurate calculation — keeping material balance and piece activity in your favor."
                .to_string(),
        )
    } else {
        (
            format!("{played_san} is a solid {} move", classification.label()),
            "Improves piece harmony while respecting tactical threats across the board.".to_string(),
        )
    };

    CoachExplanation {
        headline,
        detail,
        refutation_sans,
        refutation_fens,
    }
}
