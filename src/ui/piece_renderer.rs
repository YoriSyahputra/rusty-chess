use crate::{config::PieceSetStyle, engine::bot::BotId};
use egui::{Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use shakmaty::{Color, Role};

/// Menggambar bidak catur vektor prosedural dengan gaya default Chess.com Neo.
pub fn draw_chess_piece(
    painter: &Painter,
    center: Pos2,
    square_size: f32,
    color: Color,
    role: Role,
    is_dragged: bool,
) {
    draw_chess_piece_styled(
        painter,
        center,
        square_size,
        color,
        role,
        is_dragged,
        PieceSetStyle::ChessComNeo,
    );
}

/// Menggambar bidak catur vektor prosedural dalam 5 gaya pilihan:
/// `ChessComNeo` (default autentik Chess.com), `ClassicStaunton`, `MinimalistAlpha`, `Pixel8Bit`, `NeonWireframe`.
pub fn draw_chess_piece_styled(
    painter: &Painter,
    center: Pos2,
    square_size: f32,
    color: Color,
    role: Role,
    is_dragged: bool,
    style: PieceSetStyle,
) {
    let scale = if is_dragged {
        square_size * 1.12
    } else {
        square_size
    };

    if style == PieceSetStyle::Pixel8Bit {
        draw_pixel_8bit_piece(painter, center, scale, color, role, is_dragged);
        return;
    }

    let (fill_color, shade_color, inner_accent, outline_color, stroke_mult) = match (style, color) {
        (PieceSetStyle::NeonWireframe, Color::White) => (
            Color32::from_rgba_unmultiplied(0, 235, 255, 36),
            Color32::from_rgba_unmultiplied(0, 180, 220, 48),
            Color32::from_rgb(130, 250, 255),
            Color32::from_rgb(0, 235, 255),
            1.15,
        ),
        (PieceSetStyle::NeonWireframe, Color::Black) => (
            Color32::from_rgba_unmultiplied(255, 55, 185, 36),
            Color32::from_rgba_unmultiplied(190, 30, 140, 48),
            Color32::from_rgb(255, 150, 225),
            Color32::from_rgb(255, 55, 185),
            1.15,
        ),
        (PieceSetStyle::ChessComNeo, Color::White) => (
            Color32::from_rgb(0xFF, 0xFF, 0xFF),
            Color32::from_rgb(0xD8, 0xD6, 0xD1),
            Color32::from_rgb(0xC5, 0xC2, 0xBC),
            Color32::from_rgb(0x3B, 0x39, 0x36),
            0.92,
        ),
        (PieceSetStyle::ChessComNeo, Color::Black) => (
            Color32::from_rgb(0x55, 0x52, 0x4F),
            Color32::from_rgb(0x3E, 0x3C, 0x39),
            Color32::from_rgb(0x98, 0x94, 0x8F),
            Color32::from_rgb(0x23, 0x21, 0x1F),
            0.92,
        ),
        (PieceSetStyle::MinimalistAlpha, Color::White) => (
            Color32::from_rgb(0xF6, 0xF5, 0xF0),
            Color32::from_rgb(0xE6, 0xE4, 0xDD),
            Color32::TRANSPARENT,
            Color32::from_rgb(0x38, 0x36, 0x34),
            0.78,
        ),
        (PieceSetStyle::MinimalistAlpha, Color::Black) => (
            Color32::from_rgb(0x2D, 0x2E, 0x32),
            Color32::from_rgb(0x21, 0x22, 0x26),
            Color32::TRANSPARENT,
            Color32::from_rgb(0xDF, 0xDD, 0xD7),
            0.68,
        ),
        (_, Color::White) => (
            Color32::from_rgb(0xFA, 0xF7, 0xEE),
            Color32::from_rgb(0xDF, 0xD8, 0xC6),
            Color32::from_rgb(0xB8, 0xB0, 0x9E),
            Color32::from_rgb(0x2A, 0x28, 0x26),
            1.02,
        ),
        (_, Color::Black) => (
            Color32::from_rgb(0x3A, 0x3C, 0x42),
            Color32::from_rgb(0x28, 0x2A, 0x2F),
            Color32::from_rgb(0x88, 0x8D, 0x99),
            Color32::from_rgb(0x16, 0x17, 0x1A),
            1.02,
        ),
    };

    let stroke_width = (scale * 0.026 * stroke_mult).clamp(1.0, 2.35);
    let outline = Stroke::new(stroke_width, outline_color);
    let accent_stroke = if inner_accent == Color32::TRANSPARENT {
        Stroke::NONE
    } else {
        Stroke::new((stroke_width * 0.72).max(0.95), inner_accent)
    };

    // Soft feathered elliptical ground shadow (replaces stiff solid round puck)
    draw_soft_piece_shadow(painter, center, scale, is_dragged, style, outline_color);

    match role {
        Role::Pawn => draw_pawn(painter, center, scale, fill_color, shade_color, outline, accent_stroke),
        Role::Rook => draw_rook(painter, center, scale, fill_color, shade_color, outline, accent_stroke),
        Role::Knight => draw_knight(
            painter,
            center,
            scale,
            fill_color,
            shade_color,
            inner_accent,
            outline,
            accent_stroke,
        ),
        Role::Bishop => draw_bishop(
            painter,
            center,
            scale,
            fill_color,
            shade_color,
            inner_accent,
            outline,
            accent_stroke,
        ),
        Role::Queen => draw_queen(
            painter,
            center,
            scale,
            fill_color,
            shade_color,
            inner_accent,
            outline,
            accent_stroke,
        ),
        Role::King => draw_king(
            painter,
            center,
            scale,
            fill_color,
            shade_color,
            outline,
            accent_stroke,
        ),
    }
}

/// Menggambar bayangan kontak elips bertingkat yang lembut di bawah alas bidak.
fn draw_soft_piece_shadow(
    painter: &Painter,
    center: Pos2,
    scale: f32,
    is_dragged: bool,
    style: PieceSetStyle,
    neon_color: Color32,
) {
    let base_y = if is_dragged {
        center.y + scale * 0.43
    } else {
        center.y + scale * 0.375
    };
    let offset_x = if is_dragged { scale * 0.03 } else { 0.0 };

    if style == PieceSetStyle::NeonWireframe {
        let glow_rect = Rect::from_center_size(
            Pos2::new(center.x + offset_x, base_y),
            Vec2::new(scale * 0.62, scale * 0.11),
        );
        painter.rect_filled(
            glow_rect,
            Rounding::same(scale * 0.055),
            neon_color.gamma_multiply(0.22),
        );
        return;
    }

    // 3 concentric soft feathered horizontal ellipses (outermost widest & faintest)
    let layers: [(f32, f32, u8); 3] = if is_dragged {
        [
            (0.72, 0.15, 16),
            (0.56, 0.11, 24),
            (0.40, 0.075, 30),
        ]
    } else {
        [
            (0.64, 0.11, 14),
            (0.54, 0.08, 22),
            (0.42, 0.055, 28),
        ]
    };

    for (wf, hf, alpha) in layers {
        let h = scale * hf;
        let rect = Rect::from_center_size(
            Pos2::new(center.x + offset_x, base_y),
            Vec2::new(scale * wf, h),
        );
        painter.rect_filled(
            rect,
            Rounding::same(h * 0.5),
            Color32::from_rgba_unmultiplied(0, 0, 0, alpha),
        );
    }
}

/// Menggambar bidak catur bergaya Pixel 8-Bit retro (grid 10x10 prosedural).
fn draw_pixel_8bit_piece(
    painter: &Painter,
    center: Pos2,
    scale: f32,
    color: Color,
    role: Role,
    is_dragged: bool,
) {
    let sprite: [&str; 10] = match role {
        Role::Pawn => [
            "..........",
            "...####...",
            "..######..",
            "..######..",
            "...####...",
            "....##....",
            "...####...",
            "..######..",
            ".########.",
            "..........",
        ],
        Role::Rook => [
            ".##.##.##.",
            ".########.",
            "..######..",
            "...####...",
            "...####...",
            "...####...",
            "..######..",
            ".########.",
            ".########.",
            "..........",
        ],
        Role::Knight => [
            "....###...",
            "...#####..",
            "..##.####.",
            ".########.",
            ".####..##.",
            "....####..",
            "...#####..",
            "..######..",
            ".########.",
            "..........",
        ],
        Role::Bishop => [
            "....##....",
            "...####...",
            "..##..##..",
            "..######..",
            "...####...",
            "...####...",
            "..######..",
            ".########.",
            ".########.",
            "..........",
        ],
        Role::Queen => [
            ".#..##..#.",
            ".#.#..#.#.",
            ".########.",
            "..######..",
            "..######..",
            "...####...",
            "..######..",
            ".########.",
            ".########.",
            "..........",
        ],
        Role::King => [
            "....##....",
            "..######..",
            "....##....",
            ".###..###.",
            ".########.",
            "..######..",
            "..######..",
            ".########.",
            ".########.",
            "..........",
        ],
    };

    let (fill, border) = match color {
        Color::White => (Color32::from_rgb(248, 247, 240), Color32::from_rgb(28, 30, 34)),
        Color::Black => (Color32::from_rgb(42, 44, 52), Color32::from_rgb(225, 225, 220)),
    };

    let cell = (scale * 0.78) / 10.0;
    let origin = Pos2::new(center.x - cell * 5.0, center.y - cell * 5.0);
    let shadow_off = if is_dragged { 3.0 } else { 1.5 };

    for (row_idx, row_str) in sprite.iter().enumerate() {
        for (col_idx, ch) in row_str.chars().enumerate() {
            if ch == '#' {
                let px_rect = Rect::from_min_size(
                    Pos2::new(
                        origin.x + col_idx as f32 * cell,
                        origin.y + row_idx as f32 * cell,
                    ),
                    Vec2::splat(cell + 0.4),
                );
                painter.rect_filled(
                    px_rect.translate(Vec2::splat(shadow_off)),
                    Rounding::ZERO,
                    Color32::from_rgba_unmultiplied(0, 0, 0, 80),
                );
                painter.rect(
                    px_rect,
                    Rounding::ZERO,
                    fill,
                    Stroke::new(0.8_f32, border),
                );
            }
        }
    }
}

fn draw_neo_base(
    painter: &Painter,
    c: Pos2,
    s: f32,
    width_factor: f32,
    fill: Color32,
    shade: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    let w = s * width_factor;
    // Lower sleek pill base (Chess.com Neo signature flat rounded pedestal)
    let base_rect = Rect::from_center_size(
        Pos2::new(c.x, c.y + s * 0.315),
        Vec2::new(w, s * 0.115),
    );
    painter.rect(base_rect, Rounding::same(s * 0.055), fill, outline);

    // Subtle right-half 3D shading on base
    let shade_rect = Rect::from_min_max(
        Pos2::new(c.x + w * 0.06, base_rect.min.y + outline.width * 0.6),
        Pos2::new(base_rect.max.x - outline.width * 0.6, base_rect.max.y - outline.width * 0.6),
    );
    painter.rect_filled(shade_rect, Rounding::same(s * 0.04), shade);

    // Clean upper collar step
    let step_rect = Rect::from_center_size(
        Pos2::new(c.x, c.y + s * 0.245),
        Vec2::new(w * 0.78, s * 0.058),
    );
    painter.rect(step_rect, Rounding::same(s * 0.028), fill, outline);

    // Subtle top specular highlight line
    if accent != Stroke::NONE {
        painter.line_segment(
            [
                Pos2::new(c.x - w * 0.32, c.y + s * 0.315),
                Pos2::new(c.x + w * 0.04, c.y + s * 0.315),
            ],
            accent,
        );
    }
}

fn draw_pawn(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    shade: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Sleek flared Chess.com Neo pawn body
    let body = vec![
        Pos2::new(c.x - s * 0.095, c.y - s * 0.055),
        Pos2::new(c.x + s * 0.095, c.y - s * 0.055),
        Pos2::new(c.x + s * 0.145, c.y + s * 0.08),
        Pos2::new(c.x + s * 0.225, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.225, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.145, c.y + s * 0.08),
    ];
    painter.add(Shape::convex_polygon(body, fill, outline));

    // Right-side shading slice on body
    let body_shade = vec![
        Pos2::new(c.x + s * 0.015, c.y - s * 0.04),
        Pos2::new(c.x + s * 0.085, c.y - s * 0.04),
        Pos2::new(c.x + s * 0.205, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.045, c.y + s * 0.23),
    ];
    painter.add(Shape::convex_polygon(body_shade, shade, Stroke::NONE));

    // Smooth rounded collar
    let collar = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.065),
        Vec2::new(s * 0.27, s * 0.058),
    );
    painter.rect(collar, Rounding::same(s * 0.029), fill, outline);

    // Classic large friendly spherical Neo Pawn head
    let head_center = Pos2::new(c.x, c.y - s * 0.205);
    let head_r = s * 0.148;
    painter.circle(head_center, head_r, fill, outline);
    // Soft crescent shade on lower-right of sphere
    painter.circle_filled(
        Pos2::new(head_center.x + head_r * 0.24, head_center.y + head_r * 0.14),
        head_r * 0.62,
        shade,
    );
    painter.circle_filled(
        Pos2::new(head_center.x - head_r * 0.10, head_center.y - head_r * 0.08),
        head_r * 0.68,
        fill,
    );
    painter.circle_stroke(head_center, head_r, outline);

    draw_neo_base(painter, c, s, 0.58, fill, shade, outline, accent);
}

