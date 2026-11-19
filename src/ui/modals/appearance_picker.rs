use crate::{
    config::{PieceSetStyle, UserConfig},
    ui::{
        piece_renderer::draw_chess_piece_styled,
        theme::{BoardTheme, UiTokens},
    },
};
use egui::{Pos2, Rect, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2, Window};
use shakmaty::{Color, Role};

/// Menampilkan modal visual Appearance Picker dengan preview papan mini 4x4 & preview set bidak langsung.
pub fn show_appearance_picker_modal(
    ctx: &egui::Context,
    open: &mut bool,
    config: &mut UserConfig,
) -> bool {
    if !*open {
        return false;
    }

    let mut changed = false;
    let tokens = UiTokens::for_mode(config.dark_mode);
    let mut keep_open = *open;

    Window::new("◈ Board & Piece Appearance")
        .open(&mut keep_open)
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .fixed_size(Vec2::new(560.0, 490.0))
        .show(ctx, |ui| {
            ScrollArea::vertical()
                .id_salt("appearance_picker_scroll")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if render_appearance_preview_selectors(ui, config, &tokens) {
                        changed = true;
                    }
                });
        });

    *open = keep_open;
    changed
}

/// Widget reusable untuk memilih Board Theme (dengan mini 4x4 preview + bidak) dan Piece Set
/// (dengan preview King, Queen, Knight, Pawn putih & hitam).
pub fn render_appearance_preview_selectors(
    ui: &mut Ui,
    config: &mut UserConfig,
    tokens: &UiTokens,
) -> bool {
    let mut changed = false;

    ui.label(
        RichText::new("Chessboard Theme (Click to Preview & Apply)")
            .strong()
            .size(14.0)
            .color(tokens.text_primary),
    );
    ui.add_space(6.0);

    // Grid of visual 4x4 board theme cards
    let themes = BoardTheme::all();
    for chunk in themes.chunks(4) {
        ui.horizontal(|ui| {
            let card_w = ((ui.available_width() - 24.0) / 4.0).clamp(108.0, 128.0);
            for &theme in chunk {
                let is_sel = config.board_theme == theme;
                let border = if is_sel {
                    Stroke::new(2.0_f32, tokens.accent_primary)
                } else {
                    Stroke::new(1.0_f32, tokens.border_subtle)
                };

                let frame_resp = egui::Frame::none()
                    .fill(if is_sel {
                        tokens.bg_surface_elevated
                    } else {
                        tokens.bg_surface
                    })
                    .stroke(border)
                    .rounding(Rounding::same(8.0_f32))
                    .inner_margin(egui::Margin::same(6.0))
                    .show(ui, |ui| {
                        ui.set_width(card_w - 14.0);
                        ui.vertical_centered(|ui| {
                            let thumb_sz = (card_w - 22.0).clamp(72.0, 96.0);
                            let (rect, resp) =
                                ui.allocate_exact_size(Vec2::splat(thumb_sz), Sense::click());
                            draw_mini_board_preview(
                                ui.painter(),
                                rect,
                                theme,
                                config.piece_set,
                            );
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new(theme.name())
                                    .strong()
                                    .size(11.0)
                                    .color(if is_sel {
                                        tokens.accent_primary
                                    } else {
                                        tokens.text_primary
                                    }),
                            );
                            resp
                        })
                        .inner
                    });

                if frame_resp.inner.clicked() || frame_resp.response.interact(Sense::click()).clicked()
                {
                    config.board_theme = theme;
                    changed = true;
                }
            }
        });
        ui.add_space(6.0);
    }

    if ui
        .checkbox(
            &mut config.textured_board,
            "Subtle Surface Grain Texture (Wood / Matte / Satin)",
        )
        .changed()
    {
        changed = true;
    }

    ui.add_space(10.0);
    ui.separator();
    ui.add_space(6.0);

    ui.label(
        RichText::new("Piece Set Style (Live Preview)")
            .strong()
            .size(14.0)
            .color(tokens.text_primary),
    );
    ui.add_space(6.0);

    let palette = config.board_theme.palette();
    for &ps in PieceSetStyle::all() {
        let is_sel = config.piece_set == ps;
        let border = if is_sel {
            Stroke::new(2.0_f32, tokens.accent_primary)
        } else {
            Stroke::new(1.0_f32, tokens.border_subtle)
        };

        let row_resp = egui::Frame::none()
            .fill(if is_sel {
                tokens.bg_surface_elevated
            } else {
                tokens.bg_surface
            })
            .stroke(border)
            .rounding(Rounding::same(8.0_f32))
            .inner_margin(egui::Margin::symmetric(10.0, 6.0))
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());
                ui.horizontal(|ui| {
                    // Mini 6-square checkered strip with live rendered pieces
                    let sq = 32.0;
                    let (strip_rect, strip_resp) =
                        ui.allocate_exact_size(Vec2::new(sq * 6.0, sq), Sense::click());
                    let painter = ui.painter();
                    let sample_pieces = [
                        (Color::White, Role::King),
                        (Color::White, Role::Queen),
                        (Color::White, Role::Knight),
                        (Color::Black, Role::Rook),
                        (Color::Black, Role::Bishop),
                        (Color::Black, Role::Pawn),
                    ];
                    for (i, &(col, role)) in sample_pieces.iter().enumerate() {
                        let r = Rect::from_min_size(
                            Pos2::new(strip_rect.min.x + i as f32 * sq, strip_rect.min.y),
                            Vec2::splat(sq),
                        );
                        let sq_bg = if i % 2 == 0 {
                            palette.light_square
                        } else {
                            palette.dark_square
                        };
                        painter.rect_filled(r, Rounding::ZERO, sq_bg);
                        draw_chess_piece_styled(painter, r.center(), sq * 0.94, col, role, false, ps);
                    }
                    painter.rect_stroke(
                        strip_rect,
                        Rounding::same(4.0_f32),
                        Stroke::new(1.0_f32, tokens.border_subtle),
                    );

                    ui.add_space(10.0);
                    ui.vertical(|ui| {
                        ui.label(
                            RichText::new(ps.label())
                                .strong()
                                .size(13.0)
                                .color(if is_sel {
                                    tokens.accent_primary
                                } else {
                                    tokens.text_primary
                                }),
                        );
                        let desc = match ps {
                            PieceSetStyle::ChessComNeo => "Authentic Chess.com sleek Neo style",
                            PieceSetStyle::ClassicStaunton => "Classic Staunton tournament look",
                            PieceSetStyle::MinimalistAlpha => "Clean modern flat silhouettes",
                            PieceSetStyle::Pixel8Bit => "Retro 8-bit pixel art sprites",
                            PieceSetStyle::NeonWireframe => "Cyberpunk glowing neon vectors",
                        };
                        ui.label(
                            RichText::new(desc)
                                .size(11.0)
                                .color(tokens.text_secondary),
                        );
                    });

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if is_sel {
                            ui.label(
                                RichText::new("✓ Active")
                                    .strong()
                                    .size(11.5)
                                    .color(tokens.accent_primary),
                            );
                        }
                    });
                    strip_resp
                })
                .inner
            });

        if row_resp.inner.clicked() || row_resp.response.interact(Sense::click()).clicked() {
            config.piece_set = ps;
            changed = true;
        }
        ui.add_space(5.0);
    }

    changed
}

