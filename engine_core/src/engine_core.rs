use anyhow::Result;
use chrono::Local;
use eframe::wgpu;
use glam::Vec2;
use log::info;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};

use state::{
    game_state::{GameState, GameStateAction, GameStateInputMode, SceneResolver},
    persistent::Persistent,
    schema::{
        animation_schema::{AnimationMode, AnimationSchema, AnimationStep},
        audio_schema::{AudioSchema, ChannelsSchema},
        image_schema::ImageSchema,
        playable_schema::PlayableSchema,
        save_schema::SaveSchema,
        schema::SceneSchema,
        session_schema::SessionSchema,
    },
    screens::{BaseScreenSchema, ScreensSchema, SplashType},
    settings::Settings,
    utils::get_current_time_ms,
};

use crate::io::{persistent_path, read_file, saves_dir, write_file};

const MAX_STATE_HISTORY_SIZE: usize = 100;

// Shared with cache worker.
// Think of like the backend?
pub struct EngineCore {
    pub screens: ScreensSchema,

    /// The next state(s)
    game_state_cache: VecDeque<GameState>,
    history: VecDeque<GameState>,

    pub scene_resolver: Box<dyn SceneResolver>,

    /// Cache of loaded images (path -> (image, size))
    pub texture_cache: HashMap<String, (wgpu::BindGroup, Vec2)>,
    /// Cache of loaded audio (id -> compressed bytes)
    pub audio_sources_cache: HashMap<String, Arc<[u8]>>,

    /// For animations (animation_id -> step index)
    pub animation_indices: HashMap<String, usize>,
    /// For animations (animation_id -> timestamp_ms)
    pub animation_timestamps_ms: HashMap<String, u64>,

    pub main_menu: PlayableSchema,
    pub session: SessionSchema,
    pub persistent: Persistent,

    pub cache_worker_tx: std::sync::mpsc::Sender<()>,
    pub persist_worker_tx: std::sync::mpsc::SyncSender<()>,
}

impl EngineCore {
    pub fn new(
        scene_resolver: Box<dyn SceneResolver>,
        main_menu: PlayableSchema,
        cache_worker_tx: std::sync::mpsc::Sender<()>,
        persist_worker_tx: std::sync::mpsc::SyncSender<()>,
    ) -> Self {
        // Load persistent data
        let persistent = read_file(persistent_path()).unwrap_or_default();

        Self {
            screens: ScreensSchema::new(main_menu.clone()),
            game_state_cache: VecDeque::new(),
            history: VecDeque::new(),
            scene_resolver,
            texture_cache: HashMap::new(),
            audio_sources_cache: HashMap::new(),
            animation_indices: HashMap::new(),
            animation_timestamps_ms: HashMap::new(),
            main_menu,
            session: SessionSchema::default(),
            persistent,
            cache_worker_tx,
            persist_worker_tx,
        }
    }

    pub fn back(&mut self) {
        let Some(current_state) = self.screens.get_game_state_mut() else {
            return;
        };
        if let Some(state) = self.history.pop_back() {
            // Move (instead of clone)
            let current = std::mem::replace(current_state, state);
            let (remove_keys, reset_keys) = AnimationSchema::get_update_keys(
                &current.playable.animations,
                &current_state.playable.animations,
            );
            Self::update_animation_indices_and_timestamps(
                &mut self.animation_indices,
                &mut self.animation_timestamps_ms,
                remove_keys,
                reset_keys,
            );
            self.game_state_cache.push_front(current);

            self.cache_worker_tx.send(()).unwrap();
        }
    }

    /// Checks if the next state has already been seen.
    pub fn next_if_already_seen(&mut self) {
        {
            let Some(next_state) = self.game_state_cache.front() else {
                return;
            };
            let Some(next_scene) = &next_state.scene else {
                return;
            };
            if next_scene.is_later_than(&self.session.max_scene_schema) {
                return;
            }
        }
        self.next();
    }

