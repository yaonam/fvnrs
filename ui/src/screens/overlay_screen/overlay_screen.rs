use eframe::egui::{self, Ui};
use state::screens::{BaseScreenSchema, OverlayScreenType};

use crate::{
    constants::COLUMN_SPACER,
    render::buttons::menu_button_click,
    screens::{
        overlay_screen::{
            about_view::draw_about_view, saves_view::draw_saves_view,
            settings_view::draw_settings_view,
        },
        shared::{controls_rect, draw_menu_controls, padded_ui},
    },
    ui::FvnrsApp,
};

// TODO: Optimize locks
pub(crate) fn draw_overlay_screen(
    app: &mut FvnrsApp,
    scaled_ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
) {
    // Apply padding to UI
    let mut padded_ui = padded_ui(scaled_ui);

    // Right click closes menu
    if padded_ui.input(|i| i.pointer.secondary_clicked() || i.key_pressed(egui::Key::Escape)) {
        app.engine_core.write().screens.close_menu();
    }

    let clip_rect = padded_ui.clip_rect();
    let controls_rect = controls_rect(&padded_ui);

    padded_ui.scope_builder(egui::UiBuilder::new().max_rect(clip_rect), |ui| {
        ui.horizontal_top(|ui| {
            ui.set_height(clip_rect.height());

            // Draw controls on left
            draw_controls(app, ui, controls_rect);

            ui.add_space(COLUMN_SPACER);

            // Draw view on right
            draw_overlay_view(app, ui, frame);
        });
    });
}

fn draw_controls(app: &mut FvnrsApp, ui: &mut Ui, viewport: egui::Rect) {
    draw_menu_controls(app, ui, viewport, |app, ui| {
        if menu_button_click(ui, "Return") {
            let mut core_write = app.engine_core.write();
            core_write.screens.close_menu();
        }
        if matches!(
            &app.engine_core.read().screens.base_screen,
            BaseScreenSchema::Game(_)
        ) && menu_button_click(ui, "Main Menu")
        {
            let mut core_write = app.engine_core.write();
            core_write.screens.close_menu();
            core_write.return_to_main_menu();
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

fn draw_overlay_view(app: &mut FvnrsApp, ui: &mut Ui, frame: &mut eframe::Frame) {
    let overlay_screen = app.engine_core.read().screens.overlay_screen;
    if overlay_screen.is_open {
        match overlay_screen.screen_type {
            OverlayScreenType::Saves => draw_saves_view(app, ui, frame),
            OverlayScreenType::Settings => draw_settings_view(app, ui),
            OverlayScreenType::About => draw_about_view(ui),
        }
    }
}
