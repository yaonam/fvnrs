use eframe::egui::{self, Rect, Ui};
use state::{
    game_state::GameState,
    screens::{OverlayScreenType, SplashType},
};

use crate::{
    animation_player::process_animations,
    render::buttons::menu_button_click,
    screens::shared::{controls_rect, draw_menu_controls, padded_ui},
    ui::FvnrsApp,
};

// TODO: Optimize locks
pub(crate) fn draw_main_menu_screen(
    app: &mut FvnrsApp,
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
) {
    // Process animations
    let mut core_write = app.engine_core.write();
    let (playable, animation_indices, animation_timestamps_ms) = core_write.get_animation_parts();
    let (render_anim, _) =
        process_animations(ui, playable, animation_indices, animation_timestamps_ms);
    drop(core_write);

    // Draw the images
    if render_anim {
        log::info!("Rendering Main Menu!");
        app.render_textures(frame);
    }
    let image_response = ui.add(
        // Make image clickable
        egui::Image::new((app.egui_game_tex_id, ui.max_rect().size())).sense(egui::Sense::click()),
    );
    let overlay_open = app.engine_core.read().screens.overlay_screen.is_open;
    if !overlay_open && image_response.secondary_clicked() {
        // Open menu
        app.engine_core.write().screens.open_menu();
    }

    // Draw controls
    if !overlay_open {
        ui.scope_builder(egui::UiBuilder::new().max_rect(ui.clip_rect()), |ui| {
            let mut padded_ui = padded_ui(ui);
            let controls_rect = controls_rect(&padded_ui);
            draw_controls(app, &mut padded_ui, controls_rect);
        });
    }

    // Handle audio
    let mut core_write = app.engine_core.write();
    let (channel, audio_sources_cache, settings) = core_write.get_audio_parts();
    app.audio
        .handle_audio(channel, audio_sources_cache, settings, ui.ctx());
}

fn draw_controls(app: &mut FvnrsApp, ui: &mut Ui, viewport: Rect) {
    draw_menu_controls(app, ui, viewport, |app, ui| {
        if menu_button_click(ui, "Start") {
            let mut core_write = app.engine_core.write();

            core_write.screens.set_splash(SplashType::Loading);
            core_write.screens.set_game_state(GameState::default());
            core_write.load_scene(app.start_scene);
            core_write.session.start_session();
        }
        if menu_button_click(ui, "Saves") {
            let mut core_write = app.engine_core.write();
            core_write
                .screens
                .set_overlay_screen(OverlayScreenType::Saves);
        }
        if menu_button_click(ui, "Settings") {
            let mut core_write = app.engine_core.write();
            core_write
                .screens
                .set_overlay_screen(OverlayScreenType::Settings);
        }
        if menu_button_click(ui, "About") {
            let mut core_write = app.engine_core.write();
            core_write
                .screens
                .set_overlay_screen(OverlayScreenType::About);
        }
    });
}
