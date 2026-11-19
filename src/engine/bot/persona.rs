use crate::engine::eval::EvalWeights;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BotId {
    AdaptiveJimmy,
    Martin,
    Elani,
    Wayne,
    Jimmy,
    Aron,
    Maria,
    Emir,
    Sven,
    Wendy,
    Nelson,
    Jade,
    Antonio,
    Isabel,
    David,
    Wally,
    Li,
    Noam,
    Francis,
    Beth,
    Danny,
    Hikaru,
    Magnus,
    Mittens,
    EngineCustom,
    // Legacy aliases preserved for backward compatibility
    Warung,
    Club,
    Silicon,
}

impl BotId {
    pub fn as_str(&self) -> &'static str {
        match self {
            BotId::AdaptiveJimmy => "adaptive_jimmy",
            BotId::Martin => "martin",
            BotId::Elani => "elani",
            BotId::Wayne => "wayne",
            BotId::Jimmy => "jimmy",
            BotId::Aron => "aron",
            BotId::Maria => "maria",
            BotId::Emir => "emir",
            BotId::Sven => "sven",
            BotId::Wendy => "wendy",
            BotId::Nelson => "nelson",
            BotId::Jade => "jade",
            BotId::Antonio => "antonio",
            BotId::Isabel => "isabel",
            BotId::David => "david",
            BotId::Wally => "wally",
            BotId::Li => "li",
            BotId::Noam => "noam",
            BotId::Francis => "francis",
            BotId::Beth => "beth",
            BotId::Danny => "danny",
            BotId::Hikaru => "hikaru",
            BotId::Magnus => "magnus",
            BotId::Mittens => "mittens",
            BotId::EngineCustom => "engine_custom",
            BotId::Warung => "warung",
            BotId::Club => "club",
            BotId::Silicon => "silicon",
        }
    }

    /// 24 Personality Bots utama di Play Bots Lobby (Beginner 400 hingga Cosmic 3200)
    pub fn all_lobby_personalities() -> &'static [BotId] {
        &[
            BotId::AdaptiveJimmy,
            BotId::Martin,
            BotId::Elani,
            BotId::Wayne,
            BotId::Jimmy,
            BotId::Aron,
            BotId::Maria,
            BotId::Emir,
            BotId::Sven,
            BotId::Wendy,
            BotId::Nelson,
            BotId::Jade,
            BotId::Antonio,
            BotId::Isabel,
            BotId::David,
            BotId::Wally,
            BotId::Li,
            BotId::Noam,
            BotId::Francis,
            BotId::Beth,
            BotId::Danny,
            BotId::Hikaru,
            BotId::Magnus,
            BotId::Mittens,
        ]
    }

    pub fn all() -> &'static [BotId] {
        Self::all_lobby_personalities()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BotPersona {
    pub id: BotId,
    pub code: &'static str,
    pub name: &'static str,
    pub tier: &'static str,
    pub title: &'static str,
    pub bio: &'static str,
    pub elo: u32,
    pub max_depth: u8,
    pub time_budget_ms: u64,
    pub pace_factor: f32,
    pub softmax_temp_cp: f32,
    pub noise_cp: i32,
    pub blunder_chance: f32,
    pub weights: EvalWeights,
    pub accent_rgb: [u8; 3],
    pub speed_stat: f32,
    pub tactical_stat: f32,
    pub positional_stat: f32,
}

impl BotPersona {
    pub fn by_id(id: BotId) -> Self {
        match id {
            BotId::AdaptiveJimmy => Self::adaptive_jimmy(900),
            BotId::Martin => Self::martin(),
            BotId::Elani => Self::elani(),
            BotId::Wayne => Self::wayne(),
            BotId::Jimmy => Self::jimmy(),
            BotId::Aron => Self::aron(),
            BotId::Maria => Self::maria(),
            BotId::Emir => Self::emir(),
            BotId::Sven => Self::sven(),
            BotId::Wendy => Self::wendy(),
            BotId::Nelson => Self::nelson(),
            BotId::Jade => Self::jade(),
            BotId::Antonio => Self::antonio(),
            BotId::Isabel => Self::isabel(),
            BotId::David => Self::david(),
            BotId::Wally => Self::wally(),
            BotId::Li => Self::li(),
            BotId::Noam => Self::noam(),
            BotId::Francis => Self::francis(),
            BotId::Beth => Self::beth(),
            BotId::Danny => Self::danny(),
            BotId::Hikaru => Self::hikaru(),
            BotId::Magnus => Self::magnus(),
            BotId::Mittens => Self::mittens(),
            BotId::EngineCustom => Self::engine_custom(1500),
            BotId::Warung => Self::warung(),
            BotId::Club => Self::club(),
            BotId::Silicon => Self::silicon(),
        }
    }

