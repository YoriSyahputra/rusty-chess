use crate::engine::{
    bot::BotPersona,
    search::{search_best_move, TranspositionTable},
};
use shakmaty::{Chess, Move};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone)]
pub enum EngineCommand {
    ComputeBestMove {
        request_id: u64,
        pos: Chess,
        persona: BotPersona,
        history_hashes: Vec<u64>,
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
                        } => {
                            worker_abort.store(false, Ordering::SeqCst);

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

                            if !worker_abort.load(Ordering::SeqCst) {
                                if let Some(res) = outcome {
                                    let _ = evt_tx.send(EngineEvent::MoveReady {
                                        request_id,
                                        best_move: res.best_move,
                                        eval_cp: res.eval_cp_white,
                                        depth_reached: res.depth_reached,
                                        nodes_searched: res.nodes_searched,
                                        elapsed_ms: res.elapsed_ms,
                                    });
                                }
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
        self.abort_flag.store(false, Ordering::SeqCst);
        let _ = self.cmd_tx.send(EngineCommand::ComputeBestMove {
            request_id,
            pos,
            persona,
            history_hashes,
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

impl Drop for EngineWorker {
    fn drop(&mut self) {
        self.abort_flag.store(true, Ordering::SeqCst);
        let _ = self.cmd_tx.send(EngineCommand::Shutdown);
        if let Some(handle) = self.join_handle.take() {
            let _ = handle.join();
        }
    }
}
