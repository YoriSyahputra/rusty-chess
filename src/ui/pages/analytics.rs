use crate::{
    persistence::{SkillDnaMetrics, UserProfile},
    ui::{sidebar::ActiveRoute, theme::UiTokens},
};
use egui::{Align2, Color32, FontId, Painter, Pos2, RichText, Rounding, ScrollArea, Sense, Shape, Stroke, Ui, Vec2};
use std::f32::consts::PI;

#[derive(Debug, Default)]
pub struct AnalyticsPageActions {
    pub navigate_to: Option<ActiveRoute>,
    pub open_blunder_deck: bool,
}

/// Merender halaman Analytics & Mastery Insights (`Route::Analytics`):
/// - Hexagonal "Your Chess DNA" Skill Radar Chart (Feature #3)
/// - Personal Opening Repertoire Win/Draw/Loss Table (Feature #1)
/// - Personal Mistake Vault ("Blunder Deck") Overview (Feature #5)
pub fn render_analytics_page(
    ui: &mut Ui,
    profile: &UserProfile,
    dark_mode: bool,
) -> AnalyticsPageActions {
    let mut actions = AnalyticsPageActions::default();
    let tokens = UiTokens::for_mode(dark_mode);
    let dna = profile.compute_skill_dna();

    ScrollArea::vertical()
        .id_salt("analytics_insights_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.add_space(14.0);
            ui.horizontal(|ui| {
                ui.add_space(18.0);
                ui.vertical(|ui| {
                    ui.set_max_width((ui.available_width() - 24.0).max(620.0));

                    ui.label(
                        RichText::new("📈 Chess DNA & Performance Analytics")
                            .strong()
                            .size(21.0)
                            .color(tokens.text_primary),
                    );
                    ui.label(
                        RichText::new(
                            "Real-time mastery insights derived from your saved games, opening repertoire, puzzles, and blunder drills.",
                        )
                        .size(12.5)
                        .color(tokens.text_secondary),
                    );
                    ui.add_space(14.0);

                    ui.columns(2, |cols| {
                        // Left Column: Hexagonal Skill Radar Chart ("Your Chess DNA")
                        cols[0].vertical(|ui| {
                            egui::Frame::none()
                                .fill(tokens.bg_surface)
                                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                                .rounding(Rounding::same(10.0_f32))
                                .inner_margin(egui::Margin::same(14.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.horizontal(|ui| {
                                        ui.label(
                                            RichText::new("🧬 Your Chess DNA")
                                                .strong()
                                                .size(16.0)
                                                .color(tokens.text_primary),
                                        );
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                egui::Frame::none()
                                                    .fill(Color32::from_rgba_unmultiplied(
                                                        0xFF, 0xAA, 0x00, 38,
                                                    ))
                                                    .stroke(Stroke::new(
                                                        1.0_f32,
                                                        tokens.accent_gold,
                                                    ))
                                                    .rounding(Rounding::same(5.0_f32))
                                                    .inner_margin(egui::Margin::symmetric(8.0, 2.0))
                                                    .show(ui, |ui| {
                                                        ui.label(
                                                            RichText::new(dna.archetype)
                                                                .strong()
                                                                .size(12.0)
                                                                .color(tokens.accent_gold),
                                                        );
                                                    });
                                            },
                                        );
                                    });

                                    ui.add_space(8.0);
                                    let radar_size =
                                        (ui.available_width() - 16.0).clamp(240.0, 320.0);
                                    ui.vertical_centered(|ui| {
                                        let (rect, _) = ui.allocate_exact_size(
                                            Vec2::splat(radar_size),
                                            Sense::hover(),
                                        );
                                        draw_skill_radar_chart(
                                            ui.painter(),
                                            rect.center(),
                                            radar_size * 0.36,
                                            &dna,
                                            &tokens,
                                        );
                                    });

                                    ui.add_space(6.0);
                                    ui.columns(3, |sc| {
                                        render_dna_pill(&mut sc[0], "Opening", dna.opening, &tokens);
                                        render_dna_pill(&mut sc[1], "Tactics", dna.tactics, &tokens);
                                        render_dna_pill(&mut sc[2], "Endgame", dna.endgame, &tokens);
                                    });
                                    ui.add_space(4.0);
                                    ui.columns(3, |sc| {
                                        render_dna_pill(
                                            &mut sc[0],
                                            "Time Mgmt",
                                            dna.time_mgmt,
                                            &tokens,
                                        );
                                        render_dna_pill(&mut sc[1], "Attack", dna.attack, &tokens);
                                        render_dna_pill(
                                            &mut sc[2],
                                            "Resilience",
                                            dna.resilience,
                                            &tokens,
                                        );
                                    });
                                });
                        });

                        // Right Column: Mistake Vault Card + Opening Explorer Win Rates
                        cols[1].vertical(|ui| {
                            // Mistake Vault Summary Card
                            let active_blunders = profile.active_mistakes_count();
                            let mastered_blunders = profile
                                .mistake_vault
                                .iter()
                                .filter(|m| m.is_mastered())
                                .count();

                            egui::Frame::none()
                                .fill(tokens.bg_surface)
                                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                                .rounding(Rounding::same(10.0_f32))
                                .inner_margin(egui::Margin::same(14.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(
                                        RichText::new("🔥 Personal Mistake Vault (Blunder Deck)")
                                            .strong()
                                            .size(15.5)
                                            .color(tokens.text_primary),
                                    );
                                    ui.add_space(4.0);
                                    ui.label(
                                        RichText::new(format!(
                                            "Active Blunders to Drill: {}   •   Mastered (3/3): {}",
                                            active_blunders, mastered_blunders
                                        ))
                                        .size(12.5)
                                        .color(tokens.text_secondary),
                                    );
                                    ui.add_space(8.0);
                                    if ui
                                        .add_sized(
                                            [ui.available_width(), 34.0],
                                            egui::Button::new(
                                                RichText::new("🎯 Practice My Blunder Deck")
                                                    .strong()
                                                    .color(Color32::WHITE),
                                            )
                                            .fill(tokens.accent_primary),
                                        )
                                        .clicked()
                                    {
                                        actions.open_blunder_deck = true;
                                        actions.navigate_to = Some(ActiveRoute::Puzzles);
                                    }
                                });

                            ui.add_space(12.0);

                            // Opening Repertoire Stats Card
                            egui::Frame::none()
                                .fill(tokens.bg_surface)
                                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                                .rounding(Rounding::same(10.0_f32))
                                .inner_margin(egui::Margin::same(14.0))
                                .show(ui, |ui| {
                                    ui.set_min_width(ui.available_width());
                                    ui.label(
                                        RichText::new("📖 Personal Opening Repertoire (ECO)")
                                            .strong()
                                            .size(15.5)
                                            .color(tokens.text_primary),
                                    );
                                    ui.add_space(6.0);

                                    if profile.opening_stats.is_empty() {
                                        ui.label(
                                            RichText::new(
                                                "Complete matches against bots to track your win rates across Sicilian, Ruy Lopez, London, QGD, and 130+ ECO lines!",
                                            )
                                            .italics()
                                            .size(12.0)
                                            .color(tokens.text_secondary),
                                        );
                                    } else {
                                        let mut stats: Vec<_> =
                                            profile.opening_stats.values().collect();
                                        stats.sort_by_key(|s| std::cmp::Reverse(s.total_games()));

                                        for st in stats.into_iter().take(8) {
                                            ui.horizontal(|ui| {
                                                ui.label(
                                                    RichText::new(&st.eco)
                                                        .monospace()
                                                        .strong()
                                                        .size(12.0)
                                                        .color(tokens.accent_gold),
                                                );
                                                ui.label(
                                                    RichText::new(&st.name)
                                                        .size(12.0)
                                                        .color(tokens.text_primary),
                                                );
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                "{:.0}% ({}W-{}D-{}L)",
                                                                st.win_rate_pct(),
                                                                st.wins,
                                                                st.draws,
                                                                st.losses
                                                            ))
                                                            .monospace()
                                                            .size(11.5)
                                                            .color(tokens.accent_primary),
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

pub fn draw_skill_radar_chart(
    painter: &Painter,
    center: Pos2,
    radius: f32,
    dna: &SkillDnaMetrics,
    tokens: &UiTokens,
) {
    let labels = [
        "Opening",
        "Tactics",
        "Endgame",
        "Time",
        "Attack",
        "Resilience",
    ];
    let values = [
        dna.opening,
        dna.tactics,
        dna.endgame,
        dna.time_mgmt,
        dna.attack,
        dna.resilience,
    ];

    // 4 concentric hexagonal rings
    for ring in 1..=4 {
        let r = radius * (ring as f32 / 4.0);
        let ring_pts: Vec<Pos2> = (0..6)
            .map(|i| {
                let angle = -PI * 0.5 + (i as f32) * (PI / 3.0);
                Pos2::new(center.x + r * angle.cos(), center.y + r * angle.sin())
            })
            .collect();
        painter.add(Shape::closed_line(
            ring_pts,
            Stroke::new(1.0_f32, tokens.border_subtle),
        ));
    }

    // Spokes + axis labels
    let mut data_pts = Vec::with_capacity(6);
    for i in 0..6 {
        let angle = -PI * 0.5 + (i as f32) * (PI / 3.0);
        let outer = Pos2::new(
            center.x + radius * angle.cos(),
            center.y + radius * angle.sin(),
        );
        painter.line_segment(
            [center, outer],
            Stroke::new(1.0_f32, tokens.border_subtle),
        );

        let val_r = radius * values[i].clamp(0.15, 1.0);
        data_pts.push(Pos2::new(
            center.x + val_r * angle.cos(),
            center.y + val_r * angle.sin(),
        ));

        let label_r = radius + 22.0;
        let label_pos = Pos2::new(
            center.x + label_r * angle.cos(),
            center.y + label_r * angle.sin(),
        );
        painter.text(
            label_pos,
            Align2::CENTER_CENTER,
            labels[i],
            FontId::proportional(11.5),
            tokens.text_secondary,
        );
    }

    // Translucent skill polygon
    painter.add(Shape::convex_polygon(
        data_pts.clone(),
        Color32::from_rgba_unmultiplied(0x81, 0xB6, 0x4C, 95),
        Stroke::new(2.2_f32, tokens.accent_primary),
    ));

    // Vertex dots
    for pt in data_pts {
        painter.circle_filled(pt, 3.8, tokens.accent_gold);
    }
}

fn render_dna_pill(ui: &mut Ui, label: &str, value: f32, tokens: &UiTokens) {
    let score = (value * 100.0).round() as u32;
    egui::Frame::none()
        .fill(tokens.bg_surface_elevated)
        .rounding(Rounding::same(5.0_f32))
        .inner_margin(egui::Margin::symmetric(6.0, 4.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(label)
                        .size(10.5)
                        .color(tokens.text_secondary),
                );
                ui.label(
                    RichText::new(format!("{score}"))
                        .strong()
                        .size(13.0)
                        .color(tokens.accent_primary),
                );
            });
        });
}
