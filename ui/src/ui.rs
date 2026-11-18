use eframe::{
    egui::{self, Color32},
    wgpu,
};
use parking_lot::RwLock;
use state::screens::BaseScreenSchema;
use std::{
    collections::HashMap,
    sync::{Arc, atomic::AtomicBool},
};

use crate::{
    audio_playback::AudioPlayback,
    cache_worker::CacheWorker,
    constants::{
        HOVERED_BG_FILL, HOVERED_WEAK_BG_FILL, INACTIVE_BG_FILL, INACTIVE_FG_STROKE,
        INACTIVE_WEAK_BG_FILL, NONINTERACTIVE_FG_STROKE,
    },
    inputs::handle_global_inputs,
    persist_worker::PersistWorker,
    render::{
        blur::draw_blurred_image,
        shaders::{
            BasicPipelineAndLayouts, BufferBindGroup, KawaseResources,
            create_pipelines_and_layouts, create_post_process_resources, create_sampler,
            create_transform_buffer_and_bind_group, create_vertex_buffer,
        },
        textures::load_texture_from_path,
    },
    screens::{
        game_screen::game_screen::draw_game_screen,
        main_menu_screen::draw_main_menu_screen,
        overlay_screen::{overlay_screen::draw_overlay_screen, saves_view::LoadedSaves},
        splash_screen::draw_splash_screen,
    },
};
use engine_core::EngineCore;

pub(crate) enum StartupSequence {
    Splash,      // Show splash screen
    Screenshots, // Load screenshots
    Done,        // Done loading
}

pub struct FvnrsApp {
    pub(crate) startup_splash_path: String,
    pub(crate) loading_splash_path: String,
    pub(crate) start_scene: usize,

    pub engine_core: Arc<RwLock<EngineCore>>,
    pub needs_reload: Arc<AtomicBool>,

    // For WGPU
    pub(crate) pipelines_and_layouts: BasicPipelineAndLayouts,
    pub(crate) transform_bind_group: BufferBindGroup,
    pub(crate) vertex_buffer: wgpu::Buffer,

    /// The offscreen texture where the game renders
    pub(crate) game_target_texture: wgpu::Texture,
    pub(crate) game_target_view: wgpu::TextureView,
    /// Samples game_target (e.g. kawase blur source)
    pub(crate) game_target_bind_group: wgpu::BindGroup,
    /// The ID egui uses to draw the game_target_view
    pub(crate) egui_game_tex_id: egui::TextureId,
    /// The ID egui uses to draw the kawase blurred target
    pub(crate) egui_blurred_tex_id: egui::TextureId,

    pub(crate) post_process_resources: KawaseResources,

    pub(crate) audio: AudioPlayback,

    pub(crate) loaded_saves: LoadedSaves,
    /// mapping of screenshot path (with webp extension) to egui texture handle
    pub(crate) screenshot_tex_ids: HashMap<String, egui::TextureHandle>,
    pub(crate) startup_sequence: StartupSequence,
}

pub(crate) const TARGET_WIDTH: f32 = 1920.0;
pub(crate) const TARGET_HEIGHT: f32 = 1080.0;
pub(crate) const CANVAS_SIZE: egui::Vec2 = egui::Vec2::new(TARGET_WIDTH, TARGET_HEIGHT);
const TARGET_RATIO: f32 = TARGET_WIDTH / TARGET_HEIGHT;
const STATE_CACHE_SIZE: usize = 7;

impl eframe::App for FvnrsApp {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.handle_hot_reload();

        let ctx = ui.ctx().clone();

        // Show debug window
        #[cfg(debug_assertions)]
        {
            egui::Window::new("Debug")
                .default_open(false) // Start minimized
                .show(&ctx, |ui| {
                    ctx.settings_ui(ui);
                    ui.separator();
                    ctx.inspection_ui(ui);
                });
        }

        handle_global_inputs(self, &ctx);

