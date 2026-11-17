use crate::ui::{board_view::grid::square_to_rect, piece_renderer::draw_chess_piece};
use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Color, Role, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PendingPromotion {
    pub from: Square,
    pub to: Square,
    pub color: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionChoice {
    Selected(Role),
    Cancelled,
}

/// Menampilkan overlay popup pemilihan promosi pion (Queen, Rook, Bishop, Knight + tombol Cancel)
/// tepat di atas atau sekitar petak promosi.
pub fn show_promotion_popup(
    ui: &mut Ui,
    board_rect: Rect,
    pending: PendingPromotion,
    flipped: bool,
) -> Option<PromotionChoice> {
    let sq_size = board_rect.width() / 8.0;
    let target_rect = square_to_rect(board_rect, pending.to, flipped);

    let popup_w = (sq_size * 1.15).clamp(62.0, 96.0);
    let item_h = popup_w * 0.92;
    let cancel_h = 28.0;
    let total_h = item_h * 4.0 + cancel_h + 12.0;

    let popup_x = target_rect
        .center()
        .x
        .clamp(board_rect.min.x + popup_w * 0.5, board_rect.max.x - popup_w * 0.5)
        - popup_w * 0.5;

    let popup_y = if target_rect.center().y - board_rect.min.y < board_rect.height() * 0.5 {
        target_rect.min.y
    } else {
        (target_rect.max.y - total_h).max(board_rect.min.y)
    };

    let popup_rect = Rect::from_min_size(Pos2::new(popup_x, popup_y), Vec2::new(popup_w, total_h));

    // Backdrop semi-transparan di atas papan
    let backdrop_resp = ui.interact(
        board_rect,
        ui.id().with("promo_backdrop"),
        Sense::click(),
    );

    let painter = ui.painter();
    painter.rect_filled(
        board_rect,
        Rounding::same(4.0),
        Color32::from_rgba_unmultiplied(10, 12, 16, 135),
    );

    painter.rect(
        popup_rect,
        Rounding::same(8.0),
        Color32::from_rgb(30, 34, 44),
        Stroke::new(2.0, Color32::from_rgb(110, 165, 245)),
    );

    let roles = [Role::Queen, Role::Rook, Role::Bishop, Role::Knight];
    let mut choice: Option<PromotionChoice> = None;

    for (idx, &role) in roles.iter().enumerate() {
        let cell_rect = Rect::from_min_size(
            Pos2::new(popup_rect.min.x + 4.0, popup_rect.min.y + 4.0 + (idx as f32) * item_h),
            Vec2::new(popup_w - 8.0, item_h - 2.0),
        );
        let resp = ui.interact(
            cell_rect,
            ui.id().with(("promo_role", idx)),
            Sense::click(),
        );

        let bg = if resp.hovered() {
            Color32::from_rgb(62, 78, 108)
        } else {
            Color32::from_rgb(40, 45, 58)
        };
        ui.painter().rect_filled(cell_rect, Rounding::same(6.0), bg);

        draw_chess_piece(
            ui.painter(),
            cell_rect.center(),
            item_h * 0.85,
            pending.color,
            role,
            false,
        );

        if resp.clicked() {
            choice = Some(PromotionChoice::Selected(role));
        }
    }

    let cancel_rect = Rect::from_min_size(
        Pos2::new(
            popup_rect.min.x + 4.0,
            popup_rect.max.y - cancel_h - 4.0,
        ),
        Vec2::new(popup_w - 8.0, cancel_h),
    );
    let cancel_resp = ui.interact(
        cancel_rect,
        ui.id().with("promo_cancel_btn"),
        Sense::click(),
    );
    let cancel_bg = if cancel_resp.hovered() {
        Color32::from_rgb(155, 50, 55)
    } else {
        Color32::from_rgb(95, 38, 42)
    };
    ui.painter()
        .rect_filled(cancel_rect, Rounding::same(5.0), cancel_bg);
    ui.painter().text(
        cancel_rect.center(),
        Align2::CENTER_CENTER,
        "✕ Cancel",
        FontId::proportional(11.5),
        Color32::WHITE,
    );

    if cancel_resp.clicked()
        || (backdrop_resp.clicked()
            && backdrop_resp
                .interact_pointer_pos()
                .map(|p| !popup_rect.contains(p))
                .unwrap_or(false))
    {
        choice = Some(PromotionChoice::Cancelled);
    }

    choice
}
