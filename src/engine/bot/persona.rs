use crate::engine::eval::EvalWeights;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BotId {
    Martin,
    Warung,
    Club,
    Hikaru,
    Magnus,
    Silicon,
}

impl BotId {
    pub fn as_str(&self) -> &'static str {
        match self {
            BotId::Martin => "martin",
            BotId::Warung => "warung",
            BotId::Club => "club",
            BotId::Hikaru => "hikaru",
            BotId::Magnus => "magnus",
            BotId::Silicon => "silicon",
        }
    }

    pub fn all() -> &'static [BotId] {
        &[
            BotId::Martin,
            BotId::Warung,
            BotId::Club,
            BotId::Hikaru,
            BotId::Magnus,
            BotId::Silicon,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct BotPersona {
    pub id: BotId,
    pub code: &'static str,
    pub name: &'static str,
    pub title: &'static str,
    pub bio: &'static str,
    pub elo: u32,
    pub max_depth: u8,
    pub time_budget_ms: u64,
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
            BotId::Martin => Self::martin(),
            BotId::Warung => Self::warung(),
            BotId::Club => Self::club(),
            BotId::Hikaru => Self::hikaru(),
            BotId::Magnus => Self::magnus(),
            BotId::Silicon => Self::silicon(),
        }
    }

    pub fn by_code(code: &str) -> Option<Self> {
        match code.trim().to_ascii_lowercase().as_str() {
            "martin" => Some(Self::martin()),
            "warung" => Some(Self::warung()),
            "club" | "clara" => Some(Self::club()),
            "hikaru" => Some(Self::hikaru()),
            "magnus" => Some(Self::magnus()),
            "silicon" | "rustfish" => Some(Self::silicon()),
            _ => None,
        }
    }

    pub fn all_roster() -> Vec<Self> {
        BotId::all().iter().copied().map(Self::by_id).collect()
    }

    pub fn martin() -> Self {
        Self {
            id: BotId::Martin,
            code: "martin",
            name: "Martin",
            title: "Bot Santai Pemula",
            bio: "Sering meninggalkan perwira tanpa kawalan, evaluasi posisi rendah, cocok untuk pemanasan santai.",
            elo: 400,
            max_depth: 2,
            time_budget_ms: 250,
            noise_cp: 280,
            blunder_chance: 0.35,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 0.2,
                mobility_scale: 0.4,
                king_attack_scale: 0.3,
                endgame_scale: 0.3,
                contempt_cp: 0,
            },
            accent_rgb: [120, 200, 100],
            speed_stat: 0.65,
            tactical_stat: 0.15,
            positional_stat: 0.15,
        }
    }

    pub fn warung() -> Self {
        Self {
            id: BotId::Warung,
            code: "warung",
            name: "Pak RT (Bapak Pos Ronda)",
            title: "Penjaga Warung Kopi",
            bio: "Suka tukar perwira, lumayan paham taktik 1-2 langkah sambil menyeruput kopi hitam, kadang lengah skak ganda.",
            elo: 1100,
            max_depth: 3,
            time_budget_ms: 400,
            noise_cp: 95,
            blunder_chance: 0.15,
            weights: EvalWeights {
                material_scale: 1.05,
                positional_scale: 0.7,
                mobility_scale: 0.8,
                king_attack_scale: 0.9,
                endgame_scale: 0.6,
                contempt_cp: 10,
            },
            accent_rgb: [215, 150, 70],
            speed_stat: 0.60,
            tactical_stat: 0.48,
            positional_stat: 0.42,
        }
    }

    pub fn club() -> Self {
        Self {
            id: BotId::Club,
            code: "club",
            name: "Clara",
            title: "Atlet Klub Kampus",
            bio: "Seimbang di semua lini, disiplin melakukan rokade tepat waktu dan menjaga keutuhan struktur pion.",
            elo: 1600,
            max_depth: 4,
            time_budget_ms: 600,
            noise_cp: 35,
            blunder_chance: 0.04,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.0,
                mobility_scale: 1.0,
                king_attack_scale: 1.0,
                endgame_scale: 1.0,
                contempt_cp: 15,
            },
            accent_rgb: [85, 165, 240],
            speed_stat: 0.70,
            tactical_stat: 0.68,
            positional_stat: 0.72,
        }
    }

    pub fn hikaru() -> Self {
        Self {
            id: BotId::Hikaru,
            code: "hikaru",
            name: "Hikaru Nakamura",
            title: "GM Speed Demon",
            bio: "Sangat cepat & agresif! Suka komplikasi taktis, manuver kuda liar, dan tekanan konstan ke Raja lawan.",
            elo: 2800,
            max_depth: 5,
            time_budget_ms: 450,
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
            title: "The GOAT",
            bio: "Presisi posisional & Raja Endgame. Hampir tidak pernah blunder, menolak remis, dan memeras keunggulan mikro.",
            elo: 2882,
            max_depth: 6,
            time_budget_ms: 1100,
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

    pub fn silicon() -> Self {
        Self {
            id: BotId::Silicon,
            code: "silicon",
            name: "RustFish Core",
            title: "Maximum Overdrive",
            bio: "Evaluasi silikon murni tanpa kompromi, pencarian terdalam untuk menguji batas kemampuan kalkulasi pemain.",
            elo: 3100,
            max_depth: 7,
            time_budget_ms: 1800,
            noise_cp: 0,
            blunder_chance: 0.0,
            weights: EvalWeights {
                material_scale: 1.0,
                positional_scale: 1.40,
                mobility_scale: 1.40,
                king_attack_scale: 1.45,
                endgame_scale: 1.60,
                contempt_cp: 25,
            },
            accent_rgb: [175, 95, 250],
            speed_stat: 0.95,
            tactical_stat: 1.00,
            positional_stat: 0.98,
        }
    }
}
