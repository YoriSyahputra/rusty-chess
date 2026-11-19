use crate::{
    config::{AssistanceMode, MoveInputMethod, UserConfig},
    domain::{AnnotationColor, ChessMatchState, MarkedSquare, PlayerController, TacticalArrow},
    ui::{
        board_textures::BoardTextureCache,
        board_view::{
            animation::BoardAnimationLayer,
            grid::{pos_to_square, render_board_grid_ext, square_to_rect},
            heatmap::{render_tactical_threat_lasers, render_xray_heatmap},
            promotion_popup::{show_promotion_popup, PendingPromotion, PromotionChoice},
        },
        piece_renderer::draw_chess_piece_styled,
        theme::BoardTheme,
    },
};
use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};
use shakmaty::{Move, Position, Role, Square};
use std::collections::VecDeque;

pub const MAX_QUEUED_PREMOVES: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueuedPremove {
    pub from: Square,
    pub to: Square,
    pub promotion: Option<Role>,
}

#[derive(Default)]
pub struct BoardInteractionState {
    pub selected_square: Option<Square>,
    pub dragging_from: Option<Square>,
    pub drag_pos: Option<Pos2>,
    pub drag_prev_pos: Option<Pos2>,
    pub drag_velocity: Vec2,
    pub right_drag_start: Option<Square>,
    pub right_drag_current: Option<Square>,
    pub right_drag_color: AnnotationColor,
    pub pending_promotion: Option<PendingPromotion>,
    pub hint_arrow: Option<TacticalArrow>,
    pub threat_arrow: Option<TacticalArrow>,
    pub queued_premoves: VecDeque<QueuedPremove>,
    pub anim_layer: BoardAnimationLayer,
    pub texture_cache: BoardTextureCache,
}

impl BoardInteractionState {
    pub fn reset(&mut self) {
        self.selected_square = None;
        self.dragging_from = None;
        self.drag_pos = None;
        self.drag_prev_pos = None;
        self.drag_velocity = Vec2::ZERO;
        self.right_drag_start = None;
        self.right_drag_current = None;
        self.pending_promotion = None;
        self.hint_arrow = None;
        self.threat_arrow = None;
        self.queued_premoves.clear();
        self.anim_layer.clear();
    }

    pub fn enqueue_premove(&mut self, from: Square, to: Square, promotion: Option<Role>) {
        if self.queued_premoves.len() >= MAX_QUEUED_PREMOVES {
            return;
        }
        self.queued_premoves
            .push_back(QueuedPremove { from, to, promotion });
    }

