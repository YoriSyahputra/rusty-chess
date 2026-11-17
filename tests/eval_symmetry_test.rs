use rusty_chess::{
    domain::parse_fen,
    engine::eval::{
        evaluate, evaluate_for_white,
        material::{compute_game_phase, evaluate_material, BISHOP_PAIR_BONUS},
        pst::side_pst_scores,
        EvalWeights,
    },
};
use shakmaty::{Chess, Color, Position};

#[test]
fn test_initial_position_is_exact_zero() {
    let pos = Chess::default();
    let weights = EvalWeights::default();

    let white_score = evaluate_for_white(&pos, &weights);
    let stm_score = evaluate(&pos, &weights);

    assert_eq!(
        white_score, 0,
        "Standard initial position must evaluate to 0 cp"
    );
    assert_eq!(stm_score, 0, "Side-to-move score at start must be 0 cp");
    assert_eq!(
        compute_game_phase(pos.board()),
        24,
        "Initial position must have maximum game phase 24"
    );
}

#[test]
fn test_mirror_position_symmetry() {
    let weights = EvalWeights::default();

    // Posisi 1: Putih unggul 1 Kuda dan kontrol pusat, vs Posisi 2: Hitam unggul identik secara vertikal
    let fen_white_up =
        "r1bqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 0 3";
    let fen_black_up =
        "rnbqkb1r/pppp1ppp/5n2/4p3/4P3/8/PPPP1PPP/R1BQKBNR b KQkq - 0 3";

    let pos_w = parse_fen(fen_white_up).expect("valid FEN");
    let pos_b = parse_fen(fen_black_up).expect("valid FEN");

    let eval_w_white_persp = evaluate_for_white(&pos_w, &weights);
    let eval_b_white_persp = evaluate_for_white(&pos_b, &weights);

    assert!(
        eval_w_white_persp > 300,
        "White up a knight should be > +300 cp, got {eval_w_white_persp}"
    );
    assert_eq!(
        eval_w_white_persp, -eval_b_white_persp,
        "Mirrored position must have exact opposite White-perspective score"
    );

    // Dalam skema Negamax (`evaluate`), kedua sisi yang sedang jalan sama-sama unggul dengan nilai identik
    assert_eq!(
        evaluate(&pos_w, &weights),
        evaluate(&pos_b, &weights),
        "Negamax evaluate() must return identical score for mirrored side-to-move"
    );
}

#[test]
fn test_bishop_pair_bonus() {
    // Putih memiliki sepasang Gajah (c1, f1), Hitam memiliki Gajah + Kuda
    let fen = "4k3/8/8/8/8/8/8/2B1KB2 w - - 0 1";
    let pos = parse_fen(fen).expect("valid FEN");
    let mat_diff = evaluate_material(pos.board());

    // 2 Bishops (330 * 2 = 660) + Bishop Pair Bonus (35) = 695
    assert_eq!(mat_diff, 330 * 2 + BISHOP_PAIR_BONUS);
}

#[test]
fn test_king_pst_midgame_vs_endgame_transition() {
    // Raja Putih di tengah papan (e4) vs di pojok rokade (g1)
    let pos_center = parse_fen("4k3/8/8/8/4K3/8/8/8 w - - 0 1").expect("valid FEN");
    let pos_corner = parse_fen("4k3/8/8/8/8/8/8/6K1 w - - 0 1").expect("valid FEN");

    let (mg_center, eg_center) = side_pst_scores(pos_center.board(), Color::White);
    let (mg_corner, eg_corner) = side_pst_scores(pos_corner.board(), Color::White);

    assert!(
        mg_corner > mg_center,
        "In Midgame, castled King on g1 ({mg_corner}) must score higher than exposed King on e4 ({mg_center})"
    );
    assert!(
        eg_center > eg_corner,
        "In Endgame, centralized King on e4 ({eg_center}) must score higher than passive King on g1 ({eg_corner})"
    );
}
