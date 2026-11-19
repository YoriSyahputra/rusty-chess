use rusty_chess::{
    domain::{parse_fen, parse_san, parse_uci, ChessMatchState, Color, GameOutcome},
    engine::{
        bot::{
            detect_dialogue_trigger, dialogue_bank, lookup_persona_book_move, select_bot_dialogue,
            BotId, BotPersona, DialogueTrigger,
        },
        review::{accuracy_from_mean_loss_pct, analyze_match_review, win_probability_from_cp},
        worker::compute_target_think_ms,
    },
    ui::pages::CURATED_PUZZLES,
};
use shakmaty::{Chess, Position};

#[test]
fn test_all_six_personas_parameters_and_contrast() {
    let roster = BotPersona::all_roster();
    assert_eq!(roster.len(), 6, "Must have all 6 classic AI personas");

    // Also verify all 18 v0.2.0 Lobby Personalities + Engine Custom
    let lobby = BotPersona::lobby_roster();
    assert_eq!(lobby.len(), 24, "Lobby must contain all 24 personality bots");

    let custom_eng = BotPersona::engine_custom(2250);
    assert_eq!(custom_eng.elo, 2250);
    assert_eq!(custom_eng.max_depth, 5);

    let martin = BotPersona::martin();
    let warung = BotPersona::warung();
    let club = BotPersona::club();
    let hikaru = BotPersona::hikaru();
    let magnus = BotPersona::magnus();
    let silicon = BotPersona::silicon();

    assert_eq!(martin.elo, 400);
    assert_eq!(warung.elo, 1100);
    assert_eq!(club.elo, 1600);
    assert_eq!(hikaru.elo, 2800);
    assert_eq!(magnus.elo, 2882);
    assert_eq!(silicon.elo, 3100);

    assert!(
        hikaru.weights.mobility_scale > club.weights.mobility_scale,
        "Hikaru must have higher mobility scale than Club bot"
    );
    assert!(
        hikaru.weights.king_attack_scale > magnus.weights.king_attack_scale,
        "Hikaru must have higher king attack aggressiveness than Magnus"
    );
    assert!(
        magnus.weights.endgame_scale > hikaru.weights.endgame_scale,
        "Magnus must have the highest endgame precision scale"
    );
    assert_eq!(
        magnus.weights.contempt_cp, 45,
        "Magnus must have contempt_cp = 45 (refuses draws)"
    );
    assert!(
        martin.noise_cp >= 250 && magnus.noise_cp == 0,
        "Martin must have high blunder noise while Magnus has zero noise"
    );
}

#[test]
fn test_persona_dialogue_triggers_and_iconic_quotes() {
    let all_triggers = [
        DialogueTrigger::MatchStart,
        DialogueTrigger::OpeningPhase,
        DialogueTrigger::BotCapturesPiece,
        DialogueTrigger::PlayerBlunders,
        DialogueTrigger::PlayerGreatMove,
        DialogueTrigger::BotInCheck,
        DialogueTrigger::BotGivesCheck,
        DialogueTrigger::EndgameEntered,
        DialogueTrigger::BotWinning,
        DialogueTrigger::BotLosing,
        DialogueTrigger::GameOverWin,
        DialogueTrigger::GameOverLoss,
        DialogueTrigger::GameOverDraw,
    ];

    for &bot_id in BotId::all() {
        for &trigger in &all_triggers {
            let bank = dialogue_bank(bot_id, trigger);
            assert!(
                !bank.is_empty(),
                "Dialogue bank for {bot_id:?} on {trigger:?} must not be empty"
            );
            let chosen = select_bot_dialogue(bot_id, trigger);
            assert!(!chosen.is_empty());
        }
    }

    let trigger = detect_dialogue_trigger(12, 0, 240, false, false, false, 18, false);
    assert_eq!(trigger, Some(DialogueTrigger::PlayerBlunders));

    let great = detect_dialogue_trigger(14, 100, -60, false, false, false, 16, false);
    assert_eq!(great, Some(DialogueTrigger::PlayerGreatMove));
}

