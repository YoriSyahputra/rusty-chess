use crate::{audio::synth::synthesize_wav_bytes, config::SoundTheme};
use std::{
    io::Write,
    process::{Command, Stdio},
    sync::mpsc::{self, Sender},
    thread,
};

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
    LowTimeTick,
}

#[derive(Debug, Clone)]
pub struct AudioEngine {
    pub enabled: bool,
    pub theme: SoundTheme,
    tx: Option<Sender<(SoundEffect, SoundTheme)>>,
}

impl Default for AudioEngine {
    fn default() -> Self {
        Self::new(true)
    }
}

impl AudioEngine {
    pub fn new(enabled: bool) -> Self {
        let (tx, rx) = mpsc::channel::<(SoundEffect, SoundTheme)>();
        let backend = detect_linux_audio_player();

        let _ = thread::Builder::new()
            .name("rusty-chess-audio".to_string())
            .spawn(move || {
                while let Ok((effect, theme)) = rx.recv() {
                    if let Some(cmd_name) = backend {
                        let wav = synthesize_wav_bytes(effect, theme);
                        play_wav_via_pipe(cmd_name, &wav);
                    }
                }
            });

        Self {
            enabled,
            theme: SoundTheme::ClassicWood,
            tx: Some(tx),
        }
    }

    pub fn play(&self, effect: SoundEffect) {
        if !self.enabled {
            return;
        }
        if let Some(tx) = &self.tx {
            let _ = tx.send((effect, self.theme));
        }
    }
}

fn detect_linux_audio_player() -> Option<&'static str> {
    for bin in ["aplay", "paplay", "pw-play"] {
        if Command::new("which")
            .arg(bin)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|s| s.success())
        {
            return Some(bin);
        }
    }
    None
}

fn play_wav_via_pipe(player: &str, wav_bytes: &[u8]) {
    let mut cmd = Command::new(player);
    match player {
        "aplay" => {
            cmd.arg("-q").arg("-");
        }
        "paplay" => {
            cmd.arg("--file-format=wav");
        }
        _ => {
            cmd.arg("-");
        }
    }
    if let Ok(mut child) = cmd
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(wav_bytes);
        }
        let _ = child.wait();
    }
}
