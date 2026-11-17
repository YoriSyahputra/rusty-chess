# SPEC.md — RustyChess (`rusty-chess`)

**Native Linux High-Performance Desktop Chess Game & AI Persona Engine**

---

## 1. Executive Summary & Core Constraints

### 1.1 Project Vision

`rusty-chess` adalah aplikasi permainan catur desktop native untuk sistem operasi Linux (Debian/Ubuntu) yang dibangun 100% menggunakan bahasa pemrograman **Rust** dan GUI framework **`eframe` / `egui`** (immediate-mode native OpenGL/wgpu rendering). Aplikasi ini menghadirkan pengalaman bermain catur melawan AI dengan kepribadian/persona khas (seperti **Magnus Carlsen**, **Hikaru Nakamura**, **Bot Warung**, hingga **Martin Si Pemula**) lengkap dengan dialog interaktif, evaluasi bar _real-time_, analisis langkah, dan sistem pengemasan `.deb` mandiri.

### 1.2 Strict Technical Constraints (WAJIB DIPATUHI AGENT)

1. **NO Web Wrappers**: Dilarang keras menggunakan Electron, Tauri, WebView, HTML, CSS, atau JavaScript. Seluruh tampilan dirender secara native menggunakan `egui` primitives (`egui::Painter`, vector shapes, dan embedded textures).
2. **Low Memory & Low CPU Footprint**:
   - Konsumsi RAM saat runtime (RSS) harus di bawah **45 MB**.
   - Transposition Table dibatasi maksimal **16 MB** (fixed-size allocation).
   - Saat tidak ada animasi atau proses berpikir AI, GUI berjalan pada _reactive mode_ (hanya me-render ulang saat ada input mouse/keyboard atau update dari thread AI), namun langsung mendukung **60–144 FPS** mulus saat pemain melakukan _drag-and-drop_ bidak.
   - Konfigurasi kompilasi (`[profile.dev]`) harus ramah terhadap mesin dengan RAM terbatas (4 GB RAM): batasi `debug = 1` agar linker tidak mengalami Out-Of-Memory (OOM).
3. **Zero External Runtime Asset Dependencies**:
   - Aplikasi harus berupa **Single Self-Contained Binary**.
   - Seluruh aset visual (ikon bidak SVG/vektor, font, dan avatar bot) wajib di-_embed_ ke dalam binary atau digambar secara prosedural menggunakan `egui::Painter` / embedded SVG bytes (`include_bytes!`) sehingga binary tetap berjalan sempurna meskipun dipindah ke direktori mana pun tanpa folder `assets/`.
4. **Non-Blocking UI Architecture**:
   - Kalkulasi AI Engine (Minimax / Negamax) **DILARANG** berjalan di thread utama GUI.
   - AI wajib berjalan di _background worker thread_ menggunakan `std::sync::mpsc` atau `crossbeam-channel` agar UI tidak pernah _freeze_ atau _Not Responding_ ketika bot berpikir pada _depth_ tinggi.

---

## 2. Project Directory Architecture

Antigravity wajib membuat dan menjaga struktur direktori modular berikut secara presisi:

