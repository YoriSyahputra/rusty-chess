use crate::{
    config::UserConfig,
    domain::{format_uci, parse_uci, AnnotationColor, ChessMatchState, Color, TacticalArrow},
    persistence::UserProfile,
    ui::{board_view::BoardInteractionState, theme::UiTokens},
};
use egui::{Color32, RichText, Rounding, ScrollArea, Stroke, Ui};
use shakmaty::Position;

#[derive(Debug, Clone, Copy)]
pub struct TacticalPuzzle {
    pub id: usize,
    pub title: &'static str,
    pub theme: &'static str,
    pub rating: u32,
    pub fen: &'static str,
    pub solution_uci: &'static [&'static str],
}

pub const CURATED_PUZZLES: &[TacticalPuzzle] = &[
    TacticalPuzzle {
        id: 1,
        title: "Daily Spotlight: Back-Rank Sacrifice",
        theme: "Deflection & Mate in 2",
        rating: 1200,
        fen: "r5k1/5ppp/8/8/8/8/4RPPP/4R1K1 w - - 0 1",
        solution_uci: &["e2e8", "a8e8", "e1e8"],
    },
    TacticalPuzzle {
        id: 2,
        title: "Scholar's Strike",
        theme: "Mate in 1",
        rating: 600,
        fen: "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4",
        solution_uci: &["h5f7"],
    },
    TacticalPuzzle {
        id: 3,
        title: "Corridor Execution",
        theme: "Back-Rank Mate in 1",
        rating: 700,
        fen: "6k1/5ppp/8/8/8/8/5PPP/4R1K1 w - - 0 1",
        solution_uci: &["e1e8"],
    },
    TacticalPuzzle {
        id: 4,
        title: "Pinned f7 Battery",
        theme: "Pin & Mate in 2",
        rating: 1250,
        fen: "r5k1/5ppp/8/8/2B5/8/4QPPP/4R1K1 w - - 0 1",
        solution_uci: &["e2e8", "a8e8", "e1e8"],
    },
    TacticalPuzzle {
        id: 5,
        title: "Undefended Monarch's Guard",
        theme: "Winning Queen Capture",
        rating: 850,
        fen: "rnb1kbnr/ppp1pppp/8/3q4/8/2N5/PPPP1PPP/R1BQKBNR w KQkq - 0 1",
        solution_uci: &["c3d5"],
    },
    TacticalPuzzle {
        id: 6,
        title: "Central Rook Harvest",
        theme: "Avoiding Horizon Trap",
        rating: 1100,
        fen: "rnbqkbn1/pp2pppp/2p5/3pr3/8/5N2/PPP1PPPP/RN1QKB1R w KQq - 0 1",
        solution_uci: &["f3e5"],
    },
    TacticalPuzzle {
        id: 7,
        title: "Arabian Corner Mate",
        theme: "Rook + Knight Coordination",
        rating: 1150,
        fen: "7k/7p/5N2/8/8/8/5P1P/6RK w - - 0 1",
        solution_uci: &["g1g8"],
    },
    TacticalPuzzle {
        id: 8,
        title: "Smothered Royal Trap",
        theme: "Smothered Mate in 1",
        rating: 1300,
        fen: "6rk/6pp/8/6N1/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["g5f7"],
    },
    TacticalPuzzle {
        id: 9,
        title: "Royal Family Fork",
        theme: "Knight Fork on c7",
        rating: 950,
        fen: "r3k2r/ppp2ppp/8/3N4/8/8/PPP2PPP/R3K2R w KQkq - 0 1",
        solution_uci: &["d5c7"],
    },
    TacticalPuzzle {
        id: 10,
        title: "Anastasia's Net",
        theme: "Mate in 1",
        rating: 1200,
        fen: "5r1k/4Nppp/8/8/8/8/5PP1/6KR w - - 0 1",
        solution_uci: &["h1h7"],
    },
    TacticalPuzzle {
        id: 11,
        title: "Morphy's Opera Finale",
        theme: "Queen Sacrifice & Mate in 2",
        rating: 1150,
        fen: "4kb1r/p2n1ppp/4q3/4p1B1/4P3/1Q6/PPP2PPP/2KR4 w k - 0 1",
        solution_uci: &["b3b8", "d7b8", "d1d8"],
    },
    TacticalPuzzle {
        id: 12,
        title: "Epaulette Checkmate",
        theme: "Queen Mate in 1",
        rating: 1000,
        fen: "3rkr2/8/Q7/8/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["a6e6"],
    },
    TacticalPuzzle {
        id: 13,
        title: "Boden's Crisscross",
        theme: "Double Bishop Mate",
        rating: 1350,
        fen: "2kr3r/p2n1ppp/1p6/8/1bB5/8/PPP1BPPP/2KR4 w - - 0 1",
        solution_uci: &["c4a6"],
    },
    TacticalPuzzle {
        id: 14,
        title: "Damiano's Kiss",
        theme: "Queen + Pawn Mate",
        rating: 900,
        fen: "6k1/5p1p/6pQ/8/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["h6g7"],
    },
    TacticalPuzzle {
        id: 15,
        title: "Blackburne's Diagonal",
        theme: "Bishop + Queen Battery",
        rating: 1100,
        fen: "5rk1/5p1p/6p1/8/8/1B6/5QPP/6K1 w - - 0 1",
        solution_uci: &["f2f7"],
    },
    TacticalPuzzle {
        id: 16,
        title: "Grecian Gift Strike",
        theme: "King Hunt Mate in 1",
        rating: 1150,
        fen: "r1bq1r1k/ppp2p1p/2np4/4p1NQ/2B1P3/8/PPPP1PPP/RNB1K2R w KQ - 0 1",
        solution_uci: &["h5h7"],
    },
    TacticalPuzzle {
        id: 17,
        title: "Queen & Knight Fork on f7",
        theme: "Tactical Knockout",
        rating: 1000,
        fen: "r1bqk2r/pppp1ppp/2n2n2/2b1p1N1/2B1P3/8/PPPP1PPP/RNBQK2R w KQkq - 4 5",
        solution_uci: &["g5f7"],
    },
    TacticalPuzzle {
        id: 18,
        title: "Seventh Rank Piglets",
        theme: "Double Rook Mate",
        rating: 950,
        fen: "6k1/1R6/8/8/8/8/5PPP/2R3K1 w - - 0 1",
        solution_uci: &["c1c8"],
    },
    TacticalPuzzle {
        id: 19,
        title: "Lolli's Outpost",
        theme: "Pawn + Queen Mate",
        rating: 1050,
        fen: "6k1/5p2/5PpQ/8/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["h6g7"],
    },
    TacticalPuzzle {
        id: 20,
        title: "Queen Skewer on the Diagonal",
        theme: "Diagonal Mate",
        rating: 1100,
        fen: "6k1/5p2/6p1/8/8/2Q5/6PP/6K1 w - - 0 1",
        solution_uci: &["c3h8"],
    },
    TacticalPuzzle {
        id: 21,
        title: "Promotion with Check",
        theme: "Under-pressure Promotion",
        rating: 1150,
        fen: "8/4P1k1/8/8/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["e7e8q"],
    },
    TacticalPuzzle {
        id: 22,
        title: "Back-Rank Queen Promotion",
        theme: "Mate by Promotion",
        rating: 1200,
        fen: "6k1/4Pppp/8/8/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["e7e8q"],
    },
    TacticalPuzzle {
        id: 23,
        title: "Knight Royal Fork on e6",
        theme: "Fork King & Queen",
        rating: 1300,
        fen: "5k2/2q2ppp/8/8/3N4/8/5PPP/6K1 w - - 0 1",
        solution_uci: &["d4e6"],
    },
    TacticalPuzzle {
        id: 24,
        title: "Legal's Mate Finale",
        theme: "Minor Piece Mate",
        rating: 1400,
        fen: "r2qkbnr/ppp2ppp/2np4/4N3/2B1P1b1/2N5/PPPP1PPP/R1BQK2R w KQkq - 0 1",
        solution_uci: &["c4f7"],
    },
    TacticalPuzzle {
        id: 25,
        title: "Hook Mate Pattern",
        theme: "Rook + Knight + Pawn",
        rating: 1350,
        fen: "3R4/4kp1p/5N2/4P3/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["d8e8"],
    },
    TacticalPuzzle {
        id: 26,
        title: "Swallow's Tail Mate",
        theme: "Queen + Rook Support",
        rating: 1400,
        fen: "3r1r2/4k3/Q3p3/8/8/8/6PP/4R1K1 w - - 0 1",
        solution_uci: &["a6e6"],
    },
    TacticalPuzzle {
        id: 27,
        title: "Blind Swine Exchange Mate",
        theme: "Rook Sacrifice & Mate in 2",
        rating: 1450,
        fen: "2R2rk1/5ppp/8/8/8/8/4RPPP/6K1 w - - 0 1",
        solution_uci: &["c8f8", "g8f8", "e2e8"],
    },
    TacticalPuzzle {
        id: 28,
        title: "Cornered King Execution",
        theme: "Bishop + Knight Mate",
        rating: 1500,
        fen: "7k/7p/8/5BN1/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["g5f7"],
    },
    TacticalPuzzle {
        id: 29,
        title: "Dovetail Checkmate",
        theme: "Queen Net",
        rating: 1550,
        fen: "8/8/1q1k4/2p5/2Q5/2K5/8/8 w - - 0 1",
        solution_uci: &["c4d5"],
    },
    TacticalPuzzle {
        id: 30,
        title: "Mayet's Rook Strike",
        theme: "Bishop + Rook Mate",
        rating: 1600,
        fen: "6k1/5p2/6p1/1B6/8/8/5PP1/6KR w - - 0 1",
        solution_uci: &["h1h8"],
    },
    TacticalPuzzle {
        id: 31,
        title: "Greco's H-File Mate",
        theme: "Bishop + Rook Coordination",
        rating: 1650,
        fen: "5rk1/5p1p/8/3B4/8/8/5PP1/6KR w - - 0 1",
        solution_uci: &["h1h7"],
    },
    TacticalPuzzle {
        id: 32,
        title: "Grandmaster Endgame Precision",
        theme: "Opposition & Queening",
        rating: 1800,
        fen: "8/2P2k2/8/8/8/8/6PP/6K1 w - - 0 1",
        solution_uci: &["c7c8q"],
    },
];

