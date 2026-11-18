use shakmaty::Move;

pub const TT_SIZE_BYTES: usize = 8 * 1024 * 1024; // 8 MB fixed allocation (262,144 slots)
pub const MATE_SCORE: i32 = 30_000;
pub const MATE_THRESHOLD: i32 = 29_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TTFlag {
    Exact,
    LowerBound,
    UpperBound,
}

#[derive(Debug, Clone)]
pub struct TTEntry {
    pub key: u64,
    pub depth: u8,
    pub flag: TTFlag,
    pub score: i32,
    pub best_move: Option<Move>,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    mask: usize,
}

impl TranspositionTable {
    /// Membuat Transposition Table berukuran tetap maksimal 16 MB (~524_288 entries).
    pub fn new() -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>().max(32);
        let raw_count = TT_SIZE_BYTES / entry_size;
        let num_entries = raw_count.next_power_of_two() >> 1;
        let count = num_entries.max(65_536);

        Self {
            entries: vec![None; count],
            mask: count - 1,
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.entries {
            *slot = None;
        }
    }

    #[inline]
    pub fn probe(&self, key: u64) -> Option<&TTEntry> {
        let idx = (key as usize) & self.mask;
        if let Some(entry) = &self.entries[idx] {
            if entry.key == key {
                return Some(entry);
            }
        }
        None
    }

    #[inline]
    pub fn store(
        &mut self,
        key: u64,
        depth: u8,
        flag: TTFlag,
        score: i32,
        best_move: Option<Move>,
        ply: i32,
    ) {
        let idx = (key as usize) & self.mask;
        let replace = match &self.entries[idx] {
            None => true,
            Some(existing) => existing.key != key || depth >= existing.depth,
        };

        if replace {
            let adjusted_score = score_to_tt(score, ply);
            self.entries[idx] = Some(TTEntry {
                key,
                depth,
                flag,
                score: adjusted_score,
                best_move,
            });
        }
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }
}

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[inline]
pub fn score_to_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_THRESHOLD {
        score + ply
    } else if score < -MATE_THRESHOLD {
        score - ply
    } else {
        score
    }
}

#[inline]
pub fn score_from_tt(score: i32, ply: i32) -> i32 {
    if score > MATE_THRESHOLD {
        score - ply
    } else if score < -MATE_THRESHOLD {
        score + ply
    } else {
        score
    }
}
