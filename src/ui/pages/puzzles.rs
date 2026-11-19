use crate::{
    config::UserConfig,
    domain::{format_uci, parse_uci, AnnotationColor, ChessMatchState, Color, TacticalArrow},
    persistence::UserProfile,
    ui::{board_view::BoardInteractionState, theme::UiTokens},
};
use egui::{Color32, RichText, Rounding, ScrollArea, Stroke, Ui};
use shakmaty::Position;
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PuzzleTabMode {
    Curated,
    PuzzleRush,
    BlunderDeck,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RushVariant {
    Timed3Min,
    Survival,
}

impl RushVariant {
    pub fn label(self) -> &'static str {
        match self {
            RushVariant::Timed3Min => "⏱ 3-Min Timed",
            RushVariant::Survival => "💀 3-Strike Survival",
        }
    }
}

pub struct PuzzlesTrainerState {
    pub tab_mode: PuzzleTabMode,
    pub current_idx: usize,
    pub active_mistake_id: Option<u64>,
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

    // Puzzle Rush state (Feature #20)
    pub rush_variant: RushVariant,
    pub rush_active: bool,
    pub rush_finished: bool,
    pub rush_score: u32,
    pub rush_strikes: u8,
    pub rush_started_at: Option<Instant>,
    pub rush_order: Vec<usize>,
    pub rush_cursor: usize,
}

impl Default for PuzzlesTrainerState {
    fn default() -> Self {
        let mut s = Self {
            tab_mode: PuzzleTabMode::Curated,
            current_idx: 0,
            active_mistake_id: None,
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
            rush_variant: RushVariant::Timed3Min,
            rush_active: false,
            rush_finished: false,
            rush_score: 0,
            rush_strikes: 0,
            rush_started_at: None,
            rush_order: Vec::new(),
            rush_cursor: 0,
        };
        s.load_puzzle(0);
        s
    }
}

impl PuzzlesTrainerState {
    pub fn open_puzzle_rush(&mut self) {
        self.tab_mode = PuzzleTabMode::PuzzleRush;
        if !self.rush_active && !self.rush_finished {
            self.start_puzzle_rush(self.rush_variant);
        }
    }

    pub fn start_puzzle_rush(&mut self, variant: RushVariant) {
        self.tab_mode = PuzzleTabMode::PuzzleRush;
        self.rush_variant = variant;
        self.rush_active = true;
        self.rush_finished = false;
        self.rush_score = 0;
        self.rush_strikes = 0;
        self.rush_started_at = Some(Instant::now());

        let mut order: Vec<usize> = (0..CURATED_PUZZLES.len()).collect();
        order.sort_by_key(|&i| CURATED_PUZZLES[i].rating);
        self.rush_order = order;
        self.rush_cursor = 0;

        let first_idx = self.rush_order.first().copied().unwrap_or(0);
        self.load_puzzle(first_idx);
        let p = &CURATED_PUZZLES[first_idx];
        self.status_message = format!(
            "⚡ Rush started ({})! Solve fast — ELO {}!",
            variant.label(),
            p.rating
        );
        self.status_is_positive = true;
    }

    pub fn rush_remaining_secs(&self) -> f32 {
        if let Some(started) = self.rush_started_at {
            (180.0 - started.elapsed().as_secs_f32()).max(0.0)
        } else {
            180.0
        }
    }

    pub fn advance_rush_puzzle(&mut self) {
        if self.rush_order.is_empty() {
            return;
        }
        self.rush_cursor = (self.rush_cursor + 1) % self.rush_order.len();
        let next_idx = self.rush_order[self.rush_cursor];
        self.load_puzzle(next_idx);
    }

