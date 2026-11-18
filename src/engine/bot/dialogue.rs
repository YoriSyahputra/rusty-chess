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
#[allow(clippy::too_many_arguments)]
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
        if gave_check {
            return Some(DialogueTrigger::BotInCheck);
        }
        if eval_delta_for_bot > 180 {
            return Some(DialogueTrigger::PlayerBlunders);
        }
        if eval_delta_for_bot < -120 {
            return Some(DialogueTrigger::PlayerGreatMove);
        }
        return None;
    }

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
        BotId::Martin | BotId::Elani | BotId::Jimmy | BotId::Aron => match trigger {
            DialogueTrigger::MatchStart => &[
                "Halo! Aku baru belajar cara jalan Kuda kemarin sore. Jangan galak-galak ya!",
                "Hi there! Let's have a fun, relaxed game of chess today!",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Aku majukan pion ini dulu deh, kelihatannya rapi!",
                "Knights jump over things! That's my favorite rule.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Eh, bidakmu boleh kumakan ya? Makasih lho!",
                "Ooh, a free piece right in front of me!",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Loh, kamu sengaja ngasih bidak buat menghiburku ya?",
                "Wait, did you leave that piece undefended for me?",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Waduh, langkah apa itu?! Aku jadi bingung mau jalan ke mana!",
                "Wow, that move looks super smart!",
            ],
            DialogueTrigger::BotInCheck => &[
                "Eh! Kok Rajaku diancam?! Tunggu sebentar, jangan dimakan!",
                "Ack, check! Where should my King hide?",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Skak!! Hehehe, keren kan aku bisa bilang skak!",
                "Check! Watch out for your King!",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Loh, papannya kok jadi sepi banget sekarang?",
                "So few pieces left! Now I can count them on my fingers.",
            ],
            DialogueTrigger::BotWinning => &[
                "Hah?! Kok bidakku masih banyak banget? Aku beneran unggul nih?",
                "Wait, am I actually winning a real chess game?!",
            ],
            DialogueTrigger::BotLosing => &[
                "Loh, kok Ratuku tiba-tiba hilang? Kamu pakai sulap ya?",
                "Uh oh, my pieces are disappearing fast, but I'm still having fun!",
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

        BotId::Nelson => match trigger {
            DialogueTrigger::MatchStart => &[
                "Don't blink, or my Queen will already be on h5! Let's see how you defend.",
                "Pawns are overrated—my Queen is coming straight for your King!",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Bring out the Queen! Why wait for the middlegame to attack?",
                "Target locked on f7/f2. Can you handle the early pressure?",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Boom! My Queen devours another piece!",
                "Too slow! When the Queen raids, nothing is safe.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Haha! You fell right into my Queen trap!",
                "Leaving that square unguarded against me is fatal!",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Wait—you actually parried my Queen attack AND gained tempo?!",
                "Hey, stop chasing my Queen around the board!",
            ],
            DialogueTrigger::BotInCheck => &[
                "Hey! I'm supposed to be the one giving checks here!",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Check! My Queen sends her regards!",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Wait, how did we reach an endgame? Where did my attack go?",
            ],
            DialogueTrigger::BotWinning => &[
                "Told you! Early Queen attacks work every single time!",
            ],
            DialogueTrigger::BotLosing => &[
                "Okay, maybe bringing the Queen out on move 2 was slightly ambitious...",
            ],
            DialogueTrigger::GameOverWin => &[
                "Crushed by the Queen raid! Want a rematch?",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Ugh, you refuted my entire attack! Fine, you win this one.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "A draw?! After all those fireworks?",
            ],
        },

        BotId::Mittens => match trigger {
            DialogueTrigger::MatchStart => &[
                "Meow. I am Mittens. My rating says 1, so this should be easy for you... right? Hehehe.",
                "Purrrr... look at all those shiny pieces on the board. Let us play.",
            ],
            DialogueTrigger::OpeningPhase => &[
                "A quiet little paw step forward. Nothing to worry about... meow.",
                "Hehehe... developing my claws into the center.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Nom. That piece belonged to me anyway. Meow.",
                "Swiped off the table! Just like a glass of milk.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Meow...? Did you really think I wouldn't see 14 moves ahead? Hehehe.",
                "Oh dear. The abyss gazes back at your position.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Purr... interesting. You delayed the inevitable by two whole moves.",
                "A clever human move! You earn one head pat.",
            ],
            DialogueTrigger::BotInCheck => &[
                "Hiss! Touching the royal cat is strictly forbidden.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Meow — Check. Nowhere to run, little mouse.",
            ],
            DialogueTrigger::EndgameEntered => &[
                "The board grows cold and empty. Time for the endgame pounce.",
            ],
            DialogueTrigger::BotWinning => &[
                "Hehehe... everything is proceeding according to the feline prophecy. Meow.",
            ],
            DialogueTrigger::BotLosing => &[
                "Hiss!! Impossible! Did someone unplug my yarn ball?!",
            ],
            DialogueTrigger::GameOverWin => &[
                "Checkmate. Meow! Another grandmaster soul collected.",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Meow?! You defeated Mittens?! I demand tuna and an immediate rematch!",
            ],
            DialogueTrigger::GameOverDraw => &[
                "A draw. I shall nap on the chessboard now. Purrrr.",
            ],
        },

        BotId::Warung | BotId::Emir | BotId::Sven | BotId::AdaptiveJimmy => match trigger {
            DialogueTrigger::MatchStart => &[
                "Sini duduk dulu, kopi hitamnya diseruput. Jangan buru-buru majukan Ratu!",
                "Let's have a solid tactical match! Ready when you are.",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Buka jalan tengah dulu, kontrol petak pusat sejak awal.",
                "Developing pieces smoothly toward the center.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Nah kan, meleng dikit langsung disikat! Kurang fokus nih.",
                "Clean tactical exchange in my favor!",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Waduh, itu perwiranya kok ditaruh di mulut singa? Langsung disikat!",
                "Oops, that move leaves a huge tactical hole in your camp!",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Eits, boleh juga taktiknya! Langkah yang sangat tajam!",
                "Sharp move! I didn't expect that tactical resource.",
            ],
            DialogueTrigger::BotInCheck => &[
                "Sabar-sabar, main skak segala! Raja masih aman terkendali.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Skak dulu biar nggak ngantuk! Ayo pindah ke mana Rajanya?",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Perwira sudah pada pulang kampung, sekarang adu teknik pion di babak akhir!",
            ],
            DialogueTrigger::BotWinning => &[
                "Wah, posisi sudah angin surga ini. Siap-siap bungkus kemenangan!",
            ],
            DialogueTrigger::BotLosing => &[
                "Waduh, pertahanan mulai jebol! Tapi pantang menyerah sebelum skakmat!",
            ],
            DialogueTrigger::GameOverWin => &[
                "Skakmat! Mantap jiwa, kapan-kapan kita tanding ulang lagi!",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Permainanmu rapi betul hari ini! Selamat atas kemenangannya.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "Remis! Adil sudah, pertandingan yang seimbang.",
            ],
        },

        BotId::Club
        | BotId::Antonio
        | BotId::Isabel
        | BotId::Wally
        | BotId::Li
        | BotId::Noam
        | BotId::Francis
        | BotId::Danny => match trigger {
            DialogueTrigger::MatchStart => &[
                "Halo! Mari bermain sportif. Fokus pada pengembangan perwira dan struktur pion.",
                "Let's play principled tournament chess. May the sharpest calculation win!",
            ],
            DialogueTrigger::OpeningPhase => &[
                "Pengembangan perwira minor dan persiapan rokade adalah prioritas utama.",
                "Classical development—controlling key central squares before launching operations.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "Pertukaran yang menguntungkan secara taktis maupun struktural.",
                "Winning material while improving piece coordination!",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "Ups, sepertinya kamu melewatkan kombinasi taktis di langkah barusan!",
                "That inaccuracy compromises your pawn structure and king safety.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "Langkah yang sangat akurat! Kamu menemukan sumber daya pertahanan terbaik.",
                "First-class calculation! That was the strongest continuation on the board.",
            ],
            DialogueTrigger::BotInCheck => &[
                "Skak yang bagus, tapi aku sudah menyiapkan petak aman untuk Raja.",
            ],
            DialogueTrigger::BotGivesCheck => &[
                "Skak! Memaksa Rajamu kehilangan tempo berharga.",
            ],
            DialogueTrigger::EndgameEntered => &[
                "Kita memasuki fase Endgame. Aktivitas Raja dan pion bebas akan menjadi penentu.",
            ],
            DialogueTrigger::BotWinning => &[
                "Keunggulan material dan posisi ini sudah cukup jelas untuk dikonversi.",
            ],
            DialogueTrigger::BotLosing => &[
                "Posisiku tertekan hebat, tapi aku akan tetap mencari peluang counterplay.",
            ],
            DialogueTrigger::GameOverWin => &[
                "Terima kasih atas pertandingannya! Evaluasi pasca-game pasti akan menarik.",
            ],
            DialogueTrigger::GameOverLoss => &[
                "Permainan yang luar biasa darimu! Kamu layak mendapatkan mahkota ini.",
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

        BotId::Silicon | BotId::EngineCustom => match trigger {
            DialogueTrigger::MatchStart => &[
                "[RUSTFISH CORE ONLINE] Alpha-Beta + Quiescence search initialized.",
                "[SYSTEM] Zobrist 8MB Transposition Table ready. Searching optimal continuation.",
            ],
            DialogueTrigger::OpeningPhase => &[
                "[EVAL] Center control vector optimal. Piece mobility delta increasing.",
            ],
            DialogueTrigger::BotCapturesPiece => &[
                "[MATERIAL +] Subtree pruned via MVV-LVA capture sequence.",
            ],
            DialogueTrigger::PlayerBlunders => &[
                "[WARNING] Human move caused >180 cp evaluation drop. Punishing immediately.",
            ],
            DialogueTrigger::PlayerGreatMove => &[
                "[ANALYSIS] Human matched Principal Variation top candidate move.",
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
                "[TERMINATED] Human defeated Engine Core. Respect logged.",
            ],
            DialogueTrigger::GameOverDraw => &[
                "[TERMINATED] Equilibrium reached. Match concluded in a draw.",
            ],
        },
    }
}
