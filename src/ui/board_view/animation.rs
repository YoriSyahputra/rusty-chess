use crate::{
    config::AnimationProfile,
    domain::GameOutcome,
    ui::{board_view::grid::square_to_rect, piece_renderer::draw_chess_piece},
};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Stroke, Vec2};
use shakmaty::{Chess, Color, File, Move, Position, Role, Square};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct PieceSlideAnim {
    pub from: Square,
    pub to: Square,
    pub color: Color,
    pub role: Role,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub profile: AnimationProfile,
}

#[derive(Debug, Clone)]
pub struct CaptureFadeAnim {
    pub square: Square,
    pub color: Color,
    pub role: Role,
    pub started_at: Instant,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultGlyph {
    WinnerCrown,
    LoserCross,
    DrawHalf,
}

#[derive(Debug, Clone, Default)]
pub struct BoardAnimationLayer {
    pub active_slides: Vec<PieceSlideAnim>,
    pub fading_captures: Vec<CaptureFadeAnim>,
    pub snapback_anim: Option<PieceSlideAnim>,
    pub check_pulse_sq: Option<Square>,
    pub check_pulse_started: Option<Instant>,
    pub result_glyphs: Option<(Square, ResultGlyph, Square, ResultGlyph)>,
    pub board_shake_until: Option<Instant>,
}

impl BoardAnimationLayer {
    pub fn clear(&mut self) {
        self.active_slides.clear();
        self.fading_captures.clear();
        self.snapback_anim = None;
        self.check_pulse_sq = None;
        self.check_pulse_started = None;
        self.result_glyphs = None;
        self.board_shake_until = None;
    }

    /// Mengembalikan `true` jika tidak ada animasi aktif yang memerlukan repaint frame berikutnya.
    pub fn is_idle(&self) -> bool {
        let now = Instant::now();
        let slides_active = self
            .active_slides
            .iter()
            .any(|a| now.saturating_duration_since(a.started_at).as_millis() < a.duration_ms as u128);
        let fades_active = self
            .fading_captures
            .iter()
            .any(|f| now.saturating_duration_since(f.started_at).as_millis() < f.duration_ms as u128);
        let snap_active = self.snapback_anim.as_ref().is_some_and(|s| {
            now.saturating_duration_since(s.started_at).as_millis() < s.duration_ms as u128
        });
        let shake_active = self
            .board_shake_until
            .is_some_and(|until| now < until);
        let check_pulse_active = self.check_pulse_started.is_some_and(|st| {
            now.saturating_duration_since(st) < Duration::from_millis(650)
        });

        !(slides_active || fades_active || snap_active || shake_active || check_pulse_active)
    }

    /// Memicu animasi langkah (termasuk King + Rook saat rokade, fade-out bidak yang dimakan,
    /// check pulse, dan badge hasil akhir pada Raja).
    pub fn trigger_move_animation(
        &mut self,
        pos_before: &Chess,
        pos_after: &Chess,
        mv: &Move,
        outcome: GameOutcome,
        profile: AnimationProfile,
    ) {
        let now = Instant::now();
        let dur_ms = profile.duration_ms();
        let mover_color = pos_before.turn();

        if dur_ms > 0 {
            match mv {
                Move::Normal {
                    role,
                    from,
                    capture,
                    to,
                    promotion,
                } => {
                    let final_role = promotion.unwrap_or(*role);
                    self.active_slides.push(PieceSlideAnim {
                        from: *from,
                        to: *to,
                        color: mover_color,
                        role: final_role,
                        started_at: now,
                        duration_ms: dur_ms,
                        profile,
                    });
                    if let Some(cap_role) = capture {
                        self.fading_captures.push(CaptureFadeAnim {
                            square: *to,
                            color: mover_color.other(),
                            role: *cap_role,
                            started_at: now,
                            duration_ms: 120,
                        });
                    }
                }
                Move::EnPassant { from, to } => {
                    self.active_slides.push(PieceSlideAnim {
                        from: *from,
                        to: *to,
                        color: mover_color,
                        role: Role::Pawn,
                        started_at: now,
                        duration_ms: dur_ms,
                        profile,
                    });
                    let cap_sq = Square::from_coords(to.file(), from.rank());
                    self.fading_captures.push(CaptureFadeAnim {
                        square: cap_sq,
                        color: mover_color.other(),
                        role: Role::Pawn,
                        started_at: now,
                        duration_ms: 120,
                    });
                }
                Move::Castle { king, rook } => {
                    let rank = king.rank();
                    let is_kingside = rook.file() > king.file();
                    let (king_to_file, rook_to_file) = if is_kingside {
                        (File::G, File::F)
                    } else {
                        (File::C, File::D)
                    };
                    let king_to = Square::from_coords(king_to_file, rank);
                    let rook_to = Square::from_coords(rook_to_file, rank);

                    self.active_slides.push(PieceSlideAnim {
                        from: *king,
                        to: king_to,
                        color: mover_color,
                        role: Role::King,
                        started_at: now,
                        duration_ms: dur_ms,
                        profile,
                    });
                    self.active_slides.push(PieceSlideAnim {
                        from: *rook,
                        to: rook_to,
                        color: mover_color,
                        role: Role::Rook,
                        started_at: now,
                        duration_ms: dur_ms,
                        profile,
                    });
                }
                Move::Put { .. } => {}
            }
        }

        // Check radial pulse
        if pos_after.is_check() {
            self.check_pulse_sq = pos_after.board().king_of(pos_after.turn());
            self.check_pulse_started = Some(now);
        } else {
            self.check_pulse_sq = None;
            self.check_pulse_started = None;
        }

        // Arcade micro board vibration on checkmate or queen capture
        if profile == AnimationProfile::Arcade {
            let captured_queen = matches!(
                mv,
                Move::Normal {
                    capture: Some(Role::Queen),
                    ..
                }
            );
            if pos_after.is_checkmate() || captured_queen {
                self.board_shake_until = Some(now + Duration::from_millis(80));
            }
        }

        // Update end-of-game King result badges
        self.update_result_glyphs(pos_after, outcome);
    }

