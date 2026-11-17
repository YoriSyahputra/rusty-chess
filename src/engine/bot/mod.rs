pub mod dialogue;
pub mod persona;

pub use dialogue::{detect_dialogue_trigger, dialogue_bank, select_bot_dialogue, DialogueTrigger};
pub use persona::{BotId, BotPersona};
