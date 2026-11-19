use crate::{audio::sfx::SoundEffect, config::SoundTheme};
use std::f32::consts::PI;

pub const SAMPLE_RATE: u32 = 22_050;

/// Menghasilkan buffer file WAV 16-bit mono PCM secara prosedural murni di memori (tanpa aset eksternal)
/// sesuai efek suara & tema suara pilihan (Features #16 & #17).
pub fn synthesize_wav_bytes(effect: SoundEffect, theme: SoundTheme) -> Vec<u8> {
    let samples = synthesize_pcm_samples(effect, theme);
    encode_wav_mono_16bit(&samples, SAMPLE_RATE)
}

pub fn synthesize_pcm_samples(effect: SoundEffect, theme: SoundTheme) -> Vec<i16> {
    let duration_sec = match effect {
        SoundEffect::MovePiece => 0.065,
        SoundEffect::Capture => 0.095,
        SoundEffect::Castle => 0.145,
        SoundEffect::Check => 0.160,
        SoundEffect::Promote => 0.210,
        SoundEffect::GameStart => 0.220,
        SoundEffect::GameOver => 0.360,
        SoundEffect::Illegal => 0.110,
        SoundEffect::LowTimeTick => 0.032,
    };

    let num_samples = (SAMPLE_RATE as f32 * duration_sec) as usize;
    let mut out = Vec::with_capacity(num_samples);
    let mut lcg: u32 = 0x1234_5678;

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        lcg = lcg.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let white_noise = (((lcg >> 16) as f32) / 32768.0) - 1.0;

        let raw = match theme {
            SoundTheme::Retro8Bit => synth_retro_8bit(effect, t),
            SoundTheme::MarbleClink => synth_marble(effect, t, white_noise),
            SoundTheme::TournamentCrisp => synth_wood(effect, t, white_noise, 1.35),
            SoundTheme::ClassicWood => synth_wood(effect, t, white_noise, 1.0),
        };

        let clamped = (raw * 0.65).clamp(-0.98, 0.98);
        out.push((clamped * i16::MAX as f32) as i16);
    }

    out
}

fn synth_wood(effect: SoundEffect, t: f32, noise: f32, pitch_mult: f32) -> f32 {
    match effect {
        SoundEffect::MovePiece => {
            let env = (-t * 58.0).exp();
            let body = (2.0 * PI * 195.0 * pitch_mult * t).sin() * 0.72
                + (2.0 * PI * 420.0 * pitch_mult * t).sin() * 0.22;
            let click = noise * (-t * 210.0).exp() * 0.45;
            (body + click) * env
        }
        SoundEffect::Capture => {
            let env = (-t * 42.0).exp();
            let body = (2.0 * PI * 240.0 * pitch_mult * t).sin() * 0.55
                + (2.0 * PI * 680.0 * pitch_mult * t).sin() * 0.35;
            let snap = noise * (-t * 145.0).exp() * 0.70;
            (body + snap) * env
        }
        SoundEffect::Castle => {
            let hit1 = if t < 0.07 {
                let e = (-t * 60.0).exp();
                ((2.0 * PI * 185.0 * pitch_mult * t).sin() + noise * 0.4 * (-t * 190.0).exp()) * e
            } else {
                0.0
            };
            let hit2 = if t >= 0.065 {
                let dt = t - 0.065;
                let e = (-dt * 55.0).exp();
                ((2.0 * PI * 230.0 * pitch_mult * dt).sin() + noise * 0.4 * (-dt * 190.0).exp()) * e
            } else {
                0.0
            };
            (hit1 + hit2) * 0.85
        }
        SoundEffect::Check => {
            let env = (-t * 18.0).exp();
            let f1 = 660.0 * pitch_mult;
            let f2 = 990.0 * pitch_mult;
            ((2.0 * PI * f1 * t).sin() * 0.6 + (2.0 * PI * f2 * t).sin() * 0.4) * env
        }
        SoundEffect::Promote => {
            let freq = if t < 0.095 { 523.25 } else { 783.99 };
            let dt = if t < 0.095 { t } else { t - 0.095 };
            (2.0 * PI * freq * dt).sin() * (-dt * 16.0).exp() * 0.8
        }
        SoundEffect::GameStart => {
            let env = (-t * 12.0).exp();
            ((2.0 * PI * 392.0 * t).sin() * 0.5 + (2.0 * PI * 523.25 * t).sin() * 0.5) * env
        }
        SoundEffect::GameOver => {
            let env = (-t * 6.5).exp();
            ((2.0 * PI * 261.63 * t).sin() * 0.35
                + (2.0 * PI * 329.63 * t).sin() * 0.35
                + (2.0 * PI * 392.0 * t).sin() * 0.30)
                * env
        }
        SoundEffect::Illegal => {
            let env = (-t * 28.0).exp();
            (2.0 * PI * 115.0 * t).sin() * env * 0.75
        }
        SoundEffect::LowTimeTick => {
            let env = (-t * 130.0).exp();
            ((2.0 * PI * 1150.0 * t).sin() * 0.65 + noise * 0.35) * env * 0.55
        }
    }
}

fn synth_marble(effect: SoundEffect, t: f32, noise: f32) -> f32 {
    let base = match effect {
        SoundEffect::MovePiece => 1280.0,
        SoundEffect::Capture => 1680.0,
        SoundEffect::Castle => 1120.0,
        SoundEffect::Check => 2100.0,
        SoundEffect::Promote => 1760.0,
        SoundEffect::GameStart => 880.0,
        SoundEffect::GameOver => 660.0,
        SoundEffect::Illegal => 240.0,
        SoundEffect::LowTimeTick => 1950.0,
    };
    let env = (-t * 38.0).exp();
    ((2.0 * PI * base * t).sin() * 0.65
        + (2.0 * PI * base * 1.49 * t).sin() * 0.28
        + noise * (-t * 260.0).exp() * 0.25)
        * env
}

fn synth_retro_8bit(effect: SoundEffect, t: f32) -> f32 {
    let freq = match effect {
        SoundEffect::MovePiece => 330.0,
        SoundEffect::Capture => 440.0 - t * 1400.0,
        SoundEffect::Castle => if t < 0.07 { 293.66 } else { 392.0 },
        SoundEffect::Check => if t < 0.08 { 659.25 } else { 880.0 },
        SoundEffect::Promote => if t < 0.10 { 523.25 } else { 1046.5 },
        SoundEffect::GameStart => 440.0,
        SoundEffect::GameOver => if t < 0.12 { 523.25 } else if t < 0.24 { 659.25 } else { 783.99 },
        SoundEffect::Illegal => 130.0,
        SoundEffect::LowTimeTick => 980.0,
    };
    let wave = if (2.0 * PI * freq.max(80.0) * t).sin() >= 0.0 {
        0.55
    } else {
        -0.55
    };
    wave * (-t * 14.0).exp()
}

pub fn encode_wav_mono_16bit(samples: &[i16], sample_rate: u32) -> Vec<u8> {
    let data_len = (samples.len() * 2) as u32;
    let riff_len = 36 + data_len;
    let byte_rate = sample_rate * 2;

    let mut buf = Vec::with_capacity(44 + data_len as usize);
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&riff_len.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // PCM chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&1u16.to_le_bytes()); // Mono
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    buf.extend_from_slice(&2u16.to_le_bytes()); // Block align
    buf.extend_from_slice(&16u16.to_le_bytes()); // Bits per sample
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_len.to_le_bytes());

    for &s in samples {
        buf.extend_from_slice(&s.to_le_bytes());
    }
    buf
}