    /// Checks if the next state has already been seen.
    pub fn next_if_already_seen_global(&mut self) {
        {
            let Some(next_state) = self.game_state_cache.front() else {
                return;
            };
            let Some(next_scene) = &next_state.scene else {
                return;
            };
            if next_scene.is_later_than(&self.persistent.global_max_scene) {
                return;
            }
        }
        self.next();
    }

    /// Pulls the next state from the cache and sets it as the current state.
    pub fn next(&mut self) {
        // Do nothing if assets not yet cached
        if !self.are_next_assets_cached() {
            return;
        }
        if self.screens.is_blocking(&self.animation_indices) {
            return;
        }

        // Check current state first, before we pop from cache
        let Some(current_state) = self.screens.get_game_state_mut() else {
            return;
        };
        let Some(next_state) = self.game_state_cache.pop_front() else {
            return;
        };
        if next_state.scene.is_none() {
            self.return_to_main_menu();
            return;
        }
        let old = std::mem::replace(current_state, next_state);

        let (remove_keys, reset_keys) = AnimationSchema::get_update_keys(
            &old.playable.animations,
            &current_state.playable.animations,
        );
        Self::update_animation_indices_and_timestamps(
            &mut self.animation_indices,
            &mut self.animation_timestamps_ms,
            remove_keys,
            reset_keys,
        );

        // Ignore default state
        if old.scene.is_some() {
            self.history.push_back(old); // Save snapshot
            if self.history.len() > MAX_STATE_HISTORY_SIZE {
                self.history.pop_front();
            }
        }

        // Update session
        if let Some(scene) = current_state.scene.as_ref() {
            self.session.max_scene_schema.update_if_bigger(scene);
            if self.persistent.global_max_scene.update_if_bigger(scene) {
                let _ = self.persist_worker_tx.try_send(());
            }
        }

        self.cache_worker_tx.send(()).unwrap();
        info!("Next state: {:#?}", current_state);
    }

    pub fn update_animation_indices_and_timestamps(
        animation_indices: &mut HashMap<String, usize>,
        animation_timestamps_ms: &mut HashMap<String, u64>,
        remove_keys: Vec<String>,
        reset_keys: Vec<String>,
    ) {
        animation_indices.retain(|key, _| !remove_keys.contains(key));
        animation_timestamps_ms.retain(|key, _| !remove_keys.contains(key));
        for key in reset_keys {
            animation_indices.insert(key.clone(), 0);
            animation_timestamps_ms.insert(key, get_current_time_ms());
        }
    }

