use crate::{
    config::{AnimationProfile, PieceSetStyle},
    domain::GameOutcome,
    engine::review::MoveClassification,
    ui::{board_view::grid::square_to_rect, piece_renderer::draw_chess_piece_styled},
};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Rounding, Stroke, Vec2};
use shakmaty::{Chess, Color, File, Move, Position, Role, Square};
use std::{
    f32::consts::PI,
    time::{Duration, Instant},
};

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

#[derive(Debug, Clone)]
pub struct VfxParticle {
    pub origin_sq: Square,
    pub velocity: Vec2,
    pub color: Color32,
    pub radius: f32,
    pub started_at: Instant,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ConfettiPiece {
    pub x_norm: f32,
    pub start_y_norm: f32,
    pub speed_y: f32,
    pub sway_amp: f32,
    pub sway_freq: f32,
    pub color: Color32,
    pub size: Vec2,
    pub started_at: Instant,
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub struct LandingRippleAnim {
    pub square: Square,
    pub started_at: Instant,
    pub duration_ms: u64,
    pub color: Color32,
}

#[derive(Debug, Clone)]
pub struct MoveBadgeAnim {
    pub square: Square,
    pub classification: MoveClassification,
    pub started_at: Instant,
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
    pub particles: Vec<VfxParticle>,
    pub confetti: Vec<ConfettiPiece>,
    pub landing_ripples: Vec<LandingRippleAnim>,
    pub move_badge: Option<MoveBadgeAnim>,
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
        self.particles.clear();
        self.confetti.clear();
        self.landing_ripples.clear();
        self.move_badge = None;
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
        let particles_active = self
            .particles
            .iter()
            .any(|p| now.saturating_duration_since(p.started_at).as_millis() < p.duration_ms as u128);
        let confetti_active = self
            .confetti
            .iter()
            .any(|c| now.saturating_duration_since(c.started_at).as_millis() < c.duration_ms as u128);
        let ripples_active = self
            .landing_ripples
            .iter()
            .any(|r| now.saturating_duration_since(r.started_at).as_millis() < r.duration_ms as u128);
        let badge_spring_active = self
            .move_badge
            .as_ref()
            .is_some_and(|b| now.saturating_duration_since(b.started_at) < Duration::from_millis(280));
        let shake_active = self.board_shake_until.is_some_and(|until| now < until);
        let check_pulse_active = self.check_pulse_started.is_some_and(|st| {
            now.saturating_duration_since(st) < Duration::from_millis(650)
        });

        !(slides_active
            || fades_active
            || snap_active
            || particles_active
            || confetti_active
            || ripples_active
            || badge_spring_active
            || shake_active
            || check_pulse_active)
    }

    pub fn set_move_classification_badge(&mut self, sq: Square, classification: MoveClassification) {
        let now = Instant::now();
        self.move_badge = Some(MoveBadgeAnim {
            square: sq,
            classification,
            started_at: now,
        });
        // Brilliant / Great tactical shockwave ring
        if matches!(
            classification,
            MoveClassification::Brilliant | MoveClassification::Great
        ) {
            let rgb = classification.rgb();
            self.landing_ripples.push(LandingRippleAnim {
                square: sq,
                started_at: now,
                duration_ms: 420,
                color: Color32::from_rgb(rgb[0], rgb[1], rgb[2]),
            });
        }
    }

    /// Memicu hujan konfeti emas/hijau di atas papan saat menang melawan bot (Feature #11).
    pub fn trigger_victory_confetti(&mut self) {
        let now = Instant::now();
        let palette = [
            Color32::from_rgb(0xFF, 0xAA, 0x00),
            Color32::from_rgb(0x81, 0xB6, 0x4C),
            Color32::from_rgb(0x00, 0x9F, 0xD9),
            Color32::from_rgb(0xF5, 0xF5, 0xF5),
            Color32::from_rgb(255, 115, 165),
        ];

        self.confetti.clear();
        for i in 0..42usize {
            let f = i as f32;
            let x_norm = ((f * 0.618_034) % 1.0).clamp(0.03, 0.97);
            let start_y_norm = -0.05 - ((f * 0.31) % 0.35);
            let speed_y = 0.55 + ((f * 0.19) % 0.50);
            let sway_amp = 10.0 + ((f * 7.3) % 18.0);
            let sway_freq = 3.5 + ((f * 1.7) % 4.5);
            self.confetti.push(ConfettiPiece {
                x_norm,
                start_y_norm,
                speed_y,
                sway_amp,
                sway_freq,
                color: palette[i % palette.len()],
                size: Vec2::new(6.0 + (i % 3) as f32 * 2.0, 4.0 + (i % 2) as f32 * 2.0),
                started_at: now,
                duration_ms: 1750,
            });
        }
    }

    fn spawn_capture_particles(&mut self, sq: Square, captured_color: Color) {
        let now = Instant::now();
        let base_colors = match captured_color {
            Color::White => [
                Color32::from_rgb(250, 248, 238),
                Color32::from_rgb(0xFF, 0xAA, 0x00),
            ],
            Color::Black => [
                Color32::from_rgb(65, 68, 76),
                Color32::from_rgb(0xE0, 0x4F, 0x44),
            ],
        };

        for i in 0..14usize {
            let angle = (i as f32 / 14.0) * PI * 2.0 + (i % 3) as f32 * 0.14;
            let speed = 55.0 + ((i * 29) % 75) as f32;
            self.particles.push(VfxParticle {
                origin_sq: sq,
                velocity: Vec2::new(angle.cos() * speed, angle.sin() * speed),
                color: base_colors[i % 2],
                radius: 2.2 + (i % 3) as f32 * 1.1,
                started_at: now,
                duration_ms: 360,
            });
        }
    }

    /// Memicu animasi langkah (slide bidak, capture fade + particle burst, landing ripple,
    /// check pulse, dan King result badges).
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
            // Soft landing ripple ring at destination square (Feature #15b)
            self.landing_ripples.push(LandingRippleAnim {
                square: mv.to(),
                started_at: now,
                duration_ms: 240,
                color: Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0x33, 180),
            });

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
                        self.spawn_capture_particles(*to, mover_color.other());
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
                    self.spawn_capture_particles(cap_sq, mover_color.other());
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