```text
rusty-chess/
├── .cargo/
│   └── config.toml                 # Compiler flags & linker optimizations
├── Cargo.toml                      # Dependencies & profile configurations
├── SPEC.md                         # Dokumen spesifikasi utama ini
├── Makefile                        # Shortcut build, run, test, deb, clean
├── scripts/
│   └── build-deb.sh                # Script otomatis pembuat paket .deb
├── packaging/
│   └── debian/
│       ├── DEBIAN/
│       │   ├── control             # Metadata paket Debian
│       │   └── postinst            # Post-install hook untuk desktop database
│       └── usr/
│           └── share/
│               ├── applications/
│               │   └── rusty-chess.desktop
│               └── icons/
│                   └── hicolor/
│                       └── scalable/
│                           └── apps/
│                               └── rusty-chess.svg
├── tests/
│   ├── engine_search_test.rs       # Pengujian taktik (Mate in 1, Mate in 2, Fork)
│   ├── eval_symmetry_test.rs       # Pengujian simetri fungsi evaluasi posisi
│   └── persona_behavior_test.rs    # Pengujian karakteristik bot (Magnus vs Beginner)
└── src/
    ├── main.rs                     # Entrypoint, eframe NativeOptions, window setup
    ├── app.rs                      # Struct utama RustyChessApp & eframe::App impl
    ├── config.rs                   # State pengaturan user (Theme, Sound, Orientation)
    │
    ├── domain/                     # LAYER 1: Pure Chess Rules & State Management
    │   ├── mod.rs
    │   ├── types.rs                # Square, File, Rank, PieceKind, Side, MoveRecord
    │   ├── game_state.rs           # Wrapper di atas `shakmaty::Chess`, history, undo/redo
    │   ├── notation.rs             # Parser & Formatter SAN (e4, Nf3, O-O) & FEN
    │   └── clock.rs                # Chess clock timer (Bullet, Blitz, Rapid, Unlimited)
    │
    ├── engine/                     # LAYER 2: AI Search, Evaluation & Bot Personas
    │   ├── mod.rs
    │   ├── worker.rs               # Background thread loop & MPSC channel protocol
    │   ├── eval/
    │   │   ├── mod.rs              # Master evaluation orchestrator (Tapered Eval)
    │   │   ├── material.rs         # Skor material & imbalance (Bishop pair bonus)
    │   │   ├── pst.rs              # Piece-Square Tables (Midgame & Endgame matrices)
    │   │   ├── mobility.rs         # Evaluasi aktivitas perwira & kontrol pusat
    │   │   └── king_safety.rs      # Evaluasi pawn shield & open file di depan Raja
    │   ├── search/
    │   │   ├── mod.rs
    │   │   ├── negamax.rs          # Iterative Deepening + Negamax Alpha-Beta Pruning
    │   │   ├── quiescence.rs       # Quiescence search untuk mencegah horizon effect
    │   │   ├── ordering.rs         # Move ordering: TT Move, MVV-LVA, Killer, History
    │   │   └── tt.rs               # Zobrist Hash Transposition Table (16 MB fixed)
    │   └── bot/
    │       ├── mod.rs
    │       ├── persona.rs          # Struct BotPersona, parameter bobot AI, & roster
    │       └── dialogue.rs         # Sistem dialog kontekstual & reaksi emosional bot
    │
    ├── audio/                      # LAYER 3: Lightweight Sound Effect Synthesizer
    │   ├── mod.rs
    │   └── sfx.rs                  # Non-blocking audio trigger (opsional/graceful fallback)
    │
    └── ui/                         # LAYER 4: Native Immediate-Mode GUI (`egui`)
        ├── mod.rs
        ├── theme.rs                # 4 Palet warna papan & dark/light UI styling
        ├── piece_renderer.rs       # Custom crisp vector/SVG procedural piece painter
        ├── top_bar.rs              # Menu bar: New Game, Flip, FEN/PGN modal, FPS/RAM
        ├── eval_bar.rs             # Vertical advantage bar dengan animasi smooth lerp
        ├── board_view/
        │   ├── mod.rs
        │   ├── grid.rs             # Render 64 petak, koordinat A-H/1-8, highlight petak
        │   ├── interaction.rs      # Drag-and-drop, click-to-move, right-click arrows
        │   └── promotion_popup.rs  # Overlay pemilihan promosi pion (Q, R, B, N)
        └── side_panel/
            ├── mod.rs
            ├── bot_card.rs         # Avatar bot, nama, ELO badge, & balon chat reaktif
            ├── bot_selector.rs     # Modal / daftar pemilih lawan bot & warna bidak
            ├── move_table.rs       # Tabel riwayat langkah 2 kolom + navigasi history
            └── captured_tray.rs    # Baris bidak yang dimakan + indikator selisih poin (+3)
```

---

## 3. Cargo.toml & Compiler Optimization Specification

Buat file `Cargo.toml` dengan konfigurasi dependensi yang ringan, stabil, dan cepat dikompilasi:

```toml
[package]
name = "rusty-chess"
version = "0.1.0"
edition = "2021"
authors = ["Yori Syahputra Wisnu"]
description = "Native Linux Chess Desktop Game with Grandmaster AI Personas in Pure Rust"
license = "MIT"

[dependencies]
eframe = { version = "0.29", default-features = false, features = [
    "default_fonts",
    "glow",
    "x11",
    "wayland",
] }
egui = "0.29"
shakmaty = "0.27"
rand = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Optimasi Kompilasi Development (Hemat RAM untuk laptop 4GB)
[profile.dev]
opt-level = 1               # Sedikit optimasi agar engine tidak lambat saat debug
debug = 1                   # Line tables only, menghemat ratusan MB RAM saat linking
codegen-units = 4

[profile.dev.package."*"]
opt-level = 2               # Compile dependencies dengan kecepatan tinggi

# Optimasi Kompilasi Release (Untuk paket .deb produksi)
[profile.release]
opt-level = 3
lto = "thin"                # Performa tinggi tanpa memakan RAM berlebih saat linking
codegen-units = 1
panic = "abort"
strip = "symbols"
```

---

## 4. Layer 1: Domain Specification (`src/domain/`)

