use crate::{
    config::{AssistanceMode, PlayColorChoice, UserConfig},
    domain::{ChessMatchState, TimeControl},
    engine::bot::{BotId, BotPersona},
    persistence::UserProfile,
    ui::{
        board_view::render_board_grid,
        piece_renderer::draw_bot_avatar,
        side_panel::render_player_strip,
        theme::{chess_cta_button, lerp_color, soft_card_frame, UiTokens},
    },
};
use egui::{Color32, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone)]
pub struct BotsLobbyState {
    pub selected_bot: BotId,
    pub play_color_choice: PlayColorChoice,
    pub custom_engine_elo: u32,
}

impl Default for BotsLobbyState {
    fn default() -> Self {
        Self {
            selected_bot: BotId::Magnus,
            play_color_choice: PlayColorChoice::White,
            custom_engine_elo: 1500,
        }
    }
}

#[derive(Debug, Default)]
pub struct BotsLobbyActions {
    pub start_match_requested: bool,
}

/// Merender 1:1 Play Bots Lobby (`Route::PlayBots`) sesuai spesifikasi NEW_DESIGN.md Section 4:
/// - Left Column: Centered 8x8 preview board flanked by player summary strips.
/// - Right Column (`380 px` Fixed Width):
///   1. Selected Bot Hero Card (Pinned Top)
///   2. Grouped 18-Bot Roster + 250-3200 Engine Slider (`ScrollArea` Middle)
///   3. Pre-Match Configuration (`Play As`, `Time Control`, `Assistance Mode`, dan tombol hijau `Play` 44 px)
pub fn render_bots_lobby(
    ui: &mut Ui,
    lobby_state: &mut BotsLobbyState,
    profile: &mut UserProfile,
    config: &mut UserConfig,
    preview_match: &ChessMatchState,
) -> BotsLobbyActions {
    let mut actions = BotsLobbyActions::default();
    let tokens = UiTokens::for_mode(config.dark_mode);

    let active_persona = match lobby_state.selected_bot {
        BotId::AdaptiveJimmy => BotPersona::adaptive_jimmy(profile.adaptive_jimmy_elo),
        BotId::EngineCustom => BotPersona::engine_custom(lobby_state.custom_engine_elo),
        id => BotPersona::by_id(id),
    };

    egui::SidePanel::right("bots_lobby_right_column")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(6.0);

            // 1. Selected Bot Hero Card (Pinned Top)
            let earned_crowns = profile.crowns_for(active_persona.code);
            let accent = Color32::from_rgb(
                active_persona.accent_rgb[0],
                active_persona.accent_rgb[1],
                active_persona.accent_rgb[2],
            );

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.5_f32, accent))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(10.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (av_rect, _) =
                            ui.allocate_exact_size(Vec2::splat(52.0), Sense::hover());
                        draw_bot_avatar(
                            ui.painter(),
                            av_rect,
                            active_persona.id,
                            active_persona.accent_rgb,
                            true,
                        );
                        ui.add_space(6.0);
                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(active_persona.name)
                                        .strong()
                                        .size(15.5)
                                        .color(tokens.text_primary),
                                );

                                let elo_str = if active_persona.code == "mittens" {
                                    "1".to_string()
                                } else {
                                    format!("{}", active_persona.elo)
                                };

                                egui::Frame::none()
                                    .fill(Color32::from_rgba_unmultiplied(0xFF, 0xAA, 0x00, 40))
                                    .stroke(Stroke::new(1.0_f32, tokens.accent_gold))
                                    .rounding(Rounding::same(4.0_f32))
                                    .inner_margin(egui::Margin::symmetric(5.0, 1.0))
                                    .show(ui, |ui| {
                                        ui.label(
                                            RichText::new(elo_str)
                                                .strong()
                                                .size(11.0)
                                                .color(tokens.accent_gold),
                                        );
                                    });

                                let crown_str: String = (0..3)
                                    .map(|i| if i < earned_crowns { '👑' } else { '☆' })
                                    .collect();
                                ui.label(
                                    RichText::new(crown_str)
                                        .size(11.5)
                                        .color(tokens.accent_gold),
                                );
                            });

                            ui.label(
                                RichText::new(format!(
                                    "{} • {}",
                                    active_persona.tier, active_persona.title
                                ))
                                .italics()
                                .size(11.5)
                                .color(tokens.text_secondary),
                            );
                        });
                    });

                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(active_persona.bio)
                            .size(11.5)
                            .color(tokens.text_primary),
                    );
                });

            ui.add_space(6.0);

            // Reserve space for Pinned Bottom Pre-Match Configuration
            let bottom_config_reserve = if config.assistance_mode == AssistanceMode::Custom {
                238.0
            } else {
                194.0
            };
            let roster_h = (ui.available_height() - bottom_config_reserve).max(110.0);

            // 2. Grouped Bot Roster (ScrollArea Middle)
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .id_salt("bots_lobby_roster_scroll")
                        .max_height(roster_h)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let tiers: &[(&str, &[BotId])] = &[
                                ("Adaptive", &[BotId::AdaptiveJimmy]),
                                (
                                    "Beginner (400–850)",
                                    &[
                                        BotId::Martin,
                                        BotId::Elani,
                                        BotId::Wayne,
                                        BotId::Jimmy,
                                        BotId::Aron,
                                        BotId::Maria,
                                    ],
                                ),
                                (
                                    "Intermediate (1000–1400)",
                                    &[
                                        BotId::Emir,
                                        BotId::Sven,
                                        BotId::Wendy,
                                        BotId::Nelson,
                                        BotId::Jade,
                                    ],
                                ),
                                (
                                    "Advanced (1500–2000)",
                                    &[
                                        BotId::Antonio,
                                        BotId::Isabel,
                                        BotId::David,
                                        BotId::Wally,
                                        BotId::Li,
                                    ],
                                ),
                                (
                                    "Master (2200–2450)",
                                    &[BotId::Noam, BotId::Francis, BotId::Beth],
                                ),
                                (
                                    "Top Players (2500–2882)",
                                    &[BotId::Danny, BotId::Hikaru, BotId::Magnus],
                                ),
                                ("Personality", &[BotId::Mittens]),
                            ];

                            for &(tier_name, bot_ids) in tiers {
                                ui.label(
                                    RichText::new(tier_name)
                                        .strong()
                                        .size(12.0)
                                        .color(tokens.accent_gold),
                                );
                                ui.add_space(3.0);

                                ui.horizontal_wrapped(|ui| {
                                    for &bot_id in bot_ids {
                                        let p = match bot_id {
                                            BotId::AdaptiveJimmy => BotPersona::adaptive_jimmy(
                                                profile.adaptive_jimmy_elo,
                                            ),
                                            other => BotPersona::by_id(other),
                                        };
                                        let is_sel = lobby_state.selected_bot == bot_id;
                                        let crowns = profile.crowns_for(p.code);
                                        if render_bot_roster_tile(
                                            ui, &p, crowns, is_sel, &tokens,
                                        ) {
                                            lobby_state.selected_bot = bot_id;
                                        }
                                    }
                                });
                                ui.add_space(6.0);
                            }

                            // Custom Engine (250–3200 Slider in 50 ELO increments)
                            ui.separator();
                            ui.horizontal(|ui| {
                                let is_eng = lobby_state.selected_bot == BotId::EngineCustom;
                                if ui
                                    .selectable_label(
                                        is_eng,
                                        RichText::new(format!(
                                            "⚙ Custom Engine ({} ELO)",
                                            lobby_state.custom_engine_elo
                                        ))
                                        .strong()
                                        .size(12.5),
                                    )
                                    .clicked()
                                {
                                    lobby_state.selected_bot = BotId::EngineCustom;
                                }
                            });

                            let mut slider_val = lobby_state.custom_engine_elo;
                            if ui
                                .add(
                                    egui::Slider::new(&mut slider_val, 250..=3200)
                                        .step_by(50.0)
                                        .text("ELO"),
                                )
                                .changed()
                            {
                                lobby_state.custom_engine_elo = ((slider_val + 25) / 50) * 50;
                                lobby_state.selected_bot = BotId::EngineCustom;
                                profile.custom_engine_elo = lobby_state.custom_engine_elo;
                            }
                        });
                });

            ui.add_space(6.0);

            // 3. Pre-Match Configuration (Pinned Bottom)
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(8.0))
                .show(ui, |ui| {
                    // Play As: [White] [Random] [Black] & Time Control
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Play As:")
                                .strong()
                                .size(12.0)
                                .color(tokens.text_primary),
                        );
                        ui.selectable_value(
                            &mut lobby_state.play_color_choice,
                            PlayColorChoice::White,
                            "⬜ White",
                        );
                        ui.selectable_value(
                            &mut lobby_state.play_color_choice,
                            PlayColorChoice::Random,
                            "🎲 Random",
                        );
                        ui.selectable_value(
                            &mut lobby_state.play_color_choice,
                            PlayColorChoice::Black,
                            "⬛ Black",
                        );
                    });

                    ui.add_space(4.0);

                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new("Time:")
                                .strong()
                                .size(12.0)
                                .color(tokens.text_primary),
                        );
                        egui::ComboBox::from_id_salt("lobby_time_control_combo")
                            .selected_text(format!(
                                "{} ({})",
                                config.time_control.label(),
                                config.time_control.category()
                            ))
                            .width(160.0)
                            .show_ui(ui, |ui| {
                                for &tc in TimeControl::all_presets() {
                                    ui.selectable_value(&mut config.time_control, tc, tc.label());
                                }
                            });
                    });

                    ui.add_space(4.0);

                    // Assistance Mode selector (Challenge / Friendly / Assisted / Custom)
                    ui.horizontal(|ui| {
                        for &mode in AssistanceMode::all() {
                            let label = match mode {
                                AssistanceMode::Challenge => "Challenge 3👑",
                                AssistanceMode::Friendly => "Friendly 2👑",
                                AssistanceMode::Assisted => "Assisted 1👑",
                                AssistanceMode::Custom => "Custom",
                            };
                            ui.selectable_value(&mut config.assistance_mode, mode, label);
                        }
                    });

                    if config.assistance_mode == AssistanceMode::Custom {
                        ui.horizontal_wrapped(|ui| {
                            ui.checkbox(&mut config.custom_assistance.bot_chat, "Chat");
                            ui.checkbox(&mut config.custom_assistance.eval_bar, "Eval Bar");
                            ui.checkbox(&mut config.custom_assistance.threat_arrows, "Threats");
                            ui.checkbox(
                                &mut config.custom_assistance.suggestion_arrows,
                                "Suggestions",
                            );
                            ui.checkbox(&mut config.custom_assistance.move_feedback, "Feedback");
                        });
                    } else {
                        ui.label(
                            RichText::new(config.assistance_mode.description())
                                .size(10.5)
                                .color(tokens.text_secondary),
                        );
                    }

                    ui.add_space(6.0);

                    // Primary CTA: Full-width tactile 3D Chess.com green "Play" button
                    let full_w = ui.available_width();
                    if chess_cta_button(
                        ui,
                        Vec2::new(full_w, 44.0),
                        "▶  Play",
                        tokens.accent_primary,
                    ) {
                        actions.start_match_requested = true;
                    }
                });
        });

    // Left Column (Flexible Workspace): Centered 8x8 preview board flanked by player strips
    egui::CentralPanel::default().show_inside(ui, |ui| {
        let avail = ui.available_size();
        let board_size = (avail.x - 24.0).min(avail.y - 104.0).max(320.0);
        let pad_x = ((avail.x - board_size) * 0.5).max(8.0);
        let pad_y = ((avail.y - (board_size + 96.0)) * 0.5).max(4.0);

        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            render_player_strip(
                ui,
                board_size,
                preview_match,
                &active_persona,
                true,
                false,
                config.dark_mode,
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            let (b_rect, _) =
                ui.allocate_exact_size(Vec2::splat(board_size), Sense::hover());
            render_board_grid(
                ui.painter(),
                b_rect,
                &preview_match.displayed_position(),
                &config.board_theme.palette(),
                config.board_flipped,
                config.show_coordinates,
                None,
                preview_match.displayed_last_move(),
                &[],
                &[],
                &[],
                None,
                None,
                None,
            );
        });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            render_player_strip(
                ui,
                board_size,
                preview_match,
                &active_persona,
                false,
                false,
                config.dark_mode,
            );
        });
    });

    actions
}

