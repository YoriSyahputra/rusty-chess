use crate::engine::{
    bot::{lookup_persona_book_move, BotPersona},
    eval::evaluate_for_white,
    search::{search_best_move, TranspositionTable},
    uci_bridge::query_uci_best_move,
};
use rand::Rng;
use shakmaty::{Chess, Move, Position};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    ComputeBestMove {
        request_id: u64,
        pos: Chess,
        persona: BotPersona,
        history_hashes: Vec<u64>,
        ply: usize,
        clock_factor: f32,
        enable_thinking_delay: bool,
        external_uci_path: Option<String>,
    },
    AbortSearch,
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum EngineEvent {
    ThinkingProgress {
        request_id: u64,
        depth: u8,
        max_depth: u8,
        eval_cp: i32,
        nodes: u64,
    },
    MoveReady {
        request_id: u64,
        best_move: Move,
        eval_cp: i32,
        depth_reached: u8,
        nodes_searched: u64,
        elapsed_ms: u64,
    },
}

pub struct EngineWorker {
    cmd_tx: Sender<EngineCommand>,
    evt_rx: Receiver<EngineEvent>,
    abort_flag: Arc<AtomicBool>,
    join_handle: Option<JoinHandle<()>>,
}

impl EngineWorker {
    pub fn spawn() -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<EngineCommand>();
        let (evt_tx, evt_rx) = mpsc::channel::<EngineEvent>();
        let abort_flag = Arc::new(AtomicBool::new(false));
        let worker_abort = Arc::clone(&abort_flag);