Kita menggunakan crate `shakmaty` sebagai fondasi generator langkah legal (_bitboard_), lalu membungkusnya dengan struktur state aplikasi kita sendiri agar mendukung _move history navigation_, _captured pieces calculation_, _algebraic notation (SAN)_, dan _arrow annotations_.

### 4.1 Core Types (`src/domain/types.rs`)

```rust
use shakmaty::{Color, Move, Position, Role, Square};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerController {
    Human,
    Bot,
}

#[derive(Debug, Clone)]
pub struct MoveRecord {
    pub ply: usize,                  // Nomor setengah langkah (dimulai dari 1)
    pub mv: Move,                    // Objek Move dari shakmaty
    pub san: String,                 // Notasi standar misal "Nf3", "exd5", "O-O", "Qh7#"
    pub fen_after: String,           // String FEN setelah langkah dieksekusi
    pub eval_cp: i32,                // Evaluasi posisi dalam centipawn (perspektif Putih)
    pub captured_piece: Option<Role>,// Bidak yang dimakan pada langkah ini (jika ada)
    pub time_spent_ms: u64,          // Waktu berpikir untuk langkah ini
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOutcome {
    Ongoing,
    Checkmate { winner: Color },
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
    FiftyMoveRule,
    Resignation { winner: Color },
    Timeout { winner: Color },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TacticalArrow {
    pub from: Square,
    pub to: Square,
}
```

### 4.2 Game State Controller (`src/domain/game_state.rs`)

Struct `ChessMatchState` wajib menyimpan dan mengelola:

- `initial_pos: shakmaty::Chess`: Posisi awal (default standard atau custom FEN).
- `current_pos: shakmaty::Chess`: Posisi aktif paling mutakhir.
- `viewed_ply: usize`: Indeks langkah yang sedang dilihat user di layar (memungkinkan pemain menekan tombol `<-` dan `->` untuk meninjau langkah sebelumnya tanpa merusak game yang sedang berjalan).
- `history: Vec<MoveRecord>`: Daftar seluruh langkah yang telah dimainkan.
- `position_hashes: Vec<u64>`: Menyimpan Zobrist hash dari setiap posisi untuk mendeteksi _Threefold Repetition_ secara akurat.
- `white_controller: PlayerController` & `black_controller: PlayerController`.
- `last_move: Option<(Square, Square)>`: Petak asal dan tujuan dari langkah terakhir untuk di-_highlight_.
- `captured_by_white: Vec<Role>` & `captured_by_black: Vec<Role>`: Diurutkan dari nilai tertinggi (Queen -> Pawn).
- `material_balance: i32`: Selisih poin murni (`Q=9, R=5, B=3, N=3, P=1`) untuk ditampilkan di samping nama pemain (`+2`, `+5`, dll).

**Fungsi Wajib pada `ChessMatchState`:**

1. `new_standard(human_color: Color) -> Self`
2. `from_fen(fen_str: &str, human_color: Color) -> Result<Self, String>`
3. `apply_move(&mut self, mv: Move, eval_cp: i32, elapsed_ms: u64) -> Result<GameOutcome, String>`:
   - Menghitung string SAN menggunakan `shakmaty::san::SanPlus::from_move(self.current_pos.clone(), &mv)`.
   - Mengeksekusi langkah ke `current_pos`.
   - Mencatat `MoveRecord` ke `history`.
   - Memperbarui daftar bidak yang tertangkap dan `material_balance`.
   - Memeriksa kondisi akhir permainan (`is_checkmate()`, `is_stalemate()`, `is_insufficient_material()`, `halfmoves() >= 100`, atau `position_hashes` muncul >= 3 kali).
4. `undo_move_pair(&mut self)`: Membatalkan langkah Bot dan langkah Human terakhir (Takeback) serta menyinkronkan ulang `current_pos`.
5. `export_pgn(&self, bot_name: &str) -> String`: Menghasilkan teks PGN lengkap dengan header `[Event "RustyChess Match"]`, `[White "..."]`, `[Black "..."]`, dan daftar langkah `1. e4 e5 2. Nf3 Nc6 ...`.

---

## 5. Layer 2: AI Engine & Persona Architecture (`src/engine/`)

Inilah jantung dari aplikasi. Engine tidak boleh hanya memanggil fungsi random, melainkan merupakan mesin catur sesungguhnya dengan kepribadian yang dimodulasi melalui parameter _evaluation weights_, _search depth_, _move time_, dan _controlled noise_.

### 5.1 Evaluation System (`src/engine/eval/`)

Evaluasi selalu mengembalikan nilai dalam satuan **Centipawns (`i32`)** (`100 cp = 1 Pawn`). Positif berarti menguntungkan sisi yang sedang dievaluasi (dalam skema Negamax).

#### A. Base Material Values (`material.rs`)

