use std::{collections::HashMap, io::Cursor, num::NonZero, sync::Arc, time::Duration};

use eframe::egui;
use rodio::{Decoder, Source};
use state::{
    schema::audio_schema::{AudioSchema, AudioSchemaState, ChannelsSchema},
    settings::Settings,
    utils::get_current_time_ms,
};

const RETRY_INTERVAL_MS: u64 = 1500;

#[derive(Debug, Clone, PartialEq)]
struct FadeOut {
    start_volume: f32,
    seconds: f32,
    end_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
enum AudioState {
    Looping,
    FadingOut(FadeOut),
}

#[derive(Debug, Clone, PartialEq)]
struct PersistentAudio {
    id: String,
    state: Option<AudioState>,
}

impl PersistentAudio {
    fn matches(&self, audio_schema: &AudioSchema) -> bool {
        self.id == audio_schema.id
            && match (&self.state, &audio_schema.state) {
                (Some(AudioState::Looping), Some(AudioSchemaState::Looping)) => true,
                (
                    Some(AudioState::FadingOut(fade_out)),
                    Some(AudioSchemaState::FadingOut(fade_out_schema)),
                ) => fade_out.seconds == *fade_out_schema,
                (None, None) => true,
                _ => {
                    log::info!(
                        "Audio state mismatch: {:?} vs {:?}",
                        self.state,
                        audio_schema.state
                    );
                    false
                }
            }
    }
}

pub(crate) struct AudioSink {
    pub(crate) bgm_player: rodio::Player,
    pub(crate) sfx_player: rodio::Player,
    /// Keep alive!
    _audio_handle: rodio::MixerDeviceSink,
}

pub(crate) struct AudioPlayback {
    sink: Option<AudioSink>,
    /// Used to keep track of the currently playing audio.
    playing_bgm: Option<PersistentAudio>,
    playing_sfx: Option<PersistentAudio>,
    last_retry_ms: u64,
    // True if loses focus and not play_in_bg
    paused: bool,
}

impl AudioPlayback {
    /// Attempts to open the default audio sink. Returns true if successful.
    pub fn try_init(&mut self, settings: Option<&Settings>) -> bool {
        if self.sink.is_some() {
            return true;
        }

        self.last_retry_ms = get_current_time_ms();
        match rodio::DeviceSinkBuilder::open_default_sink() {
            Ok(audio_handle) => {
                log::info!("Audio device initialized successfully.");
                let bgm_player = rodio::Player::connect_new(&audio_handle.mixer());
                let sfx_player = rodio::Player::connect_new(&audio_handle.mixer());
                if let Some(settings) = settings {
                    bgm_player.set_volume(settings.bgm_volume as f32 / 100.0);
                    sfx_player.set_volume(settings.sfx_volume as f32 / 100.0);
                }
                if self.paused {
                    bgm_player.pause();
                    sfx_player.pause();
                }
                self.sink = Some(AudioSink {
                    bgm_player,
                    sfx_player,
                    _audio_handle: audio_handle,
                });
                self.playing_bgm = None;
                self.playing_sfx = None;
                true
            }
            Err(e) => {
                log::warn!(
                    "Audio device initialization failed: {e}. Audio playback will retry in background."
                );
                false
            }
        }
    }

    /// Handles audio channels and retries initialization if the audio device is unavailable.
    /// Returns whether or not eframe needs to repaint immediately.
    pub fn handle_audio(
        &mut self,
        channel: &mut ChannelsSchema,
        audio_sources_cache: &HashMap<String, Arc<[u8]>>,
        settings: &Settings,
        ctx: &egui::Context,
    ) -> bool {
        if self.sink.is_none()
            && get_current_time_ms().saturating_sub(self.last_retry_ms) >= RETRY_INTERVAL_MS
        {
            self.try_init(Some(settings));
        }

        let bgm_volume = self.get_bgm_volume(settings);
        let sfx_volume = self.get_sfx_volume(settings);

        let Some(sink) = &mut self.sink else {
            // Audio hardware is currently unavailable.
            // Clear unlooped audio so stale one-off SFX don't play unexpectedly once audio reconnects.
            channel.clear_unlooped_audio();

            // If audio was requested, schedule a repaint after the retry interval so we try again automatically.
            if channel.bgm.is_some() || channel.sfx.is_some() {
                ctx.request_repaint_after(Duration::from_millis(RETRY_INTERVAL_MS));
            }
            return false;
        };

        let bgm_repaint_needed = Self::handle_channel(
            &channel.bgm,
            &sink.bgm_player,
            &mut self.playing_bgm,
            audio_sources_cache,
            bgm_volume,
        );
        let sfx_repaint_needed = Self::handle_channel(
            &channel.sfx,
            &sink.sfx_player,
            &mut self.playing_sfx,
            audio_sources_cache,
            sfx_volume,
        );

        // Clear unlooped audio
        channel.clear_unlooped_audio();

        let repaint_needed = bgm_repaint_needed || sfx_repaint_needed;
        if repaint_needed {
            ctx.request_repaint();
        }
        repaint_needed
    }