    pub fn trigger_snapback(&mut self, dropped_sq: Square, origin_sq: Square, color: Color, role: Role) {
        self.snapback_anim = Some(PieceSlideAnim {
            from: dropped_sq,
            to: origin_sq,
            color,
            role,
            started_at: Instant::now(),
            duration_ms: 110,
            profile: AnimationProfile::Fast,
        });
    }

    pub fn update_result_glyphs(&mut self, pos: &Chess, outcome: GameOutcome) {
        if !outcome.is_game_over() {
            self.result_glyphs = None;
            return;
        }
        let board = pos.board();
        let (Some(w_king), Some(b_king)) = (board.king_of(Color::White), board.king_of(Color::Black)) else {
            self.result_glyphs = None;
            return;
        };

        self.result_glyphs = Some(match outcome.winner() {
            Some(Color::White) => (
                w_king,
                ResultGlyph::WinnerCrown,
                b_king,
                ResultGlyph::LoserCross,
            ),
            Some(Color::Black) => (
                w_king,
                ResultGlyph::LoserCross,
                b_king,
                ResultGlyph::WinnerCrown,
            ),
            None => (w_king, ResultGlyph::DrawHalf, b_king, ResultGlyph::DrawHalf),
        });
    }

    /// Mengembalikan offset getaran mikro papan untuk mode Arcade (`Vec2::ZERO` jika tidak aktif).
    pub fn current_board_shake_offset(&self) -> Vec2 {
        let now = Instant::now();
        if let Some(until) = self.board_shake_until {
            if now < until {
                let rem = until.saturating_duration_since(now).as_millis() as f32;
                let phase = (rem * 0.45).sin() * 2.0;
                return Vec2::new(phase, -phase * 0.7);
            }
        }
        Vec2::ZERO
    }

    /// Memangkas animasi yang sudah selesai dan mengembalikan daftar petak tujuan (`to`)
    /// yang sedang dalam perjalanan animasi agar bidak statis di petak tujuan tidak digambar ganda.
    pub fn prune_and_collect_hidden_squares(&mut self) -> Vec<Square> {
        let now = Instant::now();
        self.active_slides.retain(|a| {
            now.saturating_duration_since(a.started_at).as_millis() < a.duration_ms as u128
        });
        self.fading_captures.retain(|f| {
            now.saturating_duration_since(f.started_at).as_millis() < f.duration_ms as u128
        });
        if let Some(snap) = &self.snapback_anim {
            if now.saturating_duration_since(snap.started_at).as_millis() >= snap.duration_ms as u128
            {
                self.snapback_anim = None;
            }
        }
        if let Some(until) = self.board_shake_until {
            if now >= until {
                self.board_shake_until = None;
            }
        }

        let mut hidden = Vec::with_capacity(3);
        for slide in &self.active_slides {
            hidden.push(slide.to);
        }
        if let Some(snap) = &self.snapback_anim {
            hidden.push(snap.to);
        }
        hidden
    }