- `Pawn`: `100`
- `Knight`: `320`
- `Bishop`: `330`
- `Rook`: `500`
- `Queen`: `900`
- `King`: `20000`
- **Bishop Pair Bonus**: `+35 cp` jika salah satu sisi masih memiliki kedua Gajah di petak terang dan gelap.

#### B. Tapered Piece-Square Tables (`pst.rs`)

Permainan dibagi menjadi fase **Midgame (MG)** dan **Endgame (EG)** berdasarkan `game_phase` (dihitung dari jumlah perwira non-pion yang tersisa di papan, rentang `0..=24`).

- `final_pst_score = (mg_score * phase + eg_score * (24 - phase)) / 24`
- Gunakan matriks 64-elemen standar (perspektif Putih, di-flip menggunakan `square ^ 56` untuk Hitam):
  - **Pawn PST**: Mendorong pion menguasai pusat (`d4, e4, d5, e5` bernilai `+20..+30`) dan memberikan bonus eksponensial untuk pion di baris ke-6 dan ke-7 (`+50` hingga `+90`).
  - **Knight PST**: Penalti berat di tepi papan (`a1..h1`, kolom `a` dan `h` bernilai `-30` s.d. `-50`), bonus besar di pusat (`c3..f6` bernilai `+15..+25`).
  - **Bishop PST**: Bonus di diagonal panjang dan kontrol pusat, penalti di sudut tertutup.
  - **Rook PST**: Bonus di baris ke-7 (`+25 cp`) dan kolom tengah (`d`, `e`).
  - **King MG PST**: Bonus besar di petak rokade (`g1, b1, c1` bernilai `+30`), penalti keras di tengah papan (`-40` s.d. `-50`).
  - **King EG PST**: Kebalikan dari Midgame! Raja didorong maju ke pusat papan di Endgame (`+20..+30` di pusat, `-30` di pojok) agar aktif membantu promosi pion (kunci gaya main **Magnus Carlsen**).

#### C. Dynamic Positional Modifiers (`mobility.rs` & `king_safety.rs`)

Setiap persona memiliki pengali (_multiplier_) terhadap komponen evaluasi ini:

1. **Mobility Score**: Jumlah langkah legal perwira (Knight, Bishop, Rook, Queen) dikalikan `4 cp`.
2. **Center Attack Bonus**: Setiap serangan ke 4 petak pusat (`d4, e4, d5, e5`) diberi bonus `+6 cp`.
3. **King Safety & Attack**:
   - Penalti `-20 cp` jika Raja belum rokade sementara Ratu lawan masih ada di papan dan pion pelindung di depan Raja sudah maju/hilang.
   - Bonus agresif `+12 cp` untuk setiap perwira yang menyerang zona 3x3 di sekitar Raja lawan.
4. **Endgame Pawn Structure**:
   - _Passed Pawn_ (pion bebas tanpa penghalang pion lawan di depan): bonus `+25 cp` s.d. `+120 cp` tergantung kedekatan dengan baris promosi.
   - _Doubled Pawn_ (pion bertumpuk di satu lajur): penalti `-15 cp`.
   - _Isolated Pawn_ (pion tanpa rekan di lajur kiri/kanan): penalti `-15 cp`.

#### D. Formula Evaluasi Persona Terpadu

```rust
pub struct EvalWeights {
    pub material_scale: f32,      // Default 1.0
    pub positional_scale: f32,    // Skala PST & Pawn Structure
    pub mobility_scale: f32,      // Skala agresivitas perwira & kontrol ruang
    pub king_attack_scale: f32,   // Skala keberanian menyerang Raja lawan
    pub endgame_scale: f32,       // Ketajaman evaluasi saat masuk fase Endgame
    pub contempt_cp: i32,         // Penalti terhadap hasil seri (Magnus benci remis!)
}
```

---

### 5.2 Search Engine (`src/engine/search/`)

#### A. Transposition Table (`tt.rs`)

- Ukuran tetap: `16 MB` (sekitar `524_288` entries).
- Setiap `TTEntry` berisi: `key: u64`, `depth: u8`, `flag: TTFlag (Exact, LowerBound, UpperBound)`, `score: i32`, `best_move: Option<Move>`.
- Menggunakan Zobrist Hash bawaan `shakmaty::zobrist::Zobrist64`.

#### B. Move Ordering (`ordering.rs`)

Agar _Alpha-Beta Pruning_ memotong cabang secara maksimal dan sangat cepat di CPU Celeron:

1. **Hash Move (TT Move)**: Skor prioritas `1_000_000`.
2. **Promotions**: Skor prioritas `900_000`.
3. **Winning/Equal Captures (MVV-LVA)**: `100_000 + (victim_val * 10) - attacker_val`.
4. **Killer Moves**: 2 langkah quiet per ply yang menyebabkan _beta-cutoff_ (`80_000`).
5. **History Heuristic**: Tabel `[Color][FromSquare][ToSquare]` (`0..70_000`).
6. **Quiet Moves**: Sisanya (`0`).

#### C. Quiescence Search (`quiescence.rs`)

- Dipanggil saat `depth == 0` pada Negamax untuk mencegah _Horizon Effect_ (contoh: bot mengira menang pion padahal langkah berikutnya Ratunya dimakan balik).
- Mengevaluasi `stand_pat = evaluate(pos, weights)`.
- Hanya menelusuri langkah **Captures** dan **Queen Promotions** (maksimal kedalaman tambahan 6 ply).

#### D. Iterative Deepening + Negamax (`negamax.rs`)

- Melakukan pencarian bertahap dari `depth = 1` hingga `persona.max_depth`.
- Memeriksa `Instant::now() >= deadline` setiap `1024` node agar bot tidak pernah melebihi batas waktu berpikir (`time_budget_ms`).
- Mengirimkan laporan progres pencarian (`EngineEvent::ThinkingProgress`) ke GUI setiap selesai 1 level depth (sehingga _Eval Bar_ dan indikator depth bergerak hidup!).
- **Sistem Simulasi Blunder / Ketidaksempurnaan Manusia**:
  - Pada root node, simpan daftar seluruh langkah legal beserta skor evaluasinya (`Vec<(Move, i32)>`), urutkan dari yang terbaik.
  - Jika `persona.blunder_chance > 0.0` dan RNG memicu ketidaksempurnaan, atau jika `persona.noise_cp > 0`:
    - Tambahkan gangguan acak (_Gaussian/Uniform noise_) sebesar `[-noise_cp, +noise_cp]` pada skor setiap kandidat langkah sebelum memilih langkah tertinggi.
    - Ini membuat bot level rendah melakukan kesalahan yang **terasa alami seperti manusia** (misal salah memilih langkah terbaik ke-2 atau ke-3), bukan langkah aneh yang benar-benar acak kecuali pada Bot Martin di saat blunder!

---

### 5.3 Bot Personas & Roster Specification (`src/engine/bot/`)

Semua bot didefinisikan di dalam `src/engine/bot/persona.rs` dengan karakter yang kontras dan ikonik:

| ID        | Nama Tampilan                | Gelar / Julukan       | ELO  | Max Depth | Time Budget | Noise (cp) | Karakteristik Bobot Evaluasi & Gaya Main                                                                                                                                                                |
| :-------- | :--------------------------- | :-------------------- | :--: | :-------: | :---------: | :--------: | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `martin`  | **Martin**                   | _Bot Santai Pemula_   | 400  |     2     |   250 ms    |   280 cp   | Sering meninggalkan perwira tanpa kawalan, evaluasi posisi rendah (`positional: 0.2`), cocok untuk pemanasan.                                                                                           |
| `warung`  | **Pak RT (Bapak Pos Ronda)** | _Penjaga Warung Kopi_ | 1100 |     3     |   400 ms    |   95 cp    | Suka tukar perwira, lumayan paham taktik 1-2 langkah (`mobility: 0.8`), kadang lengah terhadap skak ganda.                                                                                              |
| `club`    | **Clara**                    | _Atlet Klub Kampus_   | 1600 |     4     |   600 ms    |   35 cp    | Seimbang di semua lini (`1.0`), disiplin melakukan rokade dan menjaga struktur pion.                                                                                                                    |
| `hikaru`  | **Hikaru Nakamura**          | _GM Speed Demon_      | 2800 |     5     |   450 ms    |    5 cp    | **Sangat Cepat & Agresif**: `mobility_scale: 1.55`, `king_attack_scale: 1.65`, `positional_scale: 1.1`. Sangat suka komplikasi taktis, langkah kuda liar, dan bermain cepat!                            |
| `magnus`  | **Magnus Carlsen**           | _The GOAT_            | 2882 |     6     |   1100 ms   |    0 cp    | **Presisi Posisional & Raja Endgame**: `positional_scale: 1.60`, `endgame_scale: 1.85`, `contempt_cp: 45`. Hampir tidak pernah blunder, menolak seri, dan memeras kesalahan sekecil apa pun di endgame. |
| `silicon` | **RustFish Core**            | _Maximum Overdrive_   | 3100 |     7     |   1800 ms   |    0 cp    | Evaluasi murni tanpa kompromi, pencarian terdalam untuk menguji batas kemampuan pemain.                                                                                                                 |

#### Sistem Dialog Reaktif (`src/engine/bot/dialogue.rs`)