        // Handle fullscreen setting
        let should_fullscreen = self.engine_core.read().persistent.settings.fullscreen;
        if ctx.input(|i| i.viewport().fullscreen.unwrap_or(false)) != should_fullscreen {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(should_fullscreen));
        }

        // Create a central panel that covers the whole screen
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(Color32::BLACK)) // Remove padding
            .show_inside(ui, |ui| {
                let window_size = ui.available_size();
                let window_aspect = window_size.x / window_size.y;

                // Calculate the largest fitting rectangle
                let mut fit_size = window_size;
                if window_aspect > TARGET_RATIO {
                    fit_size.x = window_size.y * TARGET_RATIO;
                } else {
                    fit_size.y = window_size.x / TARGET_RATIO;
                }

                // Center the rectangle in the available space
                let offset = (window_size - fit_size) / 2.0;
                let fit_rect = egui::Rect::from_min_size(ui.min_rect().min + offset, fit_size);

                // Always draw at 1080p
                let canvas = egui::Rect::from_min_size(fit_rect.min, CANVAS_SIZE);
                ui.scope_builder(egui::UiBuilder::new().max_rect(canvas), |scaled_ui| {
                    // Zoom factor, so we draw everything at 1080p.
                    // set_zoom_factor applies on the next pass, so discard this one
                    // instead of presenting a frame at the old zoom (the startup splash).
                    let target = scaled_ui.zoom_factor() * fit_size.x / TARGET_WIDTH;
                    if (target / scaled_ui.zoom_factor() - 1.0).abs() > 0.001 {
                        // Avoid rounding errors
                        scaled_ui.set_zoom_factor(target);
                        scaled_ui.request_discard("zoom to fit");
                    }
                    if scaled_ui.will_discard() {
                        return;
                    }
                    // Background color
                    scaled_ui.painter().rect_filled(
                        scaled_ui.max_rect(),
                        0.0,
                        egui::Color32::from_rgb(20, 20, 20),
                    );

                    // Clip to the canvas, and to the window when 1080p is still larger than it.
                    let clip = canvas.intersect(scaled_ui.clip_rect());
                    scaled_ui.set_clip_rect(clip);

                    let splash_screen = self.engine_core.read().screens.splash_screen;
                    match splash_screen {
                        Some((splash_type, start_time_ms)) => {
                            draw_splash_screen(self, scaled_ui, frame, splash_type, start_time_ms);
                        }
                        None => {
                            let overlay_open =
                                self.engine_core.read().screens.overlay_screen.is_open;

                            if matches!(
                                self.engine_core.read().screens.base_screen,
                                BaseScreenSchema::Game(_)
                            ) {
                                draw_game_screen(self, scaled_ui, frame);
                            } else {
                                draw_main_menu_screen(self, scaled_ui, frame);
                            }

                            draw_blurred_image(self, scaled_ui);

                            if overlay_open {
                                draw_overlay_screen(self, scaled_ui, frame);
                            }
                        }
                    }
                });
            });
    }
}

impl FvnrsApp {
    pub fn new(
        cc: &eframe::CreationContext,
        mut engine_core: EngineCore,
        start_scene: usize,
        needs_reload: Arc<AtomicBool>,
        startup_splash_path: &str,
        loading_splash_path: &str,
        cache_worker_rx: std::sync::mpsc::Receiver<()>,
        persist_worker_rx: std::sync::mpsc::Receiver<()>,
    ) -> Self {
        // Set global style
        cc.egui_ctx.all_styles_mut(|s| {
            // Colors
            s.visuals.widgets.inactive.bg_fill = INACTIVE_BG_FILL;
            s.visuals.widgets.inactive.weak_bg_fill = INACTIVE_WEAK_BG_FILL;
            s.visuals.widgets.inactive.fg_stroke = INACTIVE_FG_STROKE;
            s.visuals.widgets.noninteractive.fg_stroke = NONINTERACTIVE_FG_STROKE;
            s.visuals.widgets.hovered.bg_fill = HOVERED_BG_FILL;
            s.visuals.widgets.hovered.weak_bg_fill = HOVERED_WEAK_BG_FILL;

            // Labels
            s.interaction.selectable_labels = false;
        });

        cc.egui_ctx.options_mut(|o| {
            // Increase click sensitivity for touch
            o.input_options.max_click_dist = 20.0;
        });

        let wgpu_state = cc.wgpu_render_state.as_ref().expect("WGPU must be enabled");
        let device = &wgpu_state.device;
        let format = wgpu_state.target_format;

        let pipelines_and_layouts = create_pipelines_and_layouts(device, format);

        // Create bind group for image transforms
        let transform_bind_group = create_transform_buffer_and_bind_group(
            device,
            &pipelines_and_layouts.transform_bind_group_layout,
        );

        // Create default sampler
        let default_sampler = create_sampler(device);

        // Create vertex buffer
        let vertex_buffer = create_vertex_buffer(device);

        // Create Offscreen Target & Register with egui
        let game_target = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Game Target"),
            size: wgpu::Extent3d {
                width: TARGET_WIDTH as u32,
                height: (TARGET_WIDTH / TARGET_RATIO) as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1, // Num of lower-res versions
            sample_count: 1,    // MSAA samples
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC, // Render target, shader sample for egui, and screenshot source
            view_formats: &[], // Use texture's own format for now
        });
        let game_target_view = game_target.create_view(&wgpu::TextureViewDescriptor {
            label: Some("Game Target View"),
            ..Default::default()
        });