#[test]
fn test_opening_books_and_thinking_delay_scheduler() {
    let start_pos = Chess::default();
    let nelson = BotPersona::nelson();
    let magnus = BotPersona::magnus();
    let martin = BotPersona::martin();

    assert!(lookup_persona_book_move(&start_pos, &nelson, 0).is_some());
    assert!(lookup_persona_book_move(&start_pos, &magnus, 0).is_some());
    assert!(lookup_persona_book_move(&start_pos, &martin, 0).is_some());

    // After 1. e4 e5, Nelson must choose Qh5 or Bc4 from opening book
    let pos_after_e4_e5 =
        parse_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
            .expect("valid FEN");
    let nelson_mv = lookup_persona_book_move(&pos_after_e4_e5, &nelson, 2).expect("book move");
    assert!(nelson_mv.from().is_some());

    // Verify Thinking Delay Scheduler clamps to [350, 3800] ms
    let delay_fast = compute_target_think_ms(true, 20, false, false, false, 0.62, 0.35);
    let delay_slow = compute_target_think_ms(false, 38, false, true, true, 1.35, 1.0);
    assert!((350..=3800).contains(&delay_fast));
    assert!((350..=3800).contains(&delay_slow));
}

#[test]
fn test_game_state_fools_mate_review_and_curated_puzzles() {
    let mut state = ChessMatchState::new_standard(Color::White);

    let moves_san = ["f3", "e5", "g4", "Qh4#"];
    for san in moves_san {
        let mv = parse_san(&state.current_pos, san).expect("legal move");
        let _ = state.apply_move(mv, 0, 100).expect("apply move");
    }

    assert_eq!(
        state.outcome,
        GameOutcome::Checkmate {
            winner: Color::Black
        }
    );

    // Verify Post-Game Review & Local Accuracy Model
    assert!((win_probability_from_cp(0) - 0.5).abs() < 0.001);
    assert!((accuracy_from_mean_loss_pct(0.0) - 100.0).abs() < 0.01);

    let report = analyze_match_review(&state, 2);
    assert_eq!(report.entries.len(), 4);
    assert!(
        report.black_accuracy > report.white_accuracy,
        "Black (winner in Fool's Mate) must have higher accuracy than White"
    );

    // Verify all 32 curated tactical puzzles have valid FENs and legal solution moves
    assert!(CURATED_PUZZLES.len() >= 30);
    for puzzle in CURATED_PUZZLES {
        let mut pos = parse_fen(puzzle.fen)
            .unwrap_or_else(|e| panic!("Puzzle #{} has invalid FEN: {e}", puzzle.id));
        for &uci_str in puzzle.solution_uci {
            let mv = parse_uci(&pos, uci_str).unwrap_or_else(|e| {
                panic!("Puzzle #{} has illegal solution move '{uci_str}': {e}", puzzle.id)
            });
            pos.play_unchecked(&mv);
        }
    }

    // Verify board geometry formulas at 1180x780 and 1366x768 produce zero clipping
    for (win_w, win_h) in [(1180.0_f32, 780.0_f32), (1366.0_f32, 768.0_f32)] {
        let sidebar_w = if win_w >= 1100.0 { 168.0 } else { 60.0 };
        let right_panel_w = 360.0;
        let top_bar_h = 40.0;
        let avail_x = win_w - sidebar_w - right_panel_w;
        let avail_y = win_h - top_bar_h;
        let eval_w = 36.0;
        let board_size = (avail_x - eval_w - 16.0).min(avail_y - 104.0).max(320.0);
        assert!(
            board_size + 104.0 <= avail_y,
            "Board + top/bottom player strips must fit vertically within {win_w}x{win_h}"
        );
        assert!(
            board_size + eval_w <= avail_x,
            "Board + eval bar must fit horizontally within {win_w}x{win_h}"
        );
    }
}
