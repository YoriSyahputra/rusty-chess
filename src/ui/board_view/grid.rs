use crate::{
    domain::TacticalArrow,
    ui::{piece_renderer::draw_chess_piece, theme::BoardPalette},
};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use shakmaty::{Chess, File, Move, Position, Rank, Square};

/// Mengonversi `Square` menjadi `Rect` layar di dalam `board_rect`.
pub fn square_to_rect(board_rect: Rect, sq: Square, flipped: bool) -> Rect {
    let sq_size = board_rect.width() / 8.0;
    let file_idx = sq.file() as usize;
    let rank_idx = sq.rank() as usize;

    let (col, row) = if flipped {
        (7 - file_idx, rank_idx)
    } else {
        (file_idx, 7 - rank_idx)
    };

    let min_x = board_rect.min.x + (col as f32) * sq_size;
    let min_y = board_rect.min.y + (row as f32) * sq_size;
    Rect::from_min_size(Pos2::new(min_x, min_y), Vec2::splat(sq_size))
}

/// Mengonversi posisi kursor layar (`Pos2`) menjadi `Option<Square>`.
pub fn pos_to_square(board_rect: Rect, pos: Pos2, flipped: bool) -> Option<Square> {
    if !board_rect.contains(pos) {
        return None;
    }
    let sq_size = board_rect.width() / 8.0;
    let col = ((pos.x - board_rect.min.x) / sq_size).floor() as i32;
    let row = ((pos.y - board_rect.min.y) / sq_size).floor() as i32;
    if !(0..8).contains(&col) || !(0..8).contains(&row) {
        return None;
    }

    let (file_idx, rank_idx) = if flipped {
        (7 - col, row)
    } else {
        (col, 7 - row)
    };

    Some(Square::from_coords(
        File::new(file_idx as u32),
        Rank::new(rank_idx as u32),
    ))
}