    pub fn finish_rush(&mut self, profile: &mut UserProfile, reason: &str) {
        self.rush_active = false;
        self.rush_finished = true;
        match self.rush_variant {
            RushVariant::Timed3Min => {
                if self.rush_score > profile.puzzle_rush_best_timed {
                    profile.puzzle_rush_best_timed = self.rush_score;
                    profile.save();
                }
            }
            RushVariant::Survival => {
                if self.rush_score > profile.puzzle_rush_best_survival {
                    profile.puzzle_rush_best_survival = self.rush_score;
                    profile.save();
                }
            }
        }
        self.status_message = format!("🏁 {reason} Final Score: {}", self.rush_score);
        self.status_is_positive = self.rush_score > 0;
    }

    pub fn open_blunder_deck(&mut self, profile: &UserProfile) {
        self.tab_mode = PuzzleTabMode::BlunderDeck;
        self.load_next_blunder(profile);
    }

    pub fn load_next_blunder(&mut self, profile: &UserProfile) {
        self.solved_current = false;
        self.step_idx = 0;
        let next_entry = profile
            .mistake_vault
            .iter()
            .find(|m| !m.is_mastered())
            .or_else(|| profile.mistake_vault.first());

        if let Some(entry) = next_entry {
            self.active_mistake_id = Some(entry.id);
            let side = if entry.side_is_white {
                Color::White
            } else {
                Color::Black
            };
            if let Ok(state) = ChessMatchState::from_fen(&entry.fen, side) {
                self.board_state = state;
            }
            self.board_ui.reset();
            self.status_message = format!(
                "Blunder vs {}: You played {}. Find the better move! (Mastery {}/3)",
                entry.opponent_name, entry.played_san, entry.mastery_streak
            );
            self.status_is_positive = true;
        } else {
            self.active_mistake_id = None;
            self.status_message =
                "Mistake Vault is empty! Complete matches to auto-harvest your blunders."
                    .to_string();
            self.status_is_positive = true;
        }
    }

    pub fn load_mistake_by_id(&mut self, profile: &UserProfile, id: u64) {
        if let Some(entry) = profile.mistake_vault.iter().find(|m| m.id == id) {
            self.tab_mode = PuzzleTabMode::BlunderDeck;
            self.active_mistake_id = Some(entry.id);
            self.solved_current = false;
            self.step_idx = 0;
            let side = if entry.side_is_white {
                Color::White
            } else {
                Color::Black
            };
            if let Ok(state) = ChessMatchState::from_fen(&entry.fen, side) {
                self.board_state = state;
            }
            self.board_ui.reset();
            self.status_message = format!(
                "Blunder vs {}: You played {}. Find the best move! (Mastery {}/3)",
                entry.opponent_name, entry.played_san, entry.mastery_streak
            );
            self.status_is_positive = true;
        }
    }
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

    // Tick 3-Minute Timed Puzzle Rush clock if active
    if trainer.tab_mode == PuzzleTabMode::PuzzleRush
        && trainer.rush_active
        && trainer.rush_variant == RushVariant::Timed3Min
    {
        if trainer.rush_remaining_secs() <= 0.0 {
            trainer.finish_rush(profile, "Time's Up!");
        } else {
            ui.ctx()
                .request_repaint_after(Duration::from_millis(120));
        }
    }

