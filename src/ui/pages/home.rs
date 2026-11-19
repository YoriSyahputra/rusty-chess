use crate::{
    domain::parse_fen,
    engine::bot::{BotId, BotPersona},
    persistence::UserProfile,
    ui::{
        board_view::render_board_grid,
        piece_renderer::draw_bot_avatar,
        sidebar::ActiveRoute,
        theme::{BoardTheme, UiTokens},
    },
};
use egui::{Color32, Pos2, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Default)]
pub struct HomePageActions {
    pub navigate_to: Option<ActiveRoute>,
    pub quick_start_bot: Option<BotId>,
    pub start_coach_match: bool,
}

/// Merender Home Dashboard (`Route::Home`) sesuai NEW_DESIGN.md Section 3.1:
/// - Crown Progress Overview (X / 54 Crowns)
/// - Win / Draw / Loss Record & Puzzle Rating
/// - Quick Action Cards (Play Bots, Play Coach, Daily Puzzle, Game Review)
/// - Daily Puzzle Preview Widget & Recent PGN History
pub fn render_home_page(
    ui: &mut Ui,
    profile: &UserProfile,
    board_theme: BoardTheme,
    dark_mode: bool,
) -> HomePageActions {
    let mut actions = HomePageActions::default();
    let tokens = UiTokens::for_mode(dark_mode);

    ScrollArea::vertical()
        .id_salt("home_dashboard_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.set_max_width((ui.available_width() - 24.0).max(600.0));

                    // 1. Hero Banner & Stats Strip
                    egui::Frame::none()
                        .fill(tokens.bg_surface)
                        .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                        .rounding(Rounding::same(10.0_f32))
                        .inner_margin(egui::Margin::same(18.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.horizontal_wrapped(|ui| {
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new("Welcome to Rusty Chess")
                                            .strong()
                                            .size(22.0)
                                            .color(tokens.text_primary),
                                    );
                                    ui.label(
                                        RichText::new(
                                            "Native Linux Chess Desktop • 18 AI Personalities, Tactics Trainer & Local Game Review",
                                        )
                                        .size(13.0)
                                        .color(tokens.text_secondary),
                                    );
                                    ui.add_space(10.0);

                                    // Crown Progress Bar (X / 54)
                                    let total_crowns = profile.total_crowns();
                                    let max_crowns = 54u32;
                                    let ratio =
                                        (total_crowns as f32 / max_crowns as f32).clamp(0.0, 1.0);

                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new(format!(
                                                "👑 Bot Crowns Collected: {total_crowns} / {max_crowns}"
                                            ))
                                            .strong()
                                            .size(13.5)
                                            .color(tokens.accent_gold),
                                        );
                                        ui.add_space(12.0);
                                        let dna = profile.compute_skill_dna();
                                        ui.label(
                                            RichText::new(format!(
                                                "Record: {}W - {}D - {}L   |   🧩 Puzzle: {}   |   🧬 DNA: {}",
                                                profile.wins,
                                                profile.draws,
                                                profile.losses,
                                                profile.puzzle_rating,
                                                dna.archetype
                                            ))
                                            .size(12.5)
                                            .color(tokens.text_secondary),
                                        );
                                        if ui.small_button("📈 View Insights").clicked() {
                                            actions.navigate_to = Some(ActiveRoute::Analytics);
                                        }
                                    });

                                    let bar_w = (ui.available_width() - 16.0).clamp(240.0, 520.0);
                                    let (bar_rect, _) =
                                        ui.allocate_exact_size(Vec2::new(bar_w, 10.0), Sense::hover());
                                    ui.painter().rect_filled(
                                        bar_rect,
                                        Rounding::same(5.0_f32),
                                        tokens.bg_surface_elevated,
                                    );
                                    let fill_w = (bar_rect.width() * ratio).max(if total_crowns > 0 {
                                        8.0
                                    } else {
                                        0.0
                                    });
                                    if fill_w > 0.0 {
                                        let fill_rect = Rect::from_min_max(
                                            bar_rect.min,
                                            Pos2::new(bar_rect.min.x + fill_w, bar_rect.max.y),
                                        );
                                        ui.painter().rect_filled(
                                            fill_rect,
                                            Rounding::same(5.0_f32),
                                            tokens.accent_gold,
                                        );
                                    }
                                });
                            });
                        });

                    ui.add_space(16.0);

                    // 2. Quick Action Cards Row
                    ui.label(
                        RichText::new("Quick Play & Training")
                            .strong()
                            .size(16.0)
                            .color(tokens.text_primary),
                    );
                    ui.add_space(8.0);

                    let avail_w = ui.available_width();
                    let card_w = ((avail_w - 36.0) / 4.0).max(165.0);

                    ui.horizontal_wrapped(|ui| {
                        if render_action_card(
                            ui,
                            card_w,
                            "🤖 Play Bots",
                            "Challenge 18 personalities from Martin (250) to Magnus & Mittens",
                            "Open Lobby",
                            tokens.accent_primary,
                            &tokens,
                        ) {
                            actions.navigate_to = Some(ActiveRoute::PlayBots);
                        }

                        if render_action_card(
                            ui,
                            card_w,
                            "🎓 Play Coach",
                            "Friendly guided match with move feedback, hints, and takebacks",
                            "Play Friendly",
                            tokens.accent_info,
                            &tokens,
                        ) {
                            actions.start_coach_match = true;
                        }

                        if render_action_card(
                            ui,
                            card_w,
                            "🧩 Daily Puzzle",
                            "Sharpen tactical vision with 30+ curated combinations & 5-heart rush",
                            "Solve Puzzles",
                            tokens.accent_gold,
                            &tokens,
                        ) {
                            actions.navigate_to = Some(ActiveRoute::Puzzles);
                        }

                        if render_action_card(
                            ui,
                            card_w,
                            "📊 Game Review",
                            "Analyze accuracy %, Brilliant (!!) moves, and win probability graph",
                            "Review Match",
                            Color32::from_rgb(155, 110, 235),
                            &tokens,
                        ) {
                            actions.navigate_to = Some(ActiveRoute::Review);
                        }
                    });

                    ui.add_space(18.0);

                    // 3. Featured Challengers + Daily Puzzle Preview + Recent Matches
                    ui.columns(2, |cols| {
                        // Left Column: Featured Bot Challengers & Daily Puzzle Mini Board
                        cols[0].vertical(|ui| {
                            ui.label(
                                RichText::new("Featured Challengers")
                                    .strong()
                                    .size(15.5)
                                    .color(tokens.text_primary),
                            );
                            ui.add_space(6.0);

                            let featured = [
                                BotPersona::martin(),
                                BotPersona::nelson(),
                                BotPersona::hikaru(),
                                BotPersona::magnus(),
                                BotPersona::mittens(),
                            ];

                            for bot in featured {
                                let crowns = profile.crowns_for(bot.code);
                                egui::Frame::none()
                                    .fill(tokens.bg_surface)
                                    .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                                    .rounding(Rounding::same(8.0_f32))
                                    .inner_margin(egui::Margin::symmetric(10.0, 7.0))
                                    .show(ui, |ui| {
                                        ui.horizontal(|ui| {
                                            let (av_rect, _) = ui
                                                .allocate_exact_size(Vec2::splat(36.0), Sense::hover());
                                            draw_bot_avatar(
                                                ui.painter(),
                                                av_rect,
                                                bot.id,
                                                bot.accent_rgb,
                                                false,
                                            );
                                            ui.add_space(6.0);
                                            ui.vertical(|ui| {
                                                let elo_txt = if bot.code == "mittens" {
                                                    "1".to_string()
                                                } else {
                                                    format!("{}", bot.elo)
                                                };
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        RichText::new(bot.name)
                                                            .strong()
                                                            .size(13.5)
                                                            .color(tokens.text_primary),
                                                    );
                                                    ui.label(
                                                        RichText::new(format!("({elo_txt})"))
                                                            .size(12.0)
                                                            .color(tokens.accent_gold),
                                                    );
                                                    let crown_str: String = (0..3)
                                                        .map(|i| if i < crowns { '👑' } else { '☆' })
                                                        .collect();
                                                    ui.label(RichText::new(crown_str).size(11.0));
                                                });
                                                ui.label(
                                                    RichText::new(bot.title)
                                                        .size(11.5)
                                                        .color(tokens.text_secondary),
                                                );
                                            });

                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui
                                                        .add(
                                                            egui::Button::new(
                                                                RichText::new("Challenge")
                                                                    .strong()
                                                                    .size(12.0)
                                                                    .color(Color32::WHITE),
                                                            )
                                                            .fill(tokens.accent_primary),
                                                        )
                                                        .clicked()
                                                    {
                                                        actions.quick_start_bot = Some(bot.id);
                                                    }
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(5.0);
                            }
                        });

                        // Right Column: Daily Puzzle Board Preview & Recent Match History
                        cols[1].vertical(|ui| {
                            ui.label(
                                RichText::new("Daily Tactical Spotlight")
                                    .strong()
                                    .size(15.5)
                                    .color(tokens.text_primary),
                            );
                            ui.add_space(6.0);

                            egui::Frame::none()
                                .fill(tokens.bg_surface)
                                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                                .rounding(Rounding::same(8.0_f32))
                                .inner_margin(egui::Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let preview_size = 152.0;
                                        let (b_rect, resp) = ui.allocate_exact_size(
                                            Vec2::splat(preview_size),
                                            Sense::click(),
                                        );
                                        if let Ok(puzzle_pos) = parse_fen(
                                            "r5k1/5ppp/8/8/8/8/4RPPP/4R1K1 w - - 0 1",
                                        ) {
                                            render_board_grid(
                                                ui.painter(),
                                                b_rect,
                                                &puzzle_pos,
                                                &board_theme.palette(),
                                                false,
                                                false,
                                                None,
                                                None,
                                                &[],
                                                &[],
                                                &[],
                                                None,
                                                None,
                                                None,
                                            );
                                        }
                                        if resp.clicked() {
                                            actions.navigate_to = Some(ActiveRoute::Puzzles);
                                        }

                                        ui.add_space(8.0);
                                        ui.vertical(|ui| {
                                            ui.label(
                                                RichText::new("Back-Rank Deflection")
                                                    .strong()
                                                    .size(14.5)
                                                    .color(tokens.text_primary),
                                            );
                                            ui.label(
                                                RichText::new("White to Move • Mate in 2")
                                                    .size(12.0)
                                                    .color(tokens.accent_gold),
                                            );
                                            ui.add_space(8.0);
                                            ui.label(
                                                RichText::new(
                                                    "Find the forcing rook sacrifice that breaks Black's 8th-rank defense.",
                                                )
                                                .size(12.0)
                                                .color(tokens.text_secondary),
                                            );
                                            ui.add_space(10.0);
                                            if ui
                                                .add(
                                                    egui::Button::new(
                                                        RichText::new("▶ Solve Now")
                                                            .strong()
                                                            .color(Color32::WHITE),
                                                    )
                                                    .fill(tokens.accent_primary),
                                                )
                                                .clicked()
                                            {
                                                actions.navigate_to = Some(ActiveRoute::Puzzles);
                                            }
                                        });
                                    });
                                });

                            ui.add_space(12.0);
                            ui.label(
                                RichText::new("Recent Completed Matches")
                                    .strong()
                                    .size(15.5)
                                    .color(tokens.text_primary),
                            );
                            ui.add_space(6.0);

                            egui::Frame::none()
                                .fill(tokens.bg_surface)
                                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                                .rounding(Rounding::same(8.0_f32))
                                .inner_margin(egui::Margin::same(10.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    if profile.recent_matches.is_empty() {
                                        ui.label(
                                            RichText::new(
                                                "No completed matches yet. Defeat a bot to record your PGN & accuracy!",
                                            )
                                            .italics()
                                            .size(12.0)
                                            .color(tokens.text_secondary),
                                        );
                                    } else {
                                        for m in profile.recent_matches.iter().take(4) {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(format!(
                                                        "vs {} ({})",
                                                        m.opponent_name, m.opponent_elo
                                                    ))
                                                    .strong()
                                                    .size(12.5)
                                                    .color(tokens.text_primary),
                                                );
                                                ui.label(
                                                    RichText::new(&m.result_label)
                                                        .size(12.0)
                                                        .color(tokens.accent_gold),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "Acc: {:.1}% • {} moves",
                                                                m.player_accuracy, m.moves_count
                                                            ))
                                                            .monospace()
                                                            .size(11.5)
                                                            .color(tokens.text_secondary),
                                                        );
                                                    },
                                                );
                                            });
                                            ui.separator();
                                        }
                                    }
                                });
                        });
                    });
                });
            });
            ui.add_space(16.0);
        });

    actions
}

fn render_action_card(
    ui: &mut Ui,
    width: f32,
    title: &str,
    desc: &str,
    cta: &str,
    accent: Color32,
    tokens: &UiTokens,
) -> bool {
    let mut clicked = false;
    egui::Frame::none()
        .fill(tokens.bg_surface)
        .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
        .rounding(Rounding::same(8.0_f32))
        .inner_margin(egui::Margin::same(12.0))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.set_min_height(118.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .strong()
                        .size(15.5)
                        .color(tokens.text_primary),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(desc)
                        .size(11.5)
                        .color(tokens.text_secondary),
                );
                ui.add_space(10.0);
                if ui
                    .add_sized(
                        [width - 8.0, 30.0],
                        egui::Button::new(
                            RichText::new(cta)
                                .strong()
                                .size(12.5)
                                .color(Color32::WHITE),
                        )
                        .fill(accent),
                    )
                    .clicked()
                {
                    clicked = true;
                }
            });
        });
    clicked
}
