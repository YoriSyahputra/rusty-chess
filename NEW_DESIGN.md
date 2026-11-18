# NEW_DESIGN.md — Rusty Chess 1:1 Chess.com Experience Overhaul

**Target Version:** `v0.2.0` (Full Main Menu & Left Sidebar, Play Bots Lobby, 60–144 FPS Animations, Humanized AI + Thinking Scheduler, Game Review)

---

## 1. Purpose & Problem Remediation

This specification upgrades `rusty-chess` from `v0.1.0` to `v0.2.0` to achieve a faithful 1:1 Chess.com desktop user experience while eliminating four concrete defects identified in the initial build:

1. **Instant Bot Moves:** The engine worker currently emits `MoveReady` in $< 50\text{ ms}$ immediately upon search completion. Add a cancellable **Context-Aware Thinking Delay Scheduler** ($350\text{–}3800\text{ ms}$ scaled by opening book state, recapture detection, tactical complexity, bot pacing, and time controls).
2. **Missing Main Menu & Lobby:** The application boots straight into an active match against Magnus Carlsen without any top-level routing or setup. Add a persistent **Left Navigation Sidebar** (`Home`, `Play`, `Bots`, `Puzzles`, `Review`, `Settings`) along with dedicated **Home Dashboard**, **Play Hub**, and **Play Bots Lobby** screens.
3. **Missing Animations & Premove Polish:** Pieces teleport across the board without visual interpolation. Implement a decoupled **BoardAnimationLayer** supporting all 5 Chess.com animation profiles (_Slow_, _Medium_, _Fast_, _Natural_, _Arcade_), illegal-move snap-back, capture fade-out, check radial pulse, premove queueing, 4-color right-click annotations, and end-of-game King result badges.
4. **Vertical Layout Clipping & Idle Resource Waste:** In `v0.1.0`, the right panel overflows vertically (clipping the `|<< < > >>|` stepper and pushing match action buttons off-screen) while rank 1 touches the OS dock; furthermore, idle UI continuously repaints at 62 FPS consuming $84.9\text{ MB}$ RAM. Fix board geometry using top/bottom player strips, constrain the right-panel `ScrollArea` within available height, shrink the Transposition Table to $8\text{ MB}$, and enforce strict **Reactive Repaint Mode** (0% CPU when idle).

### Core Architectural Invariants

- **Zero Web Wrappers:** Pure native Rust rendering via `eframe 0.29` / `egui 0.29` with `shakmaty 0.27`. No Electron, Tauri, WebViews, HTML, CSS, or external runtime asset dependencies.
- **Non-Blocking Worker:** AI search, thinking delay pacing, and post-game review analysis must run on background threads using `std::sync::mpsc` channels and `Arc<AtomicBool>` cancellation flags.

---

## 2. Visual Design System & Color Tokens

Grounded in Chess.com's primary brand green (`#81B64C`) and dark slate charcoal (`#2E2C29`), paired with recognizable UI accent tokens (`#FFAA00`, `#009FD9`, `#FFFF33`) and calibrated dark/light surfaces:

| Token                  | Dark UI Hex           | Light UI Hex          | Role & Placement                                            |
| :--------------------- | :-------------------- | :-------------------- | :---------------------------------------------------------- |
| `bg_canvas`            | `#262421`             | `#F1F1EE`             | Main workspace background canvas                            |
| `bg_sidebar`           | `#211F1C`             | `#E6E6E2`             | Persistent left navigation rail                             |
| `bg_surface`           | `#302E2B`             | `#FFFFFF`             | Bot cards, move list container, modal dialogues             |
| `bg_surface_elevated`  | `#3B3834`             | `#F6F6F3`             | Hovered rows, active tab headers, selected bot items        |
| `accent_primary`       | `#81B64C`             | `#81B64C`             | Primary CTA buttons (`Play`, `Play Bots`, `Review`)         |
| `accent_primary_hover` | `#95C95D`             | `#95C95D`             | Hovered state for primary CTAs                              |
| `accent_gold`          | `#FFAA00`             | `#E09400`             | Crown badges, ELO rating pills, active turn indicators      |
| `accent_info`          | `#009FD9`             | `#0088BD`             | Informational chips, hint suggestions, secondary highlights |
| `text_primary`         | `#F5F5F5`             | `#22201E`             | High-contrast headings, active clock digits, SAN notation   |
| `text_secondary`       | `#B0ACA6`             | `#66625C`             | Bot subtitles, card descriptions, inactive player clocks    |
| `board_classic_light`  | `#EBECD0`             | `#EBECD0`             | Classic Green board light squares                           |
| `board_classic_dark`   | `#739552`             | `#739552`             | Classic Green board dark squares                            |
| `highlight_last_move`  | `#FFFF33` (45% alpha) | `#FFFF33` (50% alpha) | Origin and destination squares of the last move             |
| `highlight_premove`    | `#E04F44` (55% alpha) | `#E04F44` (55% alpha) | Queued premove origin and destination squares               |

