use crate::engine::bot::BotId;
use egui::{Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use shakmaty::{Color, Role};

/// Menggambar bidak catur vektor prosedural yang tajam (Staunton silhouette)
/// lengkap dengan drop shadow, anti-aliased stroke, dan detail internal.
pub fn draw_chess_piece(
    painter: &Painter,
    center: Pos2,
    square_size: f32,
    color: Color,
    role: Role,
    is_dragged: bool,
) {
    let scale = if is_dragged {
        square_size * 1.15
    } else {
        square_size
    };

    let (fill_color, inner_accent, outline_color) = match color {
        Color::White => (
            Color32::from_rgb(248, 247, 242),
            Color32::from_rgb(218, 214, 202),
            Color32::from_rgb(32, 34, 38),
        ),
        Color::Black => (
            Color32::from_rgb(40, 43, 50),
            Color32::from_rgb(95, 102, 118),
            Color32::from_rgb(14, 15, 18),
        ),
    };

    let stroke_width = (scale * 0.032).clamp(1.4, 2.8);
    let outline = Stroke::new(stroke_width, outline_color);
    let accent_stroke = Stroke::new((stroke_width * 0.85).max(1.1), inner_accent);

    // Drop shadow
    let shadow_offset = if is_dragged {
        Vec2::new(scale * 0.04, scale * 0.07)
    } else {
        Vec2::new(scale * 0.018, scale * 0.032)
    };
    let shadow_alpha = if is_dragged { 95 } else { 65 };
    let shadow_color = Color32::from_rgba_unmultiplied(0, 0, 0, shadow_alpha);

    painter.circle_filled(
        Pos2::new(
            center.x + shadow_offset.x,
            center.y + scale * 0.30 + shadow_offset.y,
        ),
        scale * 0.31,
        shadow_color,
    );

    match role {
        Role::Pawn => draw_pawn(painter, center, scale, fill_color, outline, accent_stroke),
        Role::Rook => draw_rook(painter, center, scale, fill_color, outline, accent_stroke),
        Role::Knight => draw_knight(
            painter,
            center,
            scale,
            fill_color,
            inner_accent,
            outline,
            accent_stroke,
        ),
        Role::Bishop => draw_bishop(painter, center, scale, fill_color, outline, accent_stroke),
        Role::Queen => draw_queen(
            painter,
            center,
            scale,
            fill_color,
            inner_accent,
            outline,
            accent_stroke,
        ),
        Role::King => draw_king(painter, center, scale, fill_color, outline, accent_stroke),
    }
}

fn draw_basepedestal(
    painter: &Painter,
    center: Pos2,
    scale: f32,
    width_factor: f32,
    fill: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    let w = scale * width_factor;
    let base_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + scale * 0.33),
        Vec2::new(w, scale * 0.09),
    );
    painter.rect(base_rect, Rounding::same(scale * 0.03), fill, outline);

    let step_rect = Rect::from_center_size(
        Pos2::new(center.x, center.y + scale * 0.26),
        Vec2::new(w * 0.82, scale * 0.065),
    );
    painter.rect(step_rect, Rounding::same(scale * 0.025), fill, outline);

    painter.line_segment(
        [
            Pos2::new(center.x - w * 0.34, center.y + scale * 0.33),
            Pos2::new(center.x + w * 0.34, center.y + scale * 0.33),
        ],
        accent,
    );
}

fn draw_pawn(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Stem trapezoid
    let stem = vec![
        Pos2::new(c.x - s * 0.11, c.y - s * 0.06),
        Pos2::new(c.x + s * 0.11, c.y - s * 0.06),
        Pos2::new(c.x + s * 0.21, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.21, c.y + s * 0.24),
    ];
    painter.add(Shape::convex_polygon(stem, fill, outline));

    // Collar ring
    let collar = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.07),
        Vec2::new(s * 0.28, s * 0.055),
    );
    painter.rect(collar, Rounding::same(s * 0.025), fill, outline);

    // Spherical head
    let head_center = Pos2::new(c.x, c.y - s * 0.21);
    painter.circle(head_center, s * 0.145, fill, outline);
    painter.circle_stroke(
        Pos2::new(head_center.x - s * 0.03, head_center.y - s * 0.03),
        s * 0.05,
        accent,
    );

    draw_basepedestal(painter, c, s, 0.56, fill, outline, accent);
}