    pub fn by_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "adaptive_jimmy" => Some(Self::adaptive_jimmy(900)),
            "martin" => Some(Self::martin()),
            "elani" => Some(Self::elani()),
            "wayne" => Some(Self::wayne()),
            "jimmy" => Some(Self::jimmy()),
            "aron" => Some(Self::aron()),
            "maria" => Some(Self::maria()),
            "emir" => Some(Self::emir()),
            "sven" => Some(Self::sven()),
            "wendy" => Some(Self::wendy()),
            "nelson" => Some(Self::nelson()),
            "jade" => Some(Self::jade()),
            "antonio" => Some(Self::antonio()),
            "isabel" => Some(Self::isabel()),
            "david" => Some(Self::david()),
            "wally" => Some(Self::wally()),
            "li" => Some(Self::li()),
            "noam" => Some(Self::noam()),
            "francis" => Some(Self::francis()),
            "beth" => Some(Self::beth()),
            "danny" => Some(Self::danny()),
            "hikaru" => Some(Self::hikaru()),
            "magnus" => Some(Self::magnus()),
            "mittens" => Some(Self::mittens()),
            "engine_custom" => Some(Self::engine_custom(1500)),
            "warung" => Some(Self::warung()),
            "club" | "clara" => Some(Self::club()),
            "silicon" | "rustfish" => Some(Self::silicon()),
            _ => None,
        }
    }

    /// Seluruh 18 persona bot untuk Play Bots Lobby
    pub fn lobby_roster() -> Vec<Self> {
        BotId::all_lobby_personalities()
            .iter()
            .copied()
            .map(Self::by_id)
            .collect()
    }

    /// Roster 6 persona klasik untuk kompatibilitas pengujian
    pub fn all_roster() -> Vec<Self> {
        vec![
            Self::martin_legacy_400(),
            Self::warung(),
            Self::club(),
            Self::hikaru(),
            Self::magnus(),
            Self::silicon(),
        ]
    }

    fn martin_legacy_400() -> Self {
        let mut m = Self::martin();
        m.elo = 400;
        m
    }

    pub fn adaptive_jimmy(current_elo: u32) -> Self {
        let clamped_elo = current_elo.clamp(400, 2200);
        let depth = match clamped_elo {
            0..=650 => 2,
            651..=1250 => 3,
            1251..=1750 => 4,
            _ => 5,
        };
        let temp = ((2200 - clamped_elo) as f32 * 0.11).clamp(12.0, 200.0);
        Self {
            id: BotId::AdaptiveJimmy,
            code: "adaptive_jimmy",
            name: "Adaptive Jimmy",
            tier: "Adaptive",
            title: "Dynamic Scaling Sparring Partner",
            bio: "Starts around 900 ELO and dynamically adjusts search depth and precision based on your accuracy.",
            elo: clamped_elo,
            max_depth: depth,
            time_budget_ms: 450,
            pace_factor: 0.90,
            softmax_temp_cp: temp,
            noise_cp: temp as i32,
            blunder_chance: if clamped_elo < 1000 { 0.12 } else { 0.02 },
            weights: EvalWeights::default(),
            accent_rgb: [0x00, 0x9F, 0xD9],
            speed_stat: 0.75,
            tactical_stat: (clamped_elo as f32 / 2400.0).clamp(0.25, 0.90),
            positional_stat: (clamped_elo as f32 / 2400.0).clamp(0.25, 0.90),
        }
    }

    pub fn martin() -> Self {
        Self {
            id: BotId::Martin,
            code: "martin",
            name: "Martin",
            tier: "Beginner",
            title: "Friendly Beginner Bot",
            bio: "Just learned how the Knight moves yesterday! Pushes flank pawns and frequently leaves pieces undefended.",
            elo: 400,
            max_depth: 1,
            time_budget_ms: 180,
            pace_factor: 0.65,
            softmax_temp_cp: 260.0,
            noise_cp: 280,
            blunder_chance: 0.35,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.15,
                mobility_scale: 0.35,
                king_attack_scale: 0.25,
                endgame_scale: 0.25,
                contempt_cp: 0,
            },
            accent_rgb: [129, 182, 76],
            speed_stat: 0.65,
            tactical_stat: 0.15,
            positional_stat: 0.15,
        }
    }

    pub fn elani() -> Self {
        Self {
            id: BotId::Elani,
            code: "elani",
            name: "Elani",
            tier: "Beginner",
            title: "Enthusiastic Novice",
            bio: "Loves advancing wing pawns early; occasionally misses simple 2-move forks and back-rank pins.",
            elo: 400,
            max_depth: 2,
            time_budget_ms: 220,
            pace_factor: 0.85,
            softmax_temp_cp: 210.0,
            noise_cp: 210,
            blunder_chance: 0.25,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.30,
                mobility_scale: 0.45,
                king_attack_scale: 0.40,
                endgame_scale: 0.35,
                contempt_cp: 0,
            },
            accent_rgb: [142, 202, 92],
            speed_stat: 0.60,
            tactical_stat: 0.22,
            positional_stat: 0.20,
        }
    }

    pub fn wayne() -> Self {
        Self {
            id: BotId::Wayne,
            code: "wayne",
            name: "Wayne",
            tier: "Beginner",
            title: "Flank Pawn Enthusiast",
            bio: "Pushes rook pawns early and loves chaotic piece trades—solid practice for basic piece safety.",
            elo: 500,
            max_depth: 2,
            time_budget_ms: 230,
            pace_factor: 0.85,
            softmax_temp_cp: 175.0,
            noise_cp: 175,
            blunder_chance: 0.22,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.35,
                mobility_scale: 0.50,
                king_attack_scale: 0.45,
                endgame_scale: 0.40,
                contempt_cp: 0,
            },
            accent_rgb: [130, 195, 102],
            speed_stat: 0.62,
            tactical_stat: 0.25,
            positional_stat: 0.24,
        }
    }

    pub fn jimmy() -> Self {
        Self {
            id: BotId::Jimmy,
            code: "jimmy",
            name: "Jimmy",
            tier: "Beginner",
            title: "Casual Club Newcomer",
            bio: "Develops Knights early and plays natural moves, but leaves tactical hangers during complex exchanges.",
            elo: 600,
            max_depth: 2,
            time_budget_ms: 260,
            pace_factor: 0.90,
            softmax_temp_cp: 180.0,
            noise_cp: 180,
            blunder_chance: 0.18,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.45,
                mobility_scale: 0.55,
                king_attack_scale: 0.50,
                endgame_scale: 0.45,
                contempt_cp: 0,
            },
            accent_rgb: [115, 190, 110],
            speed_stat: 0.62,
            tactical_stat: 0.30,
            positional_stat: 0.32,
        }
    }

    pub fn aron() -> Self {
        Self {
            id: BotId::Aron,
            code: "aron",
            name: "Aron",
            tier: "Beginner",
            title: "Principled Beginner",
            bio: "Plays standard 1.e4 openings and reliably recaptures, though vulnerable to 3-ply tactical traps.",
            elo: 700,
            max_depth: 2,
            time_budget_ms: 300,
            pace_factor: 0.92,
            softmax_temp_cp: 155.0,
            noise_cp: 155,
            blunder_chance: 0.14,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.55,
                mobility_scale: 0.65,
                king_attack_scale: 0.55,
                endgame_scale: 0.50,
                contempt_cp: 5,
            },
            accent_rgb: [98, 178, 128],
            speed_stat: 0.64,
            tactical_stat: 0.36,
            positional_stat: 0.38,
        }
    }

    pub fn maria() -> Self {
        Self {
            id: BotId::Maria,
            code: "maria",
            name: "Maria",
            tier: "Beginner",
            title: "Italian Game Student",
            bio: "Develops Knights and Bishops cleanly and castles early—great warm-up before crossing 1000 ELO.",
            elo: 850,
            max_depth: 3,
            time_budget_ms: 340,
            pace_factor: 0.94,
            softmax_temp_cp: 72.0,
            noise_cp: 72,
            blunder_chance: 0.08,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.68,
                mobility_scale: 0.72,
                king_attack_scale: 0.65,
                endgame_scale: 0.65,
                contempt_cp: 5,
            },
            accent_rgb: [108, 188, 155],
            speed_stat: 0.65,
            tactical_stat: 0.44,
            positional_stat: 0.48,
        }
    }

    pub fn emir() -> Self {
        Self {
            id: BotId::Emir,
            code: "emir",
            name: "Emir",
            tier: "Intermediate",
            title: "Solid Tactician",
            bio: "Plays solid 1.d4 and Italian Game setups; defends hanging pieces and punishes loose tactics.",
            elo: 1000,
            max_depth: 3,
            time_budget_ms: 400,
            pace_factor: 0.95,
            softmax_temp_cp: 48.0,
            noise_cp: 48,
            blunder_chance: 0.03,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.85,
                mobility_scale: 0.85,
                king_attack_scale: 0.80,
                endgame_scale: 0.80,
                contempt_cp: 10,
            },
            accent_rgb: [230, 170, 55],
            speed_stat: 0.66,
            tactical_stat: 0.54,
            positional_stat: 0.56,
        }
    }

    pub fn sven() -> Self {
        Self {
            id: BotId::Sven,
            code: "sven",
            name: "Sven",
            tier: "Intermediate",
            title: "Nordic Brawler",
            bio: "Disciplined club fighter who seeks clean central breaks, defends his pieces, and punishes free hangers.",
            elo: 1100,
            max_depth: 3,
            time_budget_ms: 450,
            pace_factor: 1.00,
            softmax_temp_cp: 36.0,
            noise_cp: 36,
            blunder_chance: 0.02,
            weights: EvalWeights {
                material_scale: 1.05,
                positional_scale: 0.88,
                mobility_scale: 0.92,
                king_attack_scale: 0.98,
                endgame_scale: 0.82,
                contempt_cp: 12,
            },
            accent_rgb: [215, 150, 70],
            speed_stat: 0.64,
            tactical_stat: 0.60,
            positional_stat: 0.56,
        }
    }

    pub fn wendy() -> Self {
        Self {
            id: BotId::Wendy,
            code: "wendy",
            name: "Wendy",
            tier: "Intermediate",
            title: "Sicilian Counter-Puncher",
            bio: "Answers 1.e4 with the sharp Sicilian Defense and launches ferocious queenside counterplay.",
            elo: 1200,
            max_depth: 3,
            time_budget_ms: 470,
            pace_factor: 0.90,
            softmax_temp_cp: 32.0,
            noise_cp: 32,
            blunder_chance: 0.01,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.90,
                mobility_scale: 1.05,
                king_attack_scale: 1.12,
                endgame_scale: 0.85,
                contempt_cp: 15,
            },
            accent_rgb: [232, 132, 72],
            speed_stat: 0.72,
            tactical_stat: 0.64,
            positional_stat: 0.58,
        }
    }

    pub fn nelson() -> Self {
        Self {
            id: BotId::Nelson,
            code: "nelson",
            name: "Nelson",
            tier: "Intermediate",
            title: "The Queen Attack Menace",
            bio: "Brings his Queen out on move 2 (Qh5 / Qf3) to hunt your f-pawn! Punish his overextended Queen.",
            elo: 1300,
            max_depth: 4,
            time_budget_ms: 500,
            pace_factor: 0.80,
            softmax_temp_cp: 28.0,
            noise_cp: 28,
            blunder_chance: 0.01,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.85,
                mobility_scale: 1.25,
                king_attack_scale: 1.50,
                endgame_scale: 0.85,
                contempt_cp: 25,
            },
            accent_rgb: [235, 110, 55],
            speed_stat: 0.78,
            tactical_stat: 0.72,
            positional_stat: 0.52,
        }
    }

    pub fn jade() -> Self {
        Self {
            id: BotId::Jade,
            code: "jade",
            name: "Jade",
            tier: "Intermediate",
            title: "Gambit Swashbuckler",
            bio: "Plays the King's Gambit and daring sacrifices to drag your King into open tactical battles.",
            elo: 1400,
            max_depth: 4,
            time_budget_ms: 530,
            pace_factor: 0.86,
            softmax_temp_cp: 24.0,
            noise_cp: 24,
            blunder_chance: 0.01,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.92,
                mobility_scale: 1.20,
                king_attack_scale: 1.38,
                endgame_scale: 0.90,
                contempt_cp: 22,
            },
            accent_rgb: [64, 192, 160],
            speed_stat: 0.74,
            tactical_stat: 0.74,
            positional_stat: 0.62,
        }
    }

    pub fn antonio() -> Self {
        Self {
            id: BotId::Antonio,
            code: "antonio",
            name: "Antonio",
            tier: "Advanced",
            title: "Classical Club Champion",
            bio: "Disciplined Ruy Lopez and Queen's Gambit Declined specialist with confident positional play.",
            elo: 1500,
            max_depth: 4,
            time_budget_ms: 560,
            pace_factor: 1.00,
            softmax_temp_cp: 20.0,
            noise_cp: 20,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.05,
                mobility_scale: 1.0,
                king_attack_scale: 1.0,
                endgame_scale: 1.0,
                contempt_cp: 15,
            },
            accent_rgb: [85, 165, 240],
            speed_stat: 0.68,
            tactical_stat: 0.72,
            positional_stat: 0.76,
        }
    }

    pub fn isabel() -> Self {
        Self {
            id: BotId::Isabel,
            code: "isabel",
            name: "Isabel",
            tier: "Advanced",
            title: "Fortress Architect",
            bio: "Maintains rock-solid pawn structures, prioritizes kingside safety, and refutes unsound sacrifices.",
            elo: 1600,
            max_depth: 4,
            time_budget_ms: 600,
            pace_factor: 1.05,
            softmax_temp_cp: 16.0,
            noise_cp: 16,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.15,
                mobility_scale: 1.05,
                king_attack_scale: 1.0,
                endgame_scale: 1.10,
                contempt_cp: 15,
            },
            accent_rgb: [75, 180, 225],
            speed_stat: 0.66,
            tactical_stat: 0.75,
            positional_stat: 0.82,
        }
    }

    pub fn david() -> Self {
        Self {
            id: BotId::David,
            code: "david",
            name: "David",
            tier: "Advanced",
            title: "Caro-Kann Wall",
            bio: "Ultra-resilient Caro-Kann & Slav expert who neutralizes attacks and outplays you in rook endgames.",
            elo: 1700,
            max_depth: 4,
            time_budget_ms: 640,
            pace_factor: 1.08,
            softmax_temp_cp: 13.0,
            noise_cp: 13,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.22,
                mobility_scale: 1.05,
                king_attack_scale: 0.95,
                endgame_scale: 1.25,
                contempt_cp: 12,
            },
            accent_rgb: [95, 160, 215],
            speed_stat: 0.64,
            tactical_stat: 0.76,
            positional_stat: 0.86,
        }
    }

    pub fn wally() -> Self {
        Self {
            id: BotId::Wally,
            code: "wally",
            name: "Wally",
            tier: "Advanced",
            title: "Calculation Grinder",
            bio: "Sharp tactical vision in the middlegame; severely punishes greedy pawn-grabbing inaccuracies.",
            elo: 1800,
            max_depth: 5,
            time_budget_ms: 700,
            pace_factor: 1.10,
            softmax_temp_cp: 10.0,
            noise_cp: 10,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.05,
                positional_scale: 1.15,
                mobility_scale: 1.18,
                king_attack_scale: 1.18,
                endgame_scale: 1.15,
                contempt_cp: 20,
            },
            accent_rgb: [95, 145, 245],
            speed_stat: 0.65,
            tactical_stat: 0.84,
            positional_stat: 0.80,
        }
    }

    pub fn li() -> Self {
        Self {
            id: BotId::Li,
            code: "li",
            name: "Li",
            tier: "Advanced",
            title: "Endgame Technician",
            bio: "Patient positional maneuvers paired with clinical endgame conversions.",
            elo: 2000,
            max_depth: 5,
            time_budget_ms: 820,
            pace_factor: 1.15,
            softmax_temp_cp: 6.0,
            noise_cp: 6,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.28,
                mobility_scale: 1.18,
                king_attack_scale: 1.12,
                endgame_scale: 1.40,
                contempt_cp: 25,
            },
            accent_rgb: [130, 120, 245],
            speed_stat: 0.64,
            tactical_stat: 0.86,
            positional_stat: 0.90,
        }
    }

    pub fn noam() -> Self {
        Self {
            id: BotId::Noam,
            code: "noam",
            name: "Noam",
            tier: "Master",
            title: "Veteran National Master",
            bio: "Master-tier calculation; rarely drops more than 25 centipawns across an entire game.",
            elo: 2200,
            max_depth: 5,
            time_budget_ms: 880,
            pace_factor: 1.20,
            softmax_temp_cp: 10.0,
            noise_cp: 10,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.35,
                mobility_scale: 1.25,
                king_attack_scale: 1.20,
                endgame_scale: 1.45,
                contempt_cp: 30,
            },
            accent_rgb: [175, 110, 240],
            speed_stat: 0.62,
            tactical_stat: 0.90,
            positional_stat: 0.92,
        }
    }

    pub fn francis() -> Self {
        Self {
            id: BotId::Francis,
            code: "francis",
            name: "Francis",
            tier: "Master",
            title: "Positional Outpost Master",
            bio: "Relentless knight outpost exploitation and quiet positional squeezing.",
            elo: 2300,
            max_depth: 5,
            time_budget_ms: 950,
            pace_factor: 1.25,
            softmax_temp_cp: 6.0,
            noise_cp: 6,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.45,
                mobility_scale: 1.30,
                king_attack_scale: 1.20,
                endgame_scale: 1.55,
                contempt_cp: 35,
            },
            accent_rgb: [195, 95, 225],
            speed_stat: 0.60,
            tactical_stat: 0.92,
            positional_stat: 0.95,
        }
    }

    pub fn beth() -> Self {
        Self {
            id: BotId::Beth,
            code: "beth",
            name: "Beth Harmon",
            tier: "Master",
            title: "Queen's Gambit Prodigy",
            bio: "Fearless 1.d4 Queen's Gambit specialist with razor-sharp Sicilian counterattacks and ceiling-level visualization.",
            elo: 2450,
            max_depth: 6,
            time_budget_ms: 920,
            pace_factor: 0.88,
            softmax_temp_cp: 0.0,
            noise_cp: 0,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.38,
                mobility_scale: 1.42,
                king_attack_scale: 1.48,
                endgame_scale: 1.45,
                contempt_cp: 40,
            },
            accent_rgb: [224, 98, 126],
            speed_stat: 0.86,
            tactical_stat: 0.96,
            positional_stat: 0.93,
        }
    }

    pub fn danny() -> Self {
        Self {
            id: BotId::Danny,
            code: "danny",
            name: "IM Danny",
            tier: "Top Player",
            title: "International Master & Commentator",
            bio: "Tactical banter, dangerous gambits, and heavy preference for active bishop pairs!",
            elo: 2500,
            max_depth: 6,
            time_budget_ms: 900,
            pace_factor: 0.95,
            softmax_temp_cp: 3.0,
            noise_cp: 3,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.35,
                mobility_scale: 1.45,
                king_attack_scale: 1.40,
                endgame_scale: 1.40,
                contempt_cp: 35,
            },
            accent_rgb: [129, 182, 76],
            speed_stat: 0.82,
            tactical_stat: 0.94,
            positional_stat: 0.92,
        }
    }

    pub fn hikaru() -> Self {
        Self {
            id: BotId::Hikaru,
            code: "hikaru",
            name: "Hikaru Nakamura",
            tier: "Top Player",
            title: "GM Speed Demon",
            bio: "Lightning-fast tempo, sharp mobility (1.55), relentless kingside pressure, and Sicilian/KID fireworks!",
            elo: 2800,
            max_depth: 6,
            time_budget_ms: 550,
            pace_factor: 0.62,
            softmax_temp_cp: 0.0,
            noise_cp: 5,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.1,
                mobility_scale: 1.55,
                king_attack_scale: 1.65,
                endgame_scale: 1.35,
                contempt_cp: 35,
            },
            accent_rgb: [245, 95, 65],
            speed_stat: 0.99,
            tactical_stat: 0.96,
            positional_stat: 0.88,
        }
    }

    pub fn magnus() -> Self {
        Self {
            id: BotId::Magnus,
            code: "magnus",
            name: "Magnus Carlsen",
            tier: "Top Player",
            title: "The GOAT",
            bio: "Peak positional mastery (1.60) & endgame squeeze (1.85). Refuses draws and converts microscopic edges.",
            elo: 2882,
            max_depth: 6,
            time_budget_ms: 1100,
            pace_factor: 1.35,
            softmax_temp_cp: 0.0,
            noise_cp: 0,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.60,
                mobility_scale: 1.35,
                king_attack_scale: 1.25,
                endgame_scale: 1.85,
                contempt_cp: 45,
            },
            accent_rgb: [240, 195, 55],
            speed_stat: 0.90,
            tactical_stat: 0.95,
            positional_stat: 1.00,
        }
    }

    pub fn mittens() -> Self {
        Self {
            id: BotId::Mittens,
            code: "mittens",
            name: "Mittens",
            tier: "Personality",
            title: "Cosmic Feline Overlord (ELO 1?)",
            bio: "Meow. Displays rating '1' while calculating at maximum depth 7. Hehehe... your King looks delicious.",
            elo: 3200,
            max_depth: 7,
            time_budget_ms: 1400,
            pace_factor: 0.75,
            softmax_temp_cp: 0.0,
            noise_cp: 0,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.55,
                mobility_scale: 1.50,
                king_attack_scale: 1.60,
                endgame_scale: 1.75,
                contempt_cp: 50,
            },
            accent_rgb: [255, 125, 185],
            speed_stat: 0.96,
            tactical_stat: 1.00,
            positional_stat: 1.00,
        }
    }

    pub fn engine_custom(slider_elo: u32) -> Self {
        let elo = ((slider_elo.clamp(250, 3200) + 25) / 50) * 50;
        let max_depth = match elo {
            0..=450 => 1,
            451..=900 => 2,
            901..=1400 => 3,
            1401..=1950 => 4,
            1951..=2450 => 5,
            2451..=2900 => 6,
            _ => 7,
        };
        let temp = if elo >= 2600 {
            0.0
        } else {
            ((2600 - elo) as f32 * 0.11).clamp(0.0, 260.0)
        };
        Self {
            id: BotId::EngineCustom,
            code: "engine_custom",
            name: "Custom Engine",
            tier: "Engine",
            title: "Synthetic Training Engine",
            bio: "Unbiased calibrated chess engine scaled precisely to your chosen ELO rating (250–3200).",
            elo,
            max_depth,
            time_budget_ms: (200 + (max_depth as u64) * 180).min(1500),
            pace_factor: 1.00,
            softmax_temp_cp: temp,
            noise_cp: temp as i32,
            blunder_chance: 0.0,
            weights: EvalWeights::default(),
            accent_rgb: [160, 170, 185],
            speed_stat: 0.85,
            tactical_stat: (elo as f32 / 3200.0).clamp(0.1, 1.0),
            positional_stat: (elo as f32 / 3200.0).clamp(0.1, 1.0),
        }
    }

    // Legacy constructors for backward compatibility with existing tests
    pub fn warung() -> Self {
        let mut s = Self::sven();
        s.id = BotId::Warung;
        s.code = "warung";
        s.name = "Pak RT (Bapak Pos Ronda)";
        s.title = "Penjaga Warung Kopi";
        s.elo = 1100;
        s.noise_cp = 95;
        s
    }

    pub fn club() -> Self {
        let mut c = Self::isabel();
        c.id = BotId::Club;
        c.code = "club";
        c.name = "Clara";
        c.title = "Atlet Klub Kampus";
        c.elo = 1600;
        c.noise_cp = 35;
        c.weights = EvalWeights {
            material_scale: 1.0,
            positional_scale: 1.0,
            mobility_scale: 1.0,
            king_attack_scale: 1.0,
            endgame_scale: 1.0,
            contempt_cp: 15,
        };
        c
    }

    pub fn silicon() -> Self {
        let mut m = Self::mittens();
        m.id = BotId::Silicon;
        m.code = "silicon";
        m.name = "RustFish Core";
        m.title = "Maximum Overdrive";
        m.elo = 3100;
        m
    }
}
