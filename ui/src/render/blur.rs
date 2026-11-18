use eframe::egui;
use state::{schema::animation_schema::EasingFn, utils::get_current_time_ms};

use crate::{
    constants::{BLUR_TRANSITION_MS, GREY_COLOR, GREY_FACTOR},
    ui::FvnrsApp,
};

/// Draws the blurred background over the sharp game target, fading in from
/// `overlay_screen.open_close_time`.
pub(crate) fn draw_blurred_image(app: &mut FvnrsApp, ui: &mut egui::Ui) {
    let overlay = app.engine_core.read().screens.overlay_screen;

    let elapsed_ms = get_current_time_ms().saturating_sub(overlay.open_close_time) as f32;
    let t = (elapsed_ms / BLUR_TRANSITION_MS).clamp(0.0, 1.0);

    // Early exit if overlay is closed and not transitioning
    if !overlay.is_open && t >= 1.0 {
        return;
    }

    let eased = EasingFn::QuadraticOut.ease(t);
    let alpha = match overlay.is_open {
        true => (eased * 255.0) as u8,
        false => ((1.0 - eased) * 255.0) as u8,
    };

    // Sharp game target is already drawn underneath; this fades the blur on top.
    egui::Image::new((app.egui_blurred_tex_id, ui.max_rect().size()))
        .tint(egui::Color32::from_rgba_unmultiplied(255, 255, 255, alpha))
        .paint_at(ui, ui.max_rect());

    // Pull towards grey (brighten on dark, darken on light)
    ui.painter().rect_filled(
        ui.max_rect(),
        0.,
        egui::Color32::from_rgba_unmultiplied(
            GREY_COLOR,
            GREY_COLOR,
            GREY_COLOR,
            alpha / GREY_FACTOR,
        ),
    );

    if t < 1.0 {
        ui.ctx().request_repaint();
    }
}