fn draw_rook(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Tower body
    let body = vec![
        Pos2::new(c.x - s * 0.17, c.y - s * 0.16),
        Pos2::new(c.x + s * 0.17, c.y - s * 0.16),
        Pos2::new(c.x + s * 0.23, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.23, c.y + s * 0.24),
    ];
    painter.add(Shape::convex_polygon(body, fill, outline));

    // Crown beam
    let crown_beam = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.20),
        Vec2::new(s * 0.44, s * 0.09),
    );
    painter.rect(crown_beam, Rounding::same(s * 0.02), fill, outline);

    // 3 battlements (merlons)
    for dx in [-0.155_f32, 0.0, 0.155] {
        let merlon = Rect::from_center_size(
            Pos2::new(c.x + s * dx, c.y - s * 0.285),
            Vec2::new(s * 0.105, s * 0.095),
        );
        painter.rect(merlon, Rounding::same(s * 0.015), fill, outline);
    }

    // Internal masonry vertical detail
    painter.line_segment(
        [
            Pos2::new(c.x - s * 0.07, c.y - s * 0.09),
            Pos2::new(c.x - s * 0.09, c.y + s * 0.17),
        ],
        accent,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + s * 0.07, c.y - s * 0.09),
            Pos2::new(c.x + s * 0.09, c.y + s * 0.17),
        ],
        accent,
    );

    draw_basepedestal(painter, c, s, 0.62, fill, outline, accent);
}

fn draw_knight(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    inner_accent: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Neck and chest convex polygon
    let neck_chest = vec![
        Pos2::new(c.x - s * 0.08, c.y - s * 0.26),
        Pos2::new(c.x + s * 0.19, c.y - s * 0.18),
        Pos2::new(c.x + s * 0.24, c.y + s * 0.04),
        Pos2::new(c.x + s * 0.23, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.22, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.16, c.y + s * 0.03),
    ];
    painter.add(Shape::convex_polygon(neck_chest, fill, outline));

    // Horse snout & jaw (facing left)
    let head_snout = vec![
        Pos2::new(c.x - s * 0.04, c.y - s * 0.30),
        Pos2::new(c.x + s * 0.14, c.y - s * 0.22),
        Pos2::new(c.x + s * 0.04, c.y - s * 0.06),
        Pos2::new(c.x - s * 0.25, c.y - s * 0.08),
        Pos2::new(c.x - s * 0.27, c.y - s * 0.17),
    ];
    painter.add(Shape::convex_polygon(head_snout, fill, outline));

    // Pointed ear
    let ear = vec![
        Pos2::new(c.x - s * 0.05, c.y - s * 0.28),
        Pos2::new(c.x + s * 0.02, c.y - s * 0.39),
        Pos2::new(c.x + s * 0.10, c.y - s * 0.25),
    ];
    painter.add(Shape::convex_polygon(ear, fill, outline));

    // Eye & nostril detail
    painter.circle_filled(
        Pos2::new(c.x - s * 0.08, c.y - s * 0.20),
        s * 0.025,
        inner_accent,
    );
    painter.circle_filled(
        Pos2::new(c.x - s * 0.21, c.y - s * 0.13),
        s * 0.016,
        inner_accent,
    );

    // Mane curve line
    painter.line_segment(
        [
            Pos2::new(c.x + s * 0.11, c.y - s * 0.19),
            Pos2::new(c.x + s * 0.16, c.y + s * 0.14),
        ],
        accent,
    );

    draw_basepedestal(painter, c, s, 0.62, fill, outline, accent);
}