    /// Processes as many steps as possible for this tick.
    /// Returns if render needed, the repaint after ms, and the new animation timestamp ms.
    pub fn process_animation_step(
        animation_id: &str,
        animation: &mut AnimationSchema,
        images: &mut HashMap<String, Vec<ImageSchema>>,
        animation_index: &mut usize,
        animation_timestamp_ms: Option<u64>,
        now_ms: u64,
    ) -> (bool, Option<u64>, Option<u64>) {
        let mut needs_render = false;
        let mut repaint_after_ms = None;
        let mut new_animation_timestamp_ms = None;
        // Only the step that was already running owns the stored timestamp.
        let mut current_timestamp = animation_timestamp_ms;

        let step_count = animation.steps.len();

        for i in 0..step_count {
            if *animation_index >= step_count {
                if animation.mode == AnimationMode::Repeat && step_count > 0 {
                    *animation_index = 0;
                } else {
                    break;
                }
            }

            match &animation.steps[*animation_index] {
                AnimationStep::Show(step_images) => {
                    info!("Showing images: {:?}", step_images);
                    images.insert(animation_id.to_string(), step_images.clone());
                    *animation_index += 1;
                    needs_render = true;
                    current_timestamp = None;
                    new_animation_timestamp_ms = None;
                }
                AnimationStep::Tween(tween) => {
                    let timestamp_ms = current_timestamp.unwrap_or(now_ms);
                    let time_elapsed = now_ms - timestamp_ms;
                    let t = tween
                        .easing
                        .ease(time_elapsed as f32 / (tween.duration * 1000.0));
                    let transform = tween.start_transform.tween(&tween.end_transform, t);
                    let transformed_images = tween
                        .base_images
                        .iter()
                        .map(|image| {
                            image
                                .clone()
                                .with_transform(image.transform.merge(&transform))
                        })
                        .collect();
                    images.insert(animation_id.to_string(), transformed_images);
                    needs_render = true;

                    // Use greater here, so we render the last frame
                    if time_elapsed > (tween.duration * 1000.0) as u64 {
                        info!("Tween time's up");
                        current_timestamp = None;
                        new_animation_timestamp_ms = None;
                        *animation_index += 1;
                    } else {
                        new_animation_timestamp_ms = Some(timestamp_ms);
                        repaint_after_ms = Some(0);
                        break;
                    }
                }
                AnimationStep::Wait(wait) => {
                    let timestamp_ms = current_timestamp.unwrap_or(now_ms);
                    let wait_ms = (wait * 1000.0) as u64;
                    let time_elapsed = now_ms - timestamp_ms;
                    // Use gte here, so we move forward on the last frame
                    if time_elapsed >= wait_ms {
                        info!("Wait time's up");
                        current_timestamp = None;
                        new_animation_timestamp_ms = None;
                        *animation_index += 1;
                    } else {
                        new_animation_timestamp_ms = Some(timestamp_ms);
                        repaint_after_ms = Some(wait_ms - time_elapsed);
                        break;
                    }
                }
                AnimationStep::Hide => {
                    info!("Hiding images");
                    images.remove(animation_id);
                    *animation_index += 1;
                    needs_render = true;
                    current_timestamp = None;
                    new_animation_timestamp_ms = None;
                }
            }

            if i == step_count - 1 && animation.mode == AnimationMode::Repeat {
                repaint_after_ms = Some(0);
            }
        }

        (needs_render, repaint_after_ms, new_animation_timestamp_ms)
    }

    /// Removes finished animations and persists any remaining images.
    pub fn handle_finished_animations(playable: &mut PlayableSchema) {
        let finished_animations = playable
            .animations
            .extract_if(|_, animation| {
                matches!(
                    animation.mode,
                    AnimationMode::Blocking | AnimationMode::OneTime
                )
            })
            .collect::<HashMap<String, AnimationSchema>>();

        for (animation_id, mut animation) in finished_animations {
            // Play each to completion
            let mut animation_index = 0;
            let animation_timestamp_ms = Some(0);
            loop {
                let (_, repaint_after_ms, _) = Self::process_animation_step(
                    &animation_id,
                    &mut animation,
                    &mut playable.images,
                    &mut animation_index,
                    animation_timestamp_ms,
                    u64::MAX,
                );

                if repaint_after_ms.is_none() {
                    break; // Animation is done
                }
            }
        }
    }

    /// Processes and caches the next state.
    /// Returns if at least one state was cached.
    pub fn cache_next(&mut self) -> bool {
        // Grab the latest scene from cache or current
        let Some(mut latest_state) = (self
            .game_state_cache
            .back()
            .or_else(|| self.screens.get_game_state()))
        .cloned() else {
            return false;
        };

        // Handle one-off animations
        Self::handle_finished_animations(&mut latest_state.playable);

        if self.process_state(&mut latest_state) {
            info!("Cached state: {:#?}", latest_state);
            self.game_state_cache.push_back(latest_state);
            return true;
        }

        false
    }

