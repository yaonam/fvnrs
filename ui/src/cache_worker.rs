use std::{collections::HashSet, sync::Arc};

use eframe::{egui::TextBuffer, wgpu};
use engine_core::EngineCore;
use parking_lot::RwLock;
use state::schema::{
    animation_schema::AnimationStep, audio_schema::ChannelsSchema, playable_schema::PlayableSchema,
};

use crate::{assets::read_asset, render::textures::load_texture_from_path};

pub(crate) struct CacheWorker {
    engine_core: Arc<RwLock<EngineCore>>,
    startup_splash: String,
    loading_splash: String,
    device: wgpu::Device,
    queue: wgpu::Queue,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,

    state_cache_size: usize,

    cache_worker_rx: std::sync::mpsc::Receiver<()>,
}

impl CacheWorker {
    pub(crate) fn new(
        engine_core: Arc<RwLock<EngineCore>>,
        startup_splash: String,
        loading_splash: String,
        device: wgpu::Device,
        queue: wgpu::Queue,
        bind_group_layout: wgpu::BindGroupLayout,
        sampler: wgpu::Sampler,
        state_cache_size: usize,
        cache_worker_rx: std::sync::mpsc::Receiver<()>,
    ) -> Self {
        Self {
            engine_core,
            startup_splash,
            loading_splash,
            device,
            queue,
            bind_group_layout,
            sampler,
            state_cache_size,
            cache_worker_rx,
        }
    }

    /// Runs a worker thread that handles loading and caching assets.
    /// Also computes the next states for the game state cache.
    pub(crate) fn start(self) {
        std::thread::spawn(move || {
            // Run once at startup (for main menu)
            self.handle_cache();

            while let Ok(()) = self.cache_worker_rx.recv() {
                // Drain channel
                while self.cache_worker_rx.try_recv().is_ok() {}
                self.handle_cache();
            }
            log::error!("Cache worker exited!");
        });
    }

    fn handle_cache(&self) {
        // Handle game state cache
        {
            let mut engine_core = self.engine_core.write();
            let need_to_cache_count = self
                .state_cache_size
                .saturating_sub(engine_core.get_cached_state_count());
            for _ in 0..need_to_cache_count {
                engine_core.cache_next();
            }
        }

        // Cache needed assets (textures, audio, etc.)
        self.cache_images();
        self.cache_audio();
    }

    fn extract_images<'a>(image_set: &mut HashSet<&'a str>, playable: &'a PlayableSchema) {
        for images in playable.images.values() {
            for image in images {
                image_set.insert(&image.path);
            }
        }
        for animation in playable.animations.values() {
            for step in &animation.steps {
                match step {
                    AnimationStep::Show(images) => {
                        for image in images {
                            image_set.insert(&image.path);
                        }
                    }
                    AnimationStep::Tween(tween) => {
                        for image in &tween.base_images {
                            image_set.insert(&image.path);
                        }
                    }
                    AnimationStep::Wait(_) => {}
                    AnimationStep::Hide => {}
                }
            }
        }
    }

    fn cache_images(&self) {
        log::info!("Caching images!");
        let stale_paths: Vec<_>;
        let missing_paths: Vec<_>;
        {
            let mut all_images: HashSet<&str> = HashSet::from_iter([self.loading_splash.as_str()]);
            let core_read = self.engine_core.read();

            log::info!("Cached images: {:?}", core_read.texture_cache.keys());

            match &core_read.screens.base_screen {
                state::screens::BaseScreenSchema::MainMenu(playable_schema) => {
                    Self::extract_images(&mut all_images, playable_schema);
                    // TODO: animations
                }
                state::screens::BaseScreenSchema::Game(_) => {
                    for state in core_read.iter_state_levels(self.state_cache_size) {
                        Self::extract_images(&mut all_images, &state.playable);
                    }
                }
            }

            log::info!("All images: {all_images:?}");

            // Remove any images in cache not in states
            stale_paths = core_read
                .texture_cache
                .keys()
                .filter(|path| !all_images.contains(path.as_str()) && **path != self.startup_splash)
                .cloned()
                .collect();

            // Cache any images in states not in cache
            missing_paths = all_images
                .into_iter()
                .filter(|path| !core_read.texture_cache.contains_key(*path))
                .map(String::from)
                .collect();
        }

        log::info!("Missing paths: {missing_paths:?}");
        log::info!("Stale paths: {stale_paths:?}");

        for path in stale_paths {
            self.engine_core.write().texture_cache.remove(&path);
        }

        for path in missing_paths {
            // Load image
            let bind_group_size = load_texture_from_path(
                &self.device,
                &self.queue,
                &self.bind_group_layout,
                &path,
                &self.sampler,
            )
            .unwrap();
            self.engine_core
                .write()
                .texture_cache
                .insert(path, bind_group_size);
        }
    }

    fn extract_audio<'a>(audio_set: &mut HashSet<&'a str>, audio: &'a ChannelsSchema) {
        if let Some(audio) = &audio.bgm {
            audio_set.insert(&audio.id);
        }
        if let Some(audio) = &audio.sfx {
            audio_set.insert(&audio.id);
        }
    }

    fn cache_audio(&self) {
        let stale_ids: Vec<_>;
        let missing_ids: Vec<_>;

        {
            let mut all_audio = HashSet::new();
            let core_read = self.engine_core.read();

            log::info!("Cached audio: {:?}", core_read.audio_sources_cache.keys());

            match &core_read.screens.base_screen {
                state::screens::BaseScreenSchema::MainMenu(playable_schema) => {
                    Self::extract_audio(&mut all_audio, &playable_schema.audio);
                }
                state::screens::BaseScreenSchema::Game(_) => {
                    for state in core_read.iter_state_levels(self.state_cache_size) {
                        Self::extract_audio(&mut all_audio, &state.playable.audio);
                    }
                }
            }

            log::info!("All audio: {all_audio:?}");

            stale_ids = core_read
                .audio_sources_cache
                .keys()
                .filter(|id| !all_audio.contains(id.as_str()))
                .cloned()
                .collect();
            missing_ids = all_audio
                .into_iter()
                .filter(|id| !core_read.audio_sources_cache.contains_key(id.as_str()))
                .map(String::from)
                .collect();
        }

        for id in stale_ids {
            self.engine_core.write().audio_sources_cache.remove(&id);
            log::info!("Removed audio: {}", id);
        }
        for id in missing_ids {
            let bytes = read_asset(&format!("audio/{id}")).unwrap();

            log::info!("Cached audio: {}", id);
            self.engine_core
                .write()
                .audio_sources_cache
                .insert(id.to_owned(), Arc::from(bytes));
        }
    }
}
