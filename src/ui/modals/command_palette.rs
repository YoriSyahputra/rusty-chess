use crate::{
    config::PieceSetStyle,
    domain::{parse_san, ChessMatchState, Move},
    engine::bot::BotId,
    ui::{sidebar::ActiveRoute, theme::{BoardTheme, UiTokens}},
};
use egui::{Align2, Color32, Context, Key, RichText, Rounding, ScrollArea, Stroke, Vec2, Window};

#[derive(Debug, Clone)]
pub enum PaletteCommand {
    Navigate(ActiveRoute),
    ChallengeBot(BotId),
    SetBoardTheme(BoardTheme),
    SetPieceSet(PieceSetStyle),
    ToggleXRay,
    ToggleZenMode,
    ToggleBlindfold,
    StartChess960,
    ToggleDarkLight,
    FlipBoard,
    OpenFenPgn,
    OpenSettings,
    OpenBlunderDeck,
    PlaySanMove(Move, String),
}

struct PaletteItem {
    label: String,
    category: &'static str,
    cmd: PaletteCommand,
}

#[derive(Debug, Default)]
pub struct CommandPaletteState {
    pub open: bool,
    pub query: String,
    pub selected_idx: usize,
    pub just_opened: bool,
}

impl CommandPaletteState {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.query.clear();
            self.selected_idx = 0;
            self.just_opened = true;
        }
    }
}

/// Menampilkan Spotlight-style Command Palette (`Ctrl+K`) dengan fuzzy search + input langkah SAN langsung!
pub fn show_command_palette(
    ctx: &Context,
    state: &mut CommandPaletteState,
    match_state: &ChessMatchState,
    in_live_match: bool,
    dark_mode: bool,
) -> Option<PaletteCommand> {
    if !state.open {
        return None;
    }

    let tokens = UiTokens::for_mode(dark_mode);
    let mut executed: Option<PaletteCommand> = None;

    // Close on Escape
    if ctx.input(|i| i.key_pressed(Key::Escape)) {
        state.open = false;
        return None;
    }

    let mut items = build_palette_items();

    // Jika sedang di Live Match dan teks yang diketik adalah langkah SAN legal (misal "Nf3", "e4", "O-O")
    let trimmed = state.query.trim();
    if in_live_match && !trimmed.is_empty() && !match_state.outcome.is_game_over() {
        if let Ok(mv) = parse_san(&match_state.current_pos, trimmed) {
            items.insert(
                0,
                PaletteItem {
                    label: format!("♟ Play Move: {trimmed}"),
                    category: "Board Move",
                    cmd: PaletteCommand::PlaySanMove(mv, trimmed.to_string()),
                },
            );
        }
    }

    let q_lower = trimmed.to_ascii_lowercase();
    let filtered: Vec<&PaletteItem> = items
        .iter()
        .filter(|it| {
            q_lower.is_empty()
                || it.label.to_ascii_lowercase().contains(&q_lower)
                || it.category.to_ascii_lowercase().contains(&q_lower)
        })
        .collect();

    if !filtered.is_empty() {
        state.selected_idx = state.selected_idx.min(filtered.len() - 1);
    } else {
        state.selected_idx = 0;
    }

    let mut enter_pressed = false;
    ctx.input(|i| {
        if i.key_pressed(Key::ArrowDown) && !filtered.is_empty() {
            state.selected_idx = (state.selected_idx + 1) % filtered.len();
        }
        if i.key_pressed(Key::ArrowUp) && !filtered.is_empty() {
            state.selected_idx = if state.selected_idx == 0 {
                filtered.len() - 1
            } else {
                state.selected_idx - 1
            };
        }
        if i.key_pressed(Key::Enter) {
            enter_pressed = true;
        }
    });

    if enter_pressed {
        if let Some(item) = filtered.get(state.selected_idx) {
            executed = Some(item.cmd.clone());
            state.open = false;
            return executed;
        }
    }

    let mut keep_open = state.open;
    Window::new("⌘ Command Palette (Ctrl+K)")
        .open(&mut keep_open)
        .collapsible(false)
        .resizable(false)
        .anchor(Align2::CENTER_TOP, Vec2::new(0.0, 72.0))
        .fixed_size(Vec2::new(480.0, 350.0))
        .show(ctx, |ui| {
            let input_resp = ui.add(
                egui::TextEdit::singleline(&mut state.query)
                    .hint_text("Search bots, themes, tools, or type a SAN move (e.g. Nf3)...")
                    .desired_width(f32::INFINITY)
                    .font(egui::TextStyle::Body),
            );
            if state.just_opened {
                input_resp.request_focus();
                state.just_opened = false;
            }

            ui.add_space(6.0);
            ui.separator();

            ScrollArea::vertical()
                .max_height(275.0)
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    if filtered.is_empty() {
                        ui.label(
                            RichText::new("No matching commands.")
                                .italics()
                                .color(tokens.text_secondary),
                        );
                    }
                    for (idx, item) in filtered.iter().enumerate() {
                        let is_sel = idx == state.selected_idx;
                        let bg = if is_sel {
                            tokens.accent_primary
                        } else {
                            Color32::TRANSPARENT
                        };
                        let fg = if is_sel {
                            Color32::WHITE
                        } else {
                            tokens.text_primary
                        };

                        let resp = egui::Frame::none()
                            .fill(bg)
                            .rounding(Rounding::same(5.0_f32))
                            .inner_margin(egui::Margin::symmetric(8.0, 5.0))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(&item.label).strong().size(13.0).color(fg));
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                RichText::new(item.category)
                                                    .size(11.0)
                                                    .color(if is_sel {
                                                        Color32::from_rgb(235, 248, 225)
                                                    } else {
                                                        tokens.text_secondary
                                                    }),
                                            );
                                        },
                                    );
                                });
                            })
                            .response
                            .interact(egui::Sense::click());

                        if resp.clicked() {
                            executed = Some(item.cmd.clone());
                        }
                    }
                });

            ui.add_space(4.0);
            ui.label(
                RichText::new("↑/↓ Navigate  •  Enter Execute  •  Esc Close")
                    .size(10.5)
                    .color(tokens.text_secondary),
            );
            let _ = Stroke::NONE;
        });

    if executed.is_some() {
        state.open = false;
    } else {
        state.open = keep_open;
    }

    executed
}

