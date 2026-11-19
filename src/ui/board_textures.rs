use crate::ui::theme::BoardTheme;
use egui::{Color32, ColorImage, Context, Painter, Pos2, Rect, TextureHandle, TextureOptions};
use std::collections::HashMap;

/// Cache tekstur permukaan papan prosedural (dibuat sekali per tema di RAM ~256 KB,
/// 0% beban CPU saat idle dan 100% bebas file aset eksternal).
#[derive(Default)]
pub struct BoardTextureCache {
    textures: HashMap<u8, TextureHandle>,
}

impl BoardTextureCache {
    fn theme_key(theme: BoardTheme) -> u8 {
        match theme {
            BoardTheme::ClassicGreen => 0,
            BoardTheme::WalnutWood => 1,
            BoardTheme::Emerald => 2,
            BoardTheme::TournamentBlue => 3,
            BoardTheme::IcySea => 4,
            BoardTheme::Newspaper => 5,
            BoardTheme::DarkSlate => 6,
        }
    }

    pub fn get_or_create(&mut self, ctx: &Context, theme: BoardTheme) -> &TextureHandle {
        let key = Self::theme_key(theme);
        self.textures.entry(key).or_insert_with(|| {
            let img = generate_procedural_board_overlay(theme, 256);
            ctx.load_texture(
                format!("board_tex_{key}"),
                img,
                TextureOptions::LINEAR_REPEAT,
            )
        })
    }

    pub fn render_overlay(&mut self, ctx: &Context, painter: &Painter, board_rect: Rect, theme: BoardTheme) {
        let tex = self.get_or_create(ctx, theme);
        let uv = Rect::from_min_max(Pos2::new(0.0, 0.0), Pos2::new(2.0, 2.0));
        painter.image(tex.id(), board_rect, uv, Color32::WHITE);
    }
}

/// Menghasilkan overlay tekstur permukaan 256x256 RGBA secara matematis untuk tiap tema papan.
pub fn generate_procedural_board_overlay(theme: BoardTheme, size: usize) -> ColorImage {
    let mut pixels = vec![Color32::TRANSPARENT; size * size];

    for y in 0..size {
        for x in 0..size {
            let fx = x as f32;
            let fy = y as f32;

            // Deterministic fast pseudo-noise hash
            let h = ((x.wrapping_mul(374_761_393)) ^ (y.wrapping_mul(668_265_263)))
                .wrapping_mul(1_274_126_177);
            let noise = (((h >> 24) & 0xFF) as f32 / 255.0) - 0.5;

            let color = match theme {
                BoardTheme::WalnutWood => {
                    // Warm horizontal oak wood grain fibers
                    let wave = ((fy * 0.24) + (fx * 0.03).sin() * 2.4 + noise * 0.9).sin();
                    if wave > 0.0 {
                        Color32::from_rgba_unmultiplied(60, 32, 12, (wave * 24.0) as u8)
                    } else {
                        Color32::from_rgba_unmultiplied(255, 232, 195, ((-wave) * 16.0) as u8)
                    }
                }
                BoardTheme::ClassicGreen | BoardTheme::Emerald | BoardTheme::Newspaper => {
                    // Subtle matte tournament surface micro-grain
                    let alpha = (2.0 + noise.abs() * 5.0).clamp(0.0, 8.0) as u8;
                    if noise > 0.0 {
                        Color32::from_rgba_unmultiplied(255, 255, 245, alpha)
                    } else {
                        Color32::from_rgba_unmultiplied(15, 20, 12, alpha)
                    }
                }
                BoardTheme::TournamentBlue => {
                    // Smooth satin tournament vinyl sheen
                    let diag = ((fx + fy) * 0.045).sin() * 0.5 + 0.5;
                    let alpha = (diag * 14.0 + noise.abs() * 7.0) as u8;
                    Color32::from_rgba_unmultiplied(255, 255, 255, alpha)
                }
                BoardTheme::IcySea => {
                    // Crystalline diagonal frost veins
                    let crystal = ((fx * 0.07 - fy * 0.05).sin() * (fx * 0.03 + fy * 0.09).cos()).abs();
                    let alpha = (crystal * 26.0 + noise.abs() * 8.0) as u8;
                    Color32::from_rgba_unmultiplied(225, 245, 255, alpha)
                }
                BoardTheme::DarkSlate => {
                    // Fine brushed slate stone texture
                    let streak = ((fy * 0.45 + noise * 1.6).sin()).abs();
                    let alpha = (streak * 18.0 + noise.abs() * 14.0) as u8;
                    if noise > 0.0 {
                        Color32::from_rgba_unmultiplied(210, 215, 225, alpha / 2)
                    } else {
                        Color32::from_rgba_unmultiplied(8, 10, 14, alpha)
                    }
                }
            };

            pixels[y * size + x] = color;
        }
    }

    ColorImage {
        size: [size, size],
        pixels,
    }
}