Setiap persona memiliki bank kalimat untuk kondisi pemicu (`DialogueTrigger`):

1. `MatchStart`: Saat permainan baru dimulai.
2. `OpeningPhase`: Langkah 1–5.
3. `BotCapturesPiece`: Saat bot memakan perwira pemain.
4. `PlayerBlunders`: Saat evaluasi pemain anjlok `> 180 cp` dalam satu langkah.
5. `PlayerGreatMove`: Saat pemain menemukan langkah terbaik yang menaikkan evaluasi pemain `> 120 cp`.
6. `BotInCheck`: Saat pemain melakukan skak ke Raja bot.
7. `BotGivesCheck`: Saat bot melakukan skak ke Raja pemain.
8. `EndgameEntered`: Saat jumlah perwira di papan tersisa sedikit.
9. `BotWinning`: Saat evaluasi bot unggul `> +350 cp`.
10. `BotLosing`: Saat evaluasi bot tertinggal `< -350 cp`.
11. `GameOverWin` / `GameOverLoss` / `GameOverDraw`.

**Contoh Kuotasi Khas yang Wajib Diimplementasikan:**

- **Martin (400)**:
  - _MatchStart_: `"Halo! Aku baru belajar cara jalan Kuda kemarin sore. Jangan galak-galak ya!"`
  - _BotLosing_: `"Loh, kok Ratuku tiba-tiba hilang? Kamu pakai sulap ya?"`
- **Pak RT (1100)**:
  - _MatchStart_: `"Sini duduk dulu, kopi hitamnya diseruput. Jangan buru-buru majukan Ratu!"`
  - _BotCapturesPiece_: `"Nah kan, meleng dikit langsung disikat! Kurang fokus nih."`
- **Hikaru Nakamura (2800)**:
  - _MatchStart_: `"Alright chat, let's see what opening we get today. Probably something dubious."`
  - _BotCapturesPiece_: `"Takes, takes, takes, and then Knight goes here... yeah, I'm just winning."`
  - _PlayerBlunders_: `"Wait, what?! Chat, did they really just play that? That loses on the spot!"`
  - _BotWinning_: `"Look at thisjuicer on the 7th rank. I don't even need to calculate anymore, just premoving."`
- **Magnus Carlsen (2882)**:
  - _MatchStart_: `"Let's play a clean game. Show me if your positional understanding holds up."`
  - _EndgameEntered_: `"Now the real chess begins. Even an equal endgame can be squeezed for a win."`
  - _PlayerBlunders_: `"Too impatient. That single tempo concession ruins your entire dark-square structure."`
  - _BotWinning_: `"Positionally, this is completely over. The rest is just a matter of simple technique."`

---

## 6. Layer 3: Multi-Threading Protocol (`src/engine/worker.rs`)

Untuk menjamin UI 144Hz tetap mulus tanpa _stutter_:

```rust
pub enum EngineCommand {
    ComputeBestMove {
        request_id: u64,
        pos: shakmaty::Chess,
        persona: BotPersona,
        history_hashes: Vec<u64>,
    },
    AbortSearch,
    Shutdown,
}

pub enum EngineEvent {
    ThinkingProgress {
        request_id: u64,
        depth: u8,
        max_depth: u8,
        eval_cp: i32,
        nodes: u64,
    },
    MoveReady {
        request_id: u64,
        best_move: shakmaty::Move,
        eval_cp: i32,
        depth_reached: u8,
        nodes_searched: u64,
        elapsed_ms: u64,
    },
}
```

- `EngineWorker` di-spawn satu kali saat `RustyChessApp::new()`.
- Menyimpan `AtomicBool` (`abort_flag`) yang dibagikan (`Arc<AtomicBool>`) antara UI thread dan Worker thread. Jika user menekan tombol **"New Game"** atau **"Undo"** saat bot sedang berpikir, UI menyetel `abort_flag.store(true, Ordering::SeqCst)` dan menaikkan `current_request_id` sehingga hasil pencarian lama langsung diabaikan.

---

## 7. Layer 4: Native GUI & UX Specification (`src/ui/`)

### 7.1 Layout Proporsi Layar (Default Window: `1180 x 780`, Min: `960 x 640`)

Menggunakan arsitektur panel `egui`:

1. **`TopBottomPanel::top("top_bar")`**:
   - Kiri: Logo vektor ikon kuda + judul bold **"RUSTY CHESS"** + badge versi `v0.1.0`.
   - Tengah: Pemilih cepat **Board Theme** (_Classic Green_, _Lichess Wood_, _Tournament Blue_, _Cyberpunk Dark_), tombol **Flip Board**, tombol **Sound Toggle**, tombol **FEN/PGN Export Modal**.
   - Kanan: Indikator status Engine (`Idle` / `Thinking Depth 5/6...`), Node per detik (`kN/s`), dan FPS counter.