    /// Clears and repopulates the cache
    pub fn refresh_cache(&mut self) {
        // Clear the cache
        self.game_state_cache.clear();

        // Manually calculate one (for hot reload)
        self.cache_next();

        // Trigger cache worker
        self.cache_worker_tx.send(()).unwrap();
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Clears history and the forward state cache
    pub fn clear_game_progression(&mut self) {
        self.history.clear();
        self.game_state_cache.clear();
        self.animation_indices.clear();
        self.animation_timestamps_ms.clear();
        self.cache_worker_tx.send(()).unwrap();
    }

    pub fn return_to_main_menu(&mut self) {
        let main_menu = self.main_menu.clone();
        self.clear_game_progression();
        self.screens.set_splash(SplashType::Loading);
        self.screens.set_main_menu(main_menu);
    }

    /// Returns the file name and save schema.
    fn get_save_schema(&self) -> Result<(String, SaveSchema)> {
        let now = Local::now();
        let elapsed_ms = now.timestamp_millis() as u64 - self.session.session_start_ms;
        Ok((
            now.format("%Y-%m-%d_%H.%M.%S").to_string(), // 2026-10-11_10.54.54.json
            SaveSchema {
                name: now.format("%b %-d, %-I:%M %p").to_string(), // Oct 11, 10:54 AM
                timestamp_ms: now.timestamp_millis() as u64,
                total_playtime_sec: self.session.previous_playtime_sec + elapsed_ms / 1000,
                max_scene_schema: self.session.max_scene_schema.clone(),
                game_state: self
                    .screens
                    .get_game_state()
                    .cloned()
                    .ok_or(anyhow::anyhow!("No game state"))?,
            },
        ))
    }

    /// Returns the file name (minus extension) and save schema.
    pub fn save(&self) -> Option<(String, SaveSchema)> {
        let Ok((name, save_schema)) = self.get_save_schema() else {
            log::error!("Failed to get save schema");
            return None;
        };
        if let Err(e) = write_file(
            saves_dir().join(format!("{name}.json")),
            save_schema.clone(),
        ) {
            log::error!("Failed to save: {}", e);
            return None;
        }
        Some((name, save_schema))
    }

    /// Loads the save file and sets the screen to game state.
    pub fn load_save(&mut self, save_schema: &SaveSchema) {
        self.session.previous_playtime_sec = save_schema.total_playtime_sec;
        self.session.max_scene_schema = save_schema.max_scene_schema.clone();
        self.session.start_session();
        self.clear_game_progression();
        self.game_state_cache
            .push_back(save_schema.game_state.clone());
        self.screens.set_game_state(GameState::default());
        self.screens.set_splash(state::screens::SplashType::Loading);
        self.cache_worker_tx.send(()).unwrap();
    }

    /// Loads the scene into the cache.
    pub fn load_scene(&mut self, scene: usize) {
        info!("Loading scene: {}", scene);
        let mut new_state = GameState::default();
        new_state.scene = Some(SceneSchema::new(scene));
        self.process_state(&mut new_state);
        self.game_state_cache.push_back(new_state);
        self.cache_worker_tx.send(()).unwrap();
    }

    /// Manually checks to see if all the assets for the given playable schema are cached.
    /// (images, animations, audio)
    fn are_playable_assets_cached(&self, playable: &PlayableSchema) -> bool {
        // Check all images, including animation steps
        let texture_cached = |i: &ImageSchema| self.texture_cache.contains_key(&i.path);
        let all_images_cached = playable
            .iter_images()
            .all(|images| images.iter().all(texture_cached))
            && playable.iter_animations().all(|(_, a)| {
                a.steps.iter().all(|s| match s {
                    AnimationStep::Show(images) => images.iter().all(texture_cached),
                    AnimationStep::Tween(tween) => tween.base_images.iter().all(texture_cached),
                    AnimationStep::Wait(_) => true,
                    AnimationStep::Hide => true,
                })
            });

        // Check audio
        let audio = &playable.audio;
        let audio_cached = |a: &AudioSchema| self.audio_sources_cache.contains_key(&a.id);
        let all_audio_cached = audio.bgm.as_ref().map_or(true, audio_cached)
            && audio.sfx.as_ref().map_or(true, audio_cached);

        all_images_cached && all_audio_cached
    }

    pub fn are_current_assets_cached(&self) -> bool {
        match &self.screens.base_screen {
            BaseScreenSchema::MainMenu(p) => self.are_playable_assets_cached(p),
            BaseScreenSchema::Game(s) => self.are_playable_assets_cached(&s.playable),
        }
    }

    pub fn are_next_assets_cached(&self) -> bool {
        match self.game_state_cache.front() {
            Some(state) => self.are_playable_assets_cached(&state.playable),
            None => false,
        }
    }

    /// Returns the playable schema, the animation indices, and the animation timestamps.
    pub fn get_animation_parts(
        &mut self,
    ) -> (
        &mut PlayableSchema,
        &mut HashMap<String, usize>,
        &mut HashMap<String, u64>,
    ) {
        let playable = match &mut self.screens.base_screen {
            BaseScreenSchema::Game(game_state) => &mut game_state.playable,
            BaseScreenSchema::MainMenu(playable) => playable,
        };
        (
            playable,
            &mut self.animation_indices,
            &mut self.animation_timestamps_ms,
        )
    }

    pub fn get_audio_parts(
        &mut self,
    ) -> (&mut ChannelsSchema, &HashMap<String, Arc<[u8]>>, &Settings) {
        let audio = match &mut self.screens.base_screen {
            BaseScreenSchema::Game(game_state) => &mut game_state.playable.audio,
            BaseScreenSchema::MainMenu(main_menu) => &mut main_menu.audio,
        };
        (audio, &self.audio_sources_cache, &self.persistent.settings)
    }

    pub fn get_cached_state_count(&self) -> usize {
        self.game_state_cache.len()
    }

    /// Iterates over the current state, and levels states from the history and cache.
    /// Not necessarily in order.
    pub fn iter_state_levels<'a>(
        &'a self,
        levels: usize,
    ) -> impl Iterator<Item = &'a GameState> + 'a {
        let current = self.screens.get_game_state();

        current
            .into_iter()
            .chain(current.into_iter().flat_map(move |_| {
                self.history
                    .iter()
                    .rev()
                    .take(levels)
                    .chain(self.game_state_cache.iter().take(levels))
            }))
    }

