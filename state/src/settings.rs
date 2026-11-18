use serde::{Deserialize, Serialize};

// Settings read from and saved to disk
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Settings {
    // Text
    // Max speed will be clipped to instant (chars/second)
    pub text_speed: u64,
    /// As a multiplier of the text length (len * wait = wait time in ms)
    pub text_wait: u64,
    pub skip_unseen: bool,
    pub comma_pause_ms: u64,
    pub period_pause_ms: u64,

    // Audio
    pub bgm_volume: u8,
    pub sfx_volume: u8,
    pub play_in_bg: bool,

    // Video
    pub fullscreen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            text_speed: 50,
            text_wait: 20,
            skip_unseen: false,
            comma_pause_ms: 250,
            period_pause_ms: 300,
            bgm_volume: 20,
            sfx_volume: 20,
            #[cfg(not(target_os = "android"))]
            play_in_bg: true, // Keep playing when app is in background
            #[cfg(target_os = "android")]
            play_in_bg: false, // Mute when app is closed
            fullscreen: false,
        }
    }
}
