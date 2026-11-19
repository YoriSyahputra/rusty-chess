use crate::{
    persistence::UserProfile,
    ui::{
        board_view::{pos_to_square, render_board_grid_ext, square_to_rect},
        theme::{BoardTheme, UiTokens},
    },
};
use egui::{Align2, Color32, FontId, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use rand::Rng;
use shakmaty::{Chess, File, Rank, Square};
use std::time::{Duration, Instant};

pub struct VisionTrainerState {
    pub active: bool,
    pub finished: bool,
    pub started_at: Option<Instant>,
    pub duration_secs: f32,
    pub flipped: bool,
    pub target_square: Square,
    pub score: u32,
    pub attempts: u32,
    pub streak: u32,
    pub best_streak: u32,
    pub last_feedback: Option<(Square, bool, Instant)>,
}

impl Default for VisionTrainerState {
    fn default() -> Self {
        Self {
            active: false,
            finished: false,
            started_at: None,
            duration_secs: 30.0,
            flipped: false,
            target_square: Square::E4,
            score: 0,
            attempts: 0,
            streak: 0,
            best_streak: 0,
            last_feedback: None,
        }
    }
}

impl VisionTrainerState {
    pub fn start_session(&mut self) {
        self.active = true;
        self.finished = false;
        self.started_at = Some(Instant::now());
        self.score = 0;
        self.attempts = 0;
        self.streak = 0;
        self.best_streak = 0;
        self.last_feedback = None;
        self.roll_next_square();
    }

    pub fn roll_next_square(&mut self) {
        let mut rng = rand::thread_rng();
        let mut next = self.target_square;
        for _ in 0..4 {
            let f = rng.gen_range(0..8u32);
            let r = rng.gen_range(0..8u32);
            let candidate = Square::from_coords(File::new(f), Rank::new(r));
            if candidate != self.target_square {
                next = candidate;
                break;
            }
        }
        self.target_square = next;
    }

    pub fn remaining_secs(&self) -> f32 {
        if let Some(t) = self.started_at {
            (self.duration_secs - t.elapsed().as_secs_f32()).max(0.0)
        } else {
            self.duration_secs
        }
    }

    pub fn accuracy_pct(&self) -> f32 {
        if self.attempts == 0 {
            100.0
        } else {
            (self.score as f32 / self.attempts as f32) * 100.0
        }
    }
}

/// Merender halaman Coordinate Speed Vision Trainer (Feature #25b).
/// Melatih kecepatan membaca koordinat papan (a1–h8) tanpa label koordinat selama 30 detik.
pub fn render_vision_trainer_page(
    ui: &mut Ui,
    state: &mut VisionTrainerState,
    profile: &mut UserProfile,
    board_theme: BoardTheme,
    dark_mode: bool,
) {
    let tokens = UiTokens::for_mode(dark_mode);

    if state.active {
        if state.remaining_secs() <= 0.0 {
            state.active = false;
            state.finished = true;
            if state.score > profile.vision_best_score {
                profile.vision_best_score = state.score;
                profile.save();
            }
        } else {
            ui.ctx()
                .request_repaint_after(Duration::from_millis(80));
        }
    }

    egui::SidePanel::right("vision_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(12.0);
            ui.label(
                RichText::new("🎯 Coordinate Vision Trainer")
                    .strong()
                    .size(18.5)
                    .color(tokens.text_primary),
            );
            ui.label(
                RichText::new("Click the prompted square as fast as possible in 30 seconds — coordinates hidden!")
                    .size(12.0)
                    .color(tokens.text_secondary),
            );
            ui.add_space(10.0);

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.5_f32, tokens.accent_primary))
                .rounding(Rounding::same(10.0_f32))
                .inner_margin(egui::Margin::same(14.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new("FIND SQUARE")
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                        let target_txt = if state.active {
                            state.target_square.to_string()
                        } else if state.finished {
                            "DONE!".to_string()
                        } else {
                            "--".to_string()
                        };
                        ui.label(
                            RichText::new(target_txt)
                                .monospace()
                                .strong()
                                .size(48.0)
                                .color(tokens.accent_gold),
                        );
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new("SCORE").size(10.5).color(tokens.text_secondary));
                            ui.label(
                                RichText::new(format!("{}", state.score))
                                    .strong()
                                    .size(26.0)
                                    .color(tokens.accent_primary),
                            );
                        });
                        ui.add_space(22.0);
                        ui.vertical(|ui| {
                            ui.label(RichText::new("TIME").size(10.5).color(tokens.text_secondary));
                            let rem = state.remaining_secs();
                            let col = if rem < 7.0 && state.active {
                                Color32::from_rgb(0xE0, 0x4F, 0x44)
                            } else {
                                tokens.text_primary
                            };
                            ui.label(
                                RichText::new(format!("{rem:04.1}s"))
                                    .monospace()
                                    .strong()
                                    .size(24.0)
                                    .color(col),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.vertical(|ui| {
                                ui.label(
                                    RichText::new("ACCURACY")
                                        .size(10.5)
                                        .color(tokens.text_secondary),
                                );
                                ui.label(
                                    RichText::new(format!("{:.0}%", state.accuracy_pct()))
                                        .strong()
                                        .size(22.0)
                                        .color(tokens.accent_info),
                                );
                            });
                        });
                    });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.label("Perspective:");
                        ui.selectable_value(&mut state.flipped, false, "⬜ White");
                        ui.selectable_value(&mut state.flipped, true, "⬛ Black");
                    });

                    ui.add_space(10.0);
                    let btn_label = if state.active {
                        "🔄 Restart 30s Drill"
                    } else {
                        "▶ Start 30s Vision Drill"
                    };
                    if ui
                        .add_sized(
                            [ui.available_width(), 38.0],
                            egui::Button::new(
                                RichText::new(btn_label)
                                    .strong()
                                    .size(14.5)
                                    .color(Color32::WHITE),
                            )
                            .fill(tokens.accent_primary),
                        )
                        .clicked()
                    {
                        state.start_session();
                    }
                });

            ui.add_space(10.0);

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new("🏆 Vision Records")
                            .strong()
                            .size(14.0)
                            .color(tokens.text_primary),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "Personal Best (30s):  {} squares",
                            profile.vision_best_score
                        ))
                        .strong()
                        .size(13.0)
                        .color(tokens.accent_gold),
                    );
                    ui.label(
                        RichText::new(format!(
                            "Current Streak: {}  •  Session Best Streak: {}",
                            state.streak, state.best_streak
                        ))
                        .size(12.0)
                        .color(tokens.text_secondary),
                    );
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
            let (b_rect, resp) =
                ui.allocate_exact_size(Vec2::splat(board_size), Sense::click());
            let empty_board = Chess::default();
            render_board_grid_ext(
                ui.painter(),
                b_rect,
                &empty_board,
                &board_theme.palette(),
                state.flipped,
                false, // Coordinates hidden during Vision Drill!
                None,
                None,
                None,
                &[],
                &[],
                &[],
                None,
                None,
                None,
                &Square::ALL, // Empty clean grid for pure coordinate recognition
                crate::config::PieceSetStyle::ClassicStaunton,
                None,
            );

            // Flash feedback highlight on clicked square
            if let Some((fb_sq, correct, ts)) = state.last_feedback {
                if ts.elapsed() < Duration::from_millis(260) {
                    let sq_r = square_to_rect(b_rect, fb_sq, state.flipped);
                    let col = if correct {
                        Color32::from_rgba_unmultiplied(80, 220, 100, 155)
                    } else {
                        Color32::from_rgba_unmultiplied(235, 65, 65, 165)
                    };
                    ui.painter().rect_filled(sq_r, Rounding::ZERO, col);
                    ui.ctx().request_repaint();
                }
            }

            // Subtle target watermark on board while running
            if state.active {
                let c = b_rect.center();
                ui.painter().circle_filled(
                    c,
                    44.0,
                    Color32::from_rgba_unmultiplied(20, 20, 22, 165),
                );
                ui.painter().text(
                    c,
                    Align2::CENTER_CENTER,
                    state.target_square.to_string(),
                    FontId::monospace(34.0),
                    Color32::from_rgb(255, 225, 95),
                );
            }

            if state.active && resp.clicked() {
                if let Some(pos) = resp.interact_pointer_pos() {
                    if let Some(clicked_sq) = pos_to_square(b_rect, pos, state.flipped) {
                        state.attempts += 1;
                        if clicked_sq == state.target_square {
                            state.score += 1;
                            state.streak += 1;
                            if state.streak > state.best_streak {
                                state.best_streak = state.streak;
                            }
                            if state.score > profile.vision_best_score {
                                profile.vision_best_score = state.score;
                                profile.save();
                            }
                            state.last_feedback = Some((clicked_sq, true, Instant::now()));
                            state.roll_next_square();
                        } else {
                            state.streak = 0;
                            state.last_feedback = Some((clicked_sq, false, Instant::now()));
                        }
                    }
                }
            }
        });
    });
}
