pub mod animation;
pub mod grid;
pub mod heatmap;
pub mod interaction;
pub mod promotion_popup;

pub use animation::{
    render_move_classification_badge, BoardAnimationLayer, CaptureFadeAnim, PieceSlideAnim,
    ResultGlyph,
};
pub use grid::{
    draw_tactical_arrow, pos_to_square, render_board_grid, render_board_grid_ext,
    resolve_annotation_color, square_to_rect,
};
pub use heatmap::{
    compute_square_control_map, render_tactical_threat_lasers, render_xray_heatmap,
    SquareControlInfo,
};
pub use interaction::{BoardInteractionState, QueuedPremove};
pub use promotion_popup::{show_promotion_popup, PendingPromotion, PromotionChoice};