#[allow(clippy::too_many_arguments)]
pub fn render_board_grid(
    painter: &Painter,
    board_rect: Rect,
    pos: &Chess,
    palette: &BoardPalette,
    flipped: bool,
    show_coordinates: bool,
    selected_sq: Option<Square>,
    last_move: Option<(Square, Square)>,
    legal_moves_for_selected: &[Move],
    marked_squares: &[Square],
    arrows: &[TacticalArrow],
    preview_arrow: Option<TacticalArrow>,
    hint_arrow: Option<TacticalArrow>,
    dragged_from: Option<Square>,
) {
    let sq_size = board_rect.width() / 8.0;
    let board = pos.board();

    // Outer frame shadow & border
    let frame_rect = board_rect.expand(4.0);
    painter.rect_filled(frame_rect, Rounding::same(6.0), palette.border_bg);

    let king_in_check_sq = if pos.is_check() {
        board.king_of(pos.turn())
    } else {
        None
    };

    // 1. Render 64 petak dasar + highlight
    for sq in Square::ALL {
        let sq_rect = square_to_rect(board_rect, sq, flipped);
        let is_light = (sq.file() as u8 + sq.rank() as u8) % 2 != 0;
        let base_color = if is_light {
            palette.light_square
        } else {
            palette.dark_square
        };

        painter.rect_filled(sq_rect, Rounding::ZERO, base_color);

        // Last move highlight
        if let Some((from, to)) = last_move {
            if sq == from {
                painter.rect_filled(sq_rect, Rounding::ZERO, palette.last_move_from);
            } else if sq == to {
                painter.rect_filled(sq_rect, Rounding::ZERO, palette.last_move_to);
            }
        }

        // Selected square highlight
        if selected_sq == Some(sq) {
            painter.rect_filled(sq_rect, Rounding::ZERO, palette.selected_square);
        }

        // Right-click marked square highlight
        if marked_squares.contains(&sq) {
            painter.rect_filled(sq_rect, Rounding::ZERO, palette.marked_square);
        }

        // King in check red radial/layered glow
        if king_in_check_sq == Some(sq) {
            painter.rect_filled(sq_rect, Rounding::ZERO, palette.check_glow_outer);
            painter.circle_filled(
                sq_rect.center(),
                sq_size * 0.44,
                palette.check_glow_inner,
            );
            painter.rect_stroke(
                sq_rect.shrink(1.5),
                Rounding::same(2.0),
                Stroke::new(2.5, Color32::from_rgb(255, 70, 70)),
            );
        }
    }

    // 2. Render koordinat A-H dan 1-8 di tepi papan
    if show_coordinates {
        let font = FontId::proportional((sq_size * 0.165).clamp(9.5, 14.0));
        for idx in 0..8usize {
            // Rank labels di kolom paling kiri visual (col = 0)
            let rank_idx = if flipped { idx } else { 7 - idx };
            let file_left = if flipped { 7 } else { 0 };
            let left_sq = Square::from_coords(
                File::new(file_left as u32),
                Rank::new(rank_idx as u32),
            );
            let sq_rect = square_to_rect(board_rect, left_sq, flipped);
            let is_light = (left_sq.file() as u8 + left_sq.rank() as u8) % 2 != 0;
            let text_color = if is_light {
                palette.dark_square
            } else {
                palette.light_square
            };
            painter.text(
                Pos2::new(sq_rect.min.x + 4.0, sq_rect.min.y + 3.0),
                Align2::LEFT_TOP,
                format!("{}", rank_idx + 1),
                font.clone(),
                text_color,
            );

            // File labels di baris paling bawah visual (row = 7)
            let file_idx = if flipped { 7 - idx } else { idx };
            let rank_bottom = if flipped { 7 } else { 0 };
            let bottom_sq = Square::from_coords(
                File::new(file_idx as u32),
                Rank::new(rank_bottom as u32),
            );
            let b_rect = square_to_rect(board_rect, bottom_sq, flipped);
            let b_light = (bottom_sq.file() as u8 + bottom_sq.rank() as u8) % 2 != 0;
            let b_color = if b_light {
                palette.dark_square
            } else {
                palette.light_square
            };
            let file_char = (b'a' + file_idx as u8) as char;
            painter.text(
                Pos2::new(b_rect.max.x - 4.0, b_rect.max.y - 3.0),
                Align2::RIGHT_BOTTOM,
                file_char.to_string(),
                font.clone(),
                b_color,
            );
        }
    }

    // 3. Render bidak di atas papan (kecuali bidak yang sedang di-drag)
    for sq in Square::ALL {
        if dragged_from == Some(sq) {
            continue;
        }
        if let Some(piece) = board.piece_at(sq) {
            let sq_rect = square_to_rect(board_rect, sq, flipped);
            draw_chess_piece(
                painter,
                sq_rect.center(),
                sq_size,
                piece.color,
                piece.role,
                false,
            );
        }
    }

    // 4. Render indikator langkah legal (titik transparan untuk langkah kosong, cincin untuk capture)
    let mut drawn_targets = [false; 64];
    for mv in legal_moves_for_selected {
        let to_sq = mv.to();
        let to_idx = to_sq as usize;
        if drawn_targets[to_idx] {
            continue;
        }
        drawn_targets[to_idx] = true;

        let target_rect = square_to_rect(board_rect, to_sq, flipped);
        if mv.is_capture() || board.piece_at(to_sq).is_some() {
            painter.circle_stroke(
                target_rect.center(),
                sq_size * 0.41,
                Stroke::new(sq_size * 0.085, palette.legal_capture_ring),
            );
        } else {
            painter.circle_filled(
                target_rect.center(),
                sq_size * 0.145,
                palette.legal_move_dot,
            );
        }
    }

    // 5. Render Hint Arrow (hijau) & Tactical Arrows (oranye)
    if let Some(hint) = hint_arrow {
        draw_tactical_arrow(
            painter,
            board_rect,
            hint,
            flipped,
            palette.arrow_hint,
        );
    }
    for &arrow in arrows {
        draw_tactical_arrow(
            painter,
            board_rect,
            arrow,
            flipped,
            palette.arrow_tactical,
        );
    }
    if let Some(preview) = preview_arrow {
        draw_tactical_arrow(
            painter,
            board_rect,
            preview,
            flipped,
            palette.arrow_tactical,
        );
    }
}

pub fn draw_tactical_arrow(
    painter: &Painter,
    board_rect: Rect,
    arrow: TacticalArrow,
    flipped: bool,
    color: Color32,
) {
    let sq_size = board_rect.width() / 8.0;
    let from_rect = square_to_rect(board_rect, arrow.from, flipped);
    let to_rect = square_to_rect(board_rect, arrow.to, flipped);

    if arrow.from == arrow.to {
        painter.circle_stroke(
            from_rect.center(),
            sq_size * 0.42,
            Stroke::new(sq_size * 0.08, color),
        );
        return;
    }

    let p1 = from_rect.center();
    let p2 = to_rect.center();
    let delta = p2 - p1;
    let dist = delta.length();
    if dist < 1.0 {
        return;
    }

    let dir = delta / dist;
    let perp = Vec2::new(-dir.y, dir.x);

    let start = p1 + dir * (sq_size * 0.15);
    let tip = p2 - dir * (sq_size * 0.06);
    let head_base = p2 - dir * (sq_size * 0.36);

    // Batang panah (shaft)
    let shaft_half = sq_size * 0.075;
    let shaft_poly = vec![
        start + perp * shaft_half,
        head_base + perp * shaft_half,
        head_base - perp * shaft_half,
        start - perp * shaft_half,
    ];
    painter.add(Shape::convex_polygon(shaft_poly, color, Stroke::NONE));

    // Kepala panah segitiga (arrowhead)
    let head_half = sq_size * 0.22;
    let head_poly = vec![
        tip,
        head_base + perp * head_half,
        head_base - perp * head_half,
    ];
    painter.add(Shape::convex_polygon(head_poly, color, Stroke::NONE));
}
