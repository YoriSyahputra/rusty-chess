use crate::domain::MoveRecord;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EcoOpening {
    pub eco: &'static str,
    pub name: &'static str,
    pub moves: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpeningMatch {
    pub eco: &'static str,
    pub name: &'static str,
    pub matched_plies: usize,
}

/// Embedded ECO Opening Encyclopedia (~130+ curated classical, tactical, and flank openings).
/// Matched by longest prefix against the live or historical game move sequence.
pub const ECO_OPENINGS: &[EcoOpening] = &[
    // --- A: Flank Openings & Indian Defenses ---
    EcoOpening { eco: "A00", name: "Polish Opening (Orangutan)", moves: &["b4"] },
    EcoOpening { eco: "A00", name: "Grob's Attack", moves: &["g4"] },
    EcoOpening { eco: "A00", name: "Anderssen's Opening", moves: &["a3"] },
    EcoOpening { eco: "A00", name: "Clemenz Opening", moves: &["h3"] },
    EcoOpening { eco: "A00", name: "King's Fianchetto Opening", moves: &["g3"] },
    EcoOpening { eco: "A01", name: "Nimzo-Larsen Attack", moves: &["b3"] },
    EcoOpening { eco: "A02", name: "Bird's Opening", moves: &["f4"] },
    EcoOpening { eco: "A03", name: "Bird's Opening: Dutch Variation", moves: &["f4", "d5"] },
    EcoOpening { eco: "A04", name: "Réti Opening", moves: &["Nf3"] },
    EcoOpening { eco: "A05", name: "Réti Opening: King's Indian Attack", moves: &["Nf3", "Nf6", "g3"] },
    EcoOpening { eco: "A06", name: "Réti Opening: Advance", moves: &["Nf3", "d5"] },
    EcoOpening { eco: "A09", name: "Réti Opening: Classic Gambit", moves: &["Nf3", "d5", "c4"] },
    EcoOpening { eco: "A10", name: "English Opening", moves: &["c4"] },
    EcoOpening { eco: "A13", name: "English Opening: Agincourt Defense", moves: &["c4", "e6"] },
    EcoOpening { eco: "A15", name: "English Opening: Anglo-Indian", moves: &["c4", "Nf6"] },
    EcoOpening { eco: "A20", name: "English Opening: Reversed Sicilian", moves: &["c4", "e5"] },
    EcoOpening { eco: "A22", name: "English Opening: Bremen System", moves: &["c4", "e5", "Nc3", "Nf6"] },
    EcoOpening { eco: "A28", name: "English Opening: Four Knights", moves: &["c4", "e5", "Nc3", "Nf6", "Nf3", "Nc6"] },
    EcoOpening { eco: "A30", name: "English Opening: Symmetrical", moves: &["c4", "c5"] },
    EcoOpening { eco: "A40", name: "Queen's Pawn Opening", moves: &["d4"] },
    EcoOpening { eco: "A41", name: "Queen's Pawn: Modern Defense", moves: &["d4", "d6"] },
    EcoOpening { eco: "A43", name: "Old Benoni Defense", moves: &["d4", "c5"] },
    EcoOpening { eco: "A45", name: "Trompowsky Attack", moves: &["d4", "Nf6", "Bg5"] },
    EcoOpening { eco: "A46", name: "Indian Defense", moves: &["d4", "Nf6"] },
    EcoOpening { eco: "A48", name: "London System (vs Nf6)", moves: &["d4", "Nf6", "Nf3", "g6", "Bf4"] },
    EcoOpening { eco: "A50", name: "Indian Game: c4 Setup", moves: &["d4", "Nf6", "c4"] },
    EcoOpening { eco: "A52", name: "Budapest Gambit", moves: &["d4", "Nf6", "c4", "e5"] },
    EcoOpening { eco: "A53", name: "Old Indian Defense", moves: &["d4", "Nf6", "c4", "d6"] },
    EcoOpening { eco: "A57", name: "Benko Gambit", moves: &["d4", "Nf6", "c4", "c5", "d5", "b5"] },
    EcoOpening { eco: "A60", name: "Benoni Defense", moves: &["d4", "Nf6", "c4", "c5", "d5"] },
    EcoOpening { eco: "A80", name: "Dutch Defense", moves: &["d4", "f5"] },
    EcoOpening { eco: "A85", name: "Dutch Defense: Classical", moves: &["d4", "f5", "c4", "Nf6", "Nc3"] },

    // --- B: Semi-Open Games (Sicilian, Caro-Kann, Pirc, Scandinavian, Alekhine) ---
    EcoOpening { eco: "B00", name: "King's Pawn Opening", moves: &["e4"] },
    EcoOpening { eco: "B00", name: "Owen's Defense", moves: &["e4", "b6"] },
    EcoOpening { eco: "B01", name: "Scandinavian Defense", moves: &["e4", "d5"] },
    EcoOpening { eco: "B01", name: "Scandinavian: Main Line", moves: &["e4", "d5", "exd5", "Qxd5"] },
    EcoOpening { eco: "B01", name: "Scandinavian: Modern Variation", moves: &["e4", "d5", "exd5", "Nf6"] },
    EcoOpening { eco: "B02", name: "Alekhine's Defense", moves: &["e4", "Nf6"] },
    EcoOpening { eco: "B03", name: "Alekhine's Defense: Four Pawns", moves: &["e4", "Nf6", "e5", "Nd5", "d4", "d6", "c4", "Nb6", "f4"] },
    EcoOpening { eco: "B06", name: "Modern Defense", moves: &["e4", "g6"] },
    EcoOpening { eco: "B07", name: "Pirc Defense", moves: &["e4", "d6", "d4", "Nf6"] },
    EcoOpening { eco: "B09", name: "Pirc Defense: Austrian Attack", moves: &["e4", "d6", "d4", "Nf6", "Nc3", "g6", "f4"] },
    EcoOpening { eco: "B10", name: "Caro-Kann Defense", moves: &["e4", "c6"] },
    EcoOpening { eco: "B12", name: "Caro-Kann: Advance Variation", moves: &["e4", "c6", "d4", "d5", "e5"] },
    EcoOpening { eco: "B13", name: "Caro-Kann: Exchange Variation", moves: &["e4", "c6", "d4", "d5", "exd5", "cxd5"] },
    EcoOpening { eco: "B14", name: "Caro-Kann: Panov-Botvinnik Attack", moves: &["e4", "c6", "d4", "d5", "exd5", "cxd5", "c4"] },
    EcoOpening { eco: "B15", name: "Caro-Kann: Tartakower / Classical", moves: &["e4", "c6", "d4", "d5", "Nc3"] },
    EcoOpening { eco: "B18", name: "Caro-Kann: Classical Variation", moves: &["e4", "c6", "d4", "d5", "Nc3", "dxe4", "Nxe4", "Bf5"] },
    EcoOpening { eco: "B20", name: "Sicilian Defense", moves: &["e4", "c5"] },
    EcoOpening { eco: "B21", name: "Sicilian: Smith-Morra Gambit", moves: &["e4", "c5", "d4", "cxd4", "c3"] },
    EcoOpening { eco: "B22", name: "Sicilian: Alapin Variation", moves: &["e4", "c5", "c3"] },
    EcoOpening { eco: "B23", name: "Sicilian: Closed Variation", moves: &["e4", "c5", "Nc3"] },
    EcoOpening { eco: "B27", name: "Sicilian: Hyperaccelerated Dragon", moves: &["e4", "c5", "Nf3", "g6"] },
    EcoOpening { eco: "B30", name: "Sicilian: Old Sicilian", moves: &["e4", "c5", "Nf3", "Nc6"] },
    EcoOpening { eco: "B31", name: "Sicilian: Rossolimo Attack", moves: &["e4", "c5", "Nf3", "Nc6", "Bb5"] },
    EcoOpening { eco: "B32", name: "Sicilian: Open", moves: &["e4", "c5", "Nf3", "Nc6", "d4", "cxd4", "Nxd4"] },
    EcoOpening { eco: "B33", name: "Sicilian: Sveshnikov Variation", moves: &["e4", "c5", "Nf3", "Nc6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "e5"] },
    EcoOpening { eco: "B35", name: "Sicilian: Accelerated Dragon", moves: &["e4", "c5", "Nf3", "Nc6", "d4", "cxd4", "Nxd4", "g6"] },
    EcoOpening { eco: "B40", name: "Sicilian: French Variation", moves: &["e4", "c5", "Nf3", "e6"] },
    EcoOpening { eco: "B42", name: "Sicilian: Kan Variation", moves: &["e4", "c5", "Nf3", "e6", "d4", "cxd4", "Nxd4", "a6"] },
    EcoOpening { eco: "B45", name: "Sicilian: Taimanov Variation", moves: &["e4", "c5", "Nf3", "e6", "d4", "cxd4", "Nxd4", "Nc6"] },
    EcoOpening { eco: "B50", name: "Sicilian: Modern", moves: &["e4", "c5", "Nf3", "d6"] },
    EcoOpening { eco: "B51", name: "Sicilian: Moscow Variation", moves: &["e4", "c5", "Nf3", "d6", "Bb5"] },
    EcoOpening { eco: "B54", name: "Sicilian: Open (d6)", moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4"] },
    EcoOpening { eco: "B70", name: "Sicilian: Dragon Variation", moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "g6"] },
    EcoOpening { eco: "B76", name: "Sicilian: Yugoslav Attack", moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "g6", "Be3", "Bg7", "f3"] },
    EcoOpening { eco: "B80", name: "Sicilian: Scheveningen", moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "e6"] },
    EcoOpening { eco: "B90", name: "Sicilian: Najdorf Variation", moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6"] },
    EcoOpening { eco: "B96", name: "Sicilian: Najdorf English Attack", moves: &["e4", "c5", "Nf3", "d6", "d4", "cxd4", "Nxd4", "Nf6", "Nc3", "a6", "Be3"] },

    // --- C: Open Games & French Defense ---
    EcoOpening { eco: "C00", name: "French Defense", moves: &["e4", "e6"] },
    EcoOpening { eco: "C01", name: "French: Exchange Variation", moves: &["e4", "e6", "d4", "d5", "exd5", "exd5"] },
    EcoOpening { eco: "C02", name: "French: Advance Variation", moves: &["e4", "e6", "d4", "d5", "e5"] },
    EcoOpening { eco: "C03", name: "French: Tarrasch Variation", moves: &["e4", "e6", "d4", "d5", "Nd2"] },
    EcoOpening { eco: "C11", name: "French: Classical Variation", moves: &["e4", "e6", "d4", "d5", "Nc3", "Nf6"] },
    EcoOpening { eco: "C15", name: "French: Winawer Variation", moves: &["e4", "e6", "d4", "d5", "Nc3", "Bb4"] },
    EcoOpening { eco: "C20", name: "King's Pawn Game", moves: &["e4", "e5"] },
    EcoOpening { eco: "C20", name: "Wayward Queen Attack (Nelson)", moves: &["e4", "e5", "Qh5"] },
    EcoOpening { eco: "C21", name: "Danish Gambit", moves: &["e4", "e5", "d4", "exd4", "c3"] },
    EcoOpening { eco: "C23", name: "Bishop's Opening", moves: &["e4", "e5", "Bc4"] },
    EcoOpening { eco: "C25", name: "Vienna Game", moves: &["e4", "e5", "Nc3"] },
    EcoOpening { eco: "C29", name: "Vienna Gambit", moves: &["e4", "e5", "Nc3", "Nf6", "f4"] },
    EcoOpening { eco: "C30", name: "King's Gambit", moves: &["e4", "e5", "f4"] },
    EcoOpening { eco: "C33", name: "King's Gambit Accepted", moves: &["e4", "e5", "f4", "exd4"] },
    EcoOpening { eco: "C40", name: "King's Knight Opening", moves: &["e4", "e5", "Nf3"] },
    EcoOpening { eco: "C41", name: "Philidor Defense", moves: &["e4", "e5", "Nf3", "d6"] },
    EcoOpening { eco: "C42", name: "Petrov's Defense", moves: &["e4", "e5", "Nf3", "Nf6"] },
    EcoOpening { eco: "C42", name: "Stafford Gambit", moves: &["e4", "e5", "Nf3", "Nf6", "Nxe5", "Nc6"] },
    EcoOpening { eco: "C44", name: "Scotch Game", moves: &["e4", "e5", "Nf3", "Nc6", "d4"] },
    EcoOpening { eco: "C45", name: "Scotch Game: Main Line", moves: &["e4", "e5", "Nf3", "Nc6", "d4", "exd4", "Nxd4"] },
    EcoOpening { eco: "C46", name: "Three Knights Opening", moves: &["e4", "e5", "Nf3", "Nc6", "Nc3"] },
    EcoOpening { eco: "C47", name: "Four Knights Game", moves: &["e4", "e5", "Nf3", "Nc6", "Nc3", "Nf6"] },
    EcoOpening { eco: "C50", name: "Italian Game", moves: &["e4", "e5", "Nf3", "Nc6", "Bc4"] },
    EcoOpening { eco: "C51", name: "Evans Gambit", moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "b4"] },
    EcoOpening { eco: "C53", name: "Italian: Giuoco Piano", moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "c3"] },
    EcoOpening { eco: "C54", name: "Italian: Giuoco Pianissimo", moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Bc5", "d3"] },
    EcoOpening { eco: "C55", name: "Italian: Two Knights Defense", moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Nf6"] },
    EcoOpening { eco: "C57", name: "Fried Liver Attack", moves: &["e4", "e5", "Nf3", "Nc6", "Bc4", "Nf6", "Ng5", "d5", "exd5", "Nxd5", "Nxf7"] },
    EcoOpening { eco: "C60", name: "Ruy Lopez (Spanish Opening)", moves: &["e4", "e5", "Nf3", "Nc6", "Bb5"] },
    EcoOpening { eco: "C65", name: "Ruy Lopez: Berlin Defense", moves: &["e4", "e5", "Nf3", "Nc6", "Bb5", "Nf6"] },
    EcoOpening { eco: "C68", name: "Ruy Lopez: Exchange Variation", moves: &["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Bxc6"] },
    EcoOpening { eco: "C70", name: "Ruy Lopez: Morphy Defense", moves: &["e4", "e5", "Nf3", "Nc6", "Bb5", "a6"] },
    EcoOpening { eco: "C78", name: "Ruy Lopez: Arkhangelsk / Main", moves: &["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O"] },
    EcoOpening { eco: "C84", name: "Ruy Lopez: Closed Spanish", moves: &["e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O", "Be7"] },

    // --- D: Closed Games & Semi-Closed Games ---
    EcoOpening { eco: "D00", name: "Queen's Pawn Game", moves: &["d4", "d5"] },
    EcoOpening { eco: "D00", name: "London System", moves: &["d4", "d5", "Bf4"] },
    EcoOpening { eco: "D00", name: "Blackmar-Diemer Gambit", moves: &["d4", "d5", "e4"] },
    EcoOpening { eco: "D02", name: "London System (Nf3)", moves: &["d4", "d5", "Nf3", "Nf6", "Bf4"] },
    EcoOpening { eco: "D04", name: "Colle System", moves: &["d4", "d5", "Nf3", "Nf6", "e3"] },
    EcoOpening { eco: "D06", name: "Queen's Gambit", moves: &["d4", "d5", "c4"] },
    EcoOpening { eco: "D10", name: "Slav Defense", moves: &["d4", "d5", "c4", "c6"] },
    EcoOpening { eco: "D15", name: "Slav Defense: Main Line", moves: &["d4", "d5", "c4", "c6", "Nf3", "Nf6", "Nc3"] },
    EcoOpening { eco: "D20", name: "Queen's Gambit Accepted", moves: &["d4", "d5", "c4", "dxc4"] },
    EcoOpening { eco: "D30", name: "Queen's Gambit Declined", moves: &["d4", "d5", "c4", "e6"] },
    EcoOpening { eco: "D35", name: "QGD: Exchange Variation", moves: &["d4", "d5", "c4", "e6", "Nc3", "Nf6", "cxd5", "exd5"] },
    EcoOpening { eco: "D37", name: "QGD: Classical Variation", moves: &["d4", "d5", "c4", "e6", "Nc3", "Nf6", "Nf3"] },
    EcoOpening { eco: "D43", name: "Semi-Slav Defense", moves: &["d4", "d5", "c4", "e6", "Nc3", "Nf6", "Nf3", "c6"] },
    EcoOpening { eco: "D70", name: "Grünfeld Defense", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "d5"] },
    EcoOpening { eco: "D85", name: "Grünfeld: Exchange Variation", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "d5", "cxd5", "Nxd5", "e4"] },

    // --- E: Indian Defenses (Catalan, Nimzo-Indian, Queen's Indian, King's Indian) ---
    EcoOpening { eco: "E00", name: "Catalan Opening", moves: &["d4", "Nf6", "c4", "e6", "g3"] },
    EcoOpening { eco: "E04", name: "Catalan: Open Variation", moves: &["d4", "Nf6", "c4", "e6", "g3", "d5", "Bg2", "dxc4"] },
    EcoOpening { eco: "E06", name: "Catalan: Closed Variation", moves: &["d4", "Nf6", "c4", "e6", "g3", "d5", "Bg2", "Be7"] },
    EcoOpening { eco: "E11", name: "Bogo-Indian Defense", moves: &["d4", "Nf6", "c4", "e6", "Nf3", "Bb4"] },
    EcoOpening { eco: "E12", name: "Queen's Indian Defense", moves: &["d4", "Nf6", "c4", "e6", "Nf3", "b6"] },
    EcoOpening { eco: "E20", name: "Nimzo-Indian Defense", moves: &["d4", "Nf6", "c4", "e6", "Nc3", "Bb4"] },
    EcoOpening { eco: "E32", name: "Nimzo-Indian: Classical", moves: &["d4", "Nf6", "c4", "e6", "Nc3", "Bb4", "Qc2"] },
    EcoOpening { eco: "E40", name: "Nimzo-Indian: Rubinstein", moves: &["d4", "Nf6", "c4", "e6", "Nc3", "Bb4", "e3"] },
    EcoOpening { eco: "E60", name: "King's Indian Defense", moves: &["d4", "Nf6", "c4", "g6"] },
    EcoOpening { eco: "E61", name: "King's Indian Defense: Classical", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "Bg7"] },
    EcoOpening { eco: "E70", name: "King's Indian: Normal Variation", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "Bg7", "e4", "d6"] },
    EcoOpening { eco: "E76", name: "King's Indian: Four Pawns Attack", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "Bg7", "e4", "d6", "f4"] },
    EcoOpening { eco: "E80", name: "King's Indian: Sämisch Variation", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "Bg7", "e4", "d6", "f3"] },
    EcoOpening { eco: "E90", name: "King's Indian: Main Line", moves: &["d4", "Nf6", "c4", "g6", "Nc3", "Bg7", "e4", "d6", "Nf3", "O-O", "Be2"] },
];

fn normalize_san(san: &str) -> &str {
    san.trim().trim_end_matches(['+', '#', '!', '?'])
}

/// Mendeteksi nama pembukaan ECO terdalam yang cocok dengan urutan langkah SAN saat ini.
pub fn detect_opening(history_sans: &[String]) -> Option<OpeningMatch> {
    if history_sans.is_empty() {
        return None;
    }

    let mut best: Option<OpeningMatch> = None;

    for entry in ECO_OPENINGS {
        let len = entry.moves.len();
        if len > history_sans.len() {
            continue;
        }
        let matches = entry
            .moves
            .iter()
            .zip(history_sans.iter())
            .all(|(&book_mv, game_mv)| normalize_san(book_mv) == normalize_san(game_mv));

        if matches {
            let should_replace = match &best {
                None => true,
                Some(prev) => len >= prev.matched_plies,
            };
            if should_replace {
                best = Some(OpeningMatch {
                    eco: entry.eco,
                    name: entry.name,
                    matched_plies: len,
                });
            }
        }
    }

    best
}

pub fn detect_opening_from_records(records: &[MoveRecord]) -> Option<OpeningMatch> {
    let sans: Vec<String> = records.iter().map(|r| r.san.clone()).collect();
    detect_opening(&sans)
}
