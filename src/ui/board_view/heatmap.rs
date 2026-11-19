use crate::ui::board_view::grid::square_to_rect;
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Stroke};
use shakmaty::{attacks, Chess, Color, Position, Role, Square};

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

/// Merender garis laser taktis untuk ancaman langsung ke bidak tak terkawal & garpu Kuda (Feature #14).
pub fn render_tactical_threat_lasers(
    painter: &Painter,
    board_rect: Rect,
    pos: &Chess,
    flipped: bool,
) {
    let board = pos.board();
    let occupied = board.occupied();
    let sq_size = board_rect.width() / 8.0;

    // 1. Knight Fork Laser Triangles (Knight attacking >= 2 valuable enemy pieces)
    for color in [Color::White, Color::Black] {
        let enemy = color.other();
        let enemy_valuable = board.by_color(enemy)
            & (board.kings() | board.queens() | board.rooks());

        for knight_sq in board.by_color(color) & board.knights() {
            let forked = attacks::knight_attacks(knight_sq) & enemy_valuable;
            if forked.count() >= 2 {
                let from_c = square_to_rect(board_rect, knight_sq, flipped).center();
                let laser_col = Color32::from_rgba_unmultiplied(0xFF, 0xAA, 0x00, 185);
                for target_sq in forked {
                    let to_c = square_to_rect(board_rect, target_sq, flipped).center();
                    painter.line_segment([from_c, to_c], Stroke::new(3.0_f32, laser_col));
                    painter.circle_stroke(to_c, sq_size * 0.36, Stroke::new(2.2_f32, laser_col));
                }
            }
        }
    }

    // 2. Direct Threat Rays against undefended friendly non-pawn pieces
    let defender = pos.turn();
    let attacker = defender.other();
    for target_sq in board.by_color(defender) {
        if let Some(role) = board.role_at(target_sq) {
            if role == Role::King || role == Role::Pawn {
                continue;
            }
            let atk_bb = board.attacks_to(target_sq, attacker, occupied);
            let def_bb = board.attacks_to(target_sq, defender, occupied);
            if atk_bb.count() > def_bb.count() {
                let to_c = square_to_rect(board_rect, target_sq, flipped).center();
                let threat_col = Color32::from_rgba_unmultiplied(0xE0, 0x4F, 0x44, 170);
                for atk_sq in atk_bb {
                    let from_c = square_to_rect(board_rect, atk_sq, flipped).center();
                    painter.line_segment([from_c, to_c], Stroke::new(2.6_f32, threat_col));
                }
                painter.circle_stroke(to_c, sq_size * 0.42, Stroke::new(2.4_f32, threat_col));
            }
        }
    }
}