        if pos_after.is_check() {
            self.check_pulse_sq = pos_after.board().king_of(pos_after.turn());
            self.check_pulse_started = Some(now);
        } else {
            self.check_pulse_sq = None;
            self.check_pulse_started = None;
        }

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

        self.update_result_glyphs(pos_after, outcome);
    }

    pub fn trigger_snapback(
        &mut self,
        dropped_sq: Square,
        origin_sq: Square,
        color: Color,
        role: Role,
    ) {
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
        let (Some(w_king), Some(b_king)) =
            (board.king_of(Color::White), board.king_of(Color::Black))
        else {
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

    pub fn prune_and_collect_hidden_squares(&mut self) -> Vec<Square> {
        let now = Instant::now();
        self.active_slides.retain(|a| {
            now.saturating_duration_since(a.started_at).as_millis() < a.duration_ms as u128
        });
        self.fading_captures.retain(|f| {
            now.saturating_duration_since(f.started_at).as_millis() < f.duration_ms as u128
        });
        self.particles.retain(|p| {
            now.saturating_duration_since(p.started_at).as_millis() < p.duration_ms as u128
        });
        self.confetti.retain(|c| {
            now.saturating_duration_since(c.started_at).as_millis() < c.duration_ms as u128
        });
        self.landing_ripples.retain(|r| {
            now.saturating_duration_since(r.started_at).as_millis() < r.duration_ms as u128
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

    pub fn render_animations(
        &self,
        painter: &Painter,
        board_rect: Rect,
        flipped: bool,
        piece_set: PieceSetStyle,
    ) {
        let now = Instant::now();
        let sq_size = board_rect.width() / 8.0;

        // 1. Pulsing radial crimson glow behind King in check
        if let (Some(check_sq), Some(started)) = (self.check_pulse_sq, self.check_pulse_started) {
            let elapsed_s = now.saturating_duration_since(started).as_secs_f32();
            if elapsed_s < 0.65 {
                let pulse = (elapsed_s * PI * 3.0).sin().abs();
                let sq_rect = square_to_rect(board_rect, check_sq, flipped);
                let alpha = (85.0 + pulse * 110.0).clamp(40.0, 210.0) as u8;
                painter.circle_filled(
                    sq_rect.center(),
                    sq_size * (0.42 + pulse * 0.10),
                    Color32::from_rgba_unmultiplied(245, 35, 35, alpha),
                );
            }
        }

        // 2. Soft landing ripple rings & brilliant shockwaves
        for ripple in &self.landing_ripples {
            let elapsed = now.saturating_duration_since(ripple.started_at).as_millis() as f32;
            let t = (elapsed / (ripple.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let sq_rect = square_to_rect(board_rect, ripple.square, flipped);
            let radius = sq_size * (0.18 + t * 0.44);
            let alpha = ((1.0 - t) * 185.0) as u8;
            let c = Color32::from_rgba_unmultiplied(
                ripple.color.r(),
                ripple.color.g(),
                ripple.color.b(),
                alpha,
            );
            painter.circle_stroke(sq_rect.center(), radius, Stroke::new(2.2_f32 * (1.0 - t * 0.5), c));
        }

        // 3. Capture fade-outs (120 ms)
        for fade in &self.fading_captures {
            let elapsed = now.saturating_duration_since(fade.started_at).as_millis() as f32;
            let t = (elapsed / (fade.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let shrink_scale = sq_size * (1.0 - t * 0.35);
            let sq_rect = square_to_rect(board_rect, fade.square, flipped);
            draw_chess_piece_styled(
                painter,
                sq_rect.center(),
                shrink_scale,
                fade.color,
                fade.role,
                false,
                piece_set,
            );
        }

        // 4. Active sliding pieces (smooth fluid glide with zero shadow snapping)
        for slide in &self.active_slides {
            let elapsed = now.saturating_duration_since(slide.started_at).as_millis() as f32;
            let raw_t = (elapsed / (slide.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let (eased_t, lift_scale) = evaluate_curve(raw_t, slide.profile);

            let from_c = square_to_rect(board_rect, slide.from, flipped).center();
            let to_c = square_to_rect(board_rect, slide.to, flipped).center();
            let arc_lift = (raw_t * PI).sin() * sq_size * 0.045;
            let pos = Pos2::new(
                from_c.x + (to_c.x - from_c.x) * eased_t,
                from_c.y + (to_c.y - from_c.y) * eased_t - arc_lift,
            );

            draw_chess_piece_styled(
                painter,
                pos,
                sq_size * lift_scale,
                slide.color,
                slide.role,
                false,
                piece_set,
            );
        }

        // 5. Snap-back animation (110 ms elastic return)
        if let Some(snap) = &self.snapback_anim {
            let elapsed = now.saturating_duration_since(snap.started_at).as_millis() as f32;
            let raw_t = (elapsed / (snap.duration_ms.max(1) as f32)).clamp(0.0, 1.0);
            let (eased_t, _) = evaluate_curve(raw_t, AnimationProfile::Fast);
            let from_c = square_to_rect(board_rect, snap.from, flipped).center();
            let to_c = square_to_rect(board_rect, snap.to, flipped).center();
            let pos = from_c + (to_c - from_c) * eased_t;
            draw_chess_piece_styled(
                painter,
                pos,
                sq_size,
                snap.color,
                snap.role,
                false,
                piece_set,
            );
        }

        // 6. Capture Particle Bursts (Feature #11)
        for p in &self.particles {
            let elapsed_s = now.saturating_duration_since(p.started_at).as_secs_f32();
            let t = (elapsed_s / (p.duration_ms as f32 / 1000.0)).clamp(0.0, 1.0);
            let origin = square_to_rect(board_rect, p.origin_sq, flipped).center();
            let gravity = 160.0 * elapsed_s * elapsed_s;
            let pos = Pos2::new(
                origin.x + p.velocity.x * elapsed_s,
                origin.y + p.velocity.y * elapsed_s + gravity,
            );
            let alpha = ((1.0 - t) * 230.0) as u8;
            let col = Color32::from_rgba_unmultiplied(p.color.r(), p.color.g(), p.color.b(), alpha);
            painter.circle_filled(pos, p.radius * (1.0 - t * 0.4), col);
        }

        // 7. Spring-Animated On-Board Move Classification Badge (Feature #12)
        if let Some(badge) = &self.move_badge {
            let elapsed_ms = now.saturating_duration_since(badge.started_at).as_millis() as f32;
            let t = (elapsed_ms / 260.0).clamp(0.0, 1.0);
            // Spring overshoot curve: 0.0 -> 1.22 -> 1.0
            let spring_scale = if t < 1.0 {
                let u = t - 1.0;
                (1.0 + 2.8 * u * u * u + 1.8 * u * u).max(0.05)
            } else {
                1.0
            };
            render_move_classification_badge(
                painter,
                board_rect,
                badge.square,
                badge.classification,
                spring_scale,
                flipped,
            );
        }

        // 8. End-of-game King result badges
        if let Some((w_sq, w_glyph, b_sq, b_glyph)) = self.result_glyphs {
            render_king_result_badge(painter, board_rect, w_sq, w_glyph, flipped);
            render_king_result_badge(painter, board_rect, b_sq, b_glyph, flipped);
        }

        // 9. Victory Confetti cascade (Feature #11)
        for c in &self.confetti {
            let elapsed_s = now.saturating_duration_since(c.started_at).as_secs_f32();
            let t = (elapsed_s / (c.duration_ms as f32 / 1000.0)).clamp(0.0, 1.0);
            let cx = board_rect.min.x
                + c.x_norm * board_rect.width()
                + (elapsed_s * c.sway_freq).sin() * c.sway_amp;
            let cy = board_rect.min.y
                + (c.start_y_norm + elapsed_s * c.speed_y) * board_rect.height();
            if board_rect.contains(Pos2::new(cx, cy)) {
                let alpha = ((1.0 - t * 0.85) * 245.0) as u8;
                let col =
                    Color32::from_rgba_unmultiplied(c.color.r(), c.color.g(), c.color.b(), alpha);
                let r = Rect::from_center_size(Pos2::new(cx, cy), c.size);
                painter.rect_filled(r, Rounding::same(1.5_f32), col);
            }
        }
    }
}

pub fn render_move_classification_badge(
    painter: &Painter,
    board_rect: Rect,
    sq: Square,
    classification: MoveClassification,
    scale_factor: f32,
    flipped: bool,
) {
    let sq_rect = square_to_rect(board_rect, sq, flipped);
    let sq_size = sq_rect.width();
    let radius = (sq_size * 0.19).clamp(9.0, 14.5) * scale_factor;
    let center = Pos2::new(
        sq_rect.max.x - radius * 0.72,
        sq_rect.min.y + radius * 0.72,
    );
    let rgb = classification.rgb();
    let bg = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);

    painter.circle_filled(center, radius, bg);
    painter.circle_stroke(center, radius, Stroke::new(1.5_f32, Color32::WHITE));
    painter.text(
        center,
        Align2::CENTER_CENTER,
        classification.symbol(),
        FontId::proportional((radius * 1.05).max(8.0)),
        Color32::WHITE,
    );
}

fn evaluate_curve(t: f32, profile: AnimationProfile) -> (f32, f32) {
    match profile {
        AnimationProfile::None => (1.0, 1.0),
        AnimationProfile::Fast | AnimationProfile::Medium => {
            let inv = 1.0 - t;
            (1.0 - inv * inv * inv, 1.0)
        }
        AnimationProfile::Slow => {
            let eased = if t < 0.5 {
                16.0 * t * t * t * t * t
            } else {
                let u = -2.0 * t + 2.0;
                1.0 - (u * u * u * u * u) * 0.5
            };
            (eased, 1.0)
        }
        AnimationProfile::Natural => {
            let inv = 1.0 - t;
            let eased = 1.0 - inv * inv * inv * inv;
            let lift = 1.0 + 0.08 * (4.0 * t * (1.0 - t));
            (eased, lift)
        }
        AnimationProfile::Arcade => {
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