2. **`SidePanel::right("control_panel")` (Lebar: `360px`)**:
   - **Bagian 1 — Bot Profile & Dialogue Box**:
     - Avatar vektor berwarna unik untuk tiap bot (dengan border glow saat bot sedang berpikir).
     - Nama Bot, badge **ELO Rating**, dan bar karakteristik singkat (Speed / Tactical / Positional).
     - **Speech Bubble (Balon Dialog)**: Kotak bergaya chat modern yang menampilkan komentar bot secara kontekstual terhadap kondisi papan saat ini.
   - **Bagian 2 — Captured Pieces & Player Info**:
     - Menampilkan deretan ikon bidak yang berhasil dimakan oleh Bot dan oleh Pemain, beserta selisih poin material (`+3`).
   - **Bagian 3 — Algebraic Move History Table**:
     - `egui::ScrollArea::vertical()` dengan tabel 3 kolom: `#No`, `White Move`, `Black Move`.
     - Klik pada langkah mana pun di tabel akan memindahkan tampilan papan ke langkah tersebut (History Inspection Mode).
     - Dilengkapi 4 tombol navigasi di bawah tabel: `|<<` (Awal), `<` (Mundur), `>` (Maju), `>>|` (Posisi Terkini).
   - **Bagian 4 — Action Bar**:
     - Tombol utama **"New Match / Ganti Bot"** (membuka pemilih bot & warna bidak).
     - Tombol **"Undo / Takeback"** (membatalkan 1 pasang langkah).
     - Tombol **"Hint"** (meminta engine menunjukkan langkah terbaik untuk pemain dengan panah hijau di papan).
3. **`CentralPanel::default()` (Area Papan Utama)**:
   - Di sebelah kiri papan terdapat **Vertical Evaluation Bar (`eval_bar.rs`)** selebar `26px` dengan tinggi mengikuti ukuran papan catur:
     - Menampilkan proporsi keunggulan Putih vs Hitam menggunakan kurva sigmoid `1.0 / (1.0 + (-eval_cp as f32 / 400.0).exp())`.
     - Transisi tinggi bar menggunakan _linear interpolation (lerp)_ di setiap frame agar pergerakannya halus.
     - Menampilkan angka evaluasi seperti `+1.4`, `-0.8`, atau `M3` (Mate in 3).
   - Di tengah terdapat **8x8 Interactive Chessboard (`board_view/`)** yang selalu mempertahankan rasio **1:1 (Persegi Sempurna)** dan otomatis menyesuaikan ukuran layar (_responsive scaling_).

### 7.2 Interaksi Papan Catur (`src/ui/board_view/`)

Harus mendukung standar UX aplikasi catur modern:

1. **Dual Input Method**:
   - **Click-to-Move**: Klik bidak sendiri -> tampilkan titik transparan di semua petak tujuan legal (dan cincin di petak bidak lawan yang bisa dimakan) -> klik petak tujuan untuk jalan.
   - **Drag-and-Drop**: Klik dan tahan bidak -> bidak mengikuti kursor mouse secara _real-time_ (diperbesar `1.15x` dengan bayangan halus) -> lepaskan di petak tujuan legal.
2. **Visual Highlights**:
   - **Selected Square**: Warna kuning-hijau terang transparan.
   - **Last Move (From & To)**: Highlight kuning hangat pada langkah terakhir yang baru saja terjadi.
   - **King in Check**: Efek gradasi/glow merah menyala pada petak Raja yang sedang terkena skak.
   - **Right-Click Tactical Annotations**:
     - Klik kanan & drag dari petak A ke petak B menggambar **Panah Taktis (Tactical Arrow)** berwarna oranye/hijau lengkap dengan kepala panah segitiga (seperti saat Hikaru menjelaskan variasi!).
     - Klik kanan di satu petak menandai petak tersebut dengan warna merah transparan.
     - Klik kiri di mana saja menghapus seluruh coretan panah.
3. **Pawn Promotion Modal (`promotion_popup.rs`)**:
   - Saat pemain menggerakkan pion ke baris ke-8 (atau ke-1), jangan langsung otomatis jadi Queen! Tampilkan _overlay popup_ tepat di atas petak promosi berisi 4 pilihan ikon besar: **Queen, Rook, Bishop, Knight** (dengan tombol _Cancel_ jika berubah pikiran).

### 7.3 Procedural Crisp Vector Piece Renderer (`src/ui/piece_renderer.rs`)

Agar aplikasi **100% mandiri tanpa ketergantungan file eksternal** dan selalu tajam di resolusi berapa pun:

- Buat modul `piece_renderer.rs` yang menggambar siluet bidak catur klasik (**King, Queen, Rook, Bishop, Knight, Pawn**) menggunakan kombinasi poligon, lingkaran, dan kurva `egui::Painter` dengan _anti-aliased stroke_ (`Stroke::new(2.0, outline_color)`), bayangan bawah (`drop shadow`), dan aksen detail internal (mahkota Ratu, salib Raja, surai dan mata Kuda, celah Gajah, gerigi Benteng).
- Dengan cara ini, aplikasi dijamin bebas dari error _missing texture_, ukuran binary sangat kecil (< 8 MB), dan tajam sempurna di layar 1080p maupun 4K.

---

## 8. Debian `.deb` Packaging Specification

### 8.1 File `packaging/debian/DEBIAN/control`

```control
Package: rusty-chess
Version: 0.1.0
Section: games
Priority: optional
Architecture: amd64
Depends: libc6 (>= 2.31), libgl1, libx11-6, libxi6, libxcursor1, libxrandr2, libwayland-client0
Maintainer: Yori Syahputra Wisnu <yori@ubuntu.local>
Description: Native Linux Chess Desktop Game with AI Grandmaster Personas
 A blazingly fast, lightweight native desktop chess application written in pure
 Rust and egui. Play against customizable AI personalities including Beginner
 bots, Hikaru-style tactical speedsters, and Magnus-style endgame masters.
```

### 8.2 File `packaging/debian/usr/share/applications/rusty-chess.desktop`

```ini
[Desktop Entry]
Name=Rusty Chess
Comment=Play Chess against AI Personas (Magnus, Hikaru, and Club Bots)
Exec=/usr/bin/rusty-chess
Icon=rusty-chess
Terminal=false
Type=Application
Categories=Game;BoardGame;StrategyGame;
Keywords=chess;catur;ai;magnus;hikaru;rust;
StartupWMClass=rusty-chess
```

### 8.3 Script `scripts/build-deb.sh`

Script ini harus:

1. Menjalankan `cargo test` untuk memastikan engine sehat.
2. Menjalankan `cargo build --release`.
3. Membuat struktur direktori staging `build-deb/rusty-chess_0.1.0_amd64/`.
4. Menyalin binary `target/release/rusty-chess` ke `usr/bin/rusty-chess` dan mengatur izin `chmod 755`.
5. Menyalin file `.desktop`, `DEBIAN/control`, dan ikon SVG aplikasi.
6. Menjalankan `dpkg-deb --build --root-owner-group` untuk menghasilkan file `rusty-chess_0.1.0_amd64.deb` di root proyek.
7. Menampilkan ukuran binary dan ukuran paket `.deb` beserta instruksi instalasi `sudo dpkg -i rusty-chess_0.1.0_amd64.deb`.

---

## 9. Step-by-Step Execution Plan for Antigravity Agent

Saat membaca `SPEC.md` ini, kerjakan secara berurutan (Milestone 1 hingga 5) dan pastikan `cargo check` lulus di setiap tahap:

- **Milestone 1 (Project & Domain Core)**:
  Buat `Cargo.toml`, `src/domain/types.rs`, `src/domain/notation.rs`, `src/domain/clock.rs`, dan `src/domain/game_state.rs`. Pastikan validasi langkah legal, deteksi skakmat/remis, undo/takeback, dan ekspor PGN/FEN berfungsi.
- **Milestone 2 (AI Engine & Personas)**:
  Buat `src/engine/eval/` (Material, PST Midgame/Endgame, Mobility, King Safety), `src/engine/search/` (TT 16MB, MVV-LVA Ordering, Quiescence, Negamax Alpha-Beta), serta `src/engine/bot/` (6 Persona + sistem dialog kontekstual) dan `src/engine/worker.rs` (MPSC background thread).
- **Milestone 3 (Vector Renderer & Board GUI)**:
  Buat `src/ui/theme.rs`, `src/ui/piece_renderer.rs`, `src/ui/eval_bar.rs`, dan `src/ui/board_view/` (grid 8x8, drag-and-drop, click-to-move, right-click tactical arrows, dan modal promosi pion).
- **Milestone 4 (Side Panels, Bot Chat & App Integration)**:
  Buat `src/ui/top_bar.rs`, `src/ui/side_panel/` (Bot card, speech bubble, captured tray, move history table, modal pemilih lawan), serta hubungkan semuanya di `src/app.rs` dan `src/main.rs`.
- **Milestone 5 (Tests & Debian Packaging)**:
  Buat unit test di `tests/`, buat `Makefile`, `packaging/debian/`, dan `scripts/build-deb.sh`. Jalankan `cargo test` dan pastikan proyek siap dibungkus menjadi `.deb`.