fn draw_rook(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    shade: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Sleek tapered fortress tower
    let tower = vec![
        Pos2::new(c.x - s * 0.165, c.y - s * 0.155),
        Pos2::new(c.x + s * 0.165, c.y - s * 0.155),
        Pos2::new(c.x + s * 0.22, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.22, c.y + s * 0.245),
    ];
    painter.add(Shape::convex_polygon(tower, fill, outline));

    // Right-side architectural shadow
    let tower_shade = vec![
        Pos2::new(c.x + s * 0.03, c.y - s * 0.14),
        Pos2::new(c.x + s * 0.155, c.y - s * 0.14),
        Pos2::new(c.x + s * 0.205, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.055, c.y + s * 0.23),
    ];
    painter.add(Shape::convex_polygon(tower_shade, shade, Stroke::NONE));

    // Upper turret parapet block (clean single-piece battlements)
    let turret = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.205),
        Vec2::new(s * 0.44, s * 0.11),
    );
    painter.rect(turret, Rounding::same(s * 0.025), fill, outline);

    // 3 battlements merged smoothly onto turret
    for dx in [-0.155_f32, 0.0, 0.155] {
        let merlon = Rect::from_center_size(
            Pos2::new(c.x + s * dx, c.y - s * 0.295),
            Vec2::new(s * 0.115, s * 0.10),
        );
        painter.rect(merlon, Rounding::same(s * 0.022), fill, outline);
    }
    // Seam-erasing interior fill strip across top of turret so merlons feel unified
    let unify = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.23),
        Vec2::new(s * 0.40, s * 0.055),
    );
    painter.rect_filled(unify, Rounding::same(s * 0.015), fill);

    // Clean recessed horizontal shadow collar under parapet
    if accent != Stroke::NONE {
        painter.line_segment(
            [
                Pos2::new(c.x - s * 0.155, c.y - s * 0.145),
                Pos2::new(c.x + s * 0.155, c.y - s * 0.145),
            ],
            accent,
        );
    }

    draw_neo_base(painter, c, s, 0.64, fill, shade, outline, accent);
}

