use crate::{
    domain::{format_fen, parse_san},
    engine::bot::{BotId, BotPersona},
};
use rand::seq::SliceRandom;
use shakmaty::{Chess, Color, Move, Position};

/// Mencari langkah pembukaan (ply 0..10 / langkah 1–6) sesuai repertoar khas tiap persona bot
/// sebagaimana didefinisikan pada NEW_DESIGN.md Section 7.2.
pub fn lookup_persona_book_move(pos: &Chess, persona: &BotPersona, ply: usize) -> Option<Move> {
    if ply > 10 {
        return None;
    }

    // Ambil bagian papan + giliran dari FEN (2 token pertama) sebagai kunci buku pembukaan
    let full_fen = format_fen(pos);
    let board_turn_key: String = full_fen
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");

    let candidate_sans: &[&str] = match persona.id {
        // 1. Martin, Elani & Wayne: Passive flank pawn pushes & quirky setups
        BotId::Martin | BotId::Elani | BotId::Wayne => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["a3", "h3", "g3", "b3", "c3"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["a6", "h6", "g6", "b6", "d6"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["a6", "h6", "c6", "g6"],
            _ => return try_flank_pawn_move(pos),
        },

        // 1b. Wendy: Sicilian Counter-Puncher (1...c5)
        BotId::Wendy => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["e4", "c4"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["c5"],
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b" => &["Nc6", "d6"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["Nf6", "c5"],
            _ => &[],
        },

        // 1c. Jade: Gambit Swashbuckler (King's Gambit 1.e4 e5 2.f4 & Benko/Scandinavian)
        BotId::Jade => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["e4"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w" => &["f4", "Bc4"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["d5", "e5"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["c5", "Nf6"],
            _ => &[],
        },

        // 1d. David: Caro-Kann & Slav Wall
        BotId::David => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["d4", "Nf3"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["c6"],
            "rnbqkbnr/pp1ppppp/2p5/8/3PP3/8/PPP2PPP/RNBQKBNR b" => &["d5"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["d5"],
            "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b" => &["c6"],
            _ => &[],
        },

        // 1e. Beth Harmon: Queen's Gambit & Sicilian Killer
        BotId::Beth => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["d4"],
            "rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w" => &["c4"],
            "rnbqkb1r/pppppppp/5n2/8/3P4/8/PPP1PPPP/RNBQKBNR w" => &["c4"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["c5"],
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b" => &["d6", "Nc6"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["d5", "Nf6"],
            _ => &[],
        },

        // 2. Nelson: Signature early Queen attacks (1. e4 e5 2. Qh5 / 2. Bc4 / Qf3)
        BotId::Nelson => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["e4"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w" => &["Qh5", "Bc4"],
            "r1bqkbnr/pppp1ppp/2n5/4p2Q/4P3/8/PPPP1PPP/RNB1KBNR w" => &["Bc4"],
            "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w" => &["exd5", "Qh5"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["e5"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b" => &["Qf6", "Qh4", "Bc5"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["e5", "c5"],
            _ => &[],
        },

        // 3. Hikaru: Sicilian Najdorf, King's Indian Defense, sharp dynamic lines
        BotId::Hikaru => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["e4", "Nf3", "b3"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w" => &["Nf3"],
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w" => &["Bc4", "Bb5", "d4"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["c5", "g6"],
            "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b" => &["d6", "Nc6"],
            "rnbqkbnr/pp2pppp/3p4/2p5/3PP3/5N2/PPP2PPP/RNBQKB1R b" => &["cxd4"],
            "rnbqkbnr/pp2pppp/3p4/8/3NP3/8/PPP2PPP/RNBQKB1R b" => &["Nf6"],
            "rnbqkb1r/pp2pppp/3p1n2/8/3NP3/2N5/PPP2PPP/R1BQKB1R b" => &["a6", "g6"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["Nf6"],
            "rnbqkb1r/pppppppp/5n2/8/2PP4/8/PP2PPPP/RNBQKBNR b" => &["g6"],
            "rnbqkb1r/pppppp1p/5np1/8/2PP4/2N5/PP2PPPP/R1BQKBNR b" => &["Bg7"],
            _ => &[],
        },

        // 4. Magnus & Mittens: Ruy Lopez, Catalan, Queen's Gambit Declined, London System
        BotId::Magnus | BotId::Mittens => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["d4", "e4", "c4", "Nf3"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w" => &["Nf3"],
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w" => &["Bb5"],
            "rnbqkbnr/ppp1pppp/8/3p4/3P4/8/PPP1PPPP/RNBQKBNR w" => &["c4", "Bf4"],
            "rnbqkb1r/pppppppp/5n2/8/3P4/8/PPP1PPPP/RNBQKBNR w" => &["c4"],
            "rnbqkb1r/pppp1ppp/4pn2/8/2PP4/8/PP2PPPP/RNBQKBNR w" => &["g3", "Nf3"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["e5", "c5"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b" => &["Nc6"],
            "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b" => &["a6", "Nf6"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["Nf6", "d5"],
            "rnbqkbnr/ppp1pppp/8/3p4/2PP4/8/PP2PPPP/RNBQKBNR b" => &["e6", "c6"],
            _ => &[],
        },

        // 5. Classical / Club / Master Personas
        _ => match board_turn_key.as_str() {
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w" => &["e4", "d4"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w" => &["Nf3"],
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w" => &["Bc4", "Bb5"],
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b" => &["e5", "c5", "e6"],
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b" => &["Nc6", "Nf6"],
            "rnbqkbnr/pppppppp/8/8/3P4/8/PPP1PPPP/RNBQKBNR b" => &["d5", "Nf6"],
            _ => &[],
        },
    };

    if candidate_sans.is_empty() {
        return None;
    }

    let mut valid_moves = Vec::new();
    for &san in candidate_sans {
        if let Ok(mv) = parse_san(pos, san) {
            valid_moves.push(mv);
        }
    }

    let mut rng = rand::thread_rng();
    valid_moves.choose(&mut rng).cloned()
}

fn try_flank_pawn_move(pos: &Chess) -> Option<Move> {
    let flank_sans: &[&str] = match pos.turn() {
        Color::White => &["a3", "h3", "a4", "h4", "b3", "g3"],
        Color::Black => &["a6", "h6", "a5", "h5", "b6", "g6"],
    };
    let mut valid = Vec::new();
    for &san in flank_sans {
        if let Ok(mv) = parse_san(pos, san) {
            valid.push(mv);
        }
    }
    let mut rng = rand::thread_rng();
    valid.choose(&mut rng).cloned()
}
