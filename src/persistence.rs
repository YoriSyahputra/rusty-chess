use crate::config::UserConfig;
use std::{collections::HashMap, fs, path::PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SavedMatchSummary {
    pub opponent_name: String,
    pub opponent_elo: u32,
    pub result_label: String,
    pub player_accuracy: f32,
    pub bot_accuracy: f32,
    pub moves_count: usize,
    pub pgn: String,
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
    pub daily_puzzle_completed: bool,
    pub adaptive_jimmy_elo: u32,
    pub custom_engine_elo: u32,
    pub recent_matches: Vec<SavedMatchSummary>,
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
            daily_puzzle_completed: false,
            adaptive_jimmy_elo: 900,
            custom_engine_elo: 1500,
            recent_matches: Vec::new(),
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

    /// Mencatat mahkota kemenangan untuk bot tertentu (menyimpan rekor mahkota tertinggi 0..=3).
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

    /// Total mahkota yang berhasil dikumpulkan dari maksimal 54 (18 bot x 3 mahkota).
    pub fn total_crowns(&self) -> u32 {
        self.bot_crowns.values().map(|&c| (c.min(3)) as u32).sum()
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
        if self.recent_matches.len() > 20 {
            self.recent_matches.truncate(20);
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
        self.daily_puzzle_completed = false;
        self.adaptive_jimmy_elo = 900;
        self.recent_matches.clear();
        self.save();
    }
}