fn draw_knight(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    shade: Color32,
    inner_accent: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Sleek sculpted Chess.com Neo Knight (sweeping arched mane + strong snout, facing left)
    let neck_body = vec![
        Pos2::new(c.x - s * 0.06, c.y - s * 0.30),
        Pos2::new(c.x + s * 0.09, c.y - s * 0.26),
        Pos2::new(c.x + s * 0.21, c.y - s * 0.12),
        Pos2::new(c.x + s * 0.25, c.y + s * 0.06),
        Pos2::new(c.x + s * 0.23, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.21, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.15, c.y + s * 0.06),
        Pos2::new(c.x - s * 0.05, c.y - s * 0.04),
    ];
    painter.add(Shape::convex_polygon(neck_body, fill, outline));

    // Curved right mane shading band
    let mane_shade = vec![
        Pos2::new(c.x + s * 0.07, c.y - s * 0.24),
        Pos2::new(c.x + s * 0.195, c.y - s * 0.11),
        Pos2::new(c.x + s * 0.235, c.y + s * 0.06),
        Pos2::new(c.x + s * 0.215, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.10, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.13, c.y + s * 0.02),
    ];
    painter.add(Shape::convex_polygon(mane_shade, shade, Stroke::NONE));

    // Expressive sculpted head + tapered muzzle
    let head = vec![
        Pos2::new(c.x - s * 0.03, c.y - s * 0.31),
        Pos2::new(c.x + s * 0.11, c.y - s * 0.23),
        Pos2::new(c.x + s * 0.05, c.y - s * 0.06),
        Pos2::new(c.x - s * 0.07, c.y - s * 0.06),
        Pos2::new(c.x - s * 0.24, c.y - s * 0.03),
        Pos2::new(c.x - s * 0.29, c.y - s * 0.10),
        Pos2::new(c.x - s * 0.25, c.y - s * 0.19),
    ];
    painter.add(Shape::convex_polygon(head, fill, outline));

    // Seamless blend patch covering internal seam between head and neck
    painter.circle_filled(Pos2::new(c.x + s * 0.01, c.y - s * 0.14), s * 0.10, fill);

    // Alert swept-back ear
    let ear = vec![
        Pos2::new(c.x - s * 0.06, c.y - s * 0.28),
        Pos2::new(c.x - s * 0.02, c.y - s * 0.40),
        Pos2::new(c.x + s * 0.09, c.y - s * 0.27),
    ];
    painter.add(Shape::convex_polygon(ear, fill, outline));
    // Smooth ear join
    painter.circle_filled(Pos2::new(c.x + s * 0.01, c.y - s * 0.275), s * 0.045, fill);

    // Expressive Neo eye & subtle nostril
    let detail_col = if inner_accent == Color32::TRANSPARENT {
        outline.color
    } else {
        outline.color
    };
    painter.circle_filled(
        Pos2::new(c.x - s * 0.095, c.y - s * 0.205),
        s * 0.026,
        detail_col,
    );
    painter.circle_filled(
        Pos2::new(c.x - s * 0.235, c.y - s * 0.105),
        s * 0.016,
        detail_col,
    );

    // Clean curved mane accent
    if accent != Stroke::NONE {
        painter.line_segment(
            [
                Pos2::new(c.x + s * 0.11, c.y - s * 0.18),
                Pos2::new(c.x + s * 0.165, c.y + s * 0.12),
            ],
            accent,
        );
    }

    draw_neo_base(painter, c, s, 0.64, fill, shade, outline, accent);
}

