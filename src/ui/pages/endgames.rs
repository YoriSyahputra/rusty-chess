use crate::{
    domain::parse_fen,
    ui::{board_view::render_board_grid, theme::{BoardTheme, UiTokens}},
};
use egui::{Color32, RichText, Rounding, ScrollArea, Sense, Stroke, Ui, Vec2};

#[derive(Debug, Clone, Copy)]
pub struct EndgameDrill {
    pub id: usize,
    pub title: &'static str,
    pub category: &'static str,
    pub goal: &'static str,
    pub difficulty: &'static str,
    pub fen: &'static str,
}

pub const ENDGAME_DRILLS: &[EndgameDrill] = &[
    EndgameDrill {
        id: 1,
        title: "Lucena Position (Building the Bridge)",
        category: "Rook Endgame",
        goal: "Win as White by shielding your King from checks and promoting the d7 pawn.",
        difficulty: "Master Technique",
        fen: "1K1k4/1P6/8/8/8/8/r7/2R5 w - - 0 1",
    },
    EndgameDrill {
        id: 2,
        title: "Philidor Defense (3rd Rank Shield)",
        category: "Rook Endgame",
        goal: "Hold the draw or outplay the opponent using active Rook checks from behind.",
        difficulty: "Advanced",
        fen: "4k3/8/4P3/4K3/8/r7/8/5R2 w - - 0 1",
    },
    EndgameDrill {
        id: 3,
        title: "King & Pawn Opposition",
        category: "Pawn Endgame",
        goal: "Seize key squares in front of your pawn to force promotion.",
        difficulty: "Fundamental",
        fen: "4k3/8/8/4K3/4P3/8/8/8 w - - 0 1",
    },
    EndgameDrill {
        id: 4,
        title: "Queen vs. 7th-Rank Pawn",
        category: "Queen Endgame",
        goal: "Force the enemy King in front of their own pawn and bring your King closer.",
        difficulty: "Advanced",
        fen: "8/8/8/8/4Q3/6K1/2p5/1k6 w - - 0 1",
    },
    EndgameDrill {
        id: 5,
        title: "King & Queen Basic Checkmate",
        category: "Checkmate Pattern",
        goal: "Box the lone Black King onto the edge without allowing stalemate.",
        difficulty: "Beginner",
        fen: "8/8/8/4k3/8/8/3Q4/4K3 w - - 0 1",
    },
    EndgameDrill {
        id: 6,
        title: "King & Rook Box Checkmate",
        category: "Checkmate Pattern",
        goal: "Cut off the Black King rank by rank and deliver checkmate on the edge.",
        difficulty: "Beginner",
        fen: "8/8/8/3k4/8/8/8/R3K3 w - - 0 1",
    },
    EndgameDrill {
        id: 7,
        title: "Two Bishops Corner Checkmate",
        category: "Minor Piece Mate",
        goal: "Coordinate the Bishop pair to shepherd the lone King into a corner.",
        difficulty: "Intermediate",
        fen: "8/8/8/4k3/8/8/8/2B1KB2 w - - 0 1",
    },
    EndgameDrill {
        id: 8,
        title: "Bishop & Knight Grandmaster Mate",
        category: "Minor Piece Mate",
        goal: "Drive the King to the corner matching your Bishop's color (light square).",
        difficulty: "Grandmaster",
        fen: "8/8/8/4k3/8/8/8/4KBN1 w - - 0 1",
    },
];

#[derive(Debug, Default)]
pub struct EndgamePageActions {
    pub launch_drill_fen: Option<String>,
}

/// Merender halaman Interactive Endgame Mastery Trainer (Feature #21).
pub fn render_endgames_page(
    ui: &mut Ui,
    selected_idx: &mut usize,
    board_theme: BoardTheme,
    dark_mode: bool,
) -> EndgamePageActions {
    let mut actions = EndgamePageActions::default();
    let tokens = UiTokens::for_mode(dark_mode);
    let idx = (*selected_idx).min(ENDGAME_DRILLS.len() - 1);
    let drill = &ENDGAME_DRILLS[idx];

    egui::SidePanel::right("endgames_right_panel")
        .exact_width(380.0)
        .resizable(false)
        .show_inside(ui, |ui| {
            ui.add_space(10.0);
            ui.label(
                RichText::new("♔ Endgame Mastery Drills")
                    .strong()
                    .size(18.0)
                    .color(tokens.text_primary),
            );
            ui.label(
                RichText::new("Prove your winning technique against GM Magnus defending at full strength.")
                    .size(12.0)
                    .color(tokens.text_secondary),
            );
            ui.add_space(8.0);

            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.5_f32, tokens.accent_gold))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(12.0))
                .show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.label(
                        RichText::new(drill.title)
                            .strong()
                            .size(15.0)
                            .color(tokens.text_primary),
                    );
                    ui.label(
                        RichText::new(format!("{}  •  {}", drill.category, drill.difficulty))
                            .size(11.5)
                            .color(tokens.accent_gold),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(drill.goal)
                            .size(12.0)
                            .color(tokens.text_secondary),
                    );
                    ui.add_space(10.0);
                    if ui
                        .add_sized(
                            [ui.available_width(), 38.0],
                            egui::Button::new(
                                RichText::new("▶ Practice Against GM Magnus")
                                    .strong()
                                    .size(14.0)
                                    .color(Color32::WHITE),
                            )
                            .fill(tokens.accent_primary),
                        )
                        .clicked()
                    {
                        actions.launch_drill_fen = Some(drill.fen.to_string());
                    }
                });

            ui.add_space(10.0);

            let list_h = (ui.available_height() - 14.0).max(140.0);
            egui::Frame::none()
                .fill(tokens.bg_surface)
                .stroke(Stroke::new(1.0_f32, tokens.border_subtle))
                .rounding(Rounding::same(8.0_f32))
                .inner_margin(egui::Margin::same(6.0))
                .show(ui, |ui| {
                    ScrollArea::vertical()
                        .id_salt("endgame_drills_scroll")
                        .max_height(list_h)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            for (i, d) in ENDGAME_DRILLS.iter().enumerate() {
                                let is_sel = i == idx;
                                let label = format!("#{} {} [{}]", d.id, d.title, d.difficulty);
                                if ui
                                    .add_sized(
                                        [ui.available_width(), 28.0],
                                        egui::SelectableLabel::new(
                                            is_sel,
                                            RichText::new(label).size(12.0),
                                        ),
                                    )
                                    .clicked()
                                {
                                    *selected_idx = i;
                                }
                            }
                        });
                });
        });

    egui::CentralPanel::default().show_inside(ui, |ui| {
        let avail = ui.available_size();
        let board_size = (avail.x - 24.0).min(avail.y - 48.0).max(320.0);
        let pad_x = ((avail.x - board_size) * 0.5).max(8.0);
        let pad_y = ((avail.y - board_size) * 0.5).max(8.0);

        ui.add_space(pad_y);
        ui.horizontal(|ui| {
            ui.add_space(pad_x);
            let (b_rect, _) =
                ui.allocate_exact_size(Vec2::splat(board_size), Sense::hover());
            if let Ok(pos) = parse_fen(drill.fen) {
                render_board_grid(
                    ui.painter(),
                    b_rect,
                    &pos,
                    &board_theme.palette(),
                    false,
                    true,
                    None,
                    None,
                    &[],
                    &[],
                    &[],
                    None,
                    None,
                    None,
                );
            }
        });
    });

    actions
}