    /// Menggambar seluruh layer animasi (capture fade-outs, sliding pieces, snapbacks, check pulse, dan King result badges).
    pub fn render_animations(
        &self,
        painter: &Painter,
        board_rect: Rect,
        flipped: bool,
    ) {
        let now = Instant::now();
        let sq_size = board_rect.width() / 8.0;

        // 1. Pulsing radial crimson glow behind King in check
        if let (Some(check_sq), Some(started)) = (self.check_pulse_sq, self.check_pulse_started) {
            let elapsed_s = now.saturating_duration_since(started).as_secs_f32();
            if elapsed_s < 0.65 {
                let pulse = (elapsed_s * std::f32::consts::PI * 3.0).sin().abs();
                let sq_rect = square_to_rect(board_rect, check_sq, flipped);
                let alpha = (85.0 + pulse * 110.0).clamp(40.0, 210.0) as u8;
                painter.circle_filled(
                    sq_rect.center(),
                    sq_size * (0.42 + pulse * 0.10),
                    Color32::from_rgba_unmultiplied(245, 35, 35, alpha),
                );
            }
        }

        // 2. Capture fade-outs (120 ms)
        for fade in &self.fading_captures {
            let elapsed = now.saturating_duration_since(fade.started_at).as_millis() as f32;
            let t = (elapsed / (fade.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let shrink_scale = sq_size * (1.0 - t * 0.35);
            let sq_rect = square_to_rect(board_rect, fade.square, flipped);
            draw_chess_piece(
                painter,
                sq_rect.center(),
                shrink_scale,
                fade.color,
                fade.role,
                false,
            );
        }

        // 3. Active sliding pieces (5 animation profiles)
        for slide in &self.active_slides {
            let elapsed = now.saturating_duration_since(slide.started_at).as_millis() as f32;
            let raw_t = (elapsed / (slide.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let (eased_t, lift_scale) = evaluate_curve(raw_t, slide.profile);

            let from_c = square_to_rect(board_rect, slide.from, flipped).center();
            let to_c = square_to_rect(board_rect, slide.to, flipped).center();
            let pos = from_c + (to_c - from_c) * eased_t;

            draw_chess_piece(
                painter,
                pos,
                sq_size * lift_scale,
                slide.color,
                slide.role,
                lift_scale > 1.02,
            );
        }

        // 4. Snap-back animation (110 ms elastic return)
        if let Some(snap) = &self.snapback_anim {
            let elapsed = now.saturating_duration_since(snap.started_at).as_millis() as f32;
            let raw_t = (elapsed / (snap.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let (eased_t, _) = evaluate_curve(raw_t, AnimationProfile::Fast);
            let from_c = square_to_rect(board_rect, snap.from, flipped).center();
            let to_c = square_to_rect(board_rect, snap.to, flipped).center();
            let pos = from_c + (to_c - from_c) * eased_t;
            draw_chess_piece(painter, pos, sq_size, snap.color, snap.role, false);
        }

        // 5. End-of-game King result badges
        if let Some((w_sq, w_glyph, b_sq, b_glyph)) = self.result_glyphs {
            render_king_result_badge(painter, board_rect, w_sq, w_glyph, flipped);
            render_king_result_badge(painter, board_rect, b_sq, b_glyph, flipped);
        }
    }
}

fn evaluate_curve(t: f32, profile: AnimationProfile) -> (f32, f32) {
    match profile {
        AnimationProfile::None => (1.0, 1.0),
        AnimationProfile::Fast | AnimationProfile::Medium => {
            // Cubic Ease-Out: 1 - (1 - t)^3
            let inv = 1.0 - t;
            (1.0 - inv * inv * inv, 1.0)
        }
        AnimationProfile::Slow => {
            // Quintic Ease-In-Out
            let eased = if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                let u = -2.0 * t + 2.0;
                1.0 - (u * u * u * u * u) * 0.5
            };
            (eased, 1.0)
        }
        AnimationProfile::Natural => {
            // Quartic Ease-Out + parabolic midpoint scale 1.0 -> 1.08 -> 1.0
            let inv = 1.0 - t;
            let eased = 1.0 - inv * inv * inv * inv;
            let lift = 1.0 + 0.08 * (4.0 * t * (1.0 - t));
            (eased, lift)
        }
        AnimationProfile::Arcade => {
            // Back-Out overshoot curve
            let c1 = 1.70158_f32;
            let c3 = c1 + 1.0;
            let u = t - 1.0;
            let eased = 1.0 + c3 * u * u * u + c1 * u * u;
            (eased, 1.04)
        }
    }
}

fn render_king_result_badge(
    painter: &Painter,
    board_rect: Rect,
    king_sq: Square,
    glyph: ResultGlyph,
    flipped: bool,
) {
    let sq_rect = square_to_rect(board_rect, king_sq, flipped);
    let sq_size = sq_rect.width();
    let badge_radius = (sq_size * 0.20).clamp(9.0, 15.0);
    let badge_center = Pos2::new(
        sq_rect.max.x - badge_radius * 0.85,
        sq_rect.min.y + badge_radius * 0.85,
    );

    let (bg_color, symbol) = match glyph {
        ResultGlyph::WinnerCrown => (Color32::from_rgb(0x81, 0xB6, 0x4C), "👑"),
        ResultGlyph::LoserCross => (Color32::from_rgb(0xE0, 0x4F, 0x44), "✗"),
        ResultGlyph::DrawHalf => (Color32::from_rgb(0xFF, 0xAA, 0x00), "½"),
    };

    painter.circle_filled(badge_center, badge_radius, bg_color);
    painter.circle_stroke(badge_center, badge_radius, Stroke::new(1.5_f32, Color32::WHITE));
    painter.text(
        badge_center,
        Align2::CENTER_CENTER,
        symbol,
        FontId::proportional(badge_radius * 1.15),
        Color32::WHITE,
    );
}
