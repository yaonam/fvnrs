use eframe::egui::{
    self, FontId, Grid, RichText, ScrollArea, Slider, TextStyle, Ui, style::HandleShape,
};

use crate::{
    constants::{
        MAX_PUNCT_PAUSE, MAX_TEXT_SPEED, MAX_TEXT_WAIT, MAX_VOLUME, MIN_TEXT_WAIT,
        SETTINGS_BUTTON_PADDING, SETTINGS_CORNER_RADIUS, SETTINGS_GRID_SPACING,
        SETTINGS_ICON_WIDTH, SETTINGS_INDENT, SETTINGS_INTERACT_SIZE, SETTINGS_SECTION_TEXT_SIZE,
        SETTINGS_SLIDER_TEXT_COLOR, SETTINGS_SLIDER_TEXT_STYLE, SETTINGS_SLIDER_WIDTH,
        SETTINGS_TEXT_SIZE,
    },
    ui::FvnrsApp,
    widgets::toggle::toggle,
};

fn section_label(ui: &mut Ui, text: &str) {
    ui.label(RichText::new(text).font(FontId::proportional(SETTINGS_SECTION_TEXT_SIZE)));
}

fn setting_label(ui: &mut Ui, text: &str) {
    ui.horizontal(|ui| {
        ui.add_space(SETTINGS_INDENT);
        ui.visuals_mut().override_text_color = None;
        ui.label(RichText::new(text).font(FontId::proportional(SETTINGS_TEXT_SIZE)));
    });
}

pub(crate) fn draw_settings_view(app: &mut FvnrsApp, ui: &mut Ui) {
    let s = ui.style_mut();
    // Slider value text
    s.text_styles
        .insert(TextStyle::Button, SETTINGS_SLIDER_TEXT_STYLE);
    s.spacing.slider_width = SETTINGS_SLIDER_WIDTH;
    s.spacing.interact_size = SETTINGS_INTERACT_SIZE;
    s.spacing.icon_width = SETTINGS_ICON_WIDTH;
    s.spacing.button_padding = SETTINGS_BUTTON_PADDING;
    s.spacing.slider_rail_height = SETTINGS_TEXT_SIZE;

    // Slider text color
    s.visuals.override_text_color = Some(SETTINGS_SLIDER_TEXT_COLOR);
    s.visuals.slider_trailing_fill = true;
    s.visuals.handle_shape = HandleShape::Circle;
    s.visuals.widgets.inactive.corner_radius = SETTINGS_CORNER_RADIUS.into();

    ui.vertical(|ui| {
        ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            Grid::new("settings_menu_grid")
                .spacing(SETTINGS_GRID_SPACING)
                .show(ui, |ui| {
                    let mut write_core = app.engine_core.write();

                    let persistent_snapshot = write_core.persistent;

                    section_label(ui, "Text");
                    ui.end_row();

                    // TODO: Handle speed clipping
                    setting_label(ui, "Text speed (c/s)");
                    ui.add(Slider::new(
                        &mut write_core.persistent.settings.text_speed,
                        1..=MAX_TEXT_SPEED,
                    ));
                    ui.end_row();

                    // Auto wait time
                    setting_label(ui, "Wait time (ms/c)");
                    ui.add(Slider::new(
                        &mut write_core.persistent.settings.text_wait,
                        MIN_TEXT_WAIT..=MAX_TEXT_WAIT,
                    ));
                    ui.end_row();

                    setting_label(ui, "Comma pause (ms)");
                    ui.add(Slider::new(
                        &mut write_core.persistent.settings.comma_pause_ms,
                        0..=MAX_PUNCT_PAUSE,
                    ));
                    ui.end_row();

                    setting_label(ui, "Period pause (ms)");
                    ui.add(Slider::new(
                        &mut write_core.persistent.settings.period_pause_ms,
                        0..=MAX_PUNCT_PAUSE,
                    ));
                    ui.end_row();

                    // Skip unseen
                    setting_label(ui, "Skip unseen");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.add(toggle(&mut write_core.persistent.settings.skip_unseen));
                    });
                    ui.end_row();

                    section_label(ui, "Audio");
                    ui.end_row();

                    // Bgm volume
                    setting_label(ui, "Bgm volume");
                    if ui
                        .add(Slider::new(
                            &mut write_core.persistent.settings.bgm_volume,
                            0..=MAX_VOLUME,
                        ))
                        .changed()
                    {
                        app.audio.set_volumes(&write_core.persistent.settings);
                    }
                    ui.end_row();

                    // Sfx volume
                    setting_label(ui, "Sfx volume");
                    if ui
                        .add(Slider::new(
                            &mut write_core.persistent.settings.sfx_volume,
                            0..=MAX_VOLUME,
                        ))
                        .changed()
                    {
                        app.audio.set_volumes(&write_core.persistent.settings);
                    }
                    ui.end_row();

                    // Keep playing in background
                    setting_label(ui, "Play in background");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.add(toggle(&mut write_core.persistent.settings.play_in_bg));
                    });
                    ui.end_row();

                    section_label(ui, "Video");
                    ui.end_row();

                    setting_label(ui, "Fullscreen");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        ui.add(toggle(&mut write_core.persistent.settings.fullscreen));
                    });
                    ui.end_row();

                    // Send persistent update to persist worker
                    if persistent_snapshot != write_core.persistent {
                        let _ = write_core.persist_worker_tx.try_send(());
                    }
                });
        });
    });
}
