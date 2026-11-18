use crate::engine::search::{MATE_SCORE, MATE_THRESHOLD};
use egui::{Align2, Color32, FontId, Pos2, Rect, Rounding, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone)]
pub struct EvalBarState {
    /// Nilai evaluasi saat ini dalam centipawn (perspektif Putih: + = Putih unggul)
    pub target_eval_cp: i32,
    /// Proporsi tinggi bar Putih (0.0..=1.0) yang di-lerp secara mulus tiap frame
    pub displayed_white_ratio: f32,
}

impl Default for EvalBarState {
    fn default() -> Self {
        Self {
            target_eval_cp: 0,
            displayed_white_ratio: 0.5,
        }
    }
}

impl EvalBarState {
    pub fn set_eval_cp(&mut self, eval_cp_white: i32) {
        self.target_eval_cp = eval_cp_white;
    }

    pub fn is_lerping(&self) -> bool {
        (self.target_ratio() - self.displayed_white_ratio).abs() > 0.005
    }

    /// Menghitung proporsi keunggulan Putih menggunakan kurva sigmoid:
    /// `1.0 / (1.0 + (-eval_cp as f32 / 400.0).exp())`
    pub fn target_ratio(&self) -> f32 {
        if self.target_eval_cp > MATE_THRESHOLD {
            return 0.99;
        }
        if self.target_eval_cp < -MATE_THRESHOLD {
            return 0.01;
        }
        let x = (-self.target_eval_cp as f32 / 400.0).clamp(-15.0, 15.0);
        let ratio = 1.0 / (1.0 + x.exp());
        ratio.clamp(0.03, 0.97)
    }

    pub fn formatted_score(&self) -> String {
        let cp = self.target_eval_cp;
        if cp.abs() > MATE_THRESHOLD {
            let plies = (MATE_SCORE - cp.abs()).max(1);
            let moves_to_mate = (plies + 1) / 2;
            if cp > 0 {
                format!("M{moves_to_mate}")
            } else {
                format!("-M{moves_to_mate}")
            }
        } else {
            let pawns = cp as f32 / 100.0;
            if pawns > 0.05 {
                format!("+{pawns:.1}")
            } else if pawns < -0.05 {
                format!("{pawns:.1}")
            } else {
                "0.0".to_string()
            }
        }
    }

    /// Merender Vertical Evaluation Bar selebar `28px` dan tinggi mengikuti papan catur.
    pub fn ui(&mut self, ui: &mut Ui, height: f32, board_flipped: bool) {
        let bar_width = 28.0;
        let (rect, _resp) =
            ui.allocate_exact_size(Vec2::new(bar_width, height), Sense::hover());

        let target = self.target_ratio();
        let diff = target - self.displayed_white_ratio;
        if diff.abs() > 0.005 {
            self.displayed_white_ratio += diff * 0.18;
            ui.ctx().request_repaint();
        } else {
            self.displayed_white_ratio = target;
        }

        let painter = ui.painter_at(rect);
        let white_fill = Color32::from_rgb(236, 238, 242);
        let black_fill = Color32::from_rgb(38, 41, 48);
        let rounding = Rounding::same(4.0_f32);

        let top_is_black = !board_flipped;
        let top_color = if top_is_black { black_fill } else { white_fill };
        let bottom_color = if top_is_black { white_fill } else { black_fill };

        painter.rect_filled(rect, rounding, top_color);

        let bottom_ratio = if board_flipped {
            1.0 - self.displayed_white_ratio
        } else {
            self.displayed_white_ratio
        };
        let bottom_h = (rect.height() * bottom_ratio).clamp(0.0, rect.height());
        let bottom_rect = Rect::from_min_max(
            Pos2::new(rect.min.x, rect.max.y - bottom_h),
            rect.max,
        );
        painter.rect_filled(bottom_rect, rounding, bottom_color);

        // Garis tengah penanda 0.0 (50%)
        let mid_y = rect.center().y;
        painter.line_segment(
            [
                Pos2::new(rect.min.x + 2.0, mid_y),
                Pos2::new(rect.max.x - 2.0, mid_y),
            ],
            Stroke::new(1.2_f32, Color32::from_rgba_unmultiplied(120, 130, 145, 160)),
        );

        painter.rect_stroke(
            rect,
            rounding,
            Stroke::new(1.0_f32, Color32::from_rgb(65, 70, 82)),
        );

        let label = self.formatted_score();
        let white_advantage = self.target_eval_cp >= 0;
        let label_at_bottom =
            (white_advantage && !board_flipped) || (!white_advantage && board_flipped);

        let (text_pos, text_align, text_color) = if label_at_bottom {
            let fg = if bottom_color == white_fill {
                Color32::from_rgb(28, 30, 36)
            } else {
                Color32::from_rgb(232, 235, 242)
            };
            (
                Pos2::new(rect.center().x, rect.max.y - 6.0),
                Align2::CENTER_BOTTOM,
                fg,
            )
        } else {
            let fg = if top_color == white_fill {
                Color32::from_rgb(28, 30, 36)
            } else {
                Color32::from_rgb(232, 235, 242)
            };
            (
                Pos2::new(rect.center().x, rect.min.y + 6.0),
                Align2::CENTER_TOP,
                fg,
            )
        };

        painter.text(
            text_pos,
            text_align,
            label,
            FontId::proportional(10.5),
            text_color,
        );
    }
}
