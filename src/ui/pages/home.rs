use crate::{
    domain::parse_fen,
    engine::bot::{BotId, BotPersona},
    persistence::UserProfile,
    ui::{
        board_view::render_board_grid,
        piece_renderer::draw_bot_avatar,
        sidebar::ActiveRoute,
        theme::{chess_cta_button, soft_card_frame, BoardTheme, UiTokens},
    },
};
use egui::{Color32, Pos2, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, TextWrapMode, Ui, Vec2};

#[derive(Debug, Default)]
pub struct HomePageActions {
    pub navigate_to: Option<ActiveRoute>,
    pub quick_start_bot: Option<BotId>,
    pub start_coach_match: bool,
}

/// Merender Home Dashboard (`Route::Home`) yang responsif tanpa overflow/clipping horizontal
/// di resolusi 1180x780 maupun 1366x768.
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
            egui::Frame::none()
                .inner_margin(egui::Margin::symmetric(18.0, 14.0))
                .show(ui, |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Wrap);

                    // 1. Hero Banner & Stats Strip
                    soft_card_frame(&tokens, egui::Margin::same(16.0))
                        .show(ui, |ui| {
                            ui.set_min_width(ui.available_width());
                            ui.label(
                                RichText::new("Welcome to Rusty Chess")
                                    .strong()
                                    .size(22.0)
                                    .color(tokens.text_primary),
                            );
                            ui.label(
                                RichText::new(format!(
                                    "Native Linux Chess Desktop • {} AI Personalities, Tactics Trainer & Real-Time Analysis",
                                    BotId::all_lobby_personalities().len()
                                ))
                                .size(13.0)
                                .color(tokens.text_secondary),
                            );
                            ui.add_space(10.0);

                            let total_crowns = profile.total_crowns();
                            let max_crowns = (BotId::all_lobby_personalities().len() as u32) * 3;
                            let ratio =
                                (total_crowns as f32 / max_crowns.max(1) as f32).clamp(0.0, 1.0);

                            ui.horizontal_wrapped(|ui| {
                                ui.label(
                                    RichText::new(format!(
                                        "👑 Bot Crowns: {total_crowns} / {max_crowns}"
                                    ))
                                    .strong()
                                    .size(13.5)
                                    .color(tokens.accent_gold),
                                );
                                ui.label(RichText::new("•").color(tokens.text_secondary));
                                ui.label(
                                    RichText::new(format!(
                                        "Record: {}W - {}D - {}L",
                                        profile.wins, profile.draws, profile.losses
                                    ))
                                    .size(12.5)
                                    .color(tokens.text_secondary),
                                );
                                ui.label(RichText::new("•").color(tokens.text_secondary));
                                ui.label(
                                    RichText::new(format!("★ Puzzle: {}", profile.puzzle_rating))
                                        .size(12.5)
                                        .color(tokens.text_secondary),
                                );
                                ui.label(RichText::new("•").color(tokens.text_secondary));
                                let dna = profile.compute_skill_dna();
                                ui.label(
                                    RichText::new(format!("◈ DNA: {}", dna.archetype))
                                        .size(12.5)
                                        .color(tokens.accent_info),
                                );
                                if ui.small_button("📈 Insights").clicked() {
                                    actions.navigate_to = Some(ActiveRoute::Analytics);
                                }
                            });

                            ui.add_space(6.0);
                            let bar_w = (ui.available_width() - 8.0).clamp(200.0, 520.0);
                            let (bar_rect, _) =
                                ui.allocate_exact_size(Vec2::new(bar_w, 9.0), Sense::hover());
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

                    ui.add_space(16.0);

                    // 2. Quick Action Cards Row — using exact responsive 4-column layout (zero overflow!)
                    ui.label(
                        RichText::new("Quick Play & Training")
                            .strong()
                            .size(16.0)
                            .color(tokens.text_primary),
                    );
                    ui.add_space(8.0);

                    ui.columns(4, |cols| {
                        if render_action_card(
                            &mut cols[0],
                            "♞ Play Bots",
                            "Challenge 24 personalities from Martin to Magnus & Mittens.",
                            "Open Lobby",
                            tokens.accent_primary,
                            &tokens,
                        ) {
                            actions.navigate_to = Some(ActiveRoute::PlayBots);
                        }

                        if render_action_card(
                            &mut cols[1],
                            "🎓 Play Coach",
                            "Guided match with live move feedback, hints, and takebacks.",
                            "Play Friendly",
                            tokens.accent_info,
                            &tokens,
                        ) {
                            actions.start_coach_match = true;
                        }

                        if render_action_card(
                            &mut cols[2],
                            "★ Daily Puzzle",
                            "Sharpen tactics with 32 curated puzzles & Puzzle Rush.",
                            "Solve Puzzles",
                            tokens.accent_gold,
                            &tokens,
                        ) {
                            actions.navigate_to = Some(ActiveRoute::Puzzles);
                        }

                        if render_action_card(
                            &mut cols[3],
                            "📊 Game Review",
                            "Inspect accuracy %, Brilliant (!!) moves, and Multi-PV.",
                            "Review Match",
                            Color32::from_rgb(145, 102, 228),
                            &tokens,
                        ) {
                            actions.navigate_to = Some(ActiveRoute::Review);
                        }
                    });

                    ui.add_space(16.0);

                    // 3. Featured Challengers + Daily Puzzle Preview & Recent Matches
                    ui.columns(2, |cols| {
                        // Left Column: Featured Challengers
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
                                    .rounding(Rounding::same(10.0_f32))
                                    .inner_margin(egui::Margin::symmetric(10.0, 8.0))
                                    .show(ui, |ui| {
                                        ui.set_min_width(ui.available_width());
                                        ui.horizontal(|ui| {
                                            let (av_rect, _) = ui.allocate_exact_size(
                                                Vec2::splat(36.0),
                                                Sense::hover(),
                                            );
                                            draw_bot_avatar(
                                                ui.painter(),
                                                av_rect,
                                                bot.id,
                                                bot.accent_rgb,
                                                false,
                                            );
                                            ui.add_space(4.0);
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
                                                    if chess_cta_button(
                                                        ui,
                                                        Vec2::new(84.0, 28.0),
                                                        "Challenge",
                                                        tokens.accent_primary,
                                                    ) {
                                                        actions.quick_start_bot = Some(bot.id);
                                                    }
                                                },
                                            );
                                        });
                                    });
                                ui.add_space(6.0);
                            }
                        });

                        // Right Column: Daily Puzzle Spotlight + Recent Matches
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
                                .rounding(Rounding::same(10.0_f32))
                                .inner_margin(egui::Margin::same(12.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    let card_w = ui.available_width();
                                    let preview_size = (card_w * 0.34).clamp(112.0, 142.0);
                                    let text_w = (card_w - preview_size - 16.0).max(120.0);

                                    ui.horizontal(|ui| {
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
                                        ui.allocate_ui_with_layout(
                                            Vec2::new(text_w, preview_size),
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                ui.style_mut().wrap_mode =
                                                    Some(TextWrapMode::Wrap);
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
                                                ui.add_space(6.0);
                                                ui.label(
                                                    RichText::new(
                                                        "Find the forcing rook sacrifice that breaks Black's 8th-rank defense.",
                                                    )
                                                    .size(12.0)
                                                    .color(tokens.text_secondary),
                                                );
                                                ui.add_space(8.0);
                                                if chess_cta_button(
                                                    ui,
                                                    Vec2::new(112.0, 30.0),
                                                    "▶ Solve Now",
                                                    tokens.accent_primary,
                                                ) {
                                                    actions.navigate_to =
                                                        Some(ActiveRoute::Puzzles);
                                                }
                                            },
                                        );
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
                                .rounding(Rounding::same(10.0_f32))
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
                                        for (i, m) in
                                            profile.recent_matches.iter().take(4).enumerate()
                                        {
                                            if i > 0 {
                                                ui.separator();
                                            }
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
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{:.1}% • {}m",
                                                                m.player_accuracy, m.moves_count
                                                            ))
                                                            .monospace()
                                                            .size(11.5)
                                                            .color(tokens.text_secondary),
                                                        );
                                                    },
                                                );
                                            });
                                            ui.label(
                                                RichText::new(&m.result_label)
                                                    .size(11.5)
                                                    .color(tokens.accent_gold),
                                            );
                                        }
                                    }
                                });
                        });
                    });
                });
        });

    actions
}

fn render_action_card(
    ui: &mut Ui,
    title: &str,
    desc: &str,
    cta: &str,
    accent: Color32,
    tokens: &UiTokens,
) -> bool {
    let mut clicked = false;
    soft_card_frame(tokens, egui::Margin::same(12.0))
        .show(ui, |ui| {
            let inner_w = ui.available_width();
            ui.set_min_width(inner_w);
            ui.set_min_height(114.0);
            ui.style_mut().wrap_mode = Some(TextWrapMode::Wrap);

            ui.vertical(|ui| {
                ui.label(
                    RichText::new(title)
                        .strong()
                        .size(14.5)
                        .color(tokens.text_primary),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new(desc)
                        .size(11.5)
                        .color(tokens.text_secondary),
                );
                ui.add_space(10.0);
                if chess_cta_button(ui, Vec2::new(inner_w, 32.0), cta, accent) {
                    clicked = true;
                }
            });
        });
    clicked
}
