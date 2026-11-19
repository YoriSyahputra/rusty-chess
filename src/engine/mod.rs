pub mod bot;
pub mod coach;
pub mod eval;
pub mod review;
pub mod search;
pub mod worker;

pub use bot::{
    detect_dialogue_trigger, dialogue_bank, lookup_persona_book_move, select_bot_dialogue, BotId,
    BotPersona, DialogueTrigger,
};
pub use coach::{explain_move, CoachExplanation};
pub use eval::{evaluate, evaluate_for_white, EvalWeights};
pub use review::{
    accuracy_from_mean_loss_pct, analyze_match_review, win_probability_from_cp, GameReviewReport,
    MoveClassification, PlyReviewEntry,
};
pub use search::{
    quiescence_search, search_best_move, SearchHeuristics, SearchOutcome, TTEntry, TTFlag,
    TranspositionTable, MATE_SCORE, MATE_THRESHOLD,
};
pub use worker::{compute_target_think_ms, EngineCommand, EngineEvent, EngineWorker};
