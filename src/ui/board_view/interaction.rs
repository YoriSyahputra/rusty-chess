use crate::{
    domain::{ChessMatchState, PlayerController, TacticalArrow},
    ui::{
        board_view::{
            grid::{pos_to_square, render_board_grid},
            promotion_popup::{show_promotion_popup, PendingPromotion, PromotionChoice},
        },
        piece_renderer::draw_chess_piece,
        theme::BoardTheme,
    },
};
use egui::{Pos2, Rect, Sense, Ui, Vec2};
use shakmaty::{Move, Position, Role, Square};

#[derive(Debug, Clone, Default)]
pub struct BoardInteractionState {
    pub selected_square: Option<Square>,
    pub dragging_from: Option<Square>,
    pub drag_pos: Option<Pos2>,
    pub right_drag_start: Option<Square>,
    pub right_drag_current: Option<Square>,
    pub pending_promotion: Option<PendingPromotion>,
    pub hint_arrow: Option<TacticalArrow>,
}

impl BoardInteractionState {
    pub fn reset(&mut self) {
        self.selected_square = None;
        self.dragging_from = None;
        self.drag_pos = None;
        self.right_drag_start = None;
        self.right_drag_current = None;
        self.pending_promotion = None;
        self.hint_arrow = None;
    }

    /// Merender papan catur 8x8 interaktif dan mengembalikan `Some(Move)` jika pemain mengeksekusi langkah legal.
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        board_size: f32,
        match_state: &mut ChessMatchState,
        theme: BoardTheme,
        flipped: bool,
        show_coordinates: bool,
    ) -> Option<Move> {
        let (board_rect, response) =
            ui.allocate_exact_size(Vec2::splat(board_size), Sense::click_and_drag());

        let displayed_pos = match_state.displayed_position();
        let displayed_last_move = match_state.displayed_last_move();
        let can_human_move = match_state.is_viewing_latest()
            && !match_state.outcome.is_game_over()
            && match_state.controller_for(match_state.current_pos.turn())
                == PlayerController::Human;

        let mut executed_move: Option<Move> = None;

        // Jika popup promosi sedang aktif, tangani input popup terlebih dahulu
        if self.pending_promotion.is_none() {
            self.handle_right_click_annotations(&response, board_rect, match_state, flipped);
            executed_move = self.handle_left_click_and_drag(
                ui,
                &response,
                board_rect,
                match_state,
                flipped,
                can_human_move,
            );
        }

        let legal_for_selected = if can_human_move {
            self.selected_square
                .map(|sq| match_state.legal_moves_from(sq))
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        let preview_arrow = match (self.right_drag_start, self.right_drag_current) {
            (Some(from), Some(to)) if from != to => Some(TacticalArrow { from, to }),
            _ => None,
        };

        let palette = theme.palette();
        let painter = ui.painter_at(board_rect.expand(6.0));

        render_board_grid(
            &painter,
            board_rect,
            &displayed_pos,
            &palette,
            flipped,
            show_coordinates,
            self.selected_square,
            displayed_last_move,
            &legal_for_selected,
            &match_state.marked_squares,
            &match_state.arrows,
            preview_arrow,
            self.hint_arrow,
            self.dragging_from,
        );

        // Render bidak yang sedang di-drag mengikuti kursor mouse secara real-time (1.15x dengan bayangan)
        if let (Some(drag_sq), Some(cursor_pos)) = (self.dragging_from, self.drag_pos) {
            if let Some(piece) = displayed_pos.board().piece_at(drag_sq) {
                let sq_size = board_rect.width() / 8.0;
                draw_chess_piece(
                    &painter,
                    cursor_pos,
                    sq_size,
                    piece.color,
                    piece.role,
                    true,
                );
            }
        }

        // Render Pawn Promotion Popup jika ada promosi yang menunggu pilihan pemain
        if let Some(pending) = self.pending_promotion {
            if let Some(choice) = show_promotion_popup(ui, board_rect, pending, flipped) {
                self.pending_promotion = None;
                self.selected_square = None;
                if let PromotionChoice::Selected(promo_role) = choice {
                    executed_move =
                        Self::find_matching_move(match_state, pending.from, pending.to, Some(promo_role));
                }
            }
        }

        if executed_move.is_some() {
            self.hint_arrow = None;
        }

        executed_move
    }

    fn handle_right_click_annotations(
        &mut self,
        response: &egui::Response,
        board_rect: Rect,
        match_state: &mut ChessMatchState,
        flipped: bool,
    ) {
        let pointer_pos = response.interact_pointer_pos();

        if response.drag_started_by(egui::PointerButton::Secondary)
            || response.clicked_by(egui::PointerButton::Secondary)
        {
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(board_rect, pos, flipped) {
                    if self.right_drag_start.is_none() {
                        self.right_drag_start = Some(sq);
                        self.right_drag_current = Some(sq);
                    }
                }
            }
        }

        if response.dragged_by(egui::PointerButton::Secondary) {
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(board_rect, pos, flipped) {
                    if self.right_drag_start.is_none() {
                        self.right_drag_start = Some(sq);
                    }
                    self.right_drag_current = Some(sq);
                }
            }
        }

        let secondary_released = response.drag_stopped_by(egui::PointerButton::Secondary)
            || response.clicked_by(egui::PointerButton::Secondary);

        if secondary_released {
            let end_sq = pointer_pos
                .and_then(|p| pos_to_square(board_rect, p, flipped))
                .or(self.right_drag_current);

            if let (Some(from), Some(to)) = (self.right_drag_start.take(), end_sq) {
                self.right_drag_current = None;
                if from == to {
                    // Toggle marked square (merah transparan)
                    if let Some(idx) = match_state.marked_squares.iter().position(|&s| s == from) {
                        match_state.marked_squares.remove(idx);
                    } else {
                        match_state.marked_squares.push(from);
                    }
                } else {
                    // Toggle tactical arrow
                    let arrow = TacticalArrow { from, to };
                    if let Some(idx) = match_state.arrows.iter().position(|&a| a == arrow) {
                        match_state.arrows.remove(idx);
                    } else {
                        match_state.arrows.push(arrow);
                    }
                }
            }
        }
    }

    fn handle_left_click_and_drag(
        &mut self,
        ui: &mut Ui,
        response: &egui::Response,
        board_rect: Rect,
        match_state: &mut ChessMatchState,
        flipped: bool,
        can_human_move: bool,
    ) -> Option<Move> {
        let pointer_pos = response
            .interact_pointer_pos()
            .or_else(|| ui.ctx().pointer_latest_pos());

        // Klik kiri di mana saja menghapus seluruh coretan panah & petak merah
        if response.clicked_by(egui::PointerButton::Primary)
            || response.drag_started_by(egui::PointerButton::Primary)
        {
            match_state.arrows.clear();
            match_state.marked_squares.clear();
        }

        if !can_human_move {
            self.dragging_from = None;
            self.drag_pos = None;
            return None;
        }

        // 1. Drag start
        if response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(board_rect, pos, flipped) {
                    let turn = match_state.current_pos.turn();
                    if let Some(piece) = match_state.current_pos.board().piece_at(sq) {
                        if piece.color == turn {
                            self.selected_square = Some(sq);
                            self.dragging_from = Some(sq);
                            self.drag_pos = Some(pos);
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        }

        // 2. Dragging in progress (60-144 FPS smooth)
        if response.dragged_by(egui::PointerButton::Primary) && self.dragging_from.is_some() {
            self.drag_pos = pointer_pos;
            ui.ctx().request_repaint();
        }

        // 3. Drag drop release
        if response.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(from_sq) = self.dragging_from.take() {
                let drop_pos = self.drag_pos.take().or(pointer_pos);
                if let Some(pos) = drop_pos {
                    if let Some(to_sq) = pos_to_square(board_rect, pos, flipped) {
                        if to_sq != from_sq {
                            if let Some(mv) =
                                self.try_prepare_or_execute_move(match_state, from_sq, to_sq)
                            {
                                self.selected_square = None;
                                return Some(mv);
                            }
                        }
                    }
                }
            }
        }

        // 4. Click-to-Move
        if response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pos) = pointer_pos {
                if let Some(clicked_sq) = pos_to_square(board_rect, pos, flipped) {
                    if let Some(from_sq) = self.selected_square {
                        if from_sq == clicked_sq {
                            self.selected_square = None;
                            return None;
                        }
                        if let Some(mv) =
                            self.try_prepare_or_execute_move(match_state, from_sq, clicked_sq)
                        {
                            self.selected_square = None;
                            return Some(mv);
                        }
                    }

                    // Jika klik pada bidak milik sendiri, pilih petak tersebut
                    let turn = match_state.current_pos.turn();
                    if let Some(piece) = match_state.current_pos.board().piece_at(clicked_sq) {
                        if piece.color == turn {
                            self.selected_square = Some(clicked_sq);
                        } else {
                            self.selected_square = None;
                        }
                    } else {
                        self.selected_square = None;
                    }
                }
            }
        }

        None
    }

    fn try_prepare_or_execute_move(
        &mut self,
        match_state: &ChessMatchState,
        from: Square,
        to: Square,
    ) -> Option<Move> {
        let candidates: Vec<Move> = match_state
            .legal_moves_from(from)
            .into_iter()
            .filter(|m| m.to() == to)
            .collect();

        if candidates.is_empty() {
            return None;
        }

        if candidates.iter().any(|m| m.is_promotion()) {
            self.pending_promotion = Some(PendingPromotion {
                from,
                to,
                color: match_state.current_pos.turn(),
            });
            return None;
        }

        candidates.into_iter().next()
    }

    fn find_matching_move(
        match_state: &ChessMatchState,
        from: Square,
        to: Square,
        promo: Option<Role>,
    ) -> Option<Move> {
        match_state
            .legal_moves_from(from)
            .into_iter()
            .find(|m| m.to() == to && m.promotion() == promo)
    }
}
