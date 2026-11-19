use crate::{
    config::UserConfig,
    ui::{piece_renderer::draw_chess_piece, theme::UiTokens},
};
use egui::{Color32, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Color, Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActiveRoute {
    Home,
    PlayHub,
    PlayBots,
    Match,
    Puzzles,
    Review,
    Analytics,
}

#[derive(Debug, Default)]
pub struct SidebarActions {
    pub navigate_to: Option<ActiveRoute>,
    pub toggled_theme_mode: bool,
    pub open_settings_modal: bool,
}

/// Merender Left Navigation Sidebar sesuai spesifikasi NEW_DESIGN.md Section 3:
/// - Width: `168 px` expanded saat lebar window >= 1100 px; otomatis `60 px` compact di bawah 1100 px (atau manual toggle).
/// - Brand Block (Top): Ikon Kuda + Crown & judul `Rusty Chess`.
/// - Navigation Destinations: `Home`, `Play`, `Bots`, `Puzzles`, `Review` (+ `Live Match` bila ada partai berjalan).
/// - Utility Footer (Bottom): `Dark UI / Light UI` toggle & `Settings` modal launcher.
pub fn render_left_sidebar(
    ui: &mut Ui,
    current_route: ActiveRoute,
    config: &mut UserConfig,
    has_active_match: bool,
    compact: bool,
) -> SidebarActions {
    let mut actions = SidebarActions::default();
    let tokens = UiTokens::for_mode(config.dark_mode);

    ui.vertical(|ui| {
        ui.add_space(10.0);

        // 1. Brand Block (Top)
        ui.horizontal(|ui| {
            ui.add_space(if compact { 12.0 } else { 10.0 });
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::splat(30.0), Sense::hover());
            draw_chess_piece(
                ui.painter(),
                icon_rect.center(),
                28.0,
                Color::White,
                Role::Knight,
                false,
            );

            if !compact {
                ui.add_space(4.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Rusty Chess")
                            .strong()
                            .size(16.0)
                            .color(tokens.text_primary),
                    );
                    ui.label(
                        RichText::new("👑 v0.2.0")
                            .size(10.5)
                            .color(tokens.accent_gold),
                    );
                });
            }
        });

        ui.add_space(14.0);

        // 2. Navigation Destinations
        let nav_items = [
            (ActiveRoute::Home, "🏠", "Home"),
            (ActiveRoute::PlayHub, "♟", "Play"),
            (ActiveRoute::PlayBots, "🤖", "Bots"),
            (ActiveRoute::Puzzles, "🧩", "Puzzles"),
            (ActiveRoute::Review, "📊", "Review"),
            (ActiveRoute::Analytics, "📈", "Insights"),
        ];

        for (route, icon, label) in nav_items {
            let is_active = current_route == route;
            if render_nav_button(ui, icon, label, is_active, compact, &tokens) {
                actions.navigate_to = Some(route);
            }
            ui.add_space(4.0);
        }

        if has_active_match {
            ui.add_space(6.0);
            let is_match = current_route == ActiveRoute::Match;
            if render_nav_button(ui, "⚔", "Live Match", is_match, compact, &tokens) {
                actions.navigate_to = Some(ActiveRoute::Match);
            }
        }

        // 3. Utility Footer (Bottom)
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(10.0);

            // Collapse / Expand Chevron
            let chevron_label = if compact {
                "»".to_string()
            } else {
                "« Collapse".to_string()
            };
            if ui
                .add_sized(
                    [ui.available_width() - 12.0, 28.0],
                    egui::Button::new(
                        RichText::new(chevron_label)
                            .size(11.5)
                            .color(tokens.text_secondary),
                    ),
                )
                .on_hover_text("Toggle Sidebar Width")
                .clicked()
            {
                config.sidebar_collapsed = Some(!compact);
            }

            ui.add_space(4.0);

            // Settings Modal Launcher
            let settings_label = if compact {
                "⚙".to_string()
            } else {
                "⚙  Settings".to_string()
            };
            if ui
                .add_sized(
                    [ui.available_width() - 12.0, 32.0],
                    egui::Button::new(
                        RichText::new(settings_label)
                            .size(12.5)
                            .color(tokens.text_primary),
                    ),
                )
                .on_hover_text("Board, Animation & Audio Settings")
                .clicked()
            {
                actions.open_settings_modal = true;
            }

            ui.add_space(4.0);

            // Dark / Light UI Toggle
            let mode_label = if compact {
                if config.dark_mode {
                    "🌙".to_string()
                } else {
                    "☀".to_string()
                }
            } else if config.dark_mode {
                "🌙  Dark UI".to_string()
            } else {
                "☀  Light UI".to_string()
            };

            if ui
                .add_sized(
                    [ui.available_width() - 12.0, 32.0],
                    egui::Button::new(
                        RichText::new(mode_label)
                            .size(12.5)
                            .color(tokens.text_primary),
                    ),
                )
                .on_hover_text("Switch between Dark UI and Light UI")
                .clicked()
            {
                config.dark_mode = !config.dark_mode;
                actions.toggled_theme_mode = true;
            }
        });
    });

    actions
}

fn render_nav_button(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    is_active: bool,
    compact: bool,
    tokens: &UiTokens,
) -> bool {
    let w = (ui.available_width() - 10.0).max(40.0);
    let bg = if is_active {
        tokens.accent_primary
    } else {
        Color32::TRANSPARENT
    };
    let fg = if is_active {
        Color32::WHITE
    } else {
        tokens.text_primary
    };
    let stroke = if is_active {
        Stroke::new(1.0_f32, tokens.accent_primary_hover)
    } else {
        Stroke::NONE
    };

    let text = if compact {
        RichText::new(icon).size(16.5).color(fg)
    } else {
        RichText::new(format!(" {icon}   {label}"))
            .strong()
            .size(14.0)
            .color(fg)
    };

    let btn = egui::Button::new(text)
        .fill(bg)
        .stroke(stroke)
        .rounding(Rounding::same(6.0_f32));

    ui.add_sized([w, 38.0], btn)
        .on_hover_text(label)
        .clicked()
}
