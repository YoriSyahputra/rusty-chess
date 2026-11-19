use crate::ui::board_view::grid::square_to_rect;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Stroke};
use shakmaty::{Chess, Color, Position, Role, Square};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SquareControlInfo {
    pub white_attackers: u8,
    pub black_attackers: u8,
    pub is_hanging: bool,
}

/// Menghitung jumlah penyerang/pelindung Putih & Hitam untuk seluruh 64 petak
/// serta mendeteksi bidak tak terkawal (hanging pieces) menggunakan bitboard `shakmaty`.
pub fn compute_square_control_map(pos: &Chess) -> [SquareControlInfo; 64] {
    let mut map = [SquareControlInfo::default(); 64];
    let board = pos.board();
    let occupied = board.occupied();

    for sq in Square::ALL {
        let w_atk = board.attacks_to(sq, Color::White, occupied).count() as u8;
        let b_atk = board.attacks_to(sq, Color::Black, occupied).count() as u8;

        let is_hanging = if let Some(piece) = board.piece_at(sq) {
            if piece.role == Role::King {
                false
            } else {
                match piece.color {
                    Color::White => b_atk > w_atk,
                    Color::Black => w_atk > b_atk,
                }
            }
        } else {
            false
        };

        map[sq as usize] = SquareControlInfo {
            white_attackers: w_atk,
            black_attackers: b_atk,
            is_hanging,
        };
    }

    map
}

/// Merender overlay X-Ray Square Control Heatmap di atas papan catur:
/// - Biru transparan: kontrol bersih Putih
/// - Merah transparan: kontrol bersih Hitam
/// - Ungu transparan: petak diperebutkan seimbang (tactical tension)
/// - Bingkai amber menyala: bidak tak terkawal (hanging piece alert!)
pub fn render_xray_heatmap(painter: &Painter, board_rect: Rect, pos: &Chess, flipped: bool) {
    let control_map = compute_square_control_map(pos);
    let sq_size = board_rect.width() / 8.0;

    for sq in Square::ALL {
        let info = control_map[sq as usize];
        let sq_rect = square_to_rect(board_rect, sq, flipped);

        let w = info.white_attackers as i32;
        let b = info.black_attackers as i32;
        let net = w - b;

        if net > 0 {
            let alpha = (34 + (net.min(4) * 20)) as u8;
            painter.rect_filled(
                sq_rect,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(0x00, 0x9F, 0xD9, alpha),
            );
        } else if net < 0 {
            let alpha = (34 + ((-net).min(4) * 20)) as u8;
            painter.rect_filled(
                sq_rect,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(0xE0, 0x4F, 0x44, alpha),
            );
        } else if w > 0 && b > 0 {
            // Contested tactical hotspot
            let alpha = (42 + (w.min(3) * 16)) as u8;
            painter.rect_filled(
                sq_rect,
                Rounding::ZERO,
                Color32::from_rgba_unmultiplied(165, 85, 235, alpha),
            );
        }

        // Hanging piece alert border & badge
        if info.is_hanging {
            painter.rect_stroke(
                sq_rect.shrink(2.0),
                Rounding::same(3.0_f32),
                Stroke::new(2.5_f32, Color32::from_rgb(0xFF, 0xAA, 0x00)),
            );
            let badge_pos = Pos2::new(sq_rect.max.x - 9.0, sq_rect.min.y + 9.0);
            painter.circle_filled(badge_pos, 7.0, Color32::from_rgb(0xFF, 0xAA, 0x00));
            painter.text(
                badge_pos,
                Align2::CENTER_CENTER,
                "!",
                FontId::proportional((sq_size * 0.16).clamp(9.0, 12.0)),
                Color32::from_rgb(20, 18, 16),
            );
        }
    }
}