#[derive(Debug, Clone)]
pub struct PuzzlesTrainerState {
    pub current_idx: usize,
    pub step_idx: usize,
    pub hearts: u8,
    pub max_hearts: u8,
    pub hard_mode: bool,
    pub solved_current: bool,
    pub failed_out_of_hearts: bool,
    pub status_message: String,
    pub status_is_positive: bool,
    pub board_state: ChessMatchState,
    pub board_ui: BoardInteractionState,
}

impl Default for PuzzlesTrainerState {
    fn default() -> Self {
        let mut s = Self {
            current_idx: 0,
            step_idx: 0,
            hearts: 5,
            max_hearts: 5,
            hard_mode: false,
            solved_current: false,
            failed_out_of_hearts: false,
            status_message: "Find the best tactical continuation!".to_string(),
            status_is_positive: true,
            board_state: ChessMatchState::new_standard(Color::White),
            board_ui: BoardInteractionState::default(),
        };
        s.load_puzzle(0);
        s
    }
}

impl PuzzlesTrainerState {
    pub fn load_puzzle(&mut self, idx: usize) {
        let clamped = idx % CURATED_PUZZLES.len();
        self.current_idx = clamped;
        self.step_idx = 0;
        self.solved_current = false;
        let puzzle = &CURATED_PUZZLES[clamped];
        if let Ok(state) = ChessMatchState::from_fen(puzzle.fen, Color::White) {
            let turn = state.current_pos.turn();
            if let Ok(aligned) = ChessMatchState::from_fen(puzzle.fen, turn) {
                self.board_state = aligned;
            } else {
                self.board_state = state;
            }
        }
        self.board_ui.reset();
        self.status_message = format!("Puzzle #{}: {} — Find the winning move!", puzzle.id, puzzle.title);
        self.status_is_positive = true;
    }

