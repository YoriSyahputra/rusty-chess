use crate::{
    config::UserConfig,
    ui::{
        piece_renderer::draw_chess_piece,
        theme::{lerp_color, tactile_surface_button, UiTokens},
    },
};
use egui::{Align2, Color32, FontId, Pos2, Rect, RichText, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Color, Role};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ActiveRoute {
    Home,
    PlayHub,
    PlayBots,
    Match,
    Puzzles,
    Endgames,
    Arena,
    Vision,
    Review,
    Analytics,
}

#[derive(Debug, Default)]
pub struct SidebarActions {
    pub navigate_to: Option<ActiveRoute>,
    pub toggled_theme_mode: bool,
    pub open_settings_modal: bool,
}

/// Merender Left Navigation Sidebar yang fluid dengan animated hover fade & glowing active pill.
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
        ui.add_space(12.0);

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
                        RichText::new("👑 v1.0.0")
                            .size(10.5)
                            .color(tokens.accent_gold),
                    );
                });
            }
        });

        ui.add_space(12.0);

        // 2. Animated Fluid Navigation Destinations
        let nav_items = [
            (ActiveRoute::Home, "🏠", "Home"),
            (ActiveRoute::PlayHub, "♟", "Play"),
            (ActiveRoute::PlayBots, "♞", "Bots"),
            (ActiveRoute::Puzzles, "★", "Puzzles"),
            (ActiveRoute::Endgames, "♔", "Endgames"),
            (ActiveRoute::Arena, "⚔", "Arena"),
            (ActiveRoute::Vision, "🎯", "Vision"),
            (ActiveRoute::Review, "📊", "Review"),
            (ActiveRoute::Analytics, "📈", "Insights"),
        ];

        for (route, icon, label) in nav_items {
            let is_active = current_route == route;
            if render_fluid_nav_button(ui, icon, label, is_active, compact, &tokens) {
                actions.navigate_to = Some(route);
            }
            ui.add_space(2.0);
        }

        if has_active_match {
            ui.add_space(6.0);
            let is_match = current_route == ActiveRoute::Match;
            if render_fluid_nav_button(ui, "⚔", "Live Match", is_match, compact, &tokens) {
                actions.navigate_to = Some(ActiveRoute::Match);
            }
        }

        // 3. Utility Footer (Bottom) — smooth tactile surface buttons
        ui.with_layout(egui::Layout::bottom_up(egui::Align::Min), |ui| {
            ui.add_space(10.0);
            let w = (ui.available_width() - 10.0).max(40.0);

            let chevron_label = if compact { "»" } else { "« Collapse" };
            if tactile_surface_button(ui, Vec2::new(w, 29.0), chevron_label, true, &tokens)
                .on_hover_text("Toggle Sidebar Width")
                .clicked()
            {
                config.sidebar_collapsed = Some(!compact);
            }

            ui.add_space(4.0);

            let settings_label = if compact { "⚙" } else { "⚙  Settings" };
            if tactile_surface_button(ui, Vec2::new(w, 32.0), settings_label, true, &tokens)
                .on_hover_text("Board, Animation & Audio Settings")
                .clicked()
            {
                actions.open_settings_modal = true;
            }

            ui.add_space(4.0);

            let mode_label = if compact {
                if config.dark_mode { "🌙" } else { "☀" }
            } else if config.dark_mode {
                "🌙  Dark UI"
            } else {
                "☀  Light UI"
            };
            if tactile_surface_button(ui, Vec2::new(w, 32.0), mode_label, true, &tokens)
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

fn render_fluid_nav_button(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    is_active: bool,
    compact: bool,
    tokens: &UiTokens,
) -> bool {
    let w = (ui.available_width() - 10.0).max(40.0);
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, 35.0), Sense::click());
    let hovered = resp.hovered();

    let hover_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("nav_h"), hovered, 0.13);
    let active_t = ui
        .ctx()
        .animate_bool_with_time(resp.id.with("nav_a"), is_active, 0.16);

    let hover_bg = if tokens.dark_mode {
        Color32::from_rgba_unmultiplied(255, 255, 255, (hover_t * 18.0) as u8)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, (hover_t * 14.0) as u8)
    };
    let active_bg = Color32::from_rgba_unmultiplied(
        tokens.accent_primary.r(),
        tokens.accent_primary.g(),
        tokens.accent_primary.b(),
        (active_t * 48.0) as u8,
    );
    let bg = if active_t > 0.05 {
        lerp_color(hover_bg, active_bg, active_t)
    } else {
        hover_bg
    };

    let painter = ui.painter();
    let rounding = Rounding::same(10.0_f32);
    painter.rect_filled(rect, rounding, bg);

    // Smooth glowing left accent indicator bar on active route
    if active_t > 0.02 {
        let bar_h = 20.0 * active_t;
        let bar_rect = Rect::from_center_size(
            Pos2::new(rect.min.x + 3.5, rect.center().y),
            Vec2::new(3.5, bar_h),
        );
        painter.rect_filled(bar_rect, Rounding::same(2.0_f32), tokens.accent_primary);
        painter.rect_stroke(
            rect,
            rounding,
            Stroke::new(
                1.0_f32,
                Color32::from_rgba_unmultiplied(
                    tokens.accent_primary.r(),
                    tokens.accent_primary.g(),
                    tokens.accent_primary.b(),
                    (active_t * 90.0) as u8,
                ),
            ),
        );
    }

    let fg = lerp_color(
        lerp_color(tokens.text_secondary, tokens.text_primary, hover_t),
        if tokens.dark_mode {
            Color32::WHITE
        } else {
            tokens.accent_primary
        },
        active_t,
    );

    if compact {
        painter.text(
            rect.center(),
            Align2::CENTER_CENTER,
            icon,
            FontId::proportional(16.0),
            fg,
        );
    } else {
        let x_slide = hover_t * 2.0;
        painter.text(
            Pos2::new(rect.min.x + 15.0 + x_slide, rect.center().y),
            Align2::LEFT_CENTER,
            icon,
            FontId::proportional(14.5),
            if active_t > 0.5 { tokens.accent_primary } else { fg },
        );
        painter.text(
            Pos2::new(rect.min.x + 39.0 + x_slide, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            FontId::proportional(13.5),
            fg,
        );
    }

    if hovered {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
    }

    resp.on_hover_text(label).clicked()
}
