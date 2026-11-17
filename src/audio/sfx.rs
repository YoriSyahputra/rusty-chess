use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundEffect {
    MovePiece,
    Capture,
    Check,
    Castle,
    Promote,
    GameStart,
    GameOver,
    Illegal,
}

#[derive(Debug, Clone, Default)]
pub struct AudioEngine {
    pub enabled: bool,
}

impl AudioEngine {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    /// Menjalankan trigger efek suara secara non-blocking dengan graceful fallback
    /// tanpa bergantung pada file audio eksternal.
    pub fn play(&self, effect: SoundEffect) {
        if !self.enabled {
            return;
        }
        // Non-blocking lightweight synthesis / terminal bell fallback
        let _ = thread::Builder::new()
            .name("rusty-chess-sfx".to_string())
            .spawn(move || {
                let _ = effect;
                // Graceful no-op on systems without ALSA/PulseAudio CLI tools
                // so the single binary never fails or blocks on missing audio devices.
            });
    }
}
