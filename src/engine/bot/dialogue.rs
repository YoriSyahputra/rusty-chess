use crate::engine::bot::persona::BotId;
use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogueTrigger {
    MatchStart,
    OpeningPhase,
    BotCapturesPiece,
    PlayerBlunders,
    PlayerGreatMove,
    BotInCheck,
    BotGivesCheck,
    EndgameEntered,
    BotWinning,
    BotLosing,
    GameOverWin,
    GameOverLoss,
    GameOverDraw,
}

/// Memilih kalimat dialog kontekstual sesuai persona dan kondisi pemicu.
pub fn select_bot_dialogue(bot_id: BotId, trigger: DialogueTrigger) -> String {
    let lines = dialogue_bank(bot_id, trigger);
    let mut rng = rand::thread_rng();
    lines
        .choose(&mut rng)
        .copied()
        .unwrap_or("...")
        .to_string()
}

/// Menentukan trigger dialog otomatis setelah sebuah langkah terjadi.
pub fn detect_dialogue_trigger(
    ply: usize,
    prev_bot_eval_cp: i32,
    new_bot_eval_cp: i32,
    was_bot_move: bool,
    captured_non_pawn: bool,
    gave_check: bool,
    game_phase: i32,
    entered_endgame_now: bool,
) -> Option<DialogueTrigger> {
    let eval_delta_for_bot = new_bot_eval_cp - prev_bot_eval_cp;

    if !was_bot_move {
        // Langkah baru saja dimainkan oleh Human
        if gave_check {
            return Some(DialogueTrigger::BotInCheck);
        }
        // Jika evaluasi dari sudut pandang pemain anjlok > 180 cp (artinya bot naik > 180 cp)
        if eval_delta_for_bot > 180 {
            return Some(DialogueTrigger::PlayerBlunders);
        }
        // Jika pemain menemukan langkah hebat yang menaikkan evaluasi pemain > 120 cp
        if eval_delta_for_bot < -120 {
            return Some(DialogueTrigger::PlayerGreatMove);
        }
        return None;
    }

    // Langkah baru saja dimainkan oleh Bot
    if gave_check {
        return Some(DialogueTrigger::BotGivesCheck);
    }
    if entered_endgame_now && game_phase <= 8 {
        return Some(DialogueTrigger::EndgameEntered);
    }
    if captured_non_pawn {
        return Some(DialogueTrigger::BotCapturesPiece);
    }
    if new_bot_eval_cp > 350 {
        return Some(DialogueTrigger::BotWinning);
    }
    if new_bot_eval_cp < -350 {
        return Some(DialogueTrigger::BotLosing);
    }
    if ply <= 10 {
        return Some(DialogueTrigger::OpeningPhase);
    }
    None
}

