use crate::{
    config::UserConfig,
    domain::{
        detect_opening_from_records, format_fen, format_uci, parse_fen, AnnotationColor,
        ChessMatchState, Color, PlayerController, TacticalArrow,
    },
    engine::{
        coach::{explain_move, CoachExplanation},
        review::{analyze_match_review, GameReviewReport, MoveClassification},
        search::{compute_multipv_lines, MultiPvLine},
    },
    ui::{
        board_view::{
            render_board_grid, render_move_classification_badge, render_xray_heatmap,
            BoardInteractionState,
        },
        eval_bar::EvalBarState,
        theme::UiTokens,
    },
};
use egui::{Color32, Pos2, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};
use shakmaty::Position;

#[derive(Default)]
pub struct ReviewViewState {
    pub report: Option<GameReviewReport>,
    pub analyzed_plies_count: usize,
    pub selected_ply: usize,
    pub cached_coach_ply: usize,
    pub cached_coach: Option<CoachExplanation>,
    pub refutation_step: Option<usize>,
    pub retry_mode_active: bool,
    pub retry_status_msg: Option<String>,
    pub retry_board_state: Option<ChessMatchState>,
    pub retry_board_ui: BoardInteractionState,

    // Feature #23: Multi-PV Top 3 Live Engine Lines & Free Analysis Sandbox
    pub sandbox_active: bool,
    pub sandbox_state: Option<ChessMatchState>,
    pub sandbox_ui: BoardInteractionState,
    pub sandbox_eval_bar: EvalBarState,
    pub cached_multipv_fen: String,
    pub cached_multipv: Vec<MultiPvLine>,
}

impl ReviewViewState {
    pub fn ensure_analyzed(&mut self, match_state: &ChessMatchState) {
        if self.report.is_none() || self.analyzed_plies_count != match_state.history.len() {
            let report = analyze_match_review(match_state, 3);
            self.analyzed_plies_count = match_state.history.len();
            self.selected_ply = report.entries.len();
            self.report = Some(report);
            self.retry_mode_active = false;
            self.retry_status_msg = None;
        }
    }

    pub fn enter_sandbox(&mut self, match_state: &ChessMatchState) {
        let fen = format_fen(&match_state.displayed_position());
        if let Ok(mut st) = ChessMatchState::from_fen(&fen, Color::White) {
            st.white_controller = PlayerController::Human;
            st.black_controller = PlayerController::Human;
            self.sandbox_state = Some(st);
            self.sandbox_ui.reset();
            self.cached_multipv_fen.clear();
            self.cached_multipv.clear();
            self.sandbox_active = true;
            self.retry_mode_active = false;
        }
    }

    pub fn refresh_sandbox_multipv(&mut self) {
        let Some(st) = &mut self.sandbox_state else {
            return;
        };
        let fen = format_fen(&st.current_pos);
        if self.cached_multipv_fen == fen && !self.cached_multipv.is_empty() {
            return;
        }
        let lines = compute_multipv_lines(&st.current_pos, 3, 3);
        if let Some(top) = lines.first() {
            self.sandbox_eval_bar.set_eval_cp(top.eval_cp_white);
        }
        // Populate color-coded Top-3 Multi-PV arrows (Green #1, Blue #2, Yellow #3)
        st.arrows.clear();
        let colors = [
            AnnotationColor::Green,
            AnnotationColor::Blue,
            AnnotationColor::Yellow,
        ];
        for (i, line) in lines.iter().enumerate() {
            if let Some(&col) = colors.get(i) {
                st.arrows.push(TacticalArrow::new(line.from, line.to, col));
            }
        }
        self.cached_multipv = lines;
        self.cached_multipv_fen = fen;
    }
}

