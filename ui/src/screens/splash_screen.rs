use std::time::Duration;

use eframe::egui;
use engine_core::io::{list_files, saves_dir};
use state::{
    schema::session_schema::TextScrollMode,
    screens::{BaseScreenSchema, SplashType},
    utils::get_current_time_ms,
};

use crate::{
    constants::MAX_TEXT_SPEED,
    ui::{FvnrsApp, StartupSequence},
};

/// Duration of the splash screen in milliseconds
const SPLASH_SCREEN_DURATION_MS: u64 = 0;

pub(crate) fn draw_splash_screen(
    app: &mut FvnrsApp,
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    splash_type: SplashType,
    start_time_ms: u64,
) {
    // Draw the images
    app.render_textures(frame);
    let image_response = ui.add(
        // Make image clickable
        egui::Image::new((app.egui_game_tex_id, ui.max_rect().size())).sense(egui::Sense::click()),
    );

    match app.startup_sequence {
        StartupSequence::Splash => {
            // Show splash screen during first frame
            app.startup_sequence = StartupSequence::Screenshots;
            ui.ctx().request_repaint();
            return;
        }
        StartupSequence::Screenshots => {
            // Start loading screenshots in second frame
            let screenshot_paths = list_files(saves_dir(), "webp").unwrap_or_default();
            for path in screenshot_paths {
                // Load each into texture cache
                let Ok(img) = image::open(&path) else {
                    log::error!("Failed to load screenshot {path}");
                    continue;
                };
                let img = img.to_rgba8();
                let size = [img.width() as usize, img.height() as usize];
                let color = egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw());
                let handle = ui
                    .ctx()
                    .load_texture(&path, color, egui::TextureOptions::LINEAR);
                app.screenshot_tex_ids.insert(path, handle);
            }
            app.startup_sequence = StartupSequence::Done;
        }
        StartupSequence::Done => {
            // Do nothing
        }
    }

    let time_elapsed_ms = get_current_time_ms() - start_time_ms;
    if image_response.clicked() || time_elapsed_ms >= SPLASH_SCREEN_DURATION_MS {
        ui.ctx().request_repaint();
        log::info!("Dismissing splash screen...");
        let mut core_write = app.engine_core.write();

        let is_game = matches!(&core_write.screens.base_screen, BaseScreenSchema::Game(_));
        if is_game {
            if !core_write.are_next_assets_cached() {
                return;
            }
            core_write.next();
            if let Some(game_state) = core_write.screens.get_game_state() {
                if game_state.textbox.start_scroll {
                    if core_write.persistent.settings.text_speed == MAX_TEXT_SPEED {
                        core_write.session.text_scroll_mode =
                            TextScrollMode::ScrollEnd(get_current_time_ms());
                    } else {
                        core_write.session.start_text_scroll();
                    }
                }
            }
        } else if !core_write.are_current_assets_cached() {
            return;
        }
        core_write.screens.splash_screen = None;

        if splash_type == SplashType::Startup {
            // Purge startup splash from cache
            if core_write
                .texture_cache
                .remove(&app.startup_splash_path)
                .is_none()
            {
                log::warn!("Failed to remove startup splash from cache!");
            }
        }

        // Render the next screen
        drop(core_write);
        app.render_textures(frame);
    } else {
        // Request repaint after delay (elapsed MUST > SPLASH...)
        let delay = std::cmp::max(SPLASH_SCREEN_DURATION_MS - time_elapsed_ms, 20);
        ui.ctx().request_repaint_after(Duration::from_millis(delay));
    }
}