fn render_bot_roster_tile(
    ui: &mut Ui,
    persona: &BotPersona,
    crowns: u8,
    is_selected: bool,
    tokens: &UiTokens,
) -> bool {
    let tile_id = ui.make_persistent_id(format!("bot_tile_{}", persona.code));
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(78.0, 74.0), Sense::click());
    let hovered = resp.hovered();

    let hover_t = ui
        .ctx()
        .animate_bool_with_time(tile_id.with("h"), hovered, 0.12);
    let sel_t = ui
        .ctx()
        .animate_bool_with_time(tile_id.with("s"), is_selected, 0.16);

    let y_lift = -1.8 * hover_t;
    let draw_rect = rect.translate(Vec2::new(0.0, y_lift));

    let accent = Color32::from_rgb(
        persona.accent_rgb[0],
        persona.accent_rgb[1],
        persona.accent_rgb[2],
    );
    let idle_bg = Color32::from_rgba_unmultiplied(
        tokens.bg_surface_elevated.r(),
        tokens.bg_surface_elevated.g(),
        tokens.bg_surface_elevated.b(),
        (75.0 + hover_t * 140.0) as u8,
    );
    let sel_bg = Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), 36);
    let fill = lerp_color(idle_bg, sel_bg, sel_t);

    let border_col = lerp_color(
        lerp_color(tokens.border_subtle, accent.gamma_multiply(0.7), hover_t),
        tokens.accent_primary,
        sel_t,
    );

    let painter = ui.painter();
    painter.rect(
        draw_rect,
        Rounding::same(10.0_f32),
        fill,
        Stroke::new(1.0_f32 + sel_t * 1.0_f32, border_col),
    );

    let av_rect = egui::Rect::from_center_size(
        egui::Pos2::new(draw_rect.center().x, draw_rect.min.y + 22.0),
        Vec2::splat(34.0),
    );
    draw_bot_avatar(
        painter,
        av_rect,
        persona.id,
        persona.accent_rgb,
        is_selected || hovered,
    );

    painter.text(
        egui::Pos2::new(draw_rect.center().x, draw_rect.min.y + 47.0),
        egui::Align2::CENTER_CENTER,
        persona.name,
        egui::FontId::proportional(11.0),
        tokens.text_primary,
    );

    let elo_txt = if persona.code == "mittens" {
        "1".to_string()
    } else {
        format!("{}", persona.elo)
    };
    let crown_txt: String = (0..3)
        .map(|i| if i < crowns { '👑' } else { '☆' })
        .collect();
    painter.text(
        egui::Pos2::new(draw_rect.center().x, draw_rect.min.y + 62.0),
        egui::Align2::CENTER_CENTER,
        format!("{elo_txt} {crown_txt}"),
        egui::FontId::proportional(9.5),
        accent,
    );

    let _ = soft_card_frame;
    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    resp.clicked()
}