    /// Pauses while the window is unfocused and resumes when it regains focus.
    pub fn set_paused(&mut self, paused: bool) {
        if self.paused == paused {
            return;
        }
        self.paused = paused;
        log::info!("Audio paused = {paused}");
        if let Some(sink) = &self.sink {
            if paused {
                sink.bgm_player.pause();
                sink.sfx_player.pause();
            } else {
                sink.bgm_player.play();
                sink.sfx_player.play();
            }
        }
    }

    pub fn set_volumes(&mut self, settings: &Settings) {
        let bgm_volume = self.get_bgm_volume(settings);
        let sfx_volume = self.get_sfx_volume(settings);
        if let Some(sink) = &mut self.sink {
            sink.bgm_player.set_volume(bgm_volume as f32 / 100.0);
            sink.sfx_player.set_volume(sfx_volume as f32 / 100.0);
        }
    }

    fn get_bgm_volume(&self, settings: &Settings) -> u8 {
        settings.bgm_volume
    }

    fn get_sfx_volume(&self, settings: &Settings) -> u8 {
        settings.sfx_volume
    }

    /// Handles the audio for a given channel.
    /// Returns if eframe repaint is needed.
    fn handle_channel(
        requested_audio: &Option<AudioSchema>,
        player: &rodio::Player,
        playing_audio: &mut Option<PersistentAudio>,
        audio_sources_cache: &HashMap<String, Arc<[u8]>>,
        volume: u8, // Should be between (0,100)
    ) -> bool {
        let mut request_repaint = false;
        // If audio requested
        if let Some(audio) = requested_audio {
            let playing_audio = playing_audio.get_or_insert_with(|| PersistentAudio {
                id: String::new(), // Don't set yet, set after playing
                state: match audio.state {
                    Some(AudioSchemaState::Looping) => Some(AudioState::Looping),
                    _ => None,
                },
            });
            // If need to fade out (runs ONCE per request)
            if let Some(AudioSchemaState::FadingOut(fade_out_seconds)) = audio.state {
                // If instantly stop
                if fade_out_seconds == 0.0 {
                    player.clear();
                    return request_repaint;
                }
                let fade_out_ms = fade_out_seconds * 1000.0;
                // Assumes playing_audio.state is not set yet
                let now = get_current_time_ms();
                let fade_out_end_ms = now + fade_out_ms as u64;
                playing_audio.state = Some(AudioState::FadingOut(FadeOut {
                    start_volume: player.volume(),
                    seconds: fade_out_seconds,
                    end_ms: fade_out_end_ms,
                }));
                request_repaint = true;
            }
            // And not playing anything or playing something else
            if !playing_audio.matches(audio) {
                // And audio is cached
                if let Some(audio_source) = audio_sources_cache.get(&audio.id) {
                    Self::play_audio(player, audio, audio_source, volume);
                } else {
                    log::warn!("Audio not cached: {}", audio.id);
                    request_repaint = true;
                    return request_repaint;
                }
                playing_audio.id = audio.id.clone();
                playing_audio.state = match audio.state {
                    Some(AudioSchemaState::Looping) => Some(AudioState::Looping),
                    _ => None,
                };
            }
        } else if playing_audio.is_some() {
            if let Some(AudioState::Looping) = playing_audio.as_ref().unwrap().state {
                // For looping audio, clear the player if no audio requested.
                player.clear();
                *playing_audio = None;
            } else if let Some(AudioState::FadingOut(fade_out)) =
                &playing_audio.as_ref().unwrap().state
            {
                if Self::handle_fade_out(player, fade_out) {
                    // Fade out completed
                    *playing_audio = None;
                } else {
                    request_repaint = true;
                }
            }
        }
        request_repaint
    }