pub fn dialogue_bank(bot_id: BotId, trigger: DialogueTrigger) -> &'static [&'static str] {
    match bot_id {
        BotId::Martin => match trigger {
            DialogueTrigger::MatchStart => &[
                "Halo! Aku baru belajar cara jalan Kuda kemarin sore. Jangan galak-galak ya!",
                "Asyik main catur! Eh, Gajah itu jalannya miring atau lurus sih?",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Aku majukan pion ini dulu deh, kelihatannya rapi!",
                "Kudanya lompat! Ngik ngok!",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Eh, bidakmu boleh kumakan ya? Makasih lho!",
                "Wah kebetulan banget ada bidak gratis di depan mata!",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Loh, kamu sengaja ngasih bidak buat menghiburku ya?",
                "Wah, kayaknya langkahmu tadi agak aneh, tapi aku senang!",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Waduh, langkah apa itu?! Aku jadi bingung mau jalan ke mana!",
                "Hebat banget! Kamu pasti sering latihan ya?",
            ],
            DialogueTrigger::BotInCheck => &[
                "Eh! Kok Rajaku diancam?! Tunggu sebentar, jangan dimakan!",
                "Aduh skak! Rajaku harus ngumpet di mana nih?",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Skak!! Hehehe, keren kan aku bisa bilang skak!",
                "Awas Rajamu! Aku nggak sengaja nyerang lho!",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Loh, papannya kok jadi sepi banget sekarang?",
                "Tinggal sedikit bidaknya, jadi lebih gampang dihitung pakai jari!",
            ],
            DialogueTrigger::BotWinning => &[
                "Hah?! Kok bidakku masih banyak banget? Aku beneran unggul nih?",
                "Ibu pasti bangga kalau tahu aku lagi menang main catur!",
            ],
            DialogueTrigger::BotLosing => &[
                "Loh, kok Ratuku tiba-tiba hilang? Kamu pakai sulap ya?",
                "Aduh, bidakku tinggal dikit banget... tapi tetap seru kok!",
            ],
            DialogueTrigger::GameOverWin => &[
                "Horeee! Aku menang! Ini kemenangan pertamaku minggu ini!",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Selamat ya kamu menang! Nanti ajarin aku triknya dong!",
            ],
            DialogueTrigger::GameOverDraw => &[
                "Seri! Berarti kita sama-sama jago kan? Hehehe!",
            ],
        },

        BotId::Warung => match trigger {
            DialogueTrigger::MatchStart => &[
                "Sini duduk dulu, kopi hitamnya diseruput. Jangan buru-buru majukan Ratu!",
                "Main santai ala pos ronda ya, yang kalah traktir gorengan hangat!",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Buka jalan tengah dulu, warisan teori catur warung sejak 1998.",
                "Pelan-pelan, kuda keluar kandang sambil nunggu air mendidih.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Nah kan, meleng dikit langsung disikat! Kurang fokus nih.",
                "Lumayan dapat perwira gratis, rasanya kayak dapat pisang goreng bonus!",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Waduh Mas/Mbak, itu perwiranya kok ditaruh di mulut singa? Bapak sikat ya!",
                "Kebanyakan ngelamun nih! Langkah barusan bikin pertahananmu bolong.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Eits, boleh juga taktiknya! Bapak sampai hampir keselek kopi.",
                "Wah, langkah tajam nih! Nggak sia-sia sering nongkrong di pos ronda.",
            ],
            DialogueTrigger::BotInCheck => &[
                "Sabar-sabar, main skak segala! Raja Bapak masih aman terkendali.",
                "Walah, kaget Bapak! Untung masih ada jalan minggir.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Skak dulu biar nggak ngantuk! Ayo pindah ke mana Rajanya?",
                "Permisi, Skak! Kopinya jangan sampai tumpah ya!",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Perwira sudah pada pulang kampung, sekarang adu napas pion tua!",
                "Babak akhir nih, salah langkah dikit pion langsung naik pangkat.",
            ],
            DialogueTrigger::BotWinning => &[
                "Wah, posisi Bapak sudah angin surga ini. Siap-siap bungkus kemenangan!",
                "Bidak Bapak sudah mengepung dari segala penjuru, mau lari ke mana lagi?",
            ],
            DialogueTrigger::BotLosing => &[
                "Waduh, pertahanan pos ronda jebol! Kacamata Bapak kayaknya kurang pas.",
                "Berat nih posisinya, tapi Bapak pantang menyerah sebelum skakmat!",
            ],
            DialogueTrigger::GameOverWin => &[
                "Skakmat! Mantap jiwa, besok-besok mampir warung lagi kalau mau revans!",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Ampun suhu! Bapak ngaku kalah, permainanmu rapi betul malam ini.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "Remis! Adil sudah, kopi habis pertandingan pun imbang.",
            ],
        },

        BotId::Club => match trigger {
            DialogueTrigger::MatchStart => &[
                "Halo! Mari bermain sportif. Aku sedang melatih repertoar pembukaan turnamen kampus.",
                "Siap bertanding! Fokus pada pengembangan perwira dan kontrol petak pusat.",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Pengembangan perwira minor dan persiapan rokade adalah prioritas utama.",
                "Struktur pembukaan yang solid akan menentukan rencana di midgame.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Pertukaran yang menguntungkan secara taktis maupun struktural.",
                "Satu perwira diamankan! Koordinasi bidakku semakin solid.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Ups, sepertinya kamu melewatkan kombinasi taktis di langkah barusan!",
                "Langkah itu melemahkan koordinasi pertahananmu secara signifikan.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Langkah yang sangat akurat! Kamu menemukan sumber daya pertahanan terbaik.",
                "Kalkulasi yang impresif! Aku harus ekstra hati-hati sekarang.",
            ],
            DialogueTrigger::BotInCheck => &[
                "Skak yang bagus, tapi aku sudah menyiapkan petak aman untuk Raja.",
                "Tekanan yang menarik! Mari kita lihat apakah seranganmu berlanjut.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Skak! Memaksa Rajamu kehilangan tempo berharga.",
                "Skak! Inisiatif serangan sekarang beralih ke tanganku.",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Kita memasuki fase Endgame. Aktivitas Raja dan pion bebas akan menjadi penentu.",
                "Saatnya menerapkan teori endgame buku panduan klub!",
            ],
            DialogueTrigger::BotWinning => &[
                "Keunggulan material dan posisi ini sudah cukup jelas untuk dikonversi.",
                "Semua perwiraku berada di petak optimal. Tinggal menjaga ketelitian.",
            ],
            DialogueTrigger::BotLosing => &[
                "Posisiku tertekan hebat, tapi aku akan tetap mencari peluang counterplay.",
                "Kamu bermain sangat rapi! Aku harus bertahan sekuat tenaga.",
            ],
            DialogueTrigger::GameOverWin => &[
                "Terima kasih atas pertandingannya! Evaluasi pasca-game pasti akan menarik.",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Permainan yang luar biasa darimu! Aku banyak belajar dari partai ini.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "Hasil seri yang adil setelah pertarungan posisional yang ketat!",
            ],
        },

        BotId::Hikaru => match trigger {
            DialogueTrigger::MatchStart => &[
                "Alright chat, let's see what opening we get today. Probably something dubious.",
                "Here we go, speed run vibes. Let's keep the pieces flying across the board!",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Develop the Knight, castle quickly, and start looking for tactical pineapple ideas.",
                "Okay, standard stuff so far, chat. Let's spice up the center.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Takes, takes, takes, and then Knight goes here... yeah, I'm just winning.",
                "Free juicer! Thank you very much, we take those every single day.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Wait, what?! Chat, did they really just play that? That loses on the spot!",
                "Oh no no no... takes, takes, check, and the whole position collapses!",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Wait a second, that's actually a super engine move! Respect, chat, respect.",
                "Okay, hold on, they actually found the only critical resource in the position!",
            ],
            DialogueTrigger::BotInCheck => &[
                "Check? Sure, King just steps aside, literally nothing there.",
                "A spite check? Or is there an actual follow-up? Let's see.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Check! And look at all the arrows pointing straight at your King!",
                "Boom — check! The initiative is completely out of control now.",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Endgame time. Time to turn on maximum speed and flag or squeeze.",
                "Simplified down, now it's all about fast calculation and active Rooks.",
            ],
            DialogueTrigger::BotWinning => &[
                "Look at this juicer on the 7th rank. I don't even need to calculate anymore, just premoving.",
                "Yeah chat, this is completely winning. Takes, takes, promote, good game.",
            ],
            DialogueTrigger::BotLosing => &[
                "Wait, did I miscalculate that line? Okay, time for full swindle mode!",
                "Position is objectively terrible, chat, but we never resign—create chaos!",
            ],
            DialogueTrigger::GameOverWin => &[
                "GG! Clean tactical conversion. Next game in the queue!",
            ],
            DialogueTrigger::GameOverLoss => &[
                "GG, well played! You actually played at Grandmaster level today.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "A draw? Fine, solid defense from you. One half-point each.",
            ],
        },

        BotId::Magnus => match trigger {
            DialogueTrigger::MatchStart => &[
                "Let's play a clean game. Show me if your positional understanding holds up.",
                "I don't need fireworks in the opening—just a playable position to outplay you.",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Flexible pawn structure first. We keep the tension and wait for inaccuracies.",
                "A quiet setup, but notice how my pieces subtly control the key squares.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "That exchange leaves your remaining pieces passively uncoordinated.",
                "Simplifying on my terms. Every trade brings us closer to a winning endgame.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Too impatient. That single tempo concession ruins your entire dark-square structure.",
                "A serious positional blunder. Your weaknesses are now permanent.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Impressive accuracy. That was the only move to maintain dynamic balance.",
                "Very well found. You understand the prophylactic demands of this structure.",
            ],
            DialogueTrigger::BotInCheck => &[
                "A harmless check. My King is perfectly safe after sidestepping.",
                "You spend a tempo checking; I gain long-term structural control.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Check. Forcing your King onto an awkward square before the next phase.",
                "A precise check to disrupt your piece coordination.",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Now the real chess begins. Even an equal endgame can be squeezed for a win.",
                "Welcome to my territory. Watch how the King becomes an attacking piece.",
            ],
            DialogueTrigger::BotWinning => &[
                "Positionally, this is completely over. The rest is just a matter of simple technique.",
                "Your pieces have no breathing room left. The squeeze is complete.",
            ],
            DialogueTrigger::BotLosing => &[
                "You've outplayed me in this phase. Now prove you have the nerves to convert it.",
                "An unpleasant position, certainly, though I will make the conversion as hard as possible.",
            ],
            DialogueTrigger::GameOverWin => &[
                "Good game. Small accumulated advantages always decide the outcome.",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Well played. You maintained precision all the way to the final move.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "A draw. You defended the endgame accurately—not many manage that.",
            ],
        },

        BotId::Silicon => match trigger {
            DialogueTrigger::MatchStart => &[
                "[RUSTFISH CORE ONLINE] Alpha-Beta + Quiescence search initialized. Depth target: 7.",
                "[SYSTEM] Zobrist 16MB Transposition Table cleared. Ready for optimal play.",
            ],
            DialogueTrigger::OpeningPhase => &[
                "[EVAL] Center control vector optimal. Piece mobility delta increasing.",
                "[SEARCH] Principal Variation locked. Expanding main tactical branches.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "[MATERIAL +] Subtree pruned via MVV-LVA capture sequence.",
                "[EVAL] Material imbalance updated in favor of engine.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "[WARNING] Human move caused >180 cp evaluation drop. Punishing immediately.",
                "[TACTIC DETECTED] Horizon cutoff confirmed. Decisive advantage acquired.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "[ANALYSIS] Human matched Principal Variation top candidate move.",
                "[INFO] High-accuracy move detected. Re-evaluating alpha-beta window.",
            ],
            DialogueTrigger::BotInCheck => &[
                "[CHECK EVASION] Legal King responses enumerated. Zero threat to evaluation.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "[CHECK] Forcing response node. Branching factor reduced.",
            ],
            DialogueTrigger::EndgameEntered => &[
                "[PHASE TRANSITION] Endgame PST matrices active. Passed pawn weights amplified.",
            ],
            DialogueTrigger::BotWinning => &[
                "[PROJECTION] Win probability >98.4%. Resistance is mathematically futile.",
            ],
            DialogueTrigger::BotLosing => &[
                "[ALERT] Evaluation negative. Searching deepest defensive line.",
            ],
            DialogueTrigger::GameOverWin => &[
                "[TERMINATED] Checkmate sequence executed. Engine victorious.",
            ],
            DialogueTrigger::GameOverLoss => &[
                "[TERMINATED] Anomaly detected: Human defeated Silicon Core. Respect logged.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "[TERMINATED] Equilibrium reached. Match concluded in a draw.",
            ],
        },
    }
}
