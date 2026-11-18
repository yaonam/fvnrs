use serde::{Deserialize, Serialize};

use crate::{schema::schema::SceneSchema, utils::get_current_time_ms};

pub enum TextScrollMode {
    ScrollStart(u64), // Start time ms
    ScrollEnd(u64),   // End time ms
}

impl Default for TextScrollMode {
    fn default() -> Self {
        TextScrollMode::ScrollEnd(0)
    }
}

pub struct SessionSchema {
    /// Timestamp for scrolling text (set to 0 for instant)
    pub text_scroll_mode: TextScrollMode,
    /// Timestamp when the current GameAction was applied for Auto.
    pub scene_index_start_ms: u64,
    pub user_input_override: Option<UserInputOverride>,
    // Playtime
    pub session_start_ms: u64,
    pub previous_playtime_sec: u64,    // Playtime at start of session
    pub max_scene_schema: SceneSchema, // The last scene that was reached
}

impl Default for SessionSchema {
    fn default() -> Self {
        Self {
            text_scroll_mode: TextScrollMode::default(),
            scene_index_start_ms: 0,
            user_input_override: None,
            session_start_ms: 0,
            previous_playtime_sec: 0,
            max_scene_schema: SceneSchema::default(),
        }
    }
}

impl SessionSchema {
    /// Sets the scroll mode to ScrollEnd with current time, if not already.
    pub fn end_text_scroll(&mut self) {
        if !matches!(self.text_scroll_mode, TextScrollMode::ScrollEnd(_)) {
            self.text_scroll_mode = TextScrollMode::ScrollEnd(get_current_time_ms());
        }
    }

    /// Set to current time
    pub fn start_text_scroll(&mut self) {
        self.text_scroll_mode = TextScrollMode::ScrollStart(get_current_time_ms());
    }

    pub fn start_session(&mut self) {
        self.session_start_ms = get_current_time_ms();
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Copy)]
pub enum UserInputOverride {
    /// Hides the UI, should disable/pause everything else
    Hide,
    /// Fast forward to last read
    Skip,
    /// Auto-advance the based on the settings
    Auto,
}