### Selectable Board Themes (`Settings > Board`)

- **Classic Green:** `#EBECD0` / `#739552`
- **Walnut / Wood:** `#F0D9B5` / `#B58863`
- **Tournament Blue:** `#DEE3E6` / `#8CA2AD`
- **Icy Sea:** `#E8EDF2` / `#5B87A6`
- **Dark Slate:** `#585C62` / `#3B3E44`

---

## 3. Navigation Router & Left Sidebar (`ActiveRoute`)

```text
+------------------+-------------------------------------------------------------+
| Rusty Chess      |                                                             |
| [Knight + Crown] |                                                             |
|                  |                     ACTIVE ROUTE VIEW                       |
| [Home]           |                                                             |
| [Play]           |   - Route::Home      (Dashboard, Stats, Daily Puzzle)       |
| [Bots]           |   - Route::PlayHub   (Play Mode Selector)                   |
| [Puzzles]        |   - Route::PlayBots  (1:1 Tiered Bot Lobby)                 |
| [Review]         |   - Route::Match     (Live Game Workspace)                  |
|                  |   - Route::Puzzles   (Tactics Trainer & Daily Puzzle)       |
|                  |   - Route::Review    (Interactive Post-Game Analysis)       |
|                  |                                                             |
| [Dark / Light]   |                                                             |
| [Settings Cog]   |                                                             |
+------------------+-------------------------------------------------------------+
```

### 3.1 Left Sidebar Specifications

- **Width:** $168\text{ px}$ expanded when window width $\ge 1100\text{ px}$; collapses automatically to a $60\text{ px}$ compact icon rail below $1100\text{ px}$ (can also be manually toggled via chevron).
- **Brand Block (Top):** Procedural vector Knight + Crown insignia with bold `Rusty Chess` title.
- **Navigation Destinations:**
  - `Home`: Dashboard with crown progress overview ($X / 54$), win/draw/loss records against bots, quick-action cards, Daily Puzzle widget, and recent PGN history.
  - `Play`: Mode selection hub featuring an interactive preview board on the left and mode cards on the right (`Play Bots`, `Play Coach`, `Pass & Play`, `Custom Position [FEN]`).
  - `Bots`: Navigates directly to the comprehensive Play Bots Lobby.
  - `Puzzles`: Offline tactics trainer with 30+ curated puzzles plus a Daily Puzzle mode featuring a 5-heart life system and Hard Mode toggle.
  - `Review`: Post-game evaluation view displaying accuracy scores, move classifications, win probability graphs, and alternative best-line explorations.
- **Utility Footer (Bottom):** One-click `Dark UI / Light UI` toggle button and a `Settings` modal launcher for theme, audio, animation, and crown reset controls.

---

## 4. Play Bots Lobby (`Route::PlayBots`)

A dedicated two-column layout matching Chess.com's lobby architecture:

- **Left Column (Flexible Workspace):** Centered $8\times 8$ preview board flanked by player summary strips.
- **Right Column ($380\text{ px}$ Fixed Width):**
  1. **Selected Bot Hero Card (Pinned Top):** Circular avatar badge with earned crown indicators ($0\text{–}3$), Bot Name, ELO pill, Tier label, and signature quote/bio.
  2. **Grouped Bot Roster (`ScrollArea` Middle):** Collapsible category sections displaying all 18 personality bots plus the continuous Engine slider.
  3. **Pre-Match Configuration (Pinned Bottom):**
     - **Play As:** `[White] [Random] [Black]`.
     - **Time Control:** `Untimed`, `1 min`, `1+1`, `2+1`, `3+2`, `5 min`, `5+5`, `10 min`, `15+10`, `30 min`.
     - **Assistance Modes:**
       - `Challenge` (3 Crowns maximum): No hints, no takebacks, eval bar and visual arrows disabled.
       - `Friendly` (2 Crowns maximum): Up to 3 hints and 3 takebacks; live eval bar disabled.
       - `Assisted` (1 Crown maximum): Unlimited hints and takebacks; real-time eval bar, threat arrows (red), and suggestion arrows (green) active.
       - `Custom` (1–3 Crowns): Granular toggles for `Bot Chat`, `Eval Bar`, `Threat Arrows`, `Suggestion Arrows`, `Move Feedback`, and `Custom FEN`. Earned crowns auto-adjust based on actual help utilized during the match.
     - **Primary CTA:** Full-width `#81B64C` button with label **"Play"** ($44\text{ px}$ height).

### 4.1 Complete 18-Bot Roster + Engine Slider

