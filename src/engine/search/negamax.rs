use crate::{
    domain::notation::position_zobrist_hash,
    engine::{
        bot::BotPersona,
        eval::EvalWeights,
        search::{
            ordering::SearchHeuristics,
            quiescence::quiescence_search,
            tt::{score_from_tt, TTFlag, TranspositionTable, MATE_SCORE, MATE_THRESHOLD},
        },
    },
};
use rand::Rng;
use shakmaty::{Chess, Color, Move, Position};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const INF: i32 = 32_000;

#[derive(Debug, Clone)]
pub struct SearchOutcome {
    pub best_move: Move,
    pub eval_cp_white: i32,
    pub depth_reached: u8,
    pub nodes_searched: u64,
    pub elapsed_ms: u64,
    pub root_candidates: Vec<(Move, i32)>,
}

struct SearchContext<'a> {
    tt: &'a mut TranspositionTable,
    heuristics: SearchHeuristics,
    weights: EvalWeights,
    max_q_depth: u8,
    root_turn: Color,
    game_history_hashes: &'a [u64],
    path_hashes: Vec<u64>,
    start_time: Instant,
    deadline: Instant,
    abort_flag: &'a AtomicBool,
    aborted: bool,
    nodes: u64,
}

impl<'a> SearchContext<'a> {
    #[inline]
    fn check_stop(&mut self) -> bool {
        if self.aborted {
            return true;
        }
        if (self.nodes & 1023) == 0 {
            if self.abort_flag.load(Ordering::Relaxed) || Instant::now() >= self.deadline {
                self.aborted = true;
                return true;
            }
        }
        false
    }

    #[inline]
    fn draw_score(&self, current_turn: Color) -> i32 {
        if current_turn == self.root_turn {
            -self.weights.contempt_cp
        } else {
            self.weights.contempt_cp
        }
    }

    #[inline]
    fn is_repetition(&self, hash: u64) -> bool {
        if self.path_hashes.iter().filter(|&&h| h == hash).count() >= 1 {
            return true;
        }
        self.game_history_hashes
            .iter()
            .filter(|&&h| h == hash)
            .count()
            >= 2
    }

    fn negamax(
        &mut self,
        pos: &Chess,
        depth: u8,
        mut alpha: i32,
        beta: i32,
        ply: i32,
    ) -> i32 {
        if self.check_stop() {
            return 0;
        }

        let hash = position_zobrist_hash(pos);

        if ply > 0 {
            if pos.is_stalemate()
                || pos.is_insufficient_material()
                || pos.halfmoves() >= 100
                || self.is_repetition(hash)
            {
                return self.draw_score(pos.turn());
            }
        }

        if pos.is_checkmate() {
            return -MATE_SCORE + ply;
        }

        if depth == 0 {
            return quiescence_search(
                pos,
                alpha,
                beta,
                0,
                self.max_q_depth,
                ply,
                &self.weights,
                &mut self.nodes,
            );
        }

        self.nodes += 1;
        let orig_alpha = alpha;

        let mut tt_best_move: Option<Move> = None;
        if let Some(entry) = self.tt.probe(hash) {
            tt_best_move = entry.best_move.clone();
            if entry.depth >= depth && ply > 0 {
                let tt_score = score_from_tt(entry.score, ply);
                match entry.flag {
                    TTFlag::Exact => return tt_score,
                    TTFlag::LowerBound => {
                        if tt_score > alpha {
                            alpha = tt_score;
                        }
                    }
                    TTFlag::UpperBound => {
                        if tt_score < beta {
                            return tt_score;
                        }
                    }
                }
                if alpha >= beta {
                    return tt_score;
                }
            }
        }

        let mut moves: Vec<Move> = pos.legal_moves().into_iter().collect();
        if moves.is_empty() {
            return if pos.is_check() {
                -MATE_SCORE + ply
            } else {
                self.draw_score(pos.turn())
            };
        }

        self.heuristics
            .order_moves(pos, &mut moves, tt_best_move.as_ref(), ply as usize);

        let mut best_score = -INF;
        let mut best_move: Option<Move> = None;

        self.path_hashes.push(hash);

        for mv in moves {
            let mut child = pos.clone();
            child.play_unchecked(&mv);

            let score = -self.negamax(&child, depth - 1, -beta, -alpha, ply + 1);

            if self.aborted {
                self.path_hashes.pop();
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mv.clone());
            }

            if score > alpha {
                alpha = score;
            }

            if alpha >= beta {
                self.heuristics.record_killer(ply as usize, mv.clone());
                self.heuristics.record_history(pos.turn(), &mv, depth);
                break;
            }
        }

        self.path_hashes.pop();

        let flag = if best_score <= orig_alpha {
            TTFlag::UpperBound
        } else if best_score >= beta {
            TTFlag::LowerBound
        } else {
            TTFlag::Exact
        };

        self.tt
            .store(hash, depth, flag, best_score, best_move, ply);

        best_score
    }
}

