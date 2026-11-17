pub mod grid;
pub mod interaction;
pub mod promotion_popup;

pub use grid::{draw_tactical_arrow, pos_to_square, render_board_grid, square_to_rect};
pub use interaction::BoardInteractionState;
pub use promotion_popup::{show_promotion_popup, PendingPromotion, PromotionChoice};