        // Create bind group for game target (for kawase blur source)
        let game_target_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Game Target Bind Group"),
            layout: &pipelines_and_layouts.texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&game_target_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&default_sampler),
                },
            ],
        });

        // Create post process resources (2 kawase levels: half + quarter)
        let post_process_resources = create_post_process_resources(
            device,
            format,
            &pipelines_and_layouts.texture_bind_group_layout,
            &default_sampler,
            TARGET_WIDTH as u32,
            TARGET_HEIGHT as u32,
            3,
        );

        let egui_game_tex_id = wgpu_state.renderer.write().register_native_texture(
            device,
            &game_target_view,
            wgpu::FilterMode::Linear,
        );
        let egui_blurred_tex_id = wgpu_state.renderer.write().register_native_texture(
            device,
            &post_process_resources.blurred_target.view,
            wgpu::FilterMode::Linear,
        );

        // Load splash
        let splash_handle = load_texture_from_path(
            device,
            &wgpu_state.queue,
            &pipelines_and_layouts.texture_bind_group_layout,
            startup_splash_path,
            &default_sampler,
        )
        .unwrap();
        engine_core
            .texture_cache
            .insert(startup_splash_path.to_string(), splash_handle);

        // Create background worker for preloading states and assets
        let shared_engine_core = Arc::new(RwLock::new(engine_core));
        CacheWorker::new(
            shared_engine_core.clone(),
            startup_splash_path.to_string(),
            loading_splash_path.to_string(),
            device.clone(),
            wgpu_state.queue.clone(),
            pipelines_and_layouts.texture_bind_group_layout.clone(),
            default_sampler.clone(),
            STATE_CACHE_SIZE,
            cache_worker_rx,
        )
        .start();

        // Create persist worker
        PersistWorker::new(shared_engine_core.clone(), persist_worker_rx).start();

        // // Set debug flags
        // #[cfg(debug_assertions)]
        // cc.egui_ctx.global_style_mut(|style| {
        //     style.debug.debug_on_hover = true;
        // });

        Self {
            startup_splash_path: startup_splash_path.to_string(),
            loading_splash_path: loading_splash_path.to_string(),
            start_scene,
            engine_core: shared_engine_core,
            needs_reload,
            pipelines_and_layouts,
            transform_bind_group,
            vertex_buffer,
            game_target_texture: game_target,
            game_target_view,
            game_target_bind_group,
            egui_game_tex_id,
            egui_blurred_tex_id,
            post_process_resources,
            audio: AudioPlayback::default(),
            loaded_saves: LoadedSaves::default(),
            screenshot_tex_ids: HashMap::new(),
            startup_sequence: StartupSequence::Splash,
        }
    }

    fn handle_hot_reload(&mut self) {
        #[cfg(feature = "hot_reload")]
        if self
            .needs_reload
            .swap(false, std::sync::atomic::Ordering::SeqCst)
        {
            log::info!("Reloading scenes on main thread...");
            let mut core_write = self.engine_core.write();
            core_write.scene_resolver.reload();
            // Now replay the previous action
            core_write.back();
            core_write.refresh_cache();
            core_write.next();
        }
    }
}