fn draw_bishop(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    shade: Color32,
    _inner_accent: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Sleek flared lower body
    let body = vec![
        Pos2::new(c.x - s * 0.12, c.y - s * 0.01),
        Pos2::new(c.x + s * 0.12, c.y - s * 0.01),
        Pos2::new(c.x + s * 0.215, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.215, c.y + s * 0.245),
    ];
    painter.add(Shape::convex_polygon(body, fill, outline));

    // Right-side body shading
    let body_shade = vec![
        Pos2::new(c.x + s * 0.02, c.y),
        Pos2::new(c.x + s * 0.11, c.y),
        Pos2::new(c.x + s * 0.20, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.05, c.y + s * 0.23),
    ];
    painter.add(Shape::convex_polygon(body_shade, shade, Stroke::NONE));

    // Smooth teardrop Bishop mitre (8-point rounded silhouette)
    let mitre = vec![
        Pos2::new(c.x, c.y - s * 0.345),
        Pos2::new(c.x + s * 0.11, c.y - s * 0.27),
        Pos2::new(c.x + s * 0.185, c.y - s * 0.15),
        Pos2::new(c.x + s * 0.16, c.y - s * 0.03),
        Pos2::new(c.x - s * 0.16, c.y - s * 0.03),
        Pos2::new(c.x - s * 0.185, c.y - s * 0.15),
        Pos2::new(c.x - s * 0.11, c.y - s * 0.27),
    ];
    painter.add(Shape::convex_polygon(mitre, fill, outline));

    // Mitre right-side subtle shade
    let mitre_shade = vec![
        Pos2::new(c.x + s * 0.02, c.y - s * 0.31),
        Pos2::new(c.x + s * 0.10, c.y - s * 0.255),
        Pos2::new(c.x + s * 0.168, c.y - s * 0.145),
        Pos2::new(c.x + s * 0.145, c.y - s * 0.045),
        Pos2::new(c.x + s * 0.03, c.y - s * 0.045),
    ];
    painter.add(Shape::convex_polygon(mitre_shade, shade, Stroke::NONE));

    // Collar pill
    let collar = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.02),
        Vec2::new(s * 0.32, s * 0.058),
    );
    painter.rect(collar, Rounding::same(s * 0.029), fill, outline);

    // Top finial pearl
    painter.circle(Pos2::new(c.x, c.y - s * 0.375), s * 0.045, fill, outline);

    // Iconic Chess.com diagonal mitre slash + subtle cross
    let slash_stroke = Stroke::new((outline.width * 1.15).max(1.5), outline.color);
    painter.line_segment(
        [
            Pos2::new(c.x + s * 0.015, c.y - s * 0.16),
            Pos2::new(c.x + s * 0.14, c.y - s * 0.245),
        ],
        slash_stroke,
    );
    if accent != Stroke::NONE {
        painter.line_segment(
            [
                Pos2::new(c.x - s * 0.055, c.y - s * 0.15),
                Pos2::new(c.x + s * 0.025, c.y - s * 0.15),
            ],
            accent,
        );
    }

    draw_neo_base(painter, c, s, 0.64, fill, shade, outline, accent);
}