    egui::SidePanel::right("puzzles_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(8.0);

            // Tab Mode Switcher: Curated Tactics vs Puzzle Rush (#20) vs Blunder Deck (#5)
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        trainer.tab_mode == PuzzleTabMode::Curated,
                        RichText::new("🧩 Curated").strong().size(12.0),
                    )
                    .clicked()
                {
                    trainer.tab_mode = PuzzleTabMode::Curated;
                    let idx = trainer.current_idx;
                    trainer.load_puzzle(idx);
                }
                if ui
                    .selectable_label(
                        trainer.tab_mode == PuzzleTabMode::PuzzleRush,
                        RichText::new("⚡ Puzzle Rush").strong().size(12.0),
                    )
                    .clicked()
                {
                    trainer.open_puzzle_rush();
                }
                let vault_label = format!(
                    "🔥 Blunders ({})",
                    profile.active_mistakes_count()
                );
                if ui
                    .selectable_label(
                        trainer.tab_mode == PuzzleTabMode::BlunderDeck,
                        RichText::new(vault_label).strong().size(12.0),
                    )
                    .clicked()
                {
                    trainer.open_blunder_deck(profile);
                }
            });

            ui.add_space(6.0);

            if trainer.tab_mode == PuzzleTabMode::PuzzleRush {
                let cur_p = &CURATED_PUZZLES[trainer.current_idx];
                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.5_f32, tokens.accent_gold))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(12.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new("⚡ Puzzle Rush")
                                .strong()
                                .size(18.0)
                                .color(tokens.text_primary),
                        );
                        ui.label(
                            RichText::new(
                                "Solve escalating tactics under pressure. 3 strikes and you're out!",
                            )
                            .size(11.5)
                            .color(tokens.text_secondary),
                        );
                        ui.add_space(8.0);

                        ui.horizontal(|ui| {
                            ui.selectable_value(
                                &mut trainer.rush_variant,
                                RushVariant::Timed3Min,
                                RushVariant::Timed3Min.label(),
                            );
                            ui.selectable_value(
                                &mut trainer.rush_variant,
                                RushVariant::Survival,
                                RushVariant::Survival.label(),
                            );
                        });

                        ui.add_space(8.0);

                        // Score + Clock + Strikes HUD
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("SCORE")
                                        .size(10.5)
                                        .color(tokens.text_secondary),
                                );
                                ui.label(
                                    RichText::new(format!("{}", trainer.rush_score))
                                        .strong()
                                        .size(30.0)
                                        .color(tokens.accent_primary),
                                );
                            });

                            ui.add_space(20.0);

                            if trainer.rush_variant == RushVariant::Timed3Min {
                                let rem = trainer.rush_remaining_secs();
                                let mins = (rem as u32) / 60;
                                let secs = (rem as u32) % 60;
                                let time_col = if rem < 20.0 && trainer.rush_active {
                                    Color32::from_rgb(0xE0, 0x4F, 0x44)
                                } else {
                                    tokens.text_primary
                                };
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("TIME LEFT")
                                            .size(10.5)
                                            .color(tokens.text_secondary),
                                    );
                                    ui.label(
                                        RichText::new(format!("{mins:02}:{secs:02}"))
                                            .monospace()
                                            .strong()
                                            .size(26.0)
                                            .color(time_col),
                                    );
                                });
                            }

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    let mut strikes_txt = String::new();
                                    for i in 0..3u8 {
                                        if i < trainer.rush_strikes {
                                            strikes_txt.push_str("❌");
                                        } else {
                                            strikes_txt.push_str("⬜");
                                        }
                                    }
                                    ui.label(RichText::new(strikes_txt).size(18.0));
                                },
                            );
                        });

                        ui.add_space(8.0);
                        ui.label(
                            RichText::new(format!(
                                "Current Puzzle Target: ELO {}  •  #{}",
                                cur_p.rating, cur_p.id
                            ))
                            .size(11.5)
                            .color(tokens.accent_gold),
                        );

                        ui.add_space(8.0);
                        let btn_text = if trainer.rush_active {
                            "🔄 Restart Rush"
                        } else {
                            "▶ Start New Rush"
                        };
                        if ui
                            .add_sized(
                                [ui.available_width(), 36.0],
                                egui::Button::new(
                                    RichText::new(btn_text)
                                        .strong()
                                        .size(14.0)
                                        .color(Color32::WHITE),
                                )
                                .fill(tokens.accent_primary),
                            )
                            .clicked()
                        {
                            let v = trainer.rush_variant;
                            trainer.start_puzzle_rush(v);
                        }
                    });

                ui.add_space(8.0);

                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(
                        1.2_f32,
                        if trainer.status_is_positive {
                            tokens.accent_primary
                        } else {
                            Color32::from_rgb(0xE0, 0x4F, 0x44)
                        },
                    ))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new(&trainer.status_message)
                                .strong()
                                .size(12.5)
                                .color(tokens.text_primary),
                        );
                    });

                ui.add_space(8.0);

                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new("🏆 Personal Best Records")
                                .strong()
                                .size(13.5)
                                .color(tokens.text_primary),
                        );
                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "⏱ 3-Min Timed Best:  {}",
                                    profile.puzzle_rush_best_timed
                                ))
                                .size(12.5)
                                .color(tokens.accent_gold),
                            );
                        });
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "💀 Survival Best:     {}",
                                    profile.puzzle_rush_best_survival
                                ))
                                .size(12.5)
                                .color(tokens.accent_info),
                            );
                        });
                    });
                return;
            }

            if trainer.tab_mode == PuzzleTabMode::BlunderDeck {
                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new("🔥 Spaced-Repetition Blunder Deck")
                                .strong()
                                .size(15.0)
                                .color(tokens.text_primary),
                        );
                        ui.label(
                            RichText::new(
                                "Drill real blunders harvested from your matches. Solve 3 times cleanly to master!",
                            )
                            .size(11.5)
                            .color(tokens.text_secondary),
                        );
                        ui.add_space(6.0);
                        ui.label(
                            RichText::new(&trainer.status_message)
                                .strong()
                                .size(12.5)
                                .color(if trainer.status_is_positive {
                                    tokens.accent_primary
                                } else {
                                    Color32::from_rgb(0xE0, 0x4F, 0x44)
                                }),
                        );
                        ui.add_space(6.0);
                        ui.horizontal(|ui| {
                            if ui.button("💡 Show Best Move").clicked() {
                                if let Some(mid) = trainer.active_mistake_id {
                                    if let Some(m) =
                                        profile.mistake_vault.iter().find(|e| e.id == mid)
                                    {
                                        if let Ok(mv) =
                                            parse_uci(&trainer.board_state.current_pos, &m.best_uci)
                                        {
                                            if let Some(from) = mv.from() {
                                                trainer.board_ui.hint_arrow =
                                                    Some(TacticalArrow::new(
                                                        from,
                                                        mv.to(),
                                                        AnnotationColor::Green,
                                                    ));
                                            }
                                        }
                                    }
                                }
                            }
                            if ui
                                .add(
                                    egui::Button::new(
                                        RichText::new("Next Blunder ⏭")
                                            .strong()
                                            .color(Color32::WHITE),
                                    )
                                    .fill(tokens.accent_primary),
                                )
                                .clicked()
                            {
                                trainer.load_next_blunder(profile);
                            }
                        });
                    });

                ui.add_space(8.0);
                let list_h = (ui.available_height() - 16.0).max(120.0);
                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ScrollArea::vertical()
                            .id_salt("blunder_deck_list_scroll")
                            .max_height(list_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                if profile.mistake_vault.is_empty() {
                                    ui.label(
                                        RichText::new("No harvested blunders yet. Play matches to build your personal drill deck!")
                                            .italics()
                                            .size(12.0)
                                            .color(tokens.text_secondary),
                                    );
                                } else {
                                    let mut picked_id: Option<u64> = None;
                                    for item in &profile.mistake_vault {
                                        let is_cur = trainer.active_mistake_id == Some(item.id);
                                        let badge = if item.is_mastered() {
                                            "✓ Mastered"
                                        } else {
                                            "Drill"
                                        };
                                        let row = format!(
                                            "[{} {}/3] vs {} • {} (Best: {})",
                                            badge,
                                            item.mastery_streak,
                                            item.opponent_name,
                                            item.played_san,
                                            item.best_san
                                        );
                                        if ui
                                            .add_sized(
                                                [ui.available_width(), 24.0],
                                                egui::SelectableLabel::new(
                                                    is_cur,
                                                    RichText::new(row).size(11.5),
                                                ),
                                            )
                                            .clicked()
                                        {
                                            picked_id = Some(item.id);
                                        }
                                    }
                                    if let Some(id) = picked_id {
                                        trainer.load_mistake_by_id(profile, id);
                                    }
                                }
                            });
                    });
                return;
            }

            let puzzle = &CURATED_PUZZLES[trainer.current_idx];

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
            let puzzle = &CURATED_PUZZLES[trainer.current_idx];
            let can_interact = match trainer.tab_mode {
                PuzzleTabMode::PuzzleRush => trainer.rush_active && !trainer.rush_finished,
                _ => !trainer.solved_current && !trainer.failed_out_of_hearts,
            };
            if can_interact {
                if let Some(mv) = trainer.board_ui.ui_with_config(
                    ui,
                    board_size,
                    &mut trainer.board_state,
                    config,
                ) {
                    let played_uci = format_uci(&mv);
                    if trainer.tab_mode == PuzzleTabMode::BlunderDeck {
                        if let Some(mid) = trainer.active_mistake_id {
                            let expected = profile
                                .mistake_vault
                                .iter()
                                .find(|e| e.id == mid)
                                .map(|e| (e.best_uci.clone(), e.best_san.clone()));
                            if let Some((best_uci, best_san)) = expected {
                                if played_uci == best_uci {
                                    let prev_pos = trainer.board_state.current_pos.clone();
                                    let _ = trainer.board_state.apply_move(mv.clone(), 0, 100);
                                    trainer.board_ui.anim_layer.trigger_move_animation(
                                        &prev_pos,
                                        &trainer.board_state.current_pos,
                                        &mv,
                                        trainer.board_state.outcome,
                                        config.animation_profile,
                                    );
                                    profile.record_mistake_drill_outcome(mid, true);
                                    trainer.solved_current = true;
                                    trainer.status_message = format!(
                                        "★ Nailed it! {best_san} was the winning resource! Mastery streak increased."
                                    );
                                    trainer.status_is_positive = true;
                                } else {
                                    profile.record_mistake_drill_outcome(mid, false);
                                    trainer.status_message =
                                        "✗ Still missing the best move — try again or click Show Best Move!"
                                            .to_string();
                                    trainer.status_is_positive = false;
                                }
                            }
                        }
                        return;
                    }

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
                            if trainer.tab_mode == PuzzleTabMode::PuzzleRush {
                                trainer.rush_score += 1;
                                match trainer.rush_variant {
                                    RushVariant::Timed3Min => {
                                        if trainer.rush_score > profile.puzzle_rush_best_timed {
                                            profile.puzzle_rush_best_timed = trainer.rush_score;
                                            profile.save();
                                        }
                                    }
                                    RushVariant::Survival => {
                                        if trainer.rush_score > profile.puzzle_rush_best_survival {
                                            profile.puzzle_rush_best_survival = trainer.rush_score;
                                            profile.save();
                                        }
                                    }
                                }
                                trainer.advance_rush_puzzle();
                                let next_p = &CURATED_PUZZLES[trainer.current_idx];
                                trainer.status_message = format!(
                                    "⚡ +1 Point! Score: {} — Next Target ELO {}",
                                    trainer.rush_score, next_p.rating
                                );
                                trainer.status_is_positive = true;
                            } else {
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
                            }
                        } else {
                            trainer.status_message =
                                "✓ Great first move! Now find the finishing blow!".to_string();
                            trainer.status_is_positive = true;
                        }
                    } else if trainer.tab_mode == PuzzleTabMode::PuzzleRush {
                        trainer.rush_strikes = trainer.rush_strikes.saturating_add(1);
                        if trainer.rush_strikes >= 3 {
                            trainer.finish_rush(profile, "3 Strikes — Rush Over!");
                        } else {
                            trainer.advance_rush_puzzle();
                            trainer.status_message = format!(
                                "❌ Strike {}/3! Skipping to next puzzle — stay sharp!",
                                trainer.rush_strikes
                            );
                            trainer.status_is_positive = false;
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