    pub fn try_consume_premove(&mut self, match_state: &ChessMatchState) -> Option<Move> {
        let pm = self.queued_premoves.pop_front()?;
        let candidates: Vec<Move> = match_state
            .legal_moves_from(pm.from)
            .into_iter()
            .filter(|m| m.to() == pm.to)
            .collect();

        if candidates.is_empty() {
            // If the front premove is no longer legal, cancel any chained follow-ups
            self.queued_premoves.clear();
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

    /// Merender papan catur 8x8 interaktif lengkap dengan Multi-Premove Chaining,
    /// BoardAnimationLayer, Threat Laser Lines, dan Velocity Drag Physics.
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
            (Some(from), Some(to)) if from != to => {
                Some(TacticalArrow::new(from, to, self.right_drag_color))
            }
            _ => None,
        };

        let hidden_squares = self.anim_layer.prune_and_collect_hidden_squares();
        let first_premove_squares = self.queued_premoves.front().map(|pm| (pm.from, pm.to));

        let palette = config.board_theme.palette();
        let painter = ui.painter_at(raw_rect.expand(8.0));

        let mut combined_arrows = match_state.arrows.clone();
        if let Some(threat) = self.threat_arrow {
            combined_arrows.push(threat);
        }

        let tex_id = if config.textured_board {
            Some(
                self.texture_cache
                    .get_or_create(ui.ctx(), config.board_theme)
                    .id(),
            )
        } else {
            None
        };

        render_board_grid_ext(
            &painter,
            board_rect,
            &displayed_pos,
            &palette,
            flipped,
            config.show_coordinates,
            self.selected_square,
            displayed_last_move,
            first_premove_squares,
            &legal_for_selected,
            &match_state.marked_squares,
            &combined_arrows,
            preview_arrow,
            self.hint_arrow,
            self.dragging_from,
            &hidden_squares,
            config.piece_set,
            tex_id,
        );

        // Render chained multi-premove highlights + numbered badges (Feature #13)
        if !self.queued_premoves.is_empty() {
            let sq_size = board_rect.width() / 8.0;
            for (idx, pm) in self.queued_premoves.iter().enumerate() {
                let from_r = square_to_rect(board_rect, pm.from, flipped);
                let to_r = square_to_rect(board_rect, pm.to, flipped);
                if idx > 0 {
                    painter.rect_filled(from_r, Rounding::ZERO, palette.premove_square);
                    painter.rect_filled(to_r, Rounding::ZERO, palette.premove_square);
                }
                if self.queued_premoves.len() > 1 {
                    let badge_c = Pos2::new(to_r.min.x + 11.0, to_r.min.y + 11.0);
                    painter.circle_filled(badge_c, 8.5, Color32::from_rgb(0xE0, 0x4F, 0x44));
                    painter.text(
                        badge_c,
                        Align2::CENTER_CENTER,
                        format!("{}", idx + 1),
                        FontId::proportional((sq_size * 0.16).clamp(9.0, 12.0)),
                        Color32::WHITE,
                    );
                }
            }
        }

        // Render X-Ray Square Control Heatmap & Tactical Threat Lasers (Features #2 & #14)
        if config.show_xray_heatmap {
            render_xray_heatmap(&painter, board_rect, &displayed_pos, flipped);
        }
        let show_threat_lasers = config.show_xray_heatmap
            || config.assistance_mode == AssistanceMode::Assisted
            || (config.assistance_mode == AssistanceMode::Custom
                && config.custom_assistance.threat_arrows);
        if show_threat_lasers {
            render_tactical_threat_lasers(&painter, board_rect, &displayed_pos, flipped);
        }

        // Render active piece slide animations, capture fades, particles, confetti, spring badges & King badges
        self.anim_layer
            .render_animations(&painter, board_rect, flipped, config.piece_set);

        // Render dragged piece with Velocity-Based Lift & Shadow Physics (Feature #15a)
        if let (Some(drag_sq), Some(cursor_pos)) = (self.dragging_from, self.drag_pos) {
            if let Some(piece) = displayed_pos.board().piece_at(drag_sq) {
                let sq_size = board_rect.width() / 8.0;
                let speed = self.drag_velocity.length().clamp(0.0, 45.0);
                let lift_bonus = 1.0 + (speed / 45.0) * 0.08;
                let tilt_offset = Vec2::new(
                    (self.drag_velocity.x * 0.18).clamp(-6.0, 6.0),
                    (self.drag_velocity.y * 0.10).clamp(-4.0, 4.0) - (speed * 0.08),
                );
                // Dynamic shadow trailing behind drag velocity
                let shadow_c = cursor_pos - tilt_offset * 0.9 + Vec2::new(3.0, sq_size * 0.28);
                painter.circle_filled(
                    shadow_c,
                    sq_size * 0.34,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 55),
                );
                draw_chess_piece_styled(
                    &painter,
                    cursor_pos + tilt_offset,
                    sq_size * lift_bonus,
                    piece.color,
                    piece.role,
                    true,
                    config.piece_set,
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
        let _ = Stroke::NONE;

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
            if !self.queued_premoves.is_empty() {
                self.queued_premoves.clear();
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
            self.drag_prev_pos = None;
            self.drag_velocity = Vec2::ZERO;
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
                            self.drag_prev_pos = Some(pos);
                            self.drag_velocity = Vec2::ZERO;
                            ui.ctx().request_repaint();
                        }
                    }
                }
            }
        }

        // 2. Dragging in progress with smoothed velocity tracking
        if allow_drag && response.dragged_by(egui::PointerButton::Primary) && self.dragging_from.is_some() {
            if let Some(cur) = pointer_pos {
                if let Some(prev) = self.drag_prev_pos {
                    let delta = cur - prev;
                    self.drag_velocity = self.drag_velocity * 0.65 + delta * 0.35;
                }
                self.drag_prev_pos = Some(cur);
                self.drag_pos = Some(cur);
            }
            ui.ctx().request_repaint();
        }

        // 3. Drag drop release
        if allow_drag && response.drag_stopped_by(egui::PointerButton::Primary) {
            if let Some(from_sq) = self.dragging_from.take() {
                let drop_pos = self.drag_pos.take().or(pointer_pos);
                self.drag_prev_pos = None;
                self.drag_velocity = Vec2::ZERO;
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
                                    if let Some(p) = match_state.current_pos.board().piece_at(from_sq)
                                    {
                                        self.anim_layer
                                            .trigger_snapback(to_sq, from_sq, p.color, p.role);
                                    }
                                }
                            } else if can_premove {
                                self.enqueue_premove(from_sq, to_sq, Some(Role::Queen));
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
                            self.enqueue_premove(from_sq, clicked_sq, Some(Role::Queen));
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