    pub fn reset_hearts(&mut self) {
        self.hearts = self.max_hearts;
        self.failed_out_of_hearts = false;
        self.load_puzzle(self.current_idx);
    }
}

/// Merender halaman Offline Tactics Trainer & Daily Puzzle (`Route::Puzzles`)
/// lengkap dengan sistem 5-Heart, Hard Mode toggle, dan 32 puzzle terkurasi.
pub fn render_puzzles_page(
    ui: &mut Ui,
    trainer: &mut PuzzlesTrainerState,
    profile: &mut UserProfile,
    config: &UserConfig,
) {
    let tokens = UiTokens::for_mode(config.dark_mode);
    let puzzle = &CURATED_PUZZLES[trainer.current_idx];

    egui::SidePanel::right("puzzles_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(10.0);

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(format!("🧩 Puzzle #{}", puzzle.id))
                                .strong()
                                .size(17.0)
                                .color(tokens.text_primary),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                RichText::new(format!("Rating: {}", profile.puzzle_rating))
                                    .strong()
                                    .size(13.0)
                                    .color(tokens.accent_gold),
                            );
                        });
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(puzzle.title)
                            .strong()
                            .size(14.0)
                            .color(tokens.text_primary),
                    );
                    if !trainer.hard_mode {
                        ui.label(
                            RichText::new(format!(
                                "Theme: {} • Target ELO: {}",
                                puzzle.theme, puzzle.rating
                            ))
                            .size(12.0)
                            .color(tokens.text_secondary),
                        );
                    } else {
                        ui.label(
                            RichText::new("🔥 Hard Mode Active (Theme hidden, -2 Hearts on error)")
                                .size(11.5)
                                .color(Color32::from_rgb(245, 115, 75)),
                        );
                    }

                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        let mut hearts_str = String::new();
                        for i in 0..trainer.max_hearts {
                            if i < trainer.hearts {
                                hearts_str.push_str("❤️ ");
                            } else {
                                hearts_str.push_str("🖤 ");
                            }
                        }
                        ui.label(RichText::new(hearts_str).size(15.0));

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.checkbox(&mut trainer.hard_mode, "Hard Mode");
                        });
                    });
                });

            ui.add_space(8.0);

            let banner_border = if trainer.status_is_positive {
                tokens.accent_primary
            } else {
                Color32::from_rgb(0xE0, 0x4F, 0x44)
            };
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.5_f32, banner_border))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(&trainer.status_message)
                            .strong()
                            .size(13.0)
                            .color(tokens.text_primary),
                    );
                });

            ui.add_space(8.0);

            ui.horizontal(|ui| {
                let can_hint = !trainer.solved_current
                    && !trainer.failed_out_of_hearts
                    && !trainer.hard_mode;
                if ui
                    .add_enabled(
                        can_hint,
                        egui::Button::new(RichText::new("💡 Hint").size(12.5))
                            .min_size([96.0, 34.0].into()),
                    )
                    .clicked()
                {
                    if let Some(&uci_str) = puzzle.solution_uci.get(trainer.step_idx) {
                        if let Ok(mv) = parse_uci(&trainer.board_state.current_pos, uci_str) {
                            if let Some(from) = mv.from() {
                                trainer.board_ui.hint_arrow = Some(TacticalArrow::new(
                                    from,
                                    mv.to(),
                                    AnnotationColor::Green,
                                ));
                            }
                        }
                    }
                }

                if ui
                    .add_sized([110.0, 34.0], egui::Button::new("🔄 Retry"))
                    .clicked()
                {
                    if trainer.failed_out_of_hearts {
                        trainer.reset_hearts();
                    } else {
                        let idx = trainer.current_idx;
                        trainer.load_puzzle(idx);
                    }
                }

                if ui
                    .add_sized(
                        [126.0, 34.0],
                        egui::Button::new(
                            RichText::new("Next Puzzle ⏭")
                                .strong()
                                .color(Color32::WHITE),
                        )
                        .fill(tokens.accent_primary),
                    )
                    .clicked()
                {
                    let next = (trainer.current_idx + 1) % CURATED_PUZZLES.len();
                    trainer.load_puzzle(next);
                }
            });

            ui.add_space(10.0);

            ui.label(
                RichText::new(format!(
                    "Curated Tactics Library ({} Solved • Streak: {})",
                    profile.puzzles_solved, profile.puzzle_streak
                ))
                .strong()
                .size(13.0)
                .color(tokens.text_primary),
            );
            ui.add_space(4.0);

            let list_h = (ui.available_height() - 16.0).max(120.0);
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .id_salt("puzzles_library_scroll")
                        .max_height(list_h)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let mut select_idx: Option<usize> = None;
                            for (i, p) in CURATED_PUZZLES.iter().enumerate() {
                                let is_cur = trainer.current_idx == i;
                                let label = format!("#{:02}  {} ({})", p.id, p.title, p.rating);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 24.0],
                                        egui::SelectableLabel::new(
                                            is_cur,
                                            RichText::new(label).size(12.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    select_idx = Some(i);
                                }
                            }
                            if let Some(i) = select_idx {
                                trainer.load_puzzle(i);
                            }
                        });
                });
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let avail = ui.available_size();
        let board_size = (avail.x - 24.0).min(avail.y - 48.0).max(320.0);
        let pad_x = ((avail.x - board_size) * 0.5).max(8.0);
        let pad_y = ((avail.y - board_size) * 0.5).max(8.0);

        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            if !trainer.solved_current && !trainer.failed_out_of_hearts {
                if let Some(mv) = trainer.board_ui.ui_with_config(
                    ui,
                    board_size,
                    &mut trainer.board_state,
                    config,
                ) {
                    let played_uci = format_uci(&mv);
                    let expected_uci = puzzle
                        .solution_uci
                        .get(trainer.step_idx)
                        .copied()
                        .unwrap_or("");

                    let mut test_pos = trainer.board_state.current_pos.clone();
                    test_pos.play_unchecked(&mv);
                    let is_mate_move = test_pos.is_checkmate();

                    if played_uci == expected_uci || is_mate_move {
                        let prev_pos = trainer.board_state.current_pos.clone();
                        let _ = trainer.board_state.apply_move(mv.clone(), 0, 100);
                        trainer.board_ui.anim_layer.trigger_move_animation(
                            &prev_pos,
                            &trainer.board_state.current_pos,
                            &mv,
                            trainer.board_state.outcome,
                            config.animation_profile,
                        );
                        trainer.step_idx += 1;

                        if trainer.step_idx < puzzle.solution_uci.len() && !is_mate_move {
                            if let Some(&reply_uci) = puzzle.solution_uci.get(trainer.step_idx) {
                                if let Ok(reply_mv) =
                                    parse_uci(&trainer.board_state.current_pos, reply_uci)
                                {
                                    let b_pos = trainer.board_state.current_pos.clone();
                                    let _ =
                                        trainer.board_state.apply_move(reply_mv.clone(), 0, 100);
                                    trainer.board_ui.anim_layer.trigger_move_animation(
                                        &b_pos,
                                        &trainer.board_state.current_pos,
                                        &reply_mv,
                                        trainer.board_state.outcome,
                                        config.animation_profile,
                                    );
                                    trainer.step_idx += 1;
                                }
                            }
                        }

                        if trainer.step_idx >= puzzle.solution_uci.len() || is_mate_move {
                            trainer.solved_current = true;
                            profile.puzzles_solved += 1;
                            profile.puzzle_streak += 1;
                            profile.puzzle_rating = (profile.puzzle_rating + 15).min(3200);
                            if trainer.current_idx == 0 {
                                profile.daily_puzzle_completed = true;
                            }
                            profile.save();
                            trainer.status_message =
                                "✓ Puzzle Solved! +15 Puzzle Rating!".to_string();
                            trainer.status_is_positive = true;
                        } else {
                            trainer.status_message =
                                "✓ Great first move! Now find the finishing blow!".to_string();
                            trainer.status_is_positive = true;
                        }
                    } else {
                        let penalty = if trainer.hard_mode { 2 } else { 1 };
                        trainer.hearts = trainer.hearts.saturating_sub(penalty);
                        profile.puzzle_streak = 0;
                        profile.puzzle_rating = profile.puzzle_rating.saturating_sub(10).max(400);
                        profile.save();

                        if trainer.hearts == 0 {
                            trainer.failed_out_of_hearts = true;
                            trainer.status_message =
                                "💔 Out of Hearts! Click Retry to restore 5 hearts.".to_string();
                        } else {
                            trainer.status_message = format!(
                                "✗ Not the best tactic (-{penalty} ❤️). Try another move!"
                            );
                        }
                        trainer.status_is_positive = false;
                    }
                }
            } else {
                let _ = trainer.board_ui.ui_with_config(
                    ui,
                    board_size,
                    &mut trainer.board_state,
                    config,
                );
            }
        });
    });
}
