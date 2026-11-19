use rusty_chess::{
    audio::{synthesize_wav_bytes, SoundEffect},
    config::{SoundTheme, UserConfig},
    domain::{
        chess960_fen, parse_fen, scharnagl_back_rank, ChessMatchState, Color, Position,
    },
    engine::{
        bot::BotId,
        search::compute_multipv_lines,
        uci_bridge::{parse_uci_depth, parse_uci_score_cp},
    },
    ui::{
        pages::ENDGAME_DRILLS,
        piece_renderer::BotMood,
    },
};

#[test]
fn test_chess960_all_960_scharnagl_positions_are_valid() {
    // Classic standard starting position is Scharnagl ID #518
    assert_eq!(scharnagl_back_rank(518), "RNBQKBNR");

    let mut seen = std::collections::HashSet::new();
    for id in 0..960u16 {
        let back = scharnagl_back_rank(id);
        assert_eq!(back.len(), 8);
        assert!(
            seen.insert(back.clone()),
            "Duplicate Chess960 back rank at id {id}: {back}"
        );

        // King must be strictly between both Rooks
        let chars: Vec<char> = back.chars().collect();
        let rooks: Vec<usize> = chars
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| (c == 'R').then_some(i))
            .collect();
        let king = chars.iter().position(|&c| c == 'K').unwrap();
        assert_eq!(rooks.len(), 2);
        assert!(rooks[0] < king && king < rooks[1], "ID {id}: {back}");

        // Bishops must stand on opposite-colored squares
        let bishops: Vec<usize> = chars
            .iter()
            .enumerate()
            .filter_map(|(i, &c)| (c == 'B').then_some(i))
            .collect();
        assert_eq!(bishops.len(), 2);
        assert_ne!(bishops[0] % 2, bishops[1] % 2, "ID {id}: {back}");

        // Verify shakmaty accepts the Chess960 FEN and generates legal moves
        let state = ChessMatchState::from_chess960(id, Color::White)
            .unwrap_or_else(|e| panic!("ID {id} failed ({}): {e}", chess960_fen(id)));
        assert!(!state.current_pos.legal_moves().is_empty());
    }
    assert_eq!(seen.len(), 960);
}

#[test]
fn test_all_endgame_drills_are_legal_and_playable() {
    assert_eq!(ENDGAME_DRILLS.len(), 8);
    for drill in ENDGAME_DRILLS {
        let pos = parse_fen(drill.fen)
            .unwrap_or_else(|e| panic!("Endgame drill #{} ({}) failed: {e}", drill.id, drill.title));
        assert!(
            !pos.is_game_over(),
            "Endgame drill #{} should not start in game-over state",
            drill.id
        );
        assert!(
            !pos.legal_moves().is_empty(),
            "Endgame drill #{} must have legal moves",
            drill.id
        );
    }
}

#[test]
fn test_multipv_top3_lines_and_uci_parsers() {
    let start_pos = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
    let lines = compute_multipv_lines(&start_pos, 3, 2);
    assert_eq!(lines.len(), 3);
    assert!(lines[0].eval_cp_white >= lines[1].eval_cp_white);
    assert!(lines[1].eval_cp_white >= lines[2].eval_cp_white);
    assert_ne!(lines[0].uci, lines[1].uci);
    assert!(!lines[0].pv_sans.is_empty());

    // Mate-in-1 should surface as line #1 with huge positive score
    let mate1 = parse_fen("6k1/5ppp/8/8/8/8/5PPP/4R1K1 w - - 0 1").unwrap();
    let mate_lines = compute_multipv_lines(&mate1, 3, 2);
    assert!(!mate_lines.is_empty());
    assert_eq!(mate_lines[0].uci, "e1e8");
    assert!(mate_lines[0].eval_cp_white > 20_000);

    // External UCI info parser
    let info_cp = "info depth 18 seldepth 24 score cp 145 nodes 50000 pv e2e4 e7e5";
    assert_eq!(parse_uci_depth(info_cp), Some(18));
    assert_eq!(parse_uci_score_cp(info_cp, Color::White), Some(145));
    assert_eq!(parse_uci_score_cp(info_cp, Color::Black), Some(-145));

    let info_mate = "info depth 14 score mate 2 pv d1h5 g7g6";
    assert!(parse_uci_score_cp(info_mate, Color::White).unwrap() > 25_000);
    assert!(parse_uci_score_cp(info_mate, Color::Black).unwrap() < -25_000);
}

#[test]
fn test_procedural_audio_synth_bot_mood_and_v1_config() {
    for &theme in SoundTheme::all() {
        for effect in [
            SoundEffect::MovePiece,
            SoundEffect::Capture,
            SoundEffect::Check,
            SoundEffect::Castle,
            SoundEffect::Promote,
            SoundEffect::GameStart,
            SoundEffect::GameOver,
            SoundEffect::LowTimeTick,
        ] {
            let wav = synthesize_wav_bytes(effect, theme);
            assert!(wav.len() > 100, "WAV too small for {theme:?}/{effect:?}");
            assert_eq!(&wav[0..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
        }
    }

    assert_eq!(BotMood::from_eval(BotId::Magnus, 0), BotMood::Neutral);
    assert_eq!(BotMood::from_eval(BotId::Magnus, 350), BotMood::SmugWinning);
    assert_eq!(
        BotMood::from_eval(BotId::Magnus, -350),
        BotMood::SweatingLosing
    );
    assert_eq!(
        BotMood::from_eval(BotId::Mittens, 29_000),
        BotMood::MittensLaserMate
    );

    let cfg = UserConfig::default();
    assert!(!cfg.blindfold_mode);
    assert!(!cfg.use_external_uci);
}