        let handle = thread::Builder::new()
            .name("rusty-chess-engine".to_string())
            .spawn(move || {
                let mut tt = TranspositionTable::new();

                while let Ok(cmd) = cmd_rx.recv() {
                    match cmd {
                        EngineCommand::ComputeBestMove {
                            request_id,
                            pos,
                            persona,
                            history_hashes,
                            ply,
                            clock_factor,
                            enable_thinking_delay,
                            external_uci_path,
                        } => {
                            worker_abort.store(false, Ordering::SeqCst);

                            let legal_moves = pos.legal_moves();
                            let legal_count = legal_moves.len();
                            if legal_count == 0 {
                                continue;
                            }

                            // 0. Opsional: Coba External UCI Engine (Feature #24) bila diaktifkan
                            let uci_result = if let Some(ref path) = external_uci_path {
                                if !path.trim().is_empty() {
                                    let t0 = Instant::now();
                                    let budget = persona.time_budget_ms.clamp(120, 1200);
                                    query_uci_best_move(path, &pos, budget).ok().map(
                                        |(mv, eval_w, d)| {
                                            let ms = t0.elapsed().as_millis() as u64;
                                            let _ = evt_tx.send(EngineEvent::ThinkingProgress {
                                                request_id,
                                                depth: d,
                                                max_depth: d.max(persona.max_depth),
                                                eval_cp: eval_w,
                                                nodes: 1000,
                                            });
                                            (mv, eval_w, d, 1000u64, ms, false)
                                        },
                                    )
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            // 1. Cek apakah ada langkah Opening Book untuk persona ini (langkah 1-6)
                            let book_move = if uci_result.is_none() && enable_thinking_delay {
                                lookup_persona_book_move(&pos, &persona, ply)
                            } else {
                                None
                            };

                            let (best_move, eval_cp, depth_reached, nodes_searched, search_ms, close_top_two) =
                                if let Some(res) = uci_result {
                                    res
                                } else if let Some(bm) = book_move {
                                    let mut child = pos.clone();
                                    child.play_unchecked(&bm);
                                    let eval_w = evaluate_for_white(&child, &persona.weights);
                                    let _ = evt_tx.send(EngineEvent::ThinkingProgress {
                                        request_id,
                                        depth: 1,
                                        max_depth: persona.max_depth,
                                        eval_cp: eval_w,
                                        nodes: 1,
                                    });
                                    (bm, eval_w, 1u8, 1u64, 5u64, false)
                                } else {
                                    let outcome = search_best_move(
                                        &pos,
                                        &persona,
                                        &history_hashes,
                                        &mut tt,
                                        &worker_abort,
                                        |depth, max_depth, eval_cp, nodes| {
                                            let _ = evt_tx.send(EngineEvent::ThinkingProgress {
                                                request_id,
                                                depth,
                                                max_depth,
                                                eval_cp,
                                                nodes,
                                            });
                                        },
                                    );

                                    match outcome {
                                        Some(res) => {
                                            let is_close = if res.root_candidates.len() >= 2 {
                                                (res.root_candidates[0].1
                                                    - res.root_candidates[1].1)
                                                    .abs()
                                                    < 25
                                            } else {
                                                false
                                            };
                                            (
                                                res.best_move,
                                                res.eval_cp_white,
                                                res.depth_reached,
                                                res.nodes_searched,
                                                res.elapsed_ms,
                                                is_close,
                                            )
                                        }
                                        None => continue,
                                    }
                                };

                            if worker_abort.load(Ordering::SeqCst) {
                                continue;
                            }

                            // 2. Context-Aware Thinking Delay Scheduler (NEW_DESIGN.md Section 7.3)
                            let mut total_elapsed_ms = search_ms;
                            if enable_thinking_delay {
                                let is_book = ply <= 8 && search_ms < 15;
                                let is_recapture = best_move.is_capture() && ply >= 2;
                                let in_check = pos.is_check();

                                let target_think_ms = compute_target_think_ms(
                                    is_book,
                                    legal_count,
                                    is_recapture,
                                    close_top_two,
                                    in_check,
                                    persona.pace_factor,
                                    clock_factor,
                                );

                                let remaining_delay_ms =
                                    target_think_ms.saturating_sub(search_ms);
                                let mut slept_ms = 0u64;
                                while slept_ms < remaining_delay_ms {
                                    if worker_abort.load(Ordering::Relaxed) {
                                        break;
                                    }
                                    let step = (remaining_delay_ms - slept_ms).min(25);
                                    thread::sleep(Duration::from_millis(step));
                                    slept_ms += step;
                                }
                                total_elapsed_ms += slept_ms;
                            }

                            if !worker_abort.load(Ordering::SeqCst) {
                                let _ = evt_tx.send(EngineEvent::MoveReady {
                                    request_id,
                                    best_move,
                                    eval_cp,
                                    depth_reached,
                                    nodes_searched,
                                    elapsed_ms: total_elapsed_ms,
                                });
                            }
                        }
                        EngineCommand::AbortSearch => {
                            worker_abort.store(true, Ordering::SeqCst);
                        }
                        EngineCommand::Shutdown => {
                            worker_abort.store(true, Ordering::SeqCst);
                            break;
                        }
                    }
                }
            })
            .expect("Failed to spawn rusty-chess background engine worker thread");

        Self {
            cmd_tx,
            evt_rx,
            abort_flag,
            join_handle: Some(handle),
        }
    }

    pub fn request_move(
        &self,
        request_id: u64,
        pos: Chess,
        persona: BotPersona,
        history_hashes: Vec<u64>,
    ) {
        self.request_move_with_pacing(request_id, pos, persona, history_hashes, 0, 1.0, true);
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_move_with_pacing(
        &self,
        request_id: u64,
        pos: Chess,
        persona: BotPersona,
        history_hashes: Vec<u64>,
        ply: usize,
        clock_factor: f32,
        enable_thinking_delay: bool,
    ) {
        self.request_move_with_uci(
            request_id,
            pos,
            persona,
            history_hashes,
            ply,
            clock_factor,
            enable_thinking_delay,
            None,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn request_move_with_uci(
        &self,
        request_id: u64,
        pos: Chess,
        persona: BotPersona,
        history_hashes: Vec<u64>,
        ply: usize,
        clock_factor: f32,
        enable_thinking_delay: bool,
        external_uci_path: Option<String>,
    ) {
        self.abort_flag.store(false, Ordering::SeqCst);
        let _ = self.cmd_tx.send(EngineCommand::ComputeBestMove {
            request_id,
            pos,
            persona,
            history_hashes,
            ply,
            clock_factor,
            enable_thinking_delay,
            external_uci_path,
        });
    }

    pub fn abort_current_search(&self) {
        self.abort_flag.store(true, Ordering::SeqCst);
        let _ = self.cmd_tx.send(EngineCommand::AbortSearch);
    }

    pub fn try_recv_event(&self) -> Option<EngineEvent> {
        self.evt_rx.try_recv().ok()
    }

    pub fn abort_flag(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.abort_flag)
    }
}

/// Menghitung durasi berpikir natural (`target_think_ms`) dalam rentang [350 ms, 3800 ms]
/// sesuai spesifikasi NEW_DESIGN.md Section 7.3.
pub fn compute_target_think_ms(
    is_book: bool,
    legal_count: usize,
    is_recapture: bool,
    close_top_two: bool,
    in_check: bool,
    pace_factor: f32,
    clock_factor: f32,
) -> u64 {
    let base_ms: f32 = if is_book || legal_count <= 1 {
        400.0
    } else if is_recapture {
        550.0
    } else {
        750.0 + ((legal_count as f32 * 25.0).min(1400.0))
    };

    let mut complexity_add: f32 = 0.0;
    if close_top_two {
        complexity_add += 550.0;
    }
    if in_check && legal_count > 1 {
        complexity_add += 420.0;
    }

    let mut rng = rand::thread_rng();
    let jitter: f32 = rng.gen_range(0.82..=1.18);

    let scaled =
        (base_ms + complexity_add) * pace_factor.clamp(0.4, 1.8) * clock_factor.clamp(0.25, 1.2) * jitter;

    (scaled.round() as u64).clamp(350, 3800)
}

impl Drop for EngineWorker {
    fn drop(&mut self) {
        self.abort_flag.store(true, Ordering::SeqCst);
        let _ = self.cmd_tx.send(EngineCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}
