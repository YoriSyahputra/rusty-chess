use crate::domain::{format_fen, parse_uci};
use shakmaty::{Chess, Color, Move, Position};
use std::{
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
};

/// Mencari binary mesin UCI (seperti Stockfish) yang sudah terpasang di sistem Linux.
pub fn detect_system_uci_engine() -> Option<String> {
    let candidates = [
        "/usr/games/stockfish",
        "/usr/bin/stockfish",
        "/usr/local/bin/stockfish",
        "/usr/games/gnuchess",
        "/usr/bin/gnuchess",
    ];
    for p in candidates {
        if Path::new(p).exists() {
            return Some(p.to_string());
        }
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for dir in path_var.split(':') {
            for bin in ["stockfish", "komodo", "lc0"] {
                let full = Path::new(dir).join(bin);
                if full.exists() {
                    return Some(full.to_string_lossy().to_string());
                }
            }
        }
    }
    None
}

/// Mem-parse skor `score cp <N>` atau `score mate <M>` dari baris `info` UCI
/// dan mengonversinya ke perspektif Putih.
pub fn parse_uci_score_cp(info_line: &str, side_to_move: Color) -> Option<i32> {
    let tokens: Vec<&str> = info_line.split_whitespace().collect();
    for i in 0..tokens.len().saturating_sub(2) {
        if tokens[i] == "score" {
            let kind = tokens[i + 1];
            let val_str = tokens[i + 2];
            if kind == "cp" {
                if let Ok(cp) = val_str.parse::<i32>() {
                    return Some(match side_to_move {
                        Color::White => cp,
                        Color::Black => -cp,
                    });
                }
            } else if kind == "mate" {
                if let Ok(m) = val_str.parse::<i32>() {
                    let mate_cp = if m > 0 { 29_000 - m * 10 } else { -29_000 - m * 10 };
                    return Some(match side_to_move {
                        Color::White => mate_cp,
                        Color::Black => -mate_cp,
                    });
                }
            }
        }
    }
    None
}

/// Mem-parse depth dari baris `info depth <d> ...` UCI.
pub fn parse_uci_depth(info_line: &str) -> Option<u8> {
    let tokens: Vec<&str> = info_line.split_whitespace().collect();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i] == "depth" {
            if let Ok(d) = tokens[i + 1].parse::<u8>() {
                return Some(d);
            }
        }
    }
    None
}

/// Menguji koneksi ke binary UCI eksternal dan mengembalikan `id name` bila berhasil.
pub fn test_uci_connection(engine_path: &str) -> Result<String, String> {
    let trimmed = engine_path.trim();
    if trimmed.is_empty() {
        return Err("Engine path is empty.".to_string());
    }

    let mut child = Command::new(trimmed)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to launch '{trimmed}': {e}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let _ = writeln!(stdin, "uci");
        let _ = writeln!(stdin, "quit");
        let _ = stdin.flush();
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Engine stdout unavailable.".to_string())?;
    let reader = BufReader::new(stdout);
    let mut engine_name = String::new();
    let mut saw_uciok = false;

    for line in reader.lines().map_while(Result::ok) {
        if let Some(rest) = line.strip_prefix("id name ") {
            engine_name = rest.trim().to_string();
        }
        if line.trim() == "uciok" {
            saw_uciok = true;
            break;
        }
    }

    let _ = child.wait();

    if saw_uciok {
        if engine_name.is_empty() {
            Ok("UCI Engine Ready".to_string())
        } else {
            Ok(engine_name)
        }
    } else {
        Err("Process did not respond with 'uciok'.".to_string())
    }
}

/// Meminta langkah terbaik dari binary UCI eksternal (misal Stockfish) untuk posisi `pos`.
/// Mengembalikan `(best_move, eval_cp_white, depth_reached)`.
pub fn query_uci_best_move(
    engine_path: &str,
    pos: &Chess,
    movetime_ms: u64,
) -> Result<(Move, i32, u8), String> {
    let trimmed = engine_path.trim();
    if trimmed.is_empty() {
        return Err("Empty UCI path".to_string());
    }

    let mut child = Command::new(trimmed)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to spawn UCI engine: {e}"))?;

    let fen = format_fen(pos);
    let mt = movetime_ms.clamp(80, 2500);

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Engine stdin unavailable".to_string())?;
    writeln!(stdin, "uci").map_err(|e| e.to_string())?;
    writeln!(stdin, "isready").map_err(|e| e.to_string())?;
    writeln!(stdin, "position fen {fen}").map_err(|e| e.to_string())?;
    writeln!(stdin, "go movetime {mt}").map_err(|e| e.to_string())?;
    stdin.flush().map_err(|e| e.to_string())?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Engine stdout unavailable".to_string())?;
    let reader = BufReader::new(stdout);

    let turn = pos.turn();
    let mut latest_eval_w: i32 = 0;
    let mut latest_depth: u8 = 1;
    let mut best_uci: Option<String> = None;

    for line in reader.lines().map_while(Result::ok) {
        if line.starts_with("info ") {
            if let Some(cp) = parse_uci_score_cp(&line, turn) {
                latest_eval_w = cp;
            }
            if let Some(d) = parse_uci_depth(&line) {
                latest_depth = d;
            }
        } else if let Some(rest) = line.strip_prefix("bestmove ") {
            if let Some(tok) = rest.split_whitespace().next() {
                if tok != "(none)" {
                    best_uci = Some(tok.to_string());
                }
            }
            break;
        }
    }

    let _ = writeln!(stdin, "quit");
    let _ = stdin.flush();
    drop(stdin);
    let _ = child.wait();

    let uci_str = best_uci.ok_or_else(|| "UCI engine returned no bestmove".to_string())?;
    let mv = parse_uci(pos, &uci_str)?;
    Ok((mv, latest_eval_w, latest_depth))
}
