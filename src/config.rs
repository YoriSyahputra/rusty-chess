use crate::domain::TimeControl;
use crate::ui::theme::BoardTheme;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserConfig {
    pub board_theme: BoardTheme,
    pub sound_enabled: bool,
    pub board_flipped: bool,
    pub show_coordinates: bool,
    pub show_eval_bar: bool,
    pub time_control: TimeControl,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            board_theme: BoardTheme::ClassicGreen,
            sound_enabled: true,
            board_flipped: false,
            show_coordinates: true,
            show_eval_bar: true,
            time_control: TimeControl::Rapid10_0,
        }
    }
}