    /// This is intended to support partial updates.
    pub fn attr_changed<T, F>(&self, select: F) -> bool
    where
        T: PartialEq,
        F: Fn(&GameState) -> &T,
    {
        let Some(current_state) = self.screens.get_game_state() else {
            log::info!("Checking attr changed?");
            return true;
        };
        match self.history.back() {
            Some(prev) => select(current_state) != select(prev),
            None => true,
        }
    }

    /// Checks against cache front instead of history back.
    pub fn attr_changed_rev<T, F>(&self, select: F) -> bool
    where
        T: PartialEq,
        F: Fn(&GameState) -> &T,
    {
        let Some(current_state) = self.screens.get_game_state() else {
            log::info!("Checking attr changed?");
            return true;
        };
        match self.game_state_cache.front() {
            Some(prev) => select(current_state) != select(prev),
            None => true,
        }
    }

    /// Applies actions on the state until wait requested.
    /// Returns if at least 1 action was processed.
    fn process_state(&mut self, state: &mut GameState) -> bool {
        // Clear one offs
        state.playable.audio.clear_unlooped_audio();
        state.textbox.clear_start_scroll();

        let mut processed_actions = false;

        loop {
            // Grab next action and increment index
            let action = {
                let Some(scene) = state.scene.as_mut() else {
                    break;
                };
                let actions = self.scene_resolver.resolve(scene.current_scene_id);
                let Some(action) = actions.get(scene.index) else {
                    break;
                };
                scene.index += 1;
                processed_actions = true;
                action
            };

            // Handle action
            match action {
                GameStateAction::Say(say) => {
                    state.textbox.apply(say);
                }
                GameStateAction::Function(function) => {
                    function(state);
                }
            }

            // Break or continue
            match state.input_mode {
                GameStateInputMode::Wait | GameStateInputMode::AdvanceBlocked => break,
                GameStateInputMode::Continue => {
                    state.input_mode = GameStateInputMode::Wait;
                }
            }
        }

        processed_actions
    }
}