fn build_palette_items() -> Vec<PaletteItem> {
    vec![
        PaletteItem {
            label: "🤖 Challenge GM Magnus Carlsen (2882)".into(),
            category: "Play Bot",
            cmd: PaletteCommand::ChallengeBot(BotId::Magnus),
        },
        PaletteItem {
            label: "⚡ Challenge GM Hikaru Nakamura (2800)".into(),
            category: "Play Bot",
            cmd: PaletteCommand::ChallengeBot(BotId::Hikaru),
        },
        PaletteItem {
            label: "🐱 Challenge Mittens (ELO 1?)".into(),
            category: "Play Bot",
            cmd: PaletteCommand::ChallengeBot(BotId::Mittens),
        },
        PaletteItem {
            label: "♕ Challenge Beth Harmon (2450 Queen's Gambit)".into(),
            category: "Play Bot",
            cmd: PaletteCommand::ChallengeBot(BotId::Beth),
        },
        PaletteItem {
            label: "👑 Challenge Nelson (1300 Early Queen)".into(),
            category: "Play Bot",
            cmd: PaletteCommand::ChallengeBot(BotId::Nelson),
        },
        PaletteItem {
            label: "🌱 Challenge Martin (Beginner)".into(),
            category: "Play Bot",
            cmd: PaletteCommand::ChallengeBot(BotId::Martin),
        },
        PaletteItem {
            label: "🧘 Toggle Zen / Focus Mode (Z)".into(),
            category: "View",
            cmd: PaletteCommand::ToggleZenMode,
        },
        PaletteItem {
            label: "🙈 Toggle Blindfold Mode (Hide Pieces)".into(),
            category: "Training",
            cmd: PaletteCommand::ToggleBlindfold,
        },
        PaletteItem {
            label: "🎲 Start Chess960 (Fischer Random) Match".into(),
            category: "Modes",
            cmd: PaletteCommand::StartChess960,
        },
        PaletteItem {
            label: "🎯 Open Coordinate Speed Vision Trainer".into(),
            category: "Training",
            cmd: PaletteCommand::Navigate(ActiveRoute::Vision),
        },
        PaletteItem {
            label: "👁 Toggle X-Ray Square Control Heatmap (X)".into(),
            category: "Analytics",
            cmd: PaletteCommand::ToggleXRay,
        },
        PaletteItem {
            label: "⇅ Flip Chessboard Orientation (F)".into(),
            category: "Board",
            cmd: PaletteCommand::FlipBoard,
        },
        PaletteItem {
            label: "🌗 Toggle Dark UI / Light UI".into(),
            category: "Appearance",
            cmd: PaletteCommand::ToggleDarkLight,
        },
        PaletteItem {
            label: "📈 Open Chess DNA & Opening Insights".into(),
            category: "Navigation",
            cmd: PaletteCommand::Navigate(ActiveRoute::Analytics),
        },
        PaletteItem {
            label: "🔥 Open Personal Blunder Deck".into(),
            category: "Training",
            cmd: PaletteCommand::OpenBlunderDeck,
        },
        PaletteItem {
            label: "♔ Open Endgame Mastery Drills".into(),
            category: "Training",
            cmd: PaletteCommand::Navigate(ActiveRoute::Endgames),
        },
        PaletteItem {
            label: "⚔ Open Bot vs. Bot Exhibition Arena".into(),
            category: "Modes",
            cmd: PaletteCommand::Navigate(ActiveRoute::Arena),
        },
        PaletteItem {
            label: "📊 Open Post-Game Review".into(),
            category: "Navigation",
            cmd: PaletteCommand::Navigate(ActiveRoute::Review),
        },
        PaletteItem {
            label: "🧩 Open Tactics Puzzles".into(),
            category: "Navigation",
            cmd: PaletteCommand::Navigate(ActiveRoute::Puzzles),
        },
        PaletteItem {
            label: "🤖 Open Play Bots Lobby (18 Bots)".into(),
            category: "Navigation",
            cmd: PaletteCommand::Navigate(ActiveRoute::PlayBots),
        },
        PaletteItem {
            label: "🏠 Go to Home Dashboard".into(),
            category: "Navigation",
            cmd: PaletteCommand::Navigate(ActiveRoute::Home),
        },
        PaletteItem {
            label: "📋 FEN / PGN Inspector & Importer".into(),
            category: "Tools",
            cmd: PaletteCommand::OpenFenPgn,
        },
        PaletteItem {
            label: "⚙ Open Settings".into(),
            category: "Tools",
            cmd: PaletteCommand::OpenSettings,
        },
        PaletteItem {
            label: "🎨 Board Theme: Classic Green".into(),
            category: "Theme",
            cmd: PaletteCommand::SetBoardTheme(BoardTheme::ClassicGreen),
        },
        PaletteItem {
            label: "🎨 Board Theme: Walnut / Wood".into(),
            category: "Theme",
            cmd: PaletteCommand::SetBoardTheme(BoardTheme::WalnutWood),
        },
        PaletteItem {
            label: "🎨 Board Theme: Tournament Blue".into(),
            category: "Theme",
            cmd: PaletteCommand::SetBoardTheme(BoardTheme::TournamentBlue),
        },
        PaletteItem {
            label: "🎨 Board Theme: Icy Sea".into(),
            category: "Theme",
            cmd: PaletteCommand::SetBoardTheme(BoardTheme::IcySea),
        },
        PaletteItem {
            label: "🎨 Board Theme: Dark Slate".into(),
            category: "Theme",
            cmd: PaletteCommand::SetBoardTheme(BoardTheme::DarkSlate),
        },
        PaletteItem {
            label: "♟ Piece Set: Classic Staunton".into(),
            category: "Pieces",
            cmd: PaletteCommand::SetPieceSet(PieceSetStyle::ClassicStaunton),
        },
        PaletteItem {
            label: "♟ Piece Set: Chess.com Neo".into(),
            category: "Pieces",
            cmd: PaletteCommand::SetPieceSet(PieceSetStyle::ChessComNeo),
        },
        PaletteItem {
            label: "♟ Piece Set: Minimalist Alpha".into(),
            category: "Pieces",
            cmd: PaletteCommand::SetPieceSet(PieceSetStyle::MinimalistAlpha),
        },
        PaletteItem {
            label: "👾 Piece Set: Pixel 8-Bit".into(),
            category: "Pieces",
            cmd: PaletteCommand::SetPieceSet(PieceSetStyle::Pixel8Bit),
        },
        PaletteItem {
            label: "⚡ Piece Set: Neon Wireframe".into(),
            category: "Pieces",
            cmd: PaletteCommand::SetPieceSet(PieceSetStyle::NeonWireframe),
        },
    ]
}
