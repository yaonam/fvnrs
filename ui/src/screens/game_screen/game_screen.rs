use eframe::egui;
use state::schema::session_schema::UserInputOverride;

use crate::{
    animation_player::process_animations,
    inputs::{handle_game_screen_interaction, handle_next, handle_user_overrides},
    screens::game_screen::{dialogue_hud::draw_dialogue_hud, game_widgets::draw_game_widgets},
    ui::FvnrsApp,
};

pub(crate) fn draw_game_screen(app: &mut FvnrsApp, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    let image_response = ui.add(
        // Make image clickable
        egui::Image::new((app.egui_game_tex_id, ui.max_rect().size())).sense(egui::Sense::click()),
    );

    // Allow interaction if overlay is not open
    let overlay_open = app.engine_core.read().screens.overlay_screen.is_open;
    let mut render = false;
    if !overlay_open {
        render |= handle_game_screen_interaction(app, ui, image_response);
    }

    // If no overlay and not hidden
    if !overlay_open
        && app.engine_core.read().session.user_input_override != Some(UserInputOverride::Hide)
    {
        render |= draw_dialogue_hud(app, ui);

        // Draw widgets if any
        render |= draw_game_widgets(app, ui);

        // Handle Auto/Skip
        render |= handle_user_overrides(app, ui.ctx());
    }

    // Process animations
    let mut core_write = app.engine_core.write();
    let (playable, animation_indices, animation_timestamps_ms) = core_write.get_animation_parts();
    let (render_anim, should_advance_game_state) =
        process_animations(ui, playable, animation_indices, animation_timestamps_ms);
    render |= render_anim;
    drop(core_write);

    // Request repaint for next
    if should_advance_game_state && !overlay_open {
        render |= handle_next(app);
        ui.ctx().request_repaint();
    }

    // Render if animations or inputs changed images
    if render {
        app.render_textures(frame);
    }

    // Handle audio
    let mut core_write = app.engine_core.write();
    let (channel, audio_sources_cache, settings) = core_write.get_audio_parts();
    app.audio
        .handle_audio(channel, audio_sources_cache, settings, ui.ctx());
}