| ID               | Display Name       | Tier         |     ELO      | Max Depth |  Pace   |   Softmax $T$   | Opening Repertoire & Behavioral Traits                                                                                              |
| :--------------- | :----------------- | :----------- | :----------: | :-------: | :-----: | :-------------: | :---------------------------------------------------------------------------------------------------------------------------------- |
| `adaptive_jimmy` | **Adaptive Jimmy** | Adaptive     |   Dynamic    |    2–5    | `0.90x` |     Dynamic     | Starts ~900 ELO; increases depth/reduces $T$ when player plays accurately, softens when player blunders.                            |
| `martin`         | **Martin**         | Beginner     |     250      |     1     | `0.65x` | $260\text{ cp}$ | Passive flank pushes (`1. a3`, `1. h4`); very low positional weight ($0.15$), frequently hangs pieces.                              |
| `elani`          | **Elani**          | Beginner     |     400      |     2     | `0.85x` | $210\text{ cp}$ | Pushes flank pawns early; misses simple 2-move forks and back-rank pins.                                                            |
| `jimmy`          | **Jimmy**          | Beginner     |     600      |     2     | `0.90x` | $180\text{ cp}$ | Develops knights early; leaves undefended tactical hangers in exchange sequences.                                                   |
| `aron`           | **Aron**           | Beginner     |     700      |     2     | `0.92x` | $155\text{ cp}$ | Plays standard `1. e4`; reliably executes 1-ply recaptures but vulnerable to 3-ply tactical traps.                                  |
| `emir`           | **Emir**           | Intermediate |     1000     |     3     | `0.95x` | $105\text{ cp}$ | Solid `1. d4` / Italian Game; eliminates direct one-turn piece blunders.                                                            |
| `sven`           | **Sven**           | Intermediate |     1100     |     3     | `1.00x` | $90\text{ cp}$  | Aggressive direct trades; initiates premature pawn breaks to open files.                                                            |
| `nelson`         | **Nelson**         | Intermediate |     1300     |     3     | `0.80x` | $72\text{ cp}$  | **Signature:** Early Queen sorties (`2. Qh5` / `2. Bc4` / `3. Qf3`); high `king_attack_scale (1.5)`.                                |
| `antonio`        | **Antonio**        | Advanced     |     1500     |     4     | `1.00x` | $48\text{ cp}$  | Smug dialogue; disciplined classical repertoires (Ruy Lopez / Queen's Gambit Declined).                                             |
| `isabel`         | **Isabel**         | Advanced     |     1600     |     4     | `1.05x` | $38\text{ cp}$  | Robust pawn structures; prioritizes kingside safety and avoids unsound sacrifices.                                                  |
| `wally`          | **Wally**          | Advanced     |     1800     |     4     | `1.10x` | $26\text{ cp}$  | Deep quiescence search; severely punishes pawn-grabbing inaccuracies.                                                               |
| `li`             | **Li**             | Advanced     |     2000     |     5     | `1.15x` | $16\text{ cp}$  | Patient positional maneuvers with refined endgame conversions (`endgame_scale: 1.35`).                                              |
| `noam`           | **Noam**           | Master       |     2200     |     5     | `1.20x` | $10\text{ cp}$  | Master-tier calculation; rarely drops more than $25\text{ cp}$ across the entire game.                                              |
| `francis`        | **Francis**        | Master       |     2300     |     5     | `1.25x` |  $6\text{ cp}$  | Relentless outpost exploitation and quiet positional squeezing.                                                                     |
| `danny`          | **IM Danny**       | Top Player   |     2500     |     6     | `0.95x` |  $3\text{ cp}$  | Tactical banter; dangerous gambits; heavily favors active bishop pairs.                                                             |
| `hikaru`         | **GM Hikaru**      | Top Player   |     2800     |     6     | `0.62x` |  $0\text{ cp}$  | **Speed Demon:** Rapid tempo; sharp mobility ($1.55$) and kingside pressure ($1.60$); Sicilian Najdorf / King's Indian defense.     |
| `magnus`         | **GM Magnus**      | Top Player   |     2882     |     6     | `1.35x` |  $0\text{ cp}$  | **The GOAT:** Peak positional weighting ($1.60$), endgame squeeze ($1.85$), `contempt_cp: 45`; Catalan / Ruy Lopez / London system. |
| `mittens`        | **Mittens**        | Personality  | `1` _(3200)_ |     7     | `0.75x` |  $0\text{ cp}$  | Deceptively displays rating **`1`** while calculating at maximum depth; eerie feline dialogue upon finding forced mates.            |
| `engine_custom`  | **Engine**         | Engine       |  `250–3200`  |    1–7    | `1.00x` |     Mapped      | Unbiased slider in increments of 50 ELO for pure synthetic training.                                                                |

---

## 5. Active Match Layout & Geometry (`Route::Match`)

### 5.1 Central Workspace Dimension Formulas

To prevent board clipping and ensure the bottom rank never collides with OS docks:

1. Query available canvas: `let avail = ui.available_size();`.
2. Compute evaluation bar allocation: `let eval_w = if show_eval_bar { 28.0 } else { 0.0 };` plus $8.0\text{ px}$ margin.
3. Reserve exactly $44.0\text{ px}$ above the board for the **Opponent Player Strip** and $44.0\text{ px}$ below for the **User Player Strip** (total vertical overhead = $104.0\text{ px}$ including gutters).
4. Calculate exact board dimension:
   $$\text{board\_size} = \max\Big(320.0, \, \min(\text{avail}.x - \text{eval\_w} - 16.0, \, \text{avail}.y - 104.0)\Big)$$
5. Each Player Strip (`width = board_size`, `height = 44.0 px`) renders:
   - Avatar badge ($32\times 32\text{ px}$) with glowing turn border.
   - Player / Bot Name + ELO pill.
   - Animated `"Thinking..."` indicator dots during calculation turns.
   - Captured piece silhouettes with dynamic material advantage differential (e.g., `+3`).
   - High-contrast digital clock display (`10:00` or `∞`).

```text
+--------------------------------------------------------------------------------+
|  OPPONENT STRIP [Avatar 32x32] [Bot Name (ELO)] [Captures +2]    [Clock 09:42] |
+---+----------------------------------------------------------------------------+
| E |                                                                            |
| V |                                                                            |
| A |                         8x8 INTERACTIVE CHESSBOARD                         |
| L |                                (board_size)                                |
|   |                                                                            |
| B |                                                                            |
| A |                                                                            |
| R |                                                                            |
+---+----------------------------------------------------------------------------+
|  PLAYER STRIP   [Avatar 32x32] [You (Player)]   [Captures]       [Clock 09:58] |
+--------------------------------------------------------------------------------+
```

### 5.2 Bounded Right Control Panel ($360\text{ px}$ Fixed Width)

Vertical elements are anchored from the edges inward to guarantee zero vertical overflow:

- **Pinned Header ($\sim 112\text{ px}$):** Reactive Bot Speech Bubble with speaker pointer + active match mode badge (`Challenge • 3 Crowns` or `Friendly • Hints: 2/3, Takebacks: 3/3`).
- **Pinned Footer ($\sim 92\text{ px}$):**
  - Top navigation row ($32\text{ px}$): History stepper buttons (`|<<`, `<`, `>`, `>>|`, `Flip`).
  - Bottom control row ($40\text{ px}$): Action buttons (`Hint`, `Takeback`, `Resign`, `New Match`).
- **Dynamic Center Region (`ScrollArea::vertical().max_height(remaining_h)`):** Two-column Move Table (`#No | White | Black`) supporting both `Figurine` (symbolic pieces) and `Text` SAN notation, with inline feedback icons when assistance mode is enabled.

---

## 6. Animation Engine, Premoves & Reactive Rendering

### 6.1 Decoupled Visual State (`BoardAnimationLayer`)

`ChessMatchState` remains completely authoritative and synchronous. Visual interpolation is handled by an independent presentation layer tracked with `std::time::Instant`:

- `active_slides: Vec<PieceSlideAnim>`: Interpolates pixel coordinates between origin and destination squares (supports simultaneous King and Rook sliding during castling).
- `fading_captures: Vec<CaptureFadeAnim>`: Renders smooth alpha fade-out ($120\text{ ms}$) on captured pieces.
- `snapback_anim: Option<PieceSlideAnim>`: Returns an illegally dropped piece to its source square via an elastic curve over $110\text{ ms}$.
- `check_pulse_sq: Option<Square>`: Displays a pulsing radial crimson glow behind the King when in check.
- `result_glyphs: Option<(Square, ResultGlyph, Square, ResultGlyph)>`: Renders corner win/loss badges directly on the respective Kings upon game over.

### 6.2 Animation Speed Profiles

Configured under `Settings > Board` to match Chess.com options:

- **None / Reduced Motion:** $0\text{ ms}$ (instant movement).
- **Fast:** $95\text{ ms}$ (Cubic Ease-Out: $f(t) = 1 - (1 - t)^3$).
- **Medium (Default):** $155\text{ ms}$ (Cubic Ease-Out: $f(t) = 1 - (1 - t)^3$).
- **Slow:** $275\text{ ms}$ (Quintic Ease-In-Out).
- **Natural:** $175\text{ ms}$ (Quartic Ease-Out with a parabolic midpoint scale curve $1.0 \to 1.08 \to 1.0$ simulating manual piece lift).
- **Arcade:** $140\text{ ms}$ (Back-Out overshoot curve with micro board vibration of $2\text{ px}$ over $80\text{ ms}$ upon checkmate or queen sacrifice).

### 6.3 Input Methods, Premoves & Right-Click Annotations

- **Move Method:** Selectable between `Drag or Click` (default), `Drag Pieces`, and `Click Squares`.
- **Legal Move Indicators:** Subtle translucent dots on empty target squares and boundary rings around capturable enemy pieces.
- **Premove Queue:** While the bot is calculating (with `Enable Premoves` enabled), the player can input a tentative move. Origin and destination squares highlight in translucent red (`#E04F44`). Right-clicking cancels the premove. As soon as `MoveReady` arrives, the premove validates and executes automatically in $0\text{ ms}$.
- **4-Color Right-Click Annotations:**
  - Clean right-click drag: **Red arrow / highlight**
  - `Ctrl` + right-click drag: **Yellow arrow / highlight**
  - `Alt` + right-click drag: **Blue arrow / highlight**
  - `Shift` + right-click drag: **Green arrow / highlight**
  - Left-clicking anywhere clears all drawn annotations.

### 6.4 Reactive Rendering & Memory Footprint

- Call `ctx.request_repaint()` **only** when `!anim_layer.is_idle()`, the evaluation bar is actively lerping ($|\text{display} - \text{target}| > 0.005$), or the player is actively dragging a piece.
- Call `ctx.request_repaint_after(Duration::from_millis(100))` **only** while game clocks are ticking or the engine worker is in its thinking state.
- Emit **zero** repaint requests when the interface is idle on the Home screen, Lobby, or paused match, yielding 0% idle CPU utilization.
- Set default `TranspositionTable` allocation to **`8 MB`** ($262,144$ slots) to keep total resident RAM strictly under $\le 45\text{ MB}$.

---

## 7. Humanized Bot Intelligence & Thinking Delay Scheduler

### 7.1 Root Multi-PV Softmax Sampling (`src/engine/search/negamax.rs`)

To replace robotic blunders with human-like inaccuracies:

1. Evaluate all legal root moves up to `persona.max_depth` (with quiescence search depth limited to $0$ for $\text{ELO} \le 400$, $2$ for $450\text{–}1200$, and $6$ for $1300+$).
2. Rank moves descending by score: $[(m_0, s_0), (m_1, s_1), \dots, (m_{k-1}, s_{k-1})]$.
3. For $T = 0$ ($\text{ELO} \ge 2600$): execute $m_0$ deterministically.
4. For $\text{ELO} < 2600$: calculate score deltas $\Delta_i = s_i - s_0 \le 0$ and sample via the Boltzmann distribution:
   $$P(m_i) = \frac{\exp(\Delta_i / T)}{\sum_{j=0}^{k-1} \exp(\Delta_j / T)}$$
   Severe blunders are dampened for bots above ELO 1200 unless tactical complications exceed their maximum horizon.

### 7.2 Persona Opening Books (`src/engine/bot/opening_book.rs`)

Embedded Zobrist/FEN-keyed move dictionaries for moves 1–6:

- **Martin / Beginners:** Passive flank pawn pushes and non-standard setups.
- **Nelson:** Prioritizes early Queen attacks (`1. e4 e5 2. Qh5` / `2. Bc4 Nc6 3. Qf3`) whenever playable.
- **Hikaru:** Sharp Sicilian Najdorf, King's Indian Defense, and dynamic flank lines.
- **Magnus:** Ruy Lopez, Catalan, Queen's Gambit Declined, and London System.

### 7.3 Non-Blocking Thinking Delay Scheduler (`src/engine/worker.rs`)

Once the engine search concludes in `search_elapsed_ms`, calculate target thinking duration `target_think_ms`:

- **Base Time:** $400\text{ ms}$ for opening book moves or single legal moves; $550\text{ ms}$ for immediate square recaptures; $750\text{ ms} + \min(1400, \text{legal\_count} \times 25)$ for general middlegame positions.
- **Tactical Complexity Additions:** $+400\text{ to }+1200\text{ ms}$ if the top two moves are separated by $< 25\text{ cp}$ or if the King is currently in check.
- **Pace & Clock Modifiers:** Multiply by `persona.pace_factor` ($0.62$ for Hikaru/Martin, $1.0$ for Club bots, $1.35$ for Magnus) and `clock_factor` ($0.35$ for Bullet, $0.65$ for Blitz, $1.0$ for Rapid/Untimed), with $\pm 18\%$ random jitter. Clamp to range $[350\text{ ms}, 3800\text{ ms}]$.
- **Cancellable Polling Loop:** Sleep through the remaining duration `target_think_ms.saturating_sub(search_elapsed_ms)` in chunks of **$25\text{ ms}$**, verifying `abort_flag.load(Ordering::Relaxed)` at each iteration. User actions like `Undo` or `New Game` cancel pending moves within $< 25\text{ ms}$.

---

## 8. Post-Game Review & Local Accuracy Model (`src/engine/review.rs`)

When **Game Review** is triggered from the Game Over modal or the Left Sidebar, run a background analysis pass (depth 5–6 per ply) mapping centipawn scores ($cp$) to White win probability:
$$W(cp) = \frac{1}{1 + 10^{-cp / 400}}$$

Calculate expected-points loss $\Delta W$ ($0.0 \le \Delta W \le 1.0$) relative to the side to move to categorize every ply:

- **Forced (`□`):** Only one legal move was available.
- **Book:** Standard opening theory within the first 8 moves ($\Delta W \le 0.03$).
- **Brilliant (`!!`):** Sound material sacrifice in an undecided position ($W_{\text{before}} < 0.85$, $\Delta W \le 0.02$, $W_{\text{after}} \ge 0.45$).
- **Great (`!`):** Sole winning defense, or finding a tactic where the second-best move loses $\ge 0.18$ win probability.
- **Best (`★`):** Top engine selection (or within $10\text{ cp}$).
- **Excellent:** $\Delta W \le 0.02$.
- **Good:** $0.02 < \Delta W < 0.05$.
- **Inaccuracy (`?!`):** $0.05 \le \Delta W < 0.10$.
- **Miss (`✗`):** Overlooking a forced mate sequence or failing to punish a tactical blunder leaving $\ge 0.18$ advantage.
- **Mistake (`?`):** $0.10 \le \Delta W < 0.20$.
- **Blunder (`??`):** $\Delta W \ge 0.20$.

Compute **Rusty Chess Local Accuracy %** ($0\text{–}100$) for each player from the mean expected-points loss percentage $\overline{\Delta W_{\%}}$:
$$\text{Accuracy} = \operatorname{clamp}\Big(100.0 \times \exp(-0.065 \times \overline{\Delta W_{\%}}), \, 0.0, \, 100.0\Big)$$
Display alongside estimated game performance rating, an interactive evaluation spline graph, and a "Retry Mistakes" drill.

---

## 9. Modular File Architecture

```text
src/
├── persistence.rs              # JSON storage for ~/.local/share/rusty-chess/profile.json (crowns, stats, settings)
├── engine/
│   ├── bot/
│   │   ├── opening_book.rs     # Curated opening book dictionaries keyed by Zobrist hash
│   │   └── persona.rs          # 18 persona definitions with softmax temperature & evaluation weights
│   ├── review.rs               # Background PGN evaluator, move classifier, and accuracy estimator
│   └── worker.rs               # Non-blocking thread with 25 ms polled Thinking Delay Scheduler
└── ui/
    ├── sidebar.rs              # Left navigation rail (Expanded / Compact modes) + Dark/Light toggle
    ├── pages/
    │   ├── home.rs             # Home Dashboard with crown statistics and quick actions
    │   ├── play_hub.rs         # Game mode selector (Bots, Coach, Local 2P, FEN)
    │   ├── bots_lobby.rs       # 1:1 Tiered Play Bots lobby with crown badges & assistance modes
    │   ├── puzzles.rs          # Offline tactical puzzle trainer with 5-heart system
    │   └── review_view.rs      # Game review dashboard with classification pills & eval graph
    ├── board_view/
    │   ├── animation.rs        # BoardAnimationLayer (5 animation profiles, snapbacks, premoves)
    │   └── grid.rs             # 8x8 Board renderer with 4-color arrows & check pulses
    └── modals/
        └── game_over.rs        # Game-over modal awarding crowns, rematch, and game review triggers
```

---

## 10. Phased Implementation Milestones

- **Milestone A (Layout Remediation, Persistence & Reactive Core):**
  Add Top/Bottom Player Strips around the board; constrain right-panel scroll geometry; downscale Transposition Table to $8\text{ MB}$; enforce strict reactive repainting ($0\%$ idle CPU); create `src/persistence.rs`.
- **Milestone B (Left Sidebar, Main Menu & 1:1 Play Bots Lobby):**
  Implement `ActiveRoute` router; build the Left Sidebar; implement Dark/Light token styling; create `Home` and `PlayHub` views; assemble the 18-bot tiered lobby + Engine slider with crowns and Challenge/Friendly/Assisted/Custom modes.
- **Milestone C (Humanized AI & Thinking Delay Scheduler):**
  Implement Root Multi-PV Softmax sampling; create `src/engine/bot/opening_book.rs`; implement Adaptive bot dynamic scaling; wire the cancellable $25\text{ ms}$-polled Thinking Delay Scheduler in `worker.rs`.
- **Milestone D (60–144 FPS Animations, Premoves & Tactical Arrows):**
  Implement `BoardAnimationLayer` with 5 selectable speeds; add piece snap-backs, capture fades, check pulses, Premove Queue, Figurine notation toggles, 4-color annotations, and King result badges.
- **Milestone E (Puzzles, Game Review & Debian Packaging):**
  Implement `puzzles.rs`, `review.rs`, and `game_over.rs`; run full validation with `cargo test`; verify zero layout clipping across $1180\times 780$ and $1366\times 768$; update `scripts/build-deb.sh` for release `v0.2.0`.

---

## 11. Execution Directives for AI Agents (Antigravity)

When processing this specification, adhere strictly to the following instructions:

1. Implement Milestones A through E sequentially. Never emit `todo!()` stubs or placeholder blocks in core modules.
2. After completing each milestone, run `cargo check` and `cargo test` to verify zero compiler errors or borrow-checker regressions.
3. Automatically stage and commit progress with Conventional Commits (e.g., `refactor(ui): player strips and reactive repaint`, `feat(nav): sidebar and 18-bot lobby`, `feat(engine): softmax sampling and delay scheduler`, `feat(anim): board animation layer and premoves`, `feat(review): game review and deb packaging`).
4. Execute `git push origin main` after each verified milestone.