/// Merender halaman Post-Game Review (`Route::Review`) sesuai NEW_DESIGN.md Section 8:
/// - Accuracy % & Estimated ELO untuk Putih dan Hitam
/// - Interactive Evaluation / Win-Probability Graph
/// - Move Classification Breakdown (`!!`, `!`, `★`, `✓`, `?!`, `✗`, `?`, `??`)
/// - Best Move vs Played Move Arrow & "Retry Mistake" Drill
pub fn render_review_page(
    ui: &mut Ui,
    review_state: &mut ReviewViewState,
    match_state: &mut ChessMatchState,
    bot_name: &str,
    config: &UserConfig,
) {
    review_state.ensure_analyzed(match_state);
    let tokens = UiTokens::for_mode(config.dark_mode);

    let Some(report) = review_state.report.clone() else {
        return;
    };

    egui::SidePanel::right("review_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(6.0);

            // Top Mode Switcher: Post-Game Review vs Free Multi-PV Analysis Sandbox (Feature #23)
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(
                        !review_state.sandbox_active,
                        RichText::new("📊 Game Review").strong().size(12.5),
                    )
                    .clicked()
                {
                    review_state.sandbox_active = false;
                }
                if ui
                    .selectable_label(
                        review_state.sandbox_active,
                        RichText::new("🔬 Multi-PV Analysis Sandbox")
                            .strong()
                            .size(12.5),
                    )
                    .clicked()
                {
                    review_state.enter_sandbox(match_state);
                }
            });

            ui.add_space(6.0);

            if review_state.sandbox_active {
                if review_state.sandbox_state.is_none() {
                    review_state.enter_sandbox(match_state);
                }
                review_state.refresh_sandbox_multipv();

                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.5_f32, tokens.accent_info))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new("🔬 Top 3 Multi-PV Engine Lines")
                                .strong()
                                .size(15.5)
                                .color(tokens.text_primary),
                        );
                        ui.label(
                            RichText::new(
                                "Play moves freely for both sides or click any engine line to explore variations.",
                            )
                            .size(11.5)
                            .color(tokens.text_secondary),
                        );
                        ui.add_space(8.0);

                        let line_colors = [
                            Color32::from_rgb(0x81, 0xB6, 0x4C),
                            Color32::from_rgb(0x4A, 0x90, 0xE2),
                            Color32::from_rgb(0xE8, 0xB9, 0x31),
                        ];
                        let mut play_line_mv: Option<crate::domain::Move> = None;

                        if review_state.cached_multipv.is_empty() {
                            ui.label(
                                RichText::new("Position is game over — no legal moves available.")
                                    .italics()
                                    .size(12.0)
                                    .color(tokens.text_secondary),
                            );
                        } else {
                            for (idx, line) in review_state.cached_multipv.iter().enumerate() {
                                let col = line_colors[idx.min(2)];
                                let score_str = if line.eval_cp_white.abs() >= 25_000 {
                                    if line.eval_cp_white > 0 {
                                        "+M".to_string()
                                    } else {
                                        "-M".to_string()
                                    }
                                } else {
                                    format!("{:+.2}", line.eval_cp_white as f32 / 100.0)
                                };
                                let pv_str = line.pv_sans.join(" ");
                                let row_txt = format!("#{}  [{:>6}]  {}", idx + 1, score_str, pv_str);

                                let resp = egui::Frame::none()
                                    .fill(tokens.bg_canvas)
                                    .stroke(Stroke::new(1.0_f32, col.gamma_multiply(0.7)))
                                    .rounding(Rounding::same(6.0_f32))
                                    .inner_margin(egui::Margin::symmetric(8.0, 6.0))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.label(
                                            RichText::new(row_txt)
                                                .monospace()
                                                .strong()
                                                .size(12.5)
                                                .color(col),
                                        );
                                    })
                                    .response
                                    .interact(Sense::click());
                                if resp.clicked() {
                                    play_line_mv = Some(line.mv.clone());
                                }
                                ui.add_space(5.0);
                            }
                        }

                        if let Some(mv) = play_line_mv {
                            if let Some(st) = &mut review_state.sandbox_state {
                                if st.apply_move(mv, 0, 100).is_ok() {
                                    review_state.cached_multipv_fen.clear();
                                }
                            }
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if ui.button("↩ Undo").clicked() {
                                if let Some(st) = &mut review_state.sandbox_state {
                                    if !st.history.is_empty() {
                                        st.history.pop();
                                        if st.position_hashes.len() > 1 {
                                            st.position_hashes.pop();
                                        }
                                        let mut pos = st.initial_pos.clone();
                                        for rec in &st.history {
                                            pos.play_unchecked(&rec.mv);
                                        }
                                        st.current_pos = pos;
                                        st.viewed_ply = st.history.len();
                                        st.outcome = crate::domain::GameOutcome::Ongoing;
                                        st.last_move = st.history.last().map(|r| {
                                            (r.mv.from().unwrap_or_else(|| r.mv.to()), r.mv.to())
                                        });
                                        review_state.cached_multipv_fen.clear();
                                    }
                                }
                            }
                            if ui.button("🔄 Reset to Main Line").clicked() {
                                review_state.enter_sandbox(match_state);
                            }
                        });
                    });

                ui.add_space(8.0);

                // Sandbox variation trail
                let rem_h = (ui.available_height() - 12.0).max(90.0);
                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(8.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.label(
                            RichText::new("Exploration Trail")
                                .strong()
                                .size(12.5)
                                .color(tokens.text_primary),
                        );
                        ScrollArea::vertical()
                            .id_salt("sandbox_trail_scroll")
                            .max_height(rem_h)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let mut trail = String::new();
                                if let Some(st) = &review_state.sandbox_state {
                                    for (i, rec) in st.history.iter().enumerate() {
                                        if i % 2 == 0 {
                                            trail.push_str(&format!("{}. {} ", (i / 2) + 1, rec.san));
                                        } else {
                                            trail.push_str(&format!("{}   ", rec.san));
                                        }
                                    }
                                }
                                if trail.is_empty() {
                                    trail = "Make moves on the board or click a Top-3 line above."
                                        .to_string();
                                }
                                ui.label(
                                    RichText::new(trail)
                                        .monospace()
                                        .size(12.0)
                                        .color(tokens.text_secondary),
                                );
                            });
                    });
                return;
            }

            // 1. Accuracy Score Cards & Estimated Game Rating
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new("📊 Game Review & Accuracy Report")
                            .strong()
                            .size(15.5)
                            .color(tokens.text_primary),
                    );
                    if let Some(op) = detect_opening_from_records(&match_state.history) {
                        ui.label(
                            RichText::new(format!(
                                "📖 {} • {} (Book through ply {})",
                                op.eco, op.name, op.matched_plies
                            ))
                            .size(11.5)
                            .color(tokens.accent_gold),
                        );
                    }
                    ui.add_space(6.0);

                    ui.columns(2, |cols| {
                        cols[0].vertical_centered(|ui| {
                            ui.label(
                                RichText::new("⬜ White")
                                    .strong()
                                    .size(12.5)
                                    .color(tokens.text_secondary),
                            );
                            ui.label(
                                RichText::new(format!("{:.1}%", report.white_accuracy))
                                    .strong()
                                    .size(21.0)
                                    .color(tokens.accent_primary),
                            );
                            ui.label(
                                RichText::new(format!("Est. ELO: {}", report.white_est_elo))
                                    .monospace()
                                    .size(11.5)
                                    .color(tokens.accent_gold),
                            );
                        });

                        cols[1].vertical_centered(|ui| {
                            ui.label(
                                RichText::new(format!("⬛ Black ({bot_name})"))
                                    .strong()
                                    .size(12.5)
                                    .color(tokens.text_secondary),
                            );
                            ui.label(
                                RichText::new(format!("{:.1}%", report.black_accuracy))
                                    .strong()
                                    .size(21.0)
                                    .color(tokens.accent_info),
                            );
                            ui.label(
                                RichText::new(format!("Est. ELO: {}", report.black_est_elo))
                                    .monospace()
                                    .size(11.5)
                                    .color(tokens.accent_gold),
                            );
                        });
                    });
                });

            ui.add_space(6.0);

            // 2. Interactive Win Probability / Evaluation Graph
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    let graph_w = (ui.available_width() - 4.0).max(200.0);
                    let graph_h = 72.0;
                    let (g_rect, g_resp) =
                        ui.allocate_exact_size(Vec2::new(graph_w, graph_h), Sense::click_and_drag());
                    let painter = ui.painter_at(g_rect);

                    painter.rect_filled(
                        g_rect,
                        Rounding::same(4.0_f32),
                        Color32::from_rgb(0x21, 0x1F, 0x1C),
                    );
                    let mid_y = g_rect.center().y;
                    painter.line_segment(
                        [
                            Pos2::new(g_rect.min.x, mid_y),
                            Pos2::new(g_rect.max.x, mid_y),
                        ],
                        Stroke::new(1.0_f32, Color32::from_rgb(85, 82, 78)),
                    );

                    if !report.entries.is_empty() {
                        let n = report.entries.len().max(1);
                        let mut pts = Vec::with_capacity(n + 1);
                        pts.push(Pos2::new(g_rect.min.x, mid_y));

                        for (idx, entry) in report.entries.iter().enumerate() {
                            let x = g_rect.min.x
                                + ((idx + 1) as f32 / (n as f32)) * g_rect.width();
                            let y = g_rect.max.y
                                - entry.win_prob_white.clamp(0.02, 0.98) * g_rect.height();
                            pts.push(Pos2::new(x, y));
                        }

                        for pair in pts.windows(2) {
                            painter.line_segment(
                                [pair[0], pair[1]],
                                Stroke::new(2.0_f32, tokens.accent_primary),
                            );
                        }

                        // Marker ply yang sedang dipilih
                        let sel_x = g_rect.min.x
                            + (review_state.selected_ply.min(n) as f32 / (n as f32))
                                * g_rect.width();
                        painter.line_segment(
                            [
                                Pos2::new(sel_x, g_rect.min.y),
                                Pos2::new(sel_x, g_rect.max.y),
                            ],
                            Stroke::new(1.5_f32, tokens.accent_gold),
                        );

                        if g_resp.clicked() || g_resp.dragged() {
                            if let Some(pos) = g_resp.interact_pointer_pos() {
                                let ratio =
                                    ((pos.x - g_rect.min.x) / g_rect.width()).clamp(0.0, 1.0);
                                let ply = (ratio * (n as f32)).round() as usize;
                                review_state.selected_ply = ply.clamp(1, n);
                                match_state.jump_to_ply(review_state.selected_ply);
                                review_state.retry_mode_active = false;
                            }
                        }
                    }

                    // Feature #4: Move-by-Move Clock Usage vs. Accuracy Dual Chart
                    ui.add_space(4.0);
                    let time_h = 34.0;
                    let (t_rect, t_resp) =
                        ui.allocate_exact_size(Vec2::new(graph_w, time_h), Sense::click());
                    let t_painter = ui.painter_at(t_rect);
                    t_painter.rect_filled(
                        t_rect,
                        Rounding::same(3.0_f32),
                        Color32::from_rgb(0x1C, 0x1A, 0x18),
                    );

                    if !report.entries.is_empty() {
                        let n = report.entries.len().max(1);
                        let max_ms = match_state
                            .history
                            .iter()
                            .map(|r| r.time_spent_ms)
                            .max()
                            .unwrap_or(1000)
                            .max(500) as f32;
                        let bar_w = (t_rect.width() / n as f32).clamp(2.0, 14.0);

                        for (idx, entry) in report.entries.iter().enumerate() {
                            let ms = match_state
                                .history
                                .get(idx)
                                .map(|r| r.time_spent_ms)
                                .unwrap_or(0) as f32;
                            let h = ((ms / max_ms).clamp(0.08, 1.0)) * (t_rect.height() - 4.0);
                            let x = t_rect.min.x + (idx as f32 / n as f32) * t_rect.width();
                            let bar_rect = Rect::from_min_max(
                                Pos2::new(x, t_rect.max.y - h),
                                Pos2::new((x + bar_w - 1.0).max(x + 1.0), t_rect.max.y),
                            );
                            let rgb = entry.classification.rgb();
                            let col = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                            t_painter.rect_filled(bar_rect, Rounding::same(1.0_f32), col);
                        }

                        if t_resp.clicked() {
                            if let Some(pos) = t_resp.interact_pointer_pos() {
                                let ratio =
                                    ((pos.x - t_rect.min.x) / t_rect.width()).clamp(0.0, 1.0);
                                let ply = ((ratio * n as f32).floor() as usize + 1).clamp(1, n);
                                review_state.selected_ply = ply;
                                match_state.jump_to_ply(ply);
                                review_state.retry_mode_active = false;
                            }
                        }

                        let sel_ms = match_state
                            .history
                            .get(review_state.selected_ply.saturating_sub(1))
                            .map(|r| r.time_spent_ms)
                            .unwrap_or(0);
                        ui.label(
                            RichText::new(format!(
                                "⏱ Move Time Chart  •  Ply {} spent {:.1}s",
                                review_state.selected_ply,
                                sel_ms as f32 / 1000.0
                            ))
                            .size(10.5)
                            .color(tokens.text_secondary),
                        );
                    }
                });

            ui.add_space(6.0);

            // 3. Selected Ply Coach Insights ("Why Was That Bad?") & Refutation Player (Feature #8)
            if let Some(entry) = report
                .entries
                .get(review_state.selected_ply.saturating_sub(1))
            {
                if review_state.cached_coach_ply != entry.ply || review_state.cached_coach.is_none()
                {
                    if let Ok(pos_before) = parse_fen(&entry.fen_before) {
                        review_state.cached_coach = Some(explain_move(
                            &pos_before,
                            &entry.played_mv,
                            &entry.best_mv,
                            entry.classification,
                        ));
                        review_state.cached_coach_ply = entry.ply;
                        review_state.refutation_step = None;
                    }
                }

                let c_rgb = entry.classification.rgb();
                let badge_color = Color32::from_rgb(c_rgb[0], c_rgb[1], c_rgb[2]);

                egui::Frame::none()
                    .fill(tokens.bg_surface)
                    .stroke(Stroke::new(1.2_f32, badge_color))
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(10.0))
                    .show(ui, |ui| {
                        ui.set_min_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new(format!(
                                    "Ply {}: {} is {} ({})",
                                    entry.ply,
                                    entry.played_san,
                                    entry.classification.label(),
                                    entry.classification.symbol()
                                ))
                                .strong()
                                .size(13.5)
                                .color(badge_color),
                            );
                        });

                        if let Some(coach) = &review_state.cached_coach {
                            ui.label(
                                RichText::new(format!("🎓 {}", coach.headline))
                                    .strong()
                                    .size(12.0)
                                    .color(tokens.text_primary),
                            );
                            ui.label(
                                RichText::new(&coach.detail)
                                    .size(11.5)
                                    .color(tokens.text_secondary),
                            );
                            if !coach.refutation_sans.is_empty() {
                                ui.horizontal(|ui| {
                                    let line_txt = coach.refutation_sans.join(" → ");
                                    ui.label(
                                        RichText::new(format!("Refutation: {line_txt}"))
                                            .monospace()
                                            .size(11.0)
                                            .color(Color32::from_rgb(0xFF, 0xAA, 0x00)),
                                    );
                                });
                                ui.horizontal(|ui| {
                                    if ui.small_button("▶ Show Refutation").clicked() {
                                        let next_s = review_state
                                            .refutation_step
                                            .map(|s| (s + 1) % coach.refutation_fens.len())
                                            .unwrap_or(1.min(coach.refutation_fens.len() - 1));
                                        review_state.refutation_step = Some(next_s);
                                    }
                                    if review_state.refutation_step.is_some()
                                        && ui.small_button("✕ Reset View").clicked()
                                    {
                                        review_state.refutation_step = None;
                                    }
                                });
                            }
                        } else if entry.played_san != entry.best_san {
                            ui.label(
                                RichText::new(format!(
                                    "Best engine move was {} (shown with green arrow)",
                                    entry.best_san
                                ))
                                .size(12.0)
                                .color(tokens.text_secondary),
                            );
                        }

                        if let Some(msg) = &review_state.retry_status_msg {
                            ui.label(
                                RichText::new(msg)
                                    .strong()
                                    .size(12.0)
                                    .color(tokens.accent_gold),
                            );
                        }

                        ui.add_space(4.0);
                        ui.horizontal(|ui| {
                            if entry.classification.is_mistake_or_worse() {
                                if ui
                                    .add(
                                        egui::Button::new(
                                            RichText::new("🎯 Retry This Mistake")
                                                .strong()
                                                .size(12.0)
                                                .color(Color32::WHITE),
                                        )
                                        .fill(tokens.accent_primary),
                                    )
                                    .clicked()
                                {
                                    if let Ok(st) =
                                        ChessMatchState::from_fen(&entry.fen_before, entry.side)
                                    {
                                        review_state.retry_board_state = Some(st);
                                        review_state.retry_board_ui.reset();
                                        review_state.retry_mode_active = true;
                                        review_state.retry_status_msg = Some(format!(
                                            "Find the better move instead of {}!",
                                            entry.played_san
                                        ));
                                    }
                                }
                            }

                            if ui.button("⏭ Next Mistake").clicked() {
                                if let Some(next_err) = report
                                    .entries
                                    .iter()
                                    .find(|e| {
                                        e.ply > review_state.selected_ply
                                            && e.classification.is_mistake_or_worse()
                                    })
                                    .or_else(|| {
                                        report
                                            .entries
                                            .iter()
                                            .find(|e| e.classification.is_mistake_or_worse())
                                    })
                                {
                                    review_state.selected_ply = next_err.ply;
                                    match_state.jump_to_ply(next_err.ply);
                                    review_state.retry_mode_active = false;
                                }
                            }
                        });
                    });

                ui.add_space(6.0);
            }

            // 4. Classification Breakdown & Ply Move List
            let footer_h = 40.0;
            let list_h = (ui.available_height() - footer_h).max(95.0);

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .id_salt("review_ply_list_scroll")
                        .max_height(list_h)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            if report.entries.is_empty() {
                                ui.label(
                                    RichText::new("Play a match first to generate a move-by-move review.")
                                        .italics()
                                        .size(12.0)
                                        .color(tokens.text_secondary),
                                );
                                return;
                            }

                            for entry in &report.entries {
                                let is_sel = review_state.selected_ply == entry.ply;
                                let c_rgb = entry.classification.rgb();
                                let c_col = Color32::from_rgb(c_rgb[0], c_rgb[1], c_rgb[2]);
                                let move_prefix = if entry.side == Color::White {
                                    format!("{}.", (entry.ply + 1) / 2)
                                } else {
                                    format!("{}...", (entry.ply + 1) / 2)
                                };

                                ui.horizontal(|ui| {
                                    let row_txt = format!(
                                        "{move_prefix:<4} {:<7}  [{} {}]",
                                        entry.played_san,
                                        entry.classification.symbol(),
                                        entry.classification.label()
                                    );
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 22.0],
                                            egui::SelectableLabel::new(
                                                is_sel,
                                                RichText::new(row_txt)
                                                    .monospace()
                                                    .size(12.0)
                                                    .color(c_col),
                                            ),
                                        )
                                        .clicked()
                                    {
                                        review_state.selected_ply = entry.ply;
                                        match_state.jump_to_ply(entry.ply);
                                        review_state.retry_mode_active = false;
                                        review_state.retry_status_msg = None;
                                    }
                                });
                            }
                        });
                });

            ui.add_space(4.0);

            // Stepper Navigation Row
            let max_ply = report.entries.len();
            ui.horizontal(|ui| {
                let bw = (ui.available_width() - 24.0) / 4.0;
                if ui
                    .add_enabled(
                        review_state.selected_ply > 1,
                        egui::Button::new("|<<").min_size([bw, 28.0].into()),
                    )
                    .clicked()
                {
                    review_state.selected_ply = 1;
                    match_state.jump_to_ply(1);
                    review_state.retry_mode_active = false;
                }
                if ui
                    .add_enabled(
                        review_state.selected_ply > 1,
                        egui::Button::new("<").min_size([bw, 28.0].into()),
                    )
                    .clicked()
                {
                    review_state.selected_ply = review_state.selected_ply.saturating_sub(1).max(1);
                    match_state.jump_to_ply(review_state.selected_ply);
                    review_state.retry_mode_active = false;
                }
                if ui
                    .add_enabled(
                        review_state.selected_ply < max_ply,
                        egui::Button::new(">").min_size([bw, 28.0].into()),
                    )
                    .clicked()
                {
                    review_state.selected_ply = (review_state.selected_ply + 1).min(max_ply);
                    match_state.jump_to_ply(review_state.selected_ply);
                    review_state.retry_mode_active = false;
                }
                if ui
                    .add_enabled(
                        review_state.selected_ply < max_ply,
                        egui::Button::new(">>|").min_size([bw, 28.0].into()),
                    )
                    .clicked()
                {
                    review_state.selected_ply = max_ply;
                    match_state.jump_to_ply(max_ply);
                    review_state.retry_mode_active = false;
                }
            });
        });

    // Left Central Column: Interactive Review Board, Multi-PV Sandbox, or Retry Mistake Drill
    egui::CentralPanel::default().show_inside(ui, |ui| {
        if review_state.sandbox_active {
            review_state.refresh_sandbox_multipv();
            let avail = ui.available_size();
            let eval_w = 34.0;
            let board_size = (avail.x - eval_w - 24.0).min(avail.y - 48.0).max(320.0);
            let pad_x = ((avail.x - (board_size + eval_w)) * 0.5).max(8.0);
            let pad_y = ((avail.y - board_size) * 0.5).max(8.0);

            ui.add_space(pad_y);
            ui.horizontal(|ui| {
                ui.add_space(pad_x);
                review_state
                    .sandbox_eval_bar
                    .ui(ui, board_size, config.board_flipped);
                ui.add_space(6.0);
                if let Some(st) = &mut review_state.sandbox_state {
                    if let Some(mv) = review_state.sandbox_ui.ui_with_config(
                        ui,
                        board_size,
                        st,
                        config,
                    ) {
                        if st.apply_move(mv, 0, 100).is_ok() {
                            review_state.cached_multipv_fen.clear();
                        }
                    }
                }
            });
            return;
        }

        let avail = ui.available_size();
        let board_size = (avail.x - 24.0).min(avail.y - 48.0).max(320.0);
        let pad_x = ((avail.x - board_size) * 0.5).max(8.0);
        let pad_y = ((avail.y - board_size) * 0.5).max(8.0);

        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);

            if review_state.retry_mode_active {
                if let Some(retry_st) = &mut review_state.retry_board_state {
                    if let Some(mv) = review_state.retry_board_ui.ui_with_config(
                        ui,
                        board_size,
                        retry_st,
                        config,
                    ) {
                        if let Some(entry) = report
                            .entries
                            .get(review_state.selected_ply.saturating_sub(1))
                        {
                            if format_uci(&mv) == entry.best_uci {
                                let _ = retry_st.apply_move(mv, 0, 100);
                                review_state.retry_status_msg = Some(format!(
                                    "★ Excellent! {} is the engine's best move!",
                                    entry.best_san
                                ));
                                review_state.retry_mode_active = false;
                            } else {
                                review_state.retry_status_msg = Some(
                                    "✗ Not quite the best resource—try another move!".to_string(),
                                );
                            }
                        }
                    }
                    return;
                }
            }

            let (b_rect, _) =
                ui.allocate_exact_size(Vec2::splat(board_size), Sense::hover());

            let (display_pos, last_mv, best_arrow) = if let (Some(step), Some(coach)) =
                (review_state.refutation_step, &review_state.cached_coach)
            {
                let pos = coach
                    .refutation_fens
                    .get(step)
                    .and_then(|f| parse_fen(f).ok())
                    .unwrap_or_else(|| match_state.displayed_position());
                (pos, None, None)
            } else if let Some(entry) = report
                .entries
                .get(review_state.selected_ply.saturating_sub(1))
            {
                let pos = match_state.displayed_position();
                let from = entry.played_mv.from().unwrap_or_else(|| entry.played_mv.to());
                let played_pair = Some((from, entry.played_mv.to()));
                let best_arr = entry.best_mv.from().map(|bf| {
                    TacticalArrow::new(bf, entry.best_mv.to(), AnnotationColor::Green)
                });
                let _ = parse_fen;
                let _ = MoveClassification::Best;
                let _ = Rect::ZERO;
                (pos, played_pair, best_arr)
            } else {
                (match_state.displayed_position(), None, None)
            };

            render_board_grid(
                ui.painter(),
                b_rect,
                &display_pos,
                &config.board_theme.palette(),
                config.board_flipped,
                config.show_coordinates,
                None,
                last_mv,
                &[],
                &[],
                &[],
                None,
                best_arrow,
                None,
            );
            if config.show_xray_heatmap {
                render_xray_heatmap(ui.painter(), b_rect, &display_pos, config.board_flipped);
            }
            if review_state.refutation_step.is_none() {
                if let Some(entry) = report
                    .entries
                    .get(review_state.selected_ply.saturating_sub(1))
                {
                    render_move_classification_badge(
                        ui.painter(),
                        b_rect,
                        entry.played_mv.to(),
                        entry.classification,
                        1.0,
                        config.board_flipped,
                    );
                }
            }
        });
    });
}