fn draw_queen(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    shade: Color32,
    inner_accent: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Graceful royal gown base
    let gown = vec![
        Pos2::new(c.x - s * 0.19, c.y + s * 0.02),
        Pos2::new(c.x + s * 0.19, c.y + s * 0.02),
        Pos2::new(c.x + s * 0.25, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.25, c.y + s * 0.245),
    ];
    painter.add(Shape::convex_polygon(gown, fill, outline));

    let gown_shade = vec![
        Pos2::new(c.x + s * 0.03, c.y + s * 0.035),
        Pos2::new(c.x + s * 0.18, c.y + s * 0.035),
        Pos2::new(c.x + s * 0.235, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.06, c.y + s * 0.23),
    ];
    painter.add(Shape::convex_polygon(gown_shade, shade, Stroke::NONE));

    // Five sleek royal coronet spires
    let spike_tips = [
        (-0.28_f32, -0.245_f32),
        (-0.145, -0.32),
        (0.0, -0.35),
        (0.145, -0.32),
        (0.28, -0.245),
    ];
    let spike_bases = [-0.21_f32, -0.105, 0.0, 0.105, 0.21];

    for (i, &(tx, ty)) in spike_tips.iter().enumerate() {
        let bx = spike_bases[i];
        let tri = vec![
            Pos2::new(c.x + s * (bx - 0.07), c.y + s * 0.04),
            Pos2::new(c.x + s * tx, c.y + s * ty),
            Pos2::new(c.x + s * (bx + 0.07), c.y + s * 0.04),
        ];
        painter.add(Shape::convex_polygon(tri, fill, outline));
        painter.circle(
            Pos2::new(c.x + s * tx, c.y + s * (ty - 0.026)),
            s * 0.036,
            fill,
            outline,
        );
    }

    // Unified smooth crown waist band (covers spike base seams cleanly!)
    let band = Rect::from_center_size(
        Pos2::new(c.x, c.y + s * 0.04),
        Vec2::new(s * 0.43, s * 0.072),
    );
    painter.rect(band, Rounding::same(s * 0.036), fill, outline);

    if inner_accent != Color32::TRANSPARENT {
        for jx in [-0.115_f32, 0.0, 0.115] {
            painter.circle_filled(
                Pos2::new(c.x + s * jx, c.y + s * 0.04),
                s * 0.019,
                inner_accent,
            );
        }
    }

    draw_neo_base(painter, c, s, 0.68, fill, shade, outline, accent);
}

