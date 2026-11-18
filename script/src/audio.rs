use state::{
    game_state::{GameState, GameStateInputMode},
    schema::audio_schema::{AudioSchema, AudioSchemaState},
};

enum AudioType {
    Bgm,
    Sfx,
}

pub struct PlayAudioBuilder<'a> {
    audio_type: AudioType,
    game_state: &'a mut GameState,
}

impl<'a> PlayAudioBuilder<'a> {
    pub fn fade_in(mut self, duration_seconds: f32) -> Self {
        if let Some(channel) = self.get_channel_mut() {
            channel.fade_in_seconds = Some(duration_seconds);
        }
        self
    }

    pub fn volume(mut self, multiplier: f32) -> Self {
        if let Some(channel) = self.get_channel_mut() {
            channel.volume_multiplier = Some(multiplier);
        }
        self
    }

    pub fn delay(mut self, duration_seconds: f32) -> Self {
        if let Some(channel) = self.get_channel_mut() {
            channel.delay_seconds = Some(duration_seconds);
        }
        self
    }

    pub fn repeat(mut self) -> Self {
        if let Some(channel) = self.get_channel_mut() {
            channel.state = Some(AudioSchemaState::Looping);
        }
        self
    }

    pub fn and(self) -> Self {
        self.game_state.input_mode = GameStateInputMode::Continue;
        self
    }

    fn get_channel_mut(&mut self) -> &mut Option<AudioSchema> {
        match self.audio_type {
            AudioType::Bgm => &mut self.game_state.playable.audio.bgm,
            AudioType::Sfx => &mut self.game_state.playable.audio.sfx,
        }
    }
}

pub fn play_bgm<'a>(game_state: &'a mut GameState, audio_id: &'static str) -> PlayAudioBuilder<'a> {
    game_state.playable.audio.bgm = Some(AudioSchema::new(audio_id));
    PlayAudioBuilder {
        audio_type: AudioType::Bgm,
        game_state,
    }
}

pub fn play_sfx<'a>(game_state: &'a mut GameState, audio_id: &'static str) -> PlayAudioBuilder<'a> {
    game_state.playable.audio.sfx = Some(AudioSchema::new(audio_id));
    PlayAudioBuilder {
        audio_type: AudioType::Sfx,
        game_state,
    }
}

pub struct StopAudioBuilder<'a> {
    audio_type: AudioType,
    game_state: &'a mut GameState,
}

impl<'a> StopAudioBuilder<'a> {
    pub fn fade_out(mut self, duration_seconds: f32) -> Self {
        if let Some(channel) = self.get_channel_mut() {
            channel.state = Some(AudioSchemaState::FadingOut(duration_seconds));
        }
        self
    }

    fn get_channel_mut(&mut self) -> &mut Option<AudioSchema> {
        match self.audio_type {
            AudioType::Bgm => &mut self.game_state.playable.audio.bgm,
            AudioType::Sfx => &mut self.game_state.playable.audio.sfx,
        }
    }

    pub fn and(self) -> Self {
        self.game_state.input_mode = GameStateInputMode::Continue;
        self
    }
}

pub fn stop_bgm<'a>(game_state: &'a mut GameState) -> StopAudioBuilder<'a> {
    if let Some(channel) = game_state.playable.audio.bgm.as_mut() {
        channel.state = Some(AudioSchemaState::FadingOut(0.0));
    }
    StopAudioBuilder {
        audio_type: AudioType::Bgm,
        game_state,
    }
}

pub fn stop_sfx<'a>(game_state: &'a mut GameState) -> StopAudioBuilder<'a> {
    if let Some(channel) = game_state.playable.audio.sfx.as_mut() {
        channel.state = Some(AudioSchemaState::FadingOut(0.0));
    }
    StopAudioBuilder {
        audio_type: AudioType::Sfx,
        game_state,
    }
}
