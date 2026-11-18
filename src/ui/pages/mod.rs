pub mod bots_lobby;
pub mod home;
pub mod play_hub;
pub mod puzzles;
pub mod review_view;

pub use bots_lobby::{render_bots_lobby, BotsLobbyActions, BotsLobbyState};
pub use home::{render_home_page, HomePageActions};
pub use play_hub::{render_play_hub, PlayHubActions};
pub use puzzles::{render_puzzles_page, PuzzlesTrainerState, TacticalPuzzle, CURATED_PUZZLES};
pub use review_view::{render_review_page, ReviewViewState};