fn draw_king(
    painter: &Painter,
    c: Pos2,
    s: f32,
    fill: Color32,
    shade: Color32,
    outline: Stroke,
    accent: Stroke,
) {
    // Bold clean royal cross atop crown
    let cross_v = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.345),
        Vec2::new(s * 0.068, s * 0.17),
    );
    let cross_h = Rect::from_center_size(
        Pos2::new(c.x, c.y - s * 0.355),
        Vec2::new(s * 0.165, s * 0.062),
    );
    painter.rect(cross_v, Rounding::same(s * 0.022), fill, outline);
    painter.rect(cross_h, Rounding::same(s * 0.022), fill, outline);
    // Clean inner cross fill so bars join seamlessly
    painter.rect_filled(
        Rect::from_center_size(Pos2::new(c.x, c.y - s * 0.355), Vec2::new(s * 0.055, s * 0.055)),
        Rounding::ZERO,
        fill,
    );

    // Classic Chess.com Neo dual-lobe rounded imperial crown
    let left_lobe = Pos2::new(c.x - s * 0.13, c.y - s * 0.10);
    let right_lobe = Pos2::new(c.x + s * 0.13, c.y - s * 0.10);
    let lobe_r = s * 0.145;
    painter.circle(left_lobe, lobe_r, fill, outline);
    painter.circle(right_lobe, lobe_r, fill, outline);

    // Central crown arch rising to cross
    let center_arch = vec![
        Pos2::new(c.x, c.y - s * 0.275),
        Pos2::new(c.x + s * 0.13, c.y - s * 0.17),
        Pos2::new(c.x + s * 0.20, c.y + s * 0.03),
        Pos2::new(c.x - s * 0.20, c.y + s * 0.03),
        Pos2::new(c.x - s * 0.13, c.y - s * 0.17),
    ];
    painter.add(Shape::convex_polygon(center_arch, fill, outline));

    // Right-lobe gentle shade
    painter.circle_filled(
        Pos2::new(right_lobe.x + s * 0.025, right_lobe.y + s * 0.015),
        lobe_r * 0.72,
        shade,
    );

    // Lower robe pedestal
    let robe = vec![
        Pos2::new(c.x - s * 0.19, c.y + s * 0.03),
        Pos2::new(c.x + s * 0.19, c.y + s * 0.03),
        Pos2::new(c.x + s * 0.255, c.y + s * 0.245),
        Pos2::new(c.x - s * 0.255, c.y + s * 0.245),
    ];
    painter.add(Shape::convex_polygon(robe, fill, outline));

    let robe_shade = vec![
        Pos2::new(c.x + s * 0.03, c.y + s * 0.045),
        Pos2::new(c.x + s * 0.18, c.y + s * 0.045),
        Pos2::new(c.x + s * 0.24, c.y + s * 0.23),
        Pos2::new(c.x + s * 0.06, c.y + s * 0.23),
    ];
    painter.add(Shape::convex_polygon(robe_shade, shade, Stroke::NONE));

    // Sleek royal collar band
    let band = Rect::from_center_size(
        Pos2::new(c.x, c.y + s * 0.035),
        Vec2::new(s * 0.44, s * 0.072),
    );
    painter.rect(band, Rounding::same(s * 0.036), fill, outline);

    // Subtle vertical crown crease
    if accent != Stroke::NONE {
        painter.line_segment(
            [
                Pos2::new(c.x, c.y - s * 0.22),
                Pos2::new(c.x, c.y - s * 0.015),
            ],
            accent,
        );
    }

    draw_neo_base(painter, c, s, 0.70, fill, shade, outline, accent);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BotMood {
    Neutral,
    SmugWinning,
    SweatingLosing,
    MittensLaserMate,
}

