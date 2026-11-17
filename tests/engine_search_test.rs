use rusty_chess::{
    domain::{format_san, format_uci, parse_fen},
    engine::{
        bot::BotPersona,
        search::{search_best_move, TranspositionTable, MATE_THRESHOLD},
    },
};
use std::sync::atomic::AtomicBool;

#[test]
fn test_mate_in_1_scholars_mate_and_back_rank() {
    let mut tt = TranspositionTable::new();
    let abort = AtomicBool::new(false);
    let persona = BotPersona::magnus();

    // 1. Scholar's Mate: Qxf7#
    let pos1 = parse_fen("r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4")
        .expect("valid FEN");
    let res1 = search_best_move(&pos1, &persona, &[], &mut tt, &abort, |_, _, _, _| {})
        .expect("search result");
    let san1 = format_san(&pos1, &res1.best_move);
    assert_eq!(san1, "Qxf7#", "Engine must find Scholar's Mate in 1 (Qxf7#)");
    assert!(
        res1.eval_cp_white > MATE_THRESHOLD,
        "Mate score must exceed MATE_THRESHOLD"
    );

    // 2. Back-rank Mate: Re8#
    let pos2 = parse_fen("6k1/5ppp/8/8/8/8/5PPP/4R1K1 w - - 0 1").expect("valid FEN");
    let res2 = search_best_move(&pos2, &persona, &[], &mut tt, &abort, |_, _, _, _| {})
        .expect("search result");
    let san2 = format_san(&pos2, &res2.best_move);
    assert_eq!(san2, "Re8#", "Engine must find Back-rank Mate in 1 (Re8#)");
}

#[test]
fn test_mate_in_2_forced_back_rank_combination() {
    let mut tt = TranspositionTable::new();
    let abort = AtomicBool::new(false);
    let mut persona = BotPersona::magnus();
    persona.max_depth = 4;
    persona.time_budget_ms = 1000;

    // Posisi Forced Mate in 2: 1. Re8+ Rxe8 2. Rxe8#
    let pos = parse_fen("r5k1/5ppp/8/8/8/8/4RPPP/4R1K1 w - - 0 1").expect("valid FEN");
    let res = search_best_move(&pos, &persona, &[], &mut tt, &abort, |_, _, _, _| {})
        .expect("search result");
    let san = format_san(&pos, &res.best_move);

    assert_eq!(
        san, "Re8+",
        "Engine must find forced Mate in 2 starting with 1. Re8+"
    );
    assert!(
        res.eval_cp_white > MATE_THRESHOLD,
        "Mate in 2 must produce a mate score, got {}",
        res.eval_cp_white
    );
}

#[test]
fn test_tactical_capture_and_quiescence_horizon_protection() {
    let mut tt = TranspositionTable::new();
    let abort = AtomicBool::new(false);
    let mut persona = BotPersona::hikaru();
    persona.noise_cp = 0;

    // 1. Quiescence Horizon Test:
    // Pion hitam di d5 dikawal oleh pion c6 (Qxd5?? kalah Ratu), sedangkan Benteng hitam di e5 gratis (Nxe5!)
    let pos_horizon =
        parse_fen("rnbqkbn1/pp2pppp/2p5/3pr3/8/5N2/PPP1PPPP/RN1QKB1R w KQq - 0 1")
            .expect("valid FEN");
    let res = search_best_move(
        &pos_horizon,
        &persona,
        &[],
        &mut tt,
        &abort,
        |_, _, _, _| {},
    )
    .expect("search result");
    let uci = format_uci(&res.best_move);
    assert_eq!(
        uci, "f3e5",
        "Engine must capture the free Rook on e5 (Nxe5) and avoid the Qxd5 trap"
    );

    // 2. Unpinned free Queen capture: Ratu hitam di d5 tidak dikawal dan bisa dimakan Kuda c3 (Nxd5)
    let pos_capture =
        parse_fen("rnb1kbnr/ppp1pppp/8/3q4/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 0 1")
            .expect("valid FEN");
    let res_cap = search_best_move(
        &pos_capture,
        &persona,
        &[],
        &mut tt,
        &abort,
        |_, _, _, _| {},
    )
    .expect("search result");
    let uci_cap = format_uci(&res_cap.best_move);
    assert_eq!(
        uci_cap, "c3d5",
        "Engine must capture the undefended Queen on d5"
    );
}
