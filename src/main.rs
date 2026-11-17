use rusty_chess::domain::{ChessMatchState, Color};

fn main() {
    let state = ChessMatchState::new_standard(Color::White);
    println!(
        "RustyChess v0.1.0 initialized (FEN: {})",
        state.initial_fen
    );
}