impl BotMood {
    pub fn from_eval(bot_id: BotId, bot_eval_cp: i32) -> Self {
        if bot_id == BotId::Mittens && bot_eval_cp > 28_000 {
            BotMood::MittensLaserMate
        } else if bot_eval_cp > 300 {
            BotMood::SmugWinning
        } else if bot_eval_cp < -300 {
            BotMood::SweatingLosing
        } else {
            BotMood::Neutral
        }
    }
}

/// Menggambar avatar vektor unik untuk tiap persona bot lengkap dengan border glow saat berpikir.
pub fn draw_bot_avatar(
    painter: &Painter,
    rect: Rect,
    bot_id: BotId,
    accent_rgb: [u8; 3],
    is_thinking: bool,
) {
    draw_bot_avatar_with_mood(painter, rect, bot_id, accent_rgb, is_thinking, BotMood::Neutral);
}

/// Menggambar avatar bot dengan mikro-ekspresi reaktif terhadap evaluasi papan (Feature #19).
pub fn draw_bot_avatar_with_mood(
    painter: &Painter,
    rect: Rect,
    bot_id: BotId,
    accent_rgb: [u8; 3],
    is_thinking: bool,
    mood: BotMood,
) {
    let center = rect.center();
    let radius = rect.width().min(rect.height()) * 0.46;
    let accent = match mood {
        BotMood::SmugWinning => Color32::from_rgb(0xFF, 0xAA, 0x00),
        BotMood::MittensLaserMate => Color32::from_rgb(255, 35, 55),
        _ => Color32::from_rgb(accent_rgb[0], accent_rgb[1], accent_rgb[2]),
    };
    let dark_bg = Color32::from_rgb(24, 27, 34);

    if is_thinking || mood != BotMood::Neutral {
        let glow_outer = accent.gamma_multiply(0.28);
        let glow_mid = accent.gamma_multiply(0.55);
        painter.circle_filled(center, radius * 1.16, glow_outer);
        painter.circle_stroke(center, radius * 1.07, Stroke::new(2.5_f32, glow_mid));
    }

    painter.circle_filled(center, radius, dark_bg);
    painter.circle_stroke(center, radius, Stroke::new(2.2_f32, accent));

    // Emblem unik per persona
    match bot_id {
        BotId::Martin | BotId::Elani | BotId::Wayne | BotId::Jimmy | BotId::Aron | BotId::Maria => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Pawn, false);
        }
        BotId::Warung | BotId::Emir | BotId::Sven | BotId::Wendy | BotId::AdaptiveJimmy => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Knight, false);
        }
        BotId::Nelson | BotId::Beth => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Queen, false);
        }
        BotId::Club | BotId::Jade | BotId::Antonio | BotId::Isabel | BotId::David | BotId::Wally => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Bishop, false);
        }
        BotId::Li | BotId::Noam | BotId::Francis | BotId::Danny => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Rook, false);
            painter.circle_stroke(center, radius * 0.84, Stroke::new(1.2_f32, accent));
        }
        BotId::Hikaru => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Knight, false);
            painter.circle_stroke(center, radius * 0.84, Stroke::new(1.2_f32, accent));
        }
        BotId::Magnus => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::King, false);
            painter.circle_stroke(center, radius * 0.84, Stroke::new(1.4_f32, accent));
        }
        BotId::Mittens | BotId::Silicon | BotId::EngineCustom => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Queen, false);
            painter.circle_stroke(center, radius * 0.86, Stroke::new(1.5_f32, accent));
        }
    }

    // Reactive mood micro-expressions overlay (Feature #19)
    match mood {
        BotMood::SmugWinning => {
            // Golden sparkles at top-left and top-right
            let sp1 = Pos2::new(center.x + radius * 0.65, center.y - radius * 0.65);
            let sp2 = Pos2::new(center.x - radius * 0.68, center.y - radius * 0.52);
            painter.circle_filled(sp1, 3.5, Color32::from_rgb(0xFF, 0xD7, 0x40));
            painter.circle_filled(sp2, 2.5, Color32::from_rgb(0xFF, 0xAA, 0x00));
        }
        BotMood::SweatingLosing => {
            // Comic cyan sweat droplets
            let d1 = Pos2::new(center.x + radius * 0.68, center.y - radius * 0.45);
            let d2 = Pos2::new(center.x + radius * 0.82, center.y - radius * 0.12);
            painter.circle_filled(d1, 3.6, Color32::from_rgb(0x00, 0xB8, 0xFF));
            painter.circle_filled(d2, 2.6, Color32::from_rgb(0x40, 0xD0, 0xFF));
        }
        BotMood::MittensLaserMate => {
            // Eerie glowing crimson eyes when Mittens spots forced mate
            let e_left = Pos2::new(center.x - radius * 0.24, center.y - radius * 0.14);
            let e_right = Pos2::new(center.x + radius * 0.24, center.y - radius * 0.14);
            painter.circle_filled(e_left, 4.2, Color32::from_rgba_unmultiplied(255, 25, 45, 190));
            painter.circle_filled(e_right, 4.2, Color32::from_rgba_unmultiplied(255, 25, 45, 190));
            painter.circle_filled(e_left, 2.0, Color32::WHITE);
            painter.circle_filled(e_right, 2.0, Color32::WHITE);
        }
        BotMood::Neutral => {}
    }
}