fn draw_bishop(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Lower body skirt
    let skirt = vec![
        Pos2::new(c.x - s * 0.13, c.y - s * 0.02),
        Pos2::new(c.x + s * 0.13, c.y - s * 0.02),
        Pos2::new(c.x + s * 0.22, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.22, c.y + s * 0.24),
    ];
    painter.add(Shape::convex_polygon(skirt, fill, outline));

    // Mitre dome (upper helmet)
    let mitre = vec![
        Pos2::new(c.x, c.y - s * 0.33),
        Pos2::new(c.x + s * 0.16, c.y - s * 0.19),
        Pos2::new(c.x + s * 0.18, c.y - s * 0.05),
        Pos2::new(c.x - s * 0.18, c.y - s * 0.05),
        Pos2::new(c.x - s * 0.16, c.y - s * 0.19),
    ];
    painter.add(Shape::convex_polygon(mitre, fill, outline));

    // Collar band
    let collar = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.02),
        Vec2::new(s * 0.32, s * 0.055),
    );
    painter.rect(collar, Rounding::same(s * 0.025), fill, outline);

    // Top finial orb
    painter.circle(Pos2::new(c.x, c.y - s * 0.365), s * 0.042, fill, outline);

    // Classic Bishop mitre cross/slit
    painter.line_segment(
        [
            Pos2::new(c.x - s * 0.06, c.y - s * 0.17),
            Pos2::new(c.x + s * 0.06, c.y - s * 0.17),
        ],
        accent,
    );
    painter.line_segment(
        [
            Pos2::new(c.x, c.y - s * 0.23),
            Pos2::new(c.x, c.y - s * 0.11),
        ],
        accent,
    );

    draw_basepedestal(painter, c, s, 0.62, fill, outline, accent);
}

fn draw_queen(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    inner_accent: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Lower gown
    let gown = vec![
        Pos2::new(c.x - s * 0.18, c.y + s * 0.01),
        Pos2::new(c.x + s * 0.18, c.y + s * 0.01),
        Pos2::new(c.x + s * 0.25, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.25, c.y + s * 0.24),
    ];
    painter.add(Shape::convex_polygon(gown, fill, outline));

    // Five-pointed royal coronet spikes (drawn as individual convex triangles)
    let spike_tips = [
        (-0.27_f32, -0.25_f32),
        (-0.14, -0.32),
        (0.0, -0.35),
        (0.14, -0.32),
        (0.27, -0.25),
    ];
    let spike_bases = [-0.21_f32, -0.105, 0.0, 0.105, 0.21];

    for (i, &(tx, ty)) in spike_tips.iter().enumerate() {
        let bx = spike_bases[i];
        let tri = vec![
            Pos2::new(c.x + s * (bx - 0.065), c.y + s * 0.03),
            Pos2::new(c.x + s * tx, c.y + s * ty),
            Pos2::new(c.x + s * (bx + 0.065), c.y + s * 0.03),
        ];
        painter.add(Shape::convex_polygon(tri, fill, outline));
        painter.circle(
            Pos2::new(c.x + s * tx, c.y + s * (ty - 0.025)),
            s * 0.033,
            fill,
            outline,
        );
    }

    // Crown jewel band
    let band = Rect::from_center_size(
        Pos2::new(c.x, c.y + s * 0.04),
        Vec2::new(s * 0.42, s * 0.065),
    );
    painter.rect(band, Rounding::same(s * 0.03), fill, outline);

    // Three jewels on the band
    for jx in [-0.11_f32, 0.0, 0.11] {
        painter.circle_filled(
            Pos2::new(c.x + s * jx, c.y + s * 0.04),
            s * 0.02,
            inner_accent,
        );
    }

    draw_basepedestal(painter, c, s, 0.66, fill, outline, accent);
}