    /// Processes the audio source, if cached, and appends to the player.
    /// Plays the audio and stops any other audio that is playing.
    fn play_audio(player: &rodio::Player, audio: &AudioSchema, bytes: &Arc<[u8]>, mut volume: u8) {
        // Reset volume (in case of fade out)
        if volume > 100 {
            log::warn!("Volume greater than 100: {volume}");
            volume = 100;
        }
        player.set_volume(volume as f32 / 100.0);
        player.clear();

        let looped = audio.state == Some(AudioSchemaState::Looping);
        match LoopableSource::new(bytes.clone(), looped) {
            Some(source) => apply_effects(player, source, audio.clone()),
            None => log::error!("Failed to decode audio: {}", audio.id),
        }

        player.play();
    }

    /// Returns if fade out is complete.
    fn handle_fade_out(player: &rodio::Player, fade_out: &FadeOut) -> bool {
        let now = get_current_time_ms();
        let remaining_ms = fade_out.end_ms.saturating_sub(now);
        if remaining_ms == 0 {
            player.clear();
        } else {
            player.set_volume(
                fade_out.start_volume * remaining_ms as f32 / (fade_out.seconds * 1000.0),
            );
        }
        remaining_ms == 0
    }
}

impl Default for AudioPlayback {
    fn default() -> Self {
        let mut playback = Self {
            sink: None,
            playing_bgm: None,
            playing_sfx: None,
            last_retry_ms: 0,
            paused: false,
        };
        playback.try_init(None);
        playback
    }
}

/// Applies effects to the source based on the audio schema.
/// (Except boolean effects like looped.)
fn apply_effects(
    player: &rodio::Player,
    source: impl rodio::Source + Send + 'static,
    audio_schema: AudioSchema,
) {
    let secs = |s: Option<f32>| s.filter(|s| *s > 0.0).map(Duration::from_secs_f32);

    let mut applied: Box<dyn Source + Send> =
        Box::new(source.amplify_normalized(audio_schema.volume_multiplier.unwrap_or(1.0)));
    if let Some(d) = secs(audio_schema.fade_in_seconds) {
        applied = Box::new(applied.fade_in(d));
    }
    if let Some(d) = secs(audio_schema.delay_seconds) {
        applied = Box::new(applied.delay(d));
    }
    player.append(applied);
}

pub struct LoopableSource {
    compressed_bytes: Arc<[u8]>,
    current_decoder: Decoder<Cursor<Arc<[u8]>>>,
    looped: bool,
}

impl LoopableSource {
    /// Returns None if the bytes can't be decoded.
    pub fn new(bytes: Arc<[u8]>, looped: bool) -> Option<Self> {
        let current_decoder = Decoder::new(Cursor::new(bytes.clone())).ok()?;
        Some(Self {
            compressed_bytes: bytes,
            current_decoder,
            looped,
        })
    }
}

// Implement Iterator so rodio can pull samples
impl Iterator for LoopableSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        self.current_decoder.next().or_else(|| {
            if !self.looped {
                return None;
            }
            // Restart; end the source instead of panicking if re-decoding fails
            self.current_decoder = Decoder::new(Cursor::new(self.compressed_bytes.clone())).ok()?;
            self.current_decoder.next()
        })
    }
}

impl Source for LoopableSource {
    fn current_span_len(&self) -> Option<usize> {
        self.current_decoder.current_span_len()
    }
    fn channels(&self) -> NonZero<u16> {
        self.current_decoder.channels()
    }
    fn sample_rate(&self) -> NonZero<u32> {
        self.current_decoder.sample_rate()
    }
    fn total_duration(&self) -> Option<Duration> {
        if self.looped {
            None // Infinite for looped
        } else {
            self.current_decoder.total_duration()
        }
    }
}