fn draw_mini_board_preview(
    painter: &egui::Painter,
    rect: Rect,
    theme: BoardTheme,
    piece_set: PieceSetStyle,
) {
    let palette = theme.palette();
    let sq = rect.width() / 4.0;
    painter.rect_filled(rect.expand(1.5), Rounding::same(4.0_f32), palette.border_bg);

    for r in 0..4usize {
        for c in 0..4usize {
            let sq_rect = Rect::from_min_size(
                Pos2::new(rect.min.x + c as f32 * sq, rect.min.y + r as f32 * sq),
                Vec2::splat(sq),
            );
            let bg = if (r + c) % 2 == 0 {
                palette.light_square
            } else {
                palette.dark_square
            };
            painter.rect_filled(sq_rect, Rounding::ZERO, bg);
        }
    }

    // Place 4 iconic pieces on the 4x4 swatch so user sees real board+piece contrast
    let placements = [
        (0, 1, Color::Black, Role::King),
        (0, 2, Color::Black, Role::Knight),
        (3, 1, Color::White, Role::Queen),
        (3, 2, Color::White, Role::Pawn),
    ];
    for (r, c, col, role) in placements {
        let center = Pos2::new(
            rect.min.x + (c as f32 + 0.5) * sq,
            rect.min.y + (r as f32 + 0.5) * sq,
        );
        draw_chess_piece_styled(painter, center, sq * 0.94, col, role, false, piece_set);
    }
}
