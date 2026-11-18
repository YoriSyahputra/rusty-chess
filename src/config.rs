use crate::domain::TimeControl;
use crate::ui::theme::BoardTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AnimationProfile {
    None,
    Fast,
    Medium,
    Slow,
    Natural,
    Arcade,
}

impl AnimationProfile {
    pub fn all() -> &'static [AnimationProfile] {
        &[
            AnimationProfile::None,
            AnimationProfile::Fast,
            AnimationProfile::Medium,
            AnimationProfile::Slow,
            AnimationProfile::Natural,
            AnimationProfile::Arcade,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AnimationProfile::None => "None (0 ms)",
            AnimationProfile::Fast => "Fast (95 ms)",
            AnimationProfile::Medium => "Medium (155 ms)",
            AnimationProfile::Slow => "Slow (275 ms)",
            AnimationProfile::Natural => "Natural (175 ms)",
            AnimationProfile::Arcade => "Arcade (140 ms)",
        }
    }

    pub fn duration_ms(&self) -> u64 {
        match self {
            AnimationProfile::None => 0,
            AnimationProfile::Fast => 95,
            AnimationProfile::Medium => 155,
            AnimationProfile::Slow => 275,
            AnimationProfile::Natural => 175,
            AnimationProfile::Arcade => 140,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MoveInputMethod {
    DragOrClick,
    DragOnly,
    ClickOnly,
}

impl MoveInputMethod {
    pub fn all() -> &'static [MoveInputMethod] {
        &[
            MoveInputMethod::DragOrClick,
            MoveInputMethod::DragOnly,
            MoveInputMethod::ClickOnly,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MoveInputMethod::DragOrClick => "Drag or Click",
            MoveInputMethod::DragOnly => "Drag Pieces",
            MoveInputMethod::ClickOnly => "Click Squares",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum NotationStyle {
    Figurine,
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayColorChoice {
    White,
    Random,
    Black,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AssistanceMode {
    Challenge,
    Friendly,
    Assisted,
    Custom,
}

impl AssistanceMode {
    pub fn all() -> &'static [AssistanceMode] {
        &[
            AssistanceMode::Challenge,
            AssistanceMode::Friendly,
            AssistanceMode::Assisted,
            AssistanceMode::Custom,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            AssistanceMode::Challenge => "Challenge (3★)",
            AssistanceMode::Friendly => "Friendly (2★)",
            AssistanceMode::Assisted => "Assisted (1★)",
            AssistanceMode::Custom => "Custom",
        }
    }

    pub fn max_crowns(&self) -> u8 {
        match self {
            AssistanceMode::Challenge => 3,
            AssistanceMode::Friendly => 2,
            AssistanceMode::Assisted => 1,
            AssistanceMode::Custom => 3,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AssistanceMode::Challenge => {
                "3 Crowns max • No hints, no takebacks, eval bar & assistance arrows off."
            }
            AssistanceMode::Friendly => {
                "2 Crowns max • Up to 3 hints and 3 takebacks; live eval bar disabled."
            }
            AssistanceMode::Assisted => {
                "1 Crown max • Unlimited hints & takebacks; real-time eval bar & threat/suggestion arrows active."
            }
            AssistanceMode::Custom => {
                "1–3 Crowns • Granular toggles for chat, eval bar, arrows, and feedback."
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomAssistanceConfig {
    pub bot_chat: bool,
    pub eval_bar: bool,
    pub threat_arrows: bool,
    pub suggestion_arrows: bool,
    pub move_feedback: bool,
    pub unlimited_takebacks: bool,
}

impl Default for CustomAssistanceConfig {
    fn default() -> Self {
        Self {
            bot_chat: true,
            eval_bar: true,
            threat_arrows: false,
            suggestion_arrows: false,
            move_feedback: true,
            unlimited_takebacks: true,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct UserConfig {
    pub dark_mode: bool,
    pub sidebar_collapsed: Option<bool>,
    pub board_theme: BoardTheme,
    pub animation_profile: AnimationProfile,
    pub move_input_method: MoveInputMethod,
    pub enable_premoves: bool,
    pub notation_style: NotationStyle,
    pub sound_enabled: bool,
    pub board_flipped: bool,
    pub show_coordinates: bool,
    pub show_eval_bar: bool,
    pub time_control: TimeControl,
    pub assistance_mode: AssistanceMode,
    pub custom_assistance: CustomAssistanceConfig,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            dark_mode: true,
            sidebar_collapsed: None,
            board_theme: BoardTheme::ClassicGreen,
            animation_profile: AnimationProfile::Medium,
            move_input_method: MoveInputMethod::DragOrClick,
            enable_premoves: true,
            notation_style: NotationStyle::Figurine,
            sound_enabled: true,
            board_flipped: false,
            show_coordinates: true,
            show_eval_bar: true,
            time_control: TimeControl::Rapid10_0,
            assistance_mode: AssistanceMode::Challenge,
            custom_assistance: CustomAssistanceConfig::default(),
        }
    }
}
