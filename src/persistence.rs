use crate::config::UserConfig;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct SavedMatchSummary {
    pub opponent_name: String,
    pub opponent_elo: u32,
    pub result_label: String,
    pub opening_name: String,
    pub player_accuracy: f32,
    pub bot_accuracy: f32,
    pub moves_count: usize,
    pub pgn: String,
}

impl Default for SavedMatchSummary {
    fn default() -> Self {
        Self {
            opponent_name: "Bot".to_string(),
            opponent_elo: 1200,
            result_label: "*".to_string(),
            opening_name: "Unclassified Opening".to_string(),
            player_accuracy: 75.0,
            bot_accuracy: 75.0,
            moves_count: 0,
            pgn: String::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpeningStat {
    pub eco: String,
    pub name: String,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
}

impl OpeningStat {
    pub fn total_games(&self) -> u32 {
        self.wins + self.draws + self.losses
    }

    pub fn win_rate_pct(&self) -> f32 {
        let total = self.total_games();
        if total == 0 {
            return 0.0;
        }
        ((self.wins as f32 + self.draws as f32 * 0.5) / total as f32) * 100.0
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MistakeVaultEntry {
    pub id: u64,
    pub fen: String,
    pub side_is_white: bool,
    pub played_san: String,
    pub best_san: String,
    pub best_uci: String,
    pub classification: String,
    pub opponent_name: String,
    pub mastery_streak: u8,
    pub total_attempts: u32,
}

impl MistakeVaultEntry {
    pub fn is_mastered(&self) -> bool {
        self.mastery_streak >= 3
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SkillDnaMetrics {
    pub opening: f32,
    pub tactics: f32,
    pub endgame: f32,
    pub time_mgmt: f32,
    pub attack: f32,
    pub resilience: f32,
    pub archetype: &'static str,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct UserProfile {
    pub config: UserConfig,
    /// Map dari bot code (misal `"martin"`, `"magnus"`) ke jumlah mahkota tertinggi (`0..=3`)
    pub bot_crowns: HashMap<String, u8>,
    pub wins: u32,
    pub draws: u32,
    pub losses: u32,
    pub puzzles_solved: u32,
    pub puzzle_rating: u32,
    pub puzzle_streak: u32,
    pub puzzle_rush_best_timed: u32,
    pub puzzle_rush_best_survival: u32,
    pub vision_best_score: u32,
    pub daily_puzzle_completed: bool,
    pub adaptive_jimmy_elo: u32,
    pub custom_engine_elo: u32,
    pub recent_matches: Vec<SavedMatchSummary>,
    pub opening_stats: HashMap<String, OpeningStat>,
    pub mistake_vault: Vec<MistakeVaultEntry>,
    pub next_mistake_id: u64,
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            config: UserConfig::default(),
            bot_crowns: HashMap::new(),
            wins: 0,
            draws: 0,
            losses: 0,
            puzzles_solved: 0,
            puzzle_rating: 1200,
            puzzle_streak: 0,
            puzzle_rush_best_timed: 0,
            puzzle_rush_best_survival: 0,
            vision_best_score: 0,
            daily_puzzle_completed: false,
            adaptive_jimmy_elo: 900,
            custom_engine_elo: 1500,
            recent_matches: Vec::new(),
            opening_stats: HashMap::new(),
            mistake_vault: Vec::new(),
            next_mistake_id: 1,
        }
    }
}

impl UserProfile {
    pub fn profile_path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        let mut path = PathBuf::from(home);
        path.push(".local");
        path.push("share");
        path.push("rusty-chess");
        path.push("profile.json");
        Some(path)
    }

    pub fn load() -> Self {
        if let Some(path) = Self::profile_path() {
            if let Ok(raw) = fs::read_to_string(&path) {
                if let Ok(profile) = serde_json::from_str::<UserProfile>(&raw) {
                    return profile;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        if let Some(path) = Self::profile_path() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Ok(serialized) = serde_json::to_string_pretty(self) {
                let _ = fs::write(path, serialized);
            }
        }
    }

    pub fn crowns_for(&self, bot_code: &str) -> u8 {
        self.bot_crowns.get(bot_code).copied().unwrap_or(0).min(3)
    }

    pub fn record_bot_crown(&mut self, bot_code: &str, crowns: u8) -> bool {
        let clamped = crowns.min(3);
        let prev = self.crowns_for(bot_code);
        if clamped > prev {
            self.bot_crowns.insert(bot_code.to_string(), clamped);
            self.save();
            true
        } else {
            false
        }
    }

    pub fn total_crowns(&self) -> u32 {
        self.bot_crowns.values().map(|&c| (c.min(3)) as u32).sum()
    }

    pub fn record_opening_result(&mut self, eco: &str, name: &str, won: Option<bool>) {
        let key = format!("{eco}: {name}");
        let entry = self
            .opening_stats
            .entry(key)
            .or_insert_with(|| OpeningStat {
                eco: eco.to_string(),
                name: name.to_string(),
                wins: 0,
                draws: 0,
                losses: 0,
            });
        match won {
            Some(true) => entry.wins += 1,
            Some(false) => entry.losses += 1,
            None => entry.draws += 1,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_mistake_entry(
        &mut self,
        fen: String,
        side_is_white: bool,
        played_san: String,
        best_san: String,
        best_uci: String,
        classification: String,
        opponent_name: String,
    ) {
        // Avoid duplicating identical FEN + best_uci already in vault
        if self
            .mistake_vault
            .iter()
            .any(|m| m.fen == fen && m.best_uci == best_uci)
        {
            return;
        }
        let id = self.next_mistake_id;
        self.next_mistake_id = self.next_mistake_id.wrapping_add(1).max(1);
        self.mistake_vault.insert(
            0,
            MistakeVaultEntry {
                id,
                fen,
                side_is_white,
                played_san,
                best_san,
                best_uci,
                classification,
                opponent_name,
                mastery_streak: 0,
                total_attempts: 0,
            },
        );
        if self.mistake_vault.len() > 60 {
            self.mistake_vault.truncate(60);
        }
    }

    pub fn record_mistake_drill_outcome(&mut self, mistake_id: u64, solved_cleanly: bool) {
        if let Some(item) = self.mistake_vault.iter_mut().find(|m| m.id == mistake_id) {
            item.total_attempts += 1;
            if solved_cleanly {
                item.mastery_streak = (item.mastery_streak + 1).min(3);
            } else {
                item.mastery_streak = 0;
            }
            self.save();
        }
    }

    pub fn active_mistakes_count(&self) -> usize {
        self.mistake_vault
            .iter()
            .filter(|m| !m.is_mastered())
            .count()
    }

    /// Menghitung 6 dimensi "Your Chess DNA" untuk Hexagonal Skill Radar Chart.
    pub fn compute_skill_dna(&self) -> SkillDnaMetrics {
        let total_games = (self.wins + self.draws + self.losses) as f32;
        let avg_acc = if self.recent_matches.is_empty() {
            68.0
        } else {
            self.recent_matches
                .iter()
                .map(|m| m.player_accuracy)
                .sum::<f32>()
                / (self.recent_matches.len() as f32)
        };

        let win_rate = if total_games > 0.0 {
            ((self.wins as f32 + self.draws as f32 * 0.5) / total_games).clamp(0.15, 1.0)
        } else {
            0.55
        };

        let opening_variety = ((self.opening_stats.len() as f32) / 10.0).clamp(0.0, 0.25);
        let opening = ((avg_acc / 100.0) * 0.72 + opening_variety).clamp(0.20, 0.98);

        let puzzle_norm = ((self.puzzle_rating as f32 - 600.0) / 1800.0).clamp(0.20, 0.98);
        let tactics = (puzzle_norm * 0.65 + (avg_acc / 100.0) * 0.35).clamp(0.20, 0.98);

        let crown_norm = (self.total_crowns() as f32 / 72.0).clamp(0.0, 1.0);
        let endgame = (0.35 + win_rate * 0.40 + crown_norm * 0.25).clamp(0.20, 0.98);

        let avg_moves = if self.recent_matches.is_empty() {
            28.0
        } else {
            self.recent_matches
                .iter()
                .map(|m| m.moves_count as f32)
                .sum::<f32>()
                / self.recent_matches.len() as f32
        };
        let time_mgmt = (0.45 + (avg_acc / 100.0) * 0.35 + (self.puzzle_rush_best_timed as f32 / 40.0).min(0.20))
            .clamp(0.25, 0.98);

        let attack = (0.35 + win_rate * 0.35 + (1.0 - (avg_moves / 70.0).clamp(0.0, 0.6)) * 0.30)
            .clamp(0.22, 0.98);

        let mastered_mistakes = self.mistake_vault.iter().filter(|m| m.is_mastered()).count() as f32;
        let blunder_penalty = (self.active_mistakes_count() as f32 * 0.025).min(0.30);
        let resilience = ((avg_acc / 100.0) * 0.80 + (mastered_mistakes * 0.04).min(0.20) - blunder_penalty)
            .clamp(0.20, 0.98);

        let archetype = if avg_acc >= 90.0 && crown_norm >= 0.5 {
            "Universal Grandmaster"
        } else if tactics >= opening && tactics >= endgame && tactics >= attack {
            "Tactical Assassin"
        } else if endgame >= opening && endgame >= tactics {
            "Endgame Grinder"
        } else if opening >= attack && opening >= resilience {
            "Opening Theorist"
        } else if attack >= resilience {
            "Fearless Attacker"
        } else {
            "Solid Positionalist"
        };

        SkillDnaMetrics {
            opening,
            tactics,
            endgame,
            time_mgmt,
            attack,
            resilience,
            archetype,
        }
    }

    pub fn record_match_result(
        &mut self,
        won: Option<bool>,
        bot_code: &str,
        awarded_crowns: u8,
        summary: SavedMatchSummary,
    ) {
        match won {
            Some(true) => {
                self.wins += 1;
                if awarded_crowns > 0 {
                    let _ = self.record_bot_crown(bot_code, awarded_crowns);
                }
            }
            Some(false) => {
                self.losses += 1;
            }
            None => {
                self.draws += 1;
            }
        }
        self.recent_matches.insert(0, summary);
        if self.recent_matches.len() > 25 {
            self.recent_matches.truncate(25);
        }
        self.save();
    }

    pub fn reset_progress(&mut self) {
        self.bot_crowns.clear();
        self.wins = 0;
        self.draws = 0;
        self.losses = 0;
        self.puzzles_solved = 0;
        self.puzzle_rating = 1200;
        self.puzzle_streak = 0;
        self.puzzle_rush_best_timed = 0;
        self.puzzle_rush_best_survival = 0;
        self.vision_best_score = 0;
        self.daily_puzzle_completed = false;
        self.adaptive_jimmy_elo = 900;
        self.recent_matches.clear();
        self.opening_stats.clear();
        self.mistake_vault.clear();
        self.next_mistake_id = 1;
        self.save();
    }
}
