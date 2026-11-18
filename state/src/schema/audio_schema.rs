use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ChannelsSchema {
    pub bgm: Option<AudioSchema>,
    pub sfx: Option<AudioSchema>,
}

impl Default for ChannelsSchema {
    fn default() -> Self {
        Self {
            bgm: None,
            sfx: None,
        }
    }
}

impl ChannelsSchema {
    pub fn clear_unlooped_audio(&mut self) {
        if let Some(audio) = self.bgm.as_ref()
            && audio.state != Some(AudioSchemaState::Looping)
        {
            self.bgm = None;
        }
        if let Some(audio) = self.sfx.as_ref()
            && audio.state != Some(AudioSchemaState::Looping)
        {
            self.sfx = None;
        }
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AudioSchemaState {
    Looping,
    FadingOut(f32),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct AudioSchema {
    pub id: String,
    pub volume_multiplier: Option<f32>,
    pub fade_in_seconds: Option<f32>,
    pub delay_seconds: Option<f32>,
    pub state: Option<AudioSchemaState>,
}

impl AudioSchema {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            volume_multiplier: None,
            fade_in_seconds: None,
            delay_seconds: None,
            state: None,
        }
    }
}
