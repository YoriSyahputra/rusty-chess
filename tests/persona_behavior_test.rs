use rusty_chess::{
    domain::{parse_san, ChessMatchState, Color, GameOutcome},
    engine::bot::{
        detect_dialogue_trigger, dialogue_bank, select_bot_dialogue, BotId, BotPersona,
        DialogueTrigger,
    },
};

#[test]
fn test_all_six_personas_parameters_and_contrast() {
    let roster = BotPersona::all_roster();
    assert_eq!(roster.len(), 6, "Must have all 6 distinct AI personas");

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

    // Verifikasi kontras bobot kepribadian sesuai SPEC.md
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
    // Pastikan setiap bot memiliki kalimat untuk seluruh DialogueTrigger
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

    // Verifikasi deteksi otomatis blunder pemain (>180 cp drop bagi pemain)
    let trigger = detect_dialogue_trigger(12, 0, 240, false, false, false, 18, false);
    assert_eq!(trigger, Some(DialogueTrigger::PlayerBlunders));

    // Verifikasi deteksi langkah brilian pemain (>120 cp gain bagi pemain)
    let great = detect_dialogue_trigger(14, 100, -60, false, false, false, 16, false);
    assert_eq!(great, Some(DialogueTrigger::PlayerGreatMove));
}

#[test]
fn test_game_state_fools_mate_undo_and_pgn_export() {
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

    let pgn = state.export_pgn("Magnus Carlsen");
    assert!(pgn.contains("[Event \"RustyChess Match\"]"));
    assert!(pgn.contains("1. f3 e5 2. g4 Qh4# 0-1"));

    // Uji Takeback / Undo
    state.undo_move_pair();
    assert_eq!(state.outcome, GameOutcome::Ongoing);
    assert_eq!(state.history.len(), 2);
}
