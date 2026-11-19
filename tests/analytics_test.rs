use rusty_chess::{
    domain::{detect_opening, parse_fen, parse_san},
    engine::{coach::explain_move, review::MoveClassification},
    persistence::UserProfile,
    ui::{board_textures::generate_procedural_board_overlay, board_view::compute_square_control_map, theme::BoardTheme},
};
use shakmaty::{Chess, Square};

#[test]
fn test_eco_opening_detection_longest_prefix() {
    let ruy_berlin: Vec<String> = ["e4", "e5", "Nf3", "Nc6", "Bb5", "Nf6"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let matched = detect_opening(&ruy_berlin).expect("must detect Ruy Lopez Berlin");
    assert_eq!(matched.eco, "C65");
    assert_eq!(matched.name, "Ruy Lopez: Berlin Defense");
    assert_eq!(matched.matched_plies, 6);

    let sicilian_najdorf: Vec<String> = [
        "e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let najdorf = detect_opening(&sicilian_najdorf).expect("must detect Najdorf");
    assert_eq!(najdorf.eco, "B90");
    assert_eq!(najdorf.matched_plies, 10);
}

#[test]
fn test_xray_square_control_and_hanging_piece_detection() {
    let start = Chess::default();
    let map = compute_square_control_map(&start);

    // At start position, no piece is hanging and e3/e6 symmetry holds
    for sq in Square::ALL {
        assert!(!map[sq as usize].is_hanging);
    }
    assert_eq!(map[Square::D3 as usize].white_attackers, map[Square::D6 as usize].black_attackers);

    // Undefended Black Queen on d5 attacked by White Knight on c3 must be flagged as hanging
    let pos = parse_fen("rnb1kbnr/ppp1pppp/8/3q4/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 0 1")
        .expect("valid FEN");
    let map2 = compute_square_control_map(&pos);
    assert!(
        map2[Square::D5 as usize].is_hanging,
        "Undefended Queen on d5 attacked by Nc3 must be flagged as hanging"
    );
}

#[test]
fn test_mistake_vault_spaced_repetition_and_chess_dna() {
    let mut profile = UserProfile::default();
    profile.wins = 6;
    profile.losses = 2;
    profile.puzzle_rating = 1650;
    profile.record_opening_result("C65", "Ruy Lopez: Berlin Defense", Some(true));
    profile.record_opening_result("C65", "Ruy Lopez: Berlin Defense", Some(true));

    assert_eq!(profile.opening_stats.len(), 1);
    let st = profile.opening_stats.values().next().unwrap();
    assert!((st.win_rate_pct() - 100.0).abs() < 0.01);

    profile.add_mistake_entry(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
        true,
        "f3".to_string(),
        "e4".to_string(),
        "e2e4".to_string(),
        "Blunder".to_string(),
        "Magnus".to_string(),
    );
    assert_eq!(profile.active_mistakes_count(), 1);

    let id = profile.mistake_vault[0].id;
    profile.record_mistake_drill_outcome(id, true);
    profile.record_mistake_drill_outcome(id, true);
    assert_eq!(profile.active_mistakes_count(), 1);
    profile.record_mistake_drill_outcome(id, true);
    assert_eq!(profile.active_mistakes_count(), 0);
    assert!(profile.mistake_vault[0].is_mastered());

    let dna = profile.compute_skill_dna();
    assert!((0.2..=1.0).contains(&dna.opening));
    assert!((0.2..=1.0).contains(&dna.tactics));
    assert!(!dna.archetype.is_empty());
}

#[test]
fn test_natural_language_coach_and_procedural_board_textures() {
    let pos = parse_fen("rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2")
        .expect("valid FEN");
    let blunder_mv = parse_san(&pos, "Qh5").expect("legal");
    let best_mv = parse_san(&pos, "Nf3").expect("legal");

    let exp = explain_move(&pos, &blunder_mv, &best_mv, MoveClassification::Mistake);
    assert!(!exp.headline.is_empty());
    assert!(!exp.detail.is_empty());
    assert!(!exp.refutation_sans.is_empty());

    for &theme in BoardTheme::all() {
        let img = generate_procedural_board_overlay(theme, 64);
        assert_eq!(img.size, [64, 64]);
        assert_eq!(img.pixels.len(), 64 * 64);
    }
}
