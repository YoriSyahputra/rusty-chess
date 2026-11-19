use crate::{config::PieceSetStyle, engine::bot::BotId};
use egui::{Color32, Painter, Pos2, Rect, Rounding, Shape, Stroke, Vec2};
use shakmaty::{Color, Role};

/// Menggambar bidak catur vektor prosedural dengan gaya klasik default (`ClassicStaunton`).
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
        PieceSetStyle::ClassicStaunton,
    );
}

/// Menggambar bidak catur vektor prosedural dalam 5 gaya pilihan (Feature #6):
/// `ClassicStaunton`, `ChessComNeo`, `MinimalistAlpha`, `Pixel8Bit`, `NeonWireframe`.
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
        square_size * 1.15
    } else {
        square_size
    };

    if style == PieceSetStyle::Pixel8Bit {
        draw_pixel_8bit_piece(painter, center, scale, color, role, is_dragged);
        return;
    }

    let (fill_color, inner_accent, outline_color, stroke_mult) = match (style, color) {
        (PieceSetStyle::NeonWireframe, Color::White) => (
            Color32::from_rgba_unmultiplied(0, 235, 255, 36),
            Color32::from_rgb(120, 250, 255),
            Color32::from_rgb(0, 235, 255),
            1.15,
        ),
        (PieceSetStyle::NeonWireframe, Color::Black) => (
            Color32::from_rgba_unmultiplied(255, 55, 185, 36),
            Color32::from_rgb(255, 145, 225),
            Color32::from_rgb(255, 55, 185),
            1.15,
        ),
        (PieceSetStyle::ChessComNeo, Color::White) => (
            Color32::from_rgb(255, 255, 255),
            Color32::from_rgb(232, 232, 228),
            Color32::from_rgb(45, 44, 42),
            1.20,
        ),
        (PieceSetStyle::ChessComNeo, Color::Black) => (
            Color32::from_rgb(56, 54, 51),
            Color32::from_rgb(108, 105, 100),
            Color32::from_rgb(22, 21, 20),
            1.20,
        ),
        (PieceSetStyle::MinimalistAlpha, Color::White) => (
            Color32::from_rgb(244, 242, 236),
            Color32::TRANSPARENT,
            Color32::from_rgb(52, 50, 48),
            0.85,
        ),
        (PieceSetStyle::MinimalistAlpha, Color::Black) => (
            Color32::from_rgb(34, 36, 40),
            Color32::TRANSPARENT,
            Color32::from_rgb(218, 216, 210),
            0.75,
        ),
        (_, Color::White) => (
            Color32::from_rgb(248, 247, 242),
            Color32::from_rgb(218, 214, 202),
            Color32::from_rgb(32, 34, 38),
            1.0,
        ),
        (_, Color::Black) => (
            Color32::from_rgb(40, 43, 50),
            Color32::from_rgb(95, 102, 118),
            Color32::from_rgb(14, 15, 18),
            1.0,
        ),
    };

    let stroke_width = (scale * 0.032 * stroke_mult).clamp(1.3, 3.2);
    let outline = Stroke::new(stroke_width, outline_color);
    let accent_stroke = if inner_accent == Color32::TRANSPARENT {
        Stroke::NONE
    } else {
        Stroke::new((stroke_width * 0.85).max(1.1), inner_accent)
    };

    // Drop shadow (neon glow halo for NeonWireframe)
    let shadow_offset = if is_dragged {
        Vec2::new(scale * 0.04, scale * 0.07)
    } else {
        Vec2::new(scale * 0.018, scale * 0.032)
    };
    let shadow_alpha = if is_dragged { 95 } else { 65 };
    let shadow_color = if style == PieceSetStyle::NeonWireframe {
        outline_color.gamma_multiply(0.28)
    } else {
        Color32::from_rgba_unmultiplied(0, 0, 0, shadow_alpha)
    };

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
        BotId::Martin | BotId::Elani | BotId::Jimmy | BotId::Aron => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Pawn, false);
        }
        BotId::Warung | BotId::Emir | BotId::Sven | BotId::AdaptiveJimmy => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Knight, false);
        }
        BotId::Nelson => {
            draw_chess_piece(painter, center, radius * 1.25, Color::White, Role::Queen, false);
        }
        BotId::Club | BotId::Antonio | BotId::Isabel | BotId::Wally => {
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