/// Menjalankan Iterative Deepening + Negamax Alpha-Beta Pruning beserta Root Multi-PV Softmax Sampling.
pub fn search_best_move<F>(
    pos: &Chess,
    persona: &BotPersona,
    game_history_hashes: &[u64],
    tt: &mut TranspositionTable,
    abort_flag: &AtomicBool,
    mut on_progress: F,
) -> Option<SearchOutcome>
where
    F: FnMut(u8, u8, i32, u64),
{
    let legal_moves: Vec<Move> = pos.legal_moves().into_iter().collect();
    if legal_moves.is_empty() {
        return None;
    }

    let start_time = Instant::now();
    let budget = Duration::from_millis(persona.time_budget_ms.max(50));
    let deadline = start_time + budget;
    let max_depth = persona.max_depth.max(1);
    let root_turn = pos.turn();

    // Quiescence depth scaled by ELO (NEW_DESIGN.md Section 7.1)
    let max_q_depth = if persona.elo <= 400 {
        0
    } else if persona.elo <= 1200 {
        2
    } else {
        6
    };

    let mut ctx = SearchContext {
        tt,
        heuristics: SearchHeuristics::new(),
        weights: persona.weights,
        max_q_depth,
        root_turn,
        game_history_hashes,
        path_hashes: Vec::with_capacity(64),
        start_time,
        deadline,
        abort_flag,
        aborted: false,
        nodes: 0,
    };

    let mut completed_candidates: Vec<(Move, i32)> =
        legal_moves.iter().cloned().map(|m| (m, 0)).collect();
    let mut depth_reached = 1u8;

    for depth in 1..=max_depth {
        if depth > 1
            && (ctx.abort_flag.load(Ordering::Relaxed) || Instant::now() >= ctx.deadline)
        {
            break;
        }

        let mut root_moves: Vec<Move> = completed_candidates
            .iter()
            .map(|(m, _)| m.clone())
            .collect();

        let root_hash = position_zobrist_hash(pos);
        let tt_root_move = ctx.tt.probe(root_hash).and_then(|e| e.best_move.clone());
        ctx.heuristics
            .order_moves(pos, &mut root_moves, tt_root_move.as_ref(), 0);

        let mut current_depth_scores: Vec<(Move, i32)> = Vec::with_capacity(root_moves.len());
        let mut alpha = -INF;
        let beta = INF;
        let mut depth_aborted = false;

        ctx.path_hashes.clear();
        ctx.path_hashes.push(root_hash);

        for mv in root_moves {
            let mut child = pos.clone();
            child.play_unchecked(&mv);

            let score = -ctx.negamax(&child, depth - 1, -beta, -alpha, 1);

            if ctx.aborted && depth > 1 {
                depth_aborted = true;
                break;
            }

            if score > alpha {
                alpha = score;
            }
            current_depth_scores.push((mv, score));
        }

        if depth_aborted && current_depth_scores.is_empty() {
            break;
        }

        if !depth_aborted {
            current_depth_scores.sort_by_key(|(_, s)| -(*s));
            if let Some((best_mv, best_sc)) = current_depth_scores.first() {
                ctx.tt.store(
                    root_hash,
                    depth,
                    TTFlag::Exact,
                    *best_sc,
                    Some(best_mv.clone()),
                    0,
                );
            }
            completed_candidates = current_depth_scores;
            depth_reached = depth;

            let best_score_side = completed_candidates.first().map(|(_, s)| *s).unwrap_or(0);
            let eval_cp_white = match root_turn {
                Color::White => best_score_side,
                Color::Black => -best_score_side,
            };
            on_progress(depth_reached, max_depth, eval_cp_white, ctx.nodes);
        } else {
            break;
        }
    }

    completed_candidates.sort_by_key(|(_, s)| -(*s));

    // Root Multi-PV Boltzmann / Softmax Sampling (NEW_DESIGN.md Section 7.1)
    let chosen_pair = sample_root_move_softmax(&completed_candidates, persona);
    let eval_cp_white = match root_turn {
        Color::White => chosen_pair.1,
        Color::Black => -chosen_pair.1,
    };

    let elapsed_ms = ctx.start_time.elapsed().as_millis() as u64;

    Some(SearchOutcome {
        best_move: chosen_pair.0,
        eval_cp_white,
        depth_reached,
        nodes_searched: ctx.nodes.max(1),
        elapsed_ms,
        root_candidates: completed_candidates,
    })
}

/// Memilih langkah dari kandidat root menggunakan distribusi Boltzmann / Softmax:
/// `P(m_i) = exp((s_i - s_0) / T) / sum_j exp((s_j - s_0) / T)`
fn sample_root_move_softmax(
    candidates: &[(Move, i32)],
    persona: &BotPersona,
) -> (Move, i32) {
    let best = candidates[0].clone();

    let temp = if persona.softmax_temp_cp > 0.0 {
        persona.softmax_temp_cp
    } else if persona.noise_cp > 5 && persona.elo < 2600 {
        persona.noise_cp as f32
    } else {
        0.0
    };

    if candidates.len() == 1 || temp <= 0.0 || persona.elo >= 2600 {
        return best;
    }

    // Untuk bot ELO >= 1000, jangan pernah melewatkan skakmat paksa
    if best.1 > MATE_THRESHOLD && persona.elo >= 1000 {
        return best;
    }

    let s0 = best.1 as f32;
    let mut weights: Vec<f32> = Vec::with_capacity(candidates.len());
    let mut sum_w = 0.0_f32;

    for (_, score) in candidates {
        let delta = (*score as f32) - s0; // <= 0.0
        // Dampen severe blunders for bots above 1200 ELO
        if persona.elo >= 1200 && delta < -180.0 {
            weights.push(0.0);
            continue;
        }
        let w = (delta / temp).clamp(-20.0, 0.0).exp();
        weights.push(w);
        sum_w += w;
    }

    if sum_w <= f32::EPSILON {
        return best;
    }

    let mut rng = rand::thread_rng();
    let sample = rng.gen::<f32>() * sum_w;
    let mut accum = 0.0_f32;

    for (idx, &w) in weights.iter().enumerate() {
        accum += w;
        if sample <= accum {
            return candidates[idx].clone();
        }
    }

    best
}
