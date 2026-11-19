pub mod bot;
pub mod coach;
pub mod eval;
pub mod review;
pub mod search;
pub mod uci_bridge;
pub mod worker;

pub use bot::{
    detect_dialogue_trigger, dialogue_bank, lookup_persona_book_move, select_bot_dialogue, BotId,
    BotPersona, DialogueTrigger,
};
pub use coach::{explain_move, CoachExplanation};
pub use eval::{evaluate, evaluate_for_white, EvalWeights};
pub use review::{
    accuracy_from_mean_loss_pct, analyze_match_review, evaluate_move_live,
    win_probability_from_cp, BreakdownRow, GameReviewReport, LiveMoveEval, MoveClassification,
    PlyReviewEntry,
};
pub use search::{
    compute_multipv_lines, quiescence_search, search_best_move, MultiPvLine, SearchHeuristics,
    SearchOutcome, TTEntry, TTFlag, TranspositionTable, MATE_SCORE, MATE_THRESHOLD,
};
pub use uci_bridge::{
    detect_system_uci_engine, parse_uci_depth, parse_uci_score_cp, query_uci_best_move,
    test_uci_connection,
};
pub use worker::{compute_target_think_ms, EngineCommand, EngineEvent, EngineWorker};