fn draw_king(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Top cross vertical & horizontal bars
    let cross_v = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.355),
        Vec2::new(s * 0.065, s * 0.16),
    );
    let cross_h = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.365),
        Vec2::new(s * 0.16, s * 0.06),
    );
    painter.rect(cross_v, Rounding::same(s * 0.015), fill, outline);
    painter.rect(cross_h, Rounding::same(s * 0.015), fill, outline);

    // Royal mitre / arch crown
    let crown = vec![
        Pos2::new(c.x, c.y - s * 0.26),
        Pos2::new(c.x + s * 0.24, c.y - s * 0.16),
        Pos2::new(c.x + s * 0.19, c.y + s * 0.03),
        Pos2::new(c.x - s * 0.19, c.y + s * 0.03),
        Pos2::new(c.x - s * 0.24, c.y - s * 0.16),
    ];
    painter.add(Shape::convex_polygon(crown, fill, outline));

    // Lower robe pedestal
    let robe = vec![
        Pos2::new(c.x - s * 0.18, c.y + s * 0.03),
        Pos2::new(c.x + s * 0.18, c.y + s * 0.03),
        Pos2::new(c.x + s * 0.25, c.y + s * 0.24),
        Pos2::new(c.x - s * 0.25, c.y + s * 0.24),
    ];
    painter.add(Shape::convex_polygon(robe, fill, outline));

    // Collar band
    let band = Rect::from_center_size(
        Pos2::new(c.x, c.y + s * 0.03),
        Vec2::new(s * 0.42, s * 0.065),
    );
    painter.rect(band, Rounding::same(s * 0.03), fill, outline);

    // Inner arch lines
    painter.line_segment(
        [
            Pos2::new(c.x, c.y - s * 0.21),
            Pos2::new(c.x, c.y - s * 0.02),
        ],
        accent,
    );
    painter.line_segment(
        [
            Pos2::new(c.x - s * 0.11, c.y - s * 0.16),
            Pos2::new(c.x - s * 0.08, c.y - s * 0.02),
        ],
        accent,
    );
    painter.line_segment(
        [
            Pos2::new(c.x + s * 0.11, c.y - s * 0.16),
            Pos2::new(c.x + s * 0.08, c.y - s * 0.02),
        ],
        accent,
    );

    draw_basepedestal(painter, c, s, 0.68, fill, outline, accent);
}

/// Menggambar avatar vektor unik untuk tiap persona bot lengkap dengan border glow saat berpikir.
pub fn draw_bot_avatar(
    painter: &Painter,
    rect: Rect,
    bot_id: BotId,
    accent_rgb: [u8; 3],
    is_thinking: bool,
) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;
    let accent = Color32::from_rgb(accent_rgb[0], accent_rgb[1], accent_rgb[2]);
    let dark_bg = Color32::from_rgb(24, 27, 34);

    if is_thinking {
        let glow_outer =
            Color32::from_rgba_unmultiplied(accent_rgb[0], accent_rgb[1], accent_rgb[2], 65);
        let glow_mid =
            Color32::from_rgba_unmultiplied(accent_rgb[0], accent_rgb[1], accent_rgb[2], 130);
        painter.circle_filled(center, radius * 1.16, glow_outer);
        painter.circle_stroke(center, radius * 1.07, Stroke::new(2.5_f32, glow_mid));
    }

    painter.circle_filled(center, radius, dark_bg);
    painter.circle_stroke(center, radius, Stroke::new(2.2_f32, accent));

    // Emblem unik per persona
    match bot_id {
        BotId::Martin => {
            // FriendlyPawn + smile badge
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Pawn, false);
        }
        BotId::Warung => {
            // Knight + warm coffee aura
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Knight, false);
        }
        BotId::Club => {
            // Bishop emblem
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Bishop, false);
        }
        BotId::Hikaru => {
            // Lightning + Queen/Knight speedster emblem
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Knight, false);
            painter.circle_stroke(center, radius * 0.84, Stroke::new(1.2_f32, accent));
        }
        BotId::Magnus => {
            // Golden Crown / King GOAT emblem
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::King, false);
            painter.circle_stroke(center, radius * 0.84, Stroke::new(1.4_f32, accent));
        }
        BotId::Silicon => {
            // Cybernetic Rook / Queen core
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Queen, false);
            painter.circle_stroke(center, radius * 0.86, Stroke::new(1.5_f32, accent));
        }
    }
}
