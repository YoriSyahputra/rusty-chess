use crate::{
    config::{MoveInputMethod, UserConfig},
    domain::{AnnotationColor, ChessMatchState, MarkedSquare, PlayerController, TacticalArrow},
    ui::{
        board_view::{
            animation::BoardAnimationLayer,
            grid::{pos_to_square, render_board_grid_ext},
            heatmap::render_xray_heatmap,
            promotion_popup::{show_promotion_popup, PendingPromotion, PromotionChoice},
        },
        piece_renderer::draw_chess_piece,
        theme::BoardTheme,
    },
};
use egui::{Pos2, Rect, Sense, Ui, Vec2};
use shakmaty::{Move, Position, Role, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuedPremove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Role>,
}

#[derive(Debug, Clone, Default)]
pub struct BoardInteractionState {
    pub selected_square: Option<Square>,
    pub dragging_from: Option<Square>,
    pub drag_pos: Option<Pos2>,
    pub right_drag_start: Option<Square>,
    pub right_drag_current: Option<Square>,
    pub right_drag_color: AnnotationColor,
    pub pending_promotion: Option<PendingPromotion>,
    pub hint_arrow: Option<TacticalArrow>,
    pub threat_arrow: Option<TacticalArrow>,
    pub queued_premove: Option<QueuedPremove>,
    pub anim_layer: BoardAnimationLayer,
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
        self.threat_arrow = None;
        self.queued_premove = None;
        self.anim_layer.clear();
    }

    pub fn try_consume_premove(&mut self, match_state: &ChessMatchState) -> Option<Move> {
        let pm = self.queued_premove.take()?;
        let candidates: Vec<Move> = match_state
            .legal_moves_from(pm.from)
            .into_iter()
            .filter(|m| m.to() == pm.to)
            .collect();

        if candidates.is_empty() {
            return None;
        }
        if let Some(promo) = pm.promotion {
            candidates
                .iter()
                .find(|m| m.promotion() == Some(promo))
                .cloned()
                .or_else(|| candidates.into_iter().next())
        } else if candidates.iter().any(|m| m.is_promotion()) {
            candidates
                .into_iter()
                .find(|m| m.promotion() == Some(Role::Queen))
        } else {
            candidates.into_iter().next()
        }
    }

    /// Compatibility wrapper
    pub fn ui(
        &mut self,
        ui: &mut Ui,
        board_size: f32,
        match_state: &mut ChessMatchState,
        theme: BoardTheme,
        flipped: bool,
        show_coordinates: bool,
    ) -> Option<Move> {
        let mut cfg = UserConfig::default();
        cfg.board_theme = theme;
        cfg.board_flipped = flipped;
        cfg.show_coordinates = show_coordinates;
        self.ui_with_config(ui, board_size, match_state, &cfg)
    }

    /// Merender papan catur 8x8 interaktif lengkap dengan BoardAnimationLayer,
    /// Premove Queue, Snap-backs, dan 4-Color Right-Click Annotations.
    pub fn ui_with_config(
        &mut self,
        ui: &mut Ui,
        board_size: f32,
        match_state: &mut ChessMatchState,
        config: &UserConfig,
    ) -> Option<Move> {
        let flipped = config.board_flipped;
        let shake_offset = self.anim_layer.current_board_shake_offset();
        let (raw_rect, response) =
            ui.allocate_exact_size(Vec2::splat(board_size), Sense::click_and_drag());
        let board_rect = raw_rect.translate(shake_offset);

        let displayed_pos = match_state.displayed_position();
        let displayed_last_move = match_state.displayed_last_move();
        let can_human_move = match_state.is_viewing_latest()
            && !match_state.outcome.is_game_over()
            && match_state.controller_for(match_state.current_pos.turn())
                == PlayerController::Human;
        let can_premove = match_state.is_viewing_latest()
            && !match_state.outcome.is_game_over()
            && !can_human_move
            && config.enable_premoves;

        let mut executed_move: Option<Move> = None;

        if self.pending_promotion.is_none() {
            self.handle_right_click_annotations(ui, &response, board_rect, match_state, flipped);
            executed_move = self.handle_left_click_and_drag(
                ui,
                &response,
                board_rect,
                match_state,
                flipped,
                can_human_move,
                can_premove,
                config.move_input_method,
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
            (Some(from), Some(to)) if from != to => Some(TacticalArrow::new(
                from,
                to,
                self.right_drag_color,
            )),
            _ => None,
        };

        let hidden_squares = self.anim_layer.prune_and_collect_hidden_squares();
        let premove_squares = self.queued_premove.map(|pm| (pm.from, pm.to));

        let palette = config.board_theme.palette();
        let painter = ui.painter_at(raw_rect.expand(8.0));

        let mut combined_arrows = match_state.arrows.clone();
        if let Some(threat) = self.threat_arrow {
            combined_arrows.push(threat);
        }

        render_board_grid_ext(
            &painter,
            board_rect,
            &displayed_pos,
            &palette,
            flipped,
            config.show_coordinates,
            self.selected_square,
            displayed_last_move,
            premove_squares,
            &legal_for_selected,
            &match_state.marked_squares,
            &combined_arrows,
            preview_arrow,
            self.hint_arrow,
            self.dragging_from,
            &hidden_squares,
        );

        // Render X-Ray Square Control Heatmap Overlay when enabled (Feature #2)
        if config.show_xray_heatmap {
            render_xray_heatmap(&painter, board_rect, &displayed_pos, flipped);
        }

        // Render active piece slide animations, capture fades, snap-backs, check pulses & King result badges
        self.anim_layer
            .render_animations(&painter, board_rect, flipped);

        // Render dragged piece following cursor (1.15x with drop shadow)
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

        // Render Pawn Promotion Popup
        if let Some(pending) = self.pending_promotion {
            if let Some(choice) = show_promotion_popup(ui, board_rect, pending, flipped) {
                self.pending_promotion = None;
                self.selected_square = None;
                if let PromotionChoice::Selected(promo_role) = choice {
                    executed_move = Self::find_matching_move(
                        match_state,
                        pending.from,
                        pending.to,
                        Some(promo_role),
                    );
                }
            }
        }

        if executed_move.is_some() {
            self.hint_arrow = None;
        }

        executed_move
    }

    fn current_modifier_annotation_color(ui: &Ui) -> AnnotationColor {
        ui.ctx().input(|i| {
            if i.modifiers.ctrl || i.modifiers.command {
                AnnotationColor::Yellow
            } else if i.modifiers.alt {
                AnnotationColor::Blue
            } else if i.modifiers.shift {
                AnnotationColor::Green
            } else {
                AnnotationColor::Red
            }
        })
    }

    fn handle_right_click_annotations(
        &mut self,
        ui: &Ui,
        response: &egui::Response,
        board_rect: Rect,
        match_state: &mut ChessMatchState,
        flipped: bool,
    ) {
        let pointer_pos = response.interact_pointer_pos();

        if response.drag_started_by(egui::PointerButton::Secondary)
            || response.clicked_by(egui::PointerButton::Secondary)
        {
            // Right-clicking cancels any queued premove
            if self.queued_premove.is_some() {
                self.queued_premove = None;
            }
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(board_rect, pos, flipped) {
                    if self.right_drag_start.is_none() {
                        self.right_drag_start = Some(sq);
                        self.right_drag_current = Some(sq);
                        self.right_drag_color = Self::current_modifier_annotation_color(ui);
                    }
                }
            }
        }

        if response.dragged_by(egui::PointerButton::Secondary) {
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(board_rect, pos, flipped) {
                    if self.right_drag_start.is_none() {
                        self.right_drag_start = Some(sq);
                        self.right_drag_color = Self::current_modifier_annotation_color(ui);
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
                let color = self.right_drag_color;
                self.right_drag_current = None;
                if from == to {
                    if let Some(idx) = match_state
                        .marked_squares
                        .iter()
                        .position(|m| m.square == from)
                    {
                        match_state.marked_squares.remove(idx);
                    } else {
                        match_state
                            .marked_squares
                            .push(MarkedSquare { square: from, color });
                    }
                } else {
                    let arrow = TacticalArrow::new(from, to, color);
                    if let Some(idx) = match_state
                        .arrows
                        .iter()
                        .position(|a| a.from == from && a.to == to)
                    {
                        match_state.arrows.remove(idx);
                    } else {
                        match_state.arrows.push(arrow);
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn handle_left_click_and_drag(
        &mut self,
        ui: &mut Ui,
        response: &egui::Response,
        board_rect: Rect,
        match_state: &mut ChessMatchState,
        flipped: bool,
        can_human_move: bool,
        can_premove: bool,
        move_method: MoveInputMethod,
    ) -> Option<Move> {
        let pointer_pos = response
            .interact_pointer_pos()
            .or_else(|| ui.ctx().pointer_latest_pos());

        if response.clicked_by(egui::PointerButton::Primary)
            || response.drag_started_by(egui::PointerButton::Primary)
        {
            match_state.arrows.clear();
            match_state.marked_squares.clear();
        }

        if !can_human_move && !can_premove {
            self.dragging_from = None;
            self.drag_pos = None;
            return None;
        }

        let active_piece_color = if can_human_move {
            match_state.current_pos.turn()
        } else {
            match_state.human_color
        };

        let allow_drag = matches!(
            move_method,
            MoveInputMethod::DragOrClick | MoveInputMethod::DragOnly
        );
        let allow_click = matches!(
            move_method,
            MoveInputMethod::DragOrClick | MoveInputMethod::ClickOnly
        );

        // 1. Drag start
        if allow_drag && response.drag_started_by(egui::PointerButton::Primary) {
            if let Some(pos) = pointer_pos {
                if let Some(sq) = pos_to_square(board_rect, pos, flipped) {
                    if let Some(piece) = match_state.current_pos.board().piece_at(sq) {
                        if piece.color == active_piece_color {
                            self.selected_square = Some(sq);
                            self.dragging_from = Some(sq);
                            self.drag_pos = Some(pos);
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        }

        // 2. Dragging in progress
        if allow_drag && response.dragged_by(egui::PointerButton::Primary) && self.dragging_from.is_some() {
            self.drag_pos = pointer_pos;
            ui.ctx().request_repaint();
        }

        // 3. Drag drop release
        if allow_drag && response.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(from_sq) = self.dragging_from.take() {
                let drop_pos = self.drag_pos.take().or(pointer_pos);
                if let Some(pos) = drop_pos {
                    if let Some(to_sq) = pos_to_square(board_rect, pos, flipped) {
                        if to_sq != from_sq {
                            if can_human_move {
                                if let Some(mv) =
                                    self.try_prepare_or_execute_move(match_state, from_sq, to_sq)
                                {
                                    self.selected_square = None;
                                    return Some(mv);
                                } else if self.pending_promotion.is_none() {
                                    // Illegal move dropped -> Snap-back animation!
                                    if let Some(p) = match_state.current_pos.board().piece_at(from_sq)
                                    {
                                        self.anim_layer
                                            .trigger_snapback(to_sq, from_sq, p.color, p.role);
                                    }
                                }
                            } else if can_premove {
                                self.queued_premove = Some(QueuedPremove {
                                    from: from_sq,
                                    to: to_sq,
                                    promotion: Some(Role::Queen),
                                });
                                self.selected_square = None;
                            }
                        }
                    }
                }
            }
        }

        // 4. Click-to-Move / Click-to-Premove
        if allow_click && response.clicked_by(egui::PointerButton::Primary) {
            if let Some(pos) = pointer_pos {
                if let Some(clicked_sq) = pos_to_square(board_rect, pos, flipped) {
                    if let Some(from_sq) = self.selected_square {
                        if from_sq == clicked_sq {
                            self.selected_square = None;
                            return None;
                        }
                        if can_human_move {
                            if let Some(mv) =
                                self.try_prepare_or_execute_move(match_state, from_sq, clicked_sq)
                            {
                                self.selected_square = None;
                                return Some(mv);
                            }
                        } else if can_premove {
                            self.queued_premove = Some(QueuedPremove {
                                from: from_sq,
                                to: clicked_sq,
                                promotion: Some(Role::Queen),
                            });
                            self.selected_square = None;
                            return None;
                        }
                    }

                    if let Some(piece) = match_state.current_pos.board().piece_at(clicked_sq) {
                        if piece.color == active_piece_color {
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
