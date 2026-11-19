use shakmaty::Color;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TimeControl {
    Unlimited,
    Bullet1_0,
    Bullet1_1,
    Bullet2_1,
    Blitz3_0,
    Blitz3_2,
    Blitz5_0,
    Blitz5_5,
    Rapid10_0,
    Rapid15_10,
    Classical30_0,
    Custom { initial_ms: u64, increment_ms: u64 },
}

impl TimeControl {
    pub fn label(&self) -> &'static str {
        match self {
            TimeControl::Unlimited => "Untimed",
            TimeControl::Bullet1_0 => "1 min",
            TimeControl::Bullet1_1 => "1+1",
            TimeControl::Bullet2_1 => "2+1",
            TimeControl::Blitz3_0 => "3 min",
            TimeControl::Blitz3_2 => "3+2",
            TimeControl::Blitz5_0 => "5 min",
            TimeControl::Blitz5_5 => "5+5",
            TimeControl::Rapid10_0 => "10 min",
            TimeControl::Rapid15_10 => "15+10",
            TimeControl::Classical30_0 => "30 min",
            TimeControl::Custom { .. } => "Custom",
        }
    }

    pub fn category(&self) -> &'static str {
        match self {
            TimeControl::Unlimited => "Untimed",
            TimeControl::Bullet1_0 | TimeControl::Bullet1_1 | TimeControl::Bullet2_1 => "Bullet",
            TimeControl::Blitz3_0
            | TimeControl::Blitz3_2
            | TimeControl::Blitz5_0
            | TimeControl::Blitz5_5 => "Blitz",
            TimeControl::Rapid10_0 | TimeControl::Rapid15_10 | TimeControl::Classical30_0 => {
                "Rapid"
            }
            TimeControl::Custom { .. } => "Custom",
        }
    }

    pub fn initial_time_ms(&self) -> Option<u64> {
        match *self {
            TimeControl::Unlimited => None,
            TimeControl::Bullet1_0 | TimeControl::Bullet1_1 => Some(60_000),
            TimeControl::Bullet2_1 => Some(120_000),
            TimeControl::Blitz3_0 | TimeControl::Blitz3_2 => Some(180_000),
            TimeControl::Blitz5_0 | TimeControl::Blitz5_5 => Some(300_000),
            TimeControl::Rapid10_0 => Some(600_000),
            TimeControl::Rapid15_10 => Some(900_000),
            TimeControl::Classical30_0 => Some(1_800_000),
            TimeControl::Custom { initial_ms, .. } => Some(initial_ms),
        }
    }

    pub fn increment_ms(&self) -> u64 {
        match *self {
            TimeControl::Unlimited
            | TimeControl::Bullet1_0
            | TimeControl::Blitz3_0
            | TimeControl::Blitz5_0
            | TimeControl::Rapid10_0
            | TimeControl::Classical30_0 => 0,
            TimeControl::Bullet1_1 | TimeControl::Bullet2_1 => 1_000,
            TimeControl::Blitz3_2 => 2_000,
            TimeControl::Blitz5_5 => 5_000,
            TimeControl::Rapid15_10 => 10_000,
            TimeControl::Custom { increment_ms, .. } => increment_ms,
        }
    }

    /// Faktor skala kecepatan berpikir bot berdasarkan Time Control (Section 7.3 NEW_DESIGN.md)
    pub fn clock_factor(&self) -> f32 {
        match self {
            TimeControl::Bullet1_0 | TimeControl::Bullet1_1 | TimeControl::Bullet2_1 => 0.35,
            TimeControl::Blitz3_0
            | TimeControl::Blitz3_2
            | TimeControl::Blitz5_0
            | TimeControl::Blitz5_5 => 0.65,
            TimeControl::Rapid10_0
            | TimeControl::Rapid15_10
            | TimeControl::Classical30_0
            | TimeControl::Unlimited => 1.0,
            TimeControl::Custom { initial_ms, .. } => {
                if *initial_ms <= 120_000 {
                    0.35
                } else if *initial_ms <= 300_000 {
                    0.65
                } else {
                    1.0
                }
            }
        }
    }

    pub fn all_presets() -> &'static [TimeControl] {
        &[
            TimeControl::Unlimited,
            TimeControl::Bullet1_0,
            TimeControl::Bullet1_1,
            TimeControl::Bullet2_1,
            TimeControl::Blitz3_2,
            TimeControl::Blitz5_0,
            TimeControl::Blitz5_5,
            TimeControl::Rapid10_0,
            TimeControl::Rapid15_10,
            TimeControl::Classical30_0,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct ChessClock {
    pub time_control: TimeControl,
    pub white_remaining_ms: u64,
    pub black_remaining_ms: u64,
    pub increment_ms: u64,
    pub active_side: Option<Color>,
    pub last_tick: Option<Instant>,
    pub is_unlimited: bool,
}

impl ChessClock {
    pub fn new(time_control: TimeControl) -> Self {
        let initial = time_control.initial_time_ms().unwrap_or(0);
        let is_unlimited = time_control.initial_time_ms().is_none();
        Self {
            time_control,
            white_remaining_ms: initial,
            black_remaining_ms: initial,
            increment_ms: time_control.increment_ms(),
            active_side: None,
            last_tick: None,
            is_unlimited,
        }
    }

    pub fn start(&mut self, side: Color) {
        self.active_side = Some(side);
        self.last_tick = Some(Instant::now());
    }

    pub fn stop(&mut self) {
        let _ = self.tick();
        self.active_side = None;
        self.last_tick = None;
    }

    /// Memperbarui sisa waktu sisi aktif; mengembalikan `Some(winner_color)` jika waktu habis.
    pub fn tick(&mut self) -> Option<Color> {
        let side = self.active_side?;
        let now = Instant::now();
        if let Some(last) = self.last_tick {
            let elapsed = now.saturating_duration_since(last).as_millis() as u64;
            self.last_tick = Some(now);
            if !self.is_unlimited {
                match side {
                    Color::White => {
                        self.white_remaining_ms = self.white_remaining_ms.saturating_sub(elapsed);
                        if self.white_remaining_ms == 0 {
                            self.active_side = None;
                            return Some(Color::Black);
                        }
                    }
                    Color::Black => {
                        self.black_remaining_ms = self.black_remaining_ms.saturating_sub(elapsed);
                        if self.black_remaining_ms == 0 {
                            self.active_side = None;
                            return Some(Color::White);
                        }
                    }
                }
            }
        } else {
            self.last_tick = Some(now);
        }
        None
    }

    /// Mengalihkan giliran jam ke `new_active_side` dan mengembalikan waktu yang dihabiskan (ms).
    pub fn switch_turn(&mut self, new_active_side: Color) -> u64 {
        let now = Instant::now();
        let elapsed = self
            .last_tick
            .map(|last| now.saturating_duration_since(last).as_millis() as u64)
            .unwrap_or(0);

        if let Some(prev_side) = self.active_side {
            if !self.is_unlimited {
                match prev_side {
                    Color::White => {
                        self.white_remaining_ms = self
                            .white_remaining_ms
                            .saturating_sub(elapsed)
                            .saturating_add(self.increment_ms);
                    }
                    Color::Black => {
                        self.black_remaining_ms = self
                            .black_remaining_ms
                            .saturating_sub(elapsed)
                            .saturating_add(self.increment_ms);
                    }
                }
            }
        }

        self.active_side = Some(new_active_side);
        self.last_tick = Some(now);
        elapsed
    }

    pub fn remaining_ms(&self, side: Color) -> u64 {
        match side {
            Color::White => self.white_remaining_ms,
            Color::Black => self.black_remaining_ms,
        }
    }

    pub fn format_time(&self, side: Color) -> String {
        if self.is_unlimited {
            return "∞".to_string();
        }
        let ms = self.remaining_ms(side);
        let total_secs = ms / 1000;
        let mins = total_secs / 60;
        let secs = total_secs % 60;
        if mins == 0 && secs < 20 {
            let tenths = (ms % 1000) / 100;
            format!("00:{secs:02}.{tenths}")
        } else {
            format!("{mins:02}:{secs:02}")
        }
    }
}
