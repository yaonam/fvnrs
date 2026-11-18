use std::path::PathBuf;

use eframe::egui::{self, FontId, RichText, ScrollArea, Ui};
use engine_core::io::{list_files, read_file, remove_file, saves_dir};
use state::{
    schema::save_schema::SaveSchema, screens::BaseScreenSchema, utils::get_current_time_ms,
};

use crate::{
    constants::{
        SAVE_FADE_IN_TIME_MS, SAVE_ITEM_DELETE_PADDING, SAVE_ITEM_DELETE_TEXT_SIZE,
        SAVE_ITEM_DETAILS_TEXT_SIZE, SAVE_ITEM_DETAILS_WIDTH, SAVE_ITEM_TITLE_TEXT_SIZE,
        SAVE_SLOT_CORNER_RADIUS, SAVE_SLOT_HEIGHT, SAVE_SLOT_PADDING, SAVE_SLOT_THUMB_SIZE,
        SCREENSHOT_HEIGHT,
    },
    ui::FvnrsApp,
};

#[derive(Default)]
pub(crate) struct LoadedSaves {
    /// JSON path and the save it contains.
    pub saves: Vec<(String, SaveSchema)>,
    pub loaded_timestamp_ms: u64,
    // For animated fade in
    pub displaying_index: usize,
    pub displaying_timestamp_ms: u64,
}

pub(crate) fn draw_saves_view(app: &mut FvnrsApp, ui: &mut Ui, frame: &mut eframe::Frame) {
    ScrollArea::vertical().show(ui, |ui| {
        let now_ms = get_current_time_ms();

        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
            if matches!(
                app.engine_core.read().screens.base_screen,
                BaseScreenSchema::Game(_)
            ) {
                draw_save_button(ui, frame, app);
                ui.add_space(SAVE_SLOT_PADDING);
            }

            // Load saves if first open
            let open_close_time = app
                .engine_core
                .read()
                .screens
                .overlay_screen
                .open_close_time;
            if open_close_time != app.loaded_saves.loaded_timestamp_ms {
                load_saves(app, open_close_time, now_ms);
            }

            let elapsed_ms = now_ms.saturating_sub(app.loaded_saves.displaying_timestamp_ms);
            let fade_t = (elapsed_ms as f32 / SAVE_FADE_IN_TIME_MS as f32).clamp(0.0, 1.0);

            // Draw each save. Take the vec so each tile can mutably use `app`.
            ui.style_mut().spacing.button_padding = SAVE_ITEM_DELETE_PADDING;
            let mut saves = std::mem::take(&mut app.loaded_saves.saves);
            let mut delete_index = None;
            for (i, (json_path, save)) in saves.iter().rev().enumerate() {
                if i > app.loaded_saves.displaying_index {
                    break;
                }
                let opacity = if i < app.loaded_saves.displaying_index {
                    1.0
                } else {
                    fade_t
                };
                if draw_save_item(ui, app, json_path, save, opacity) {
                    delete_index = Some(saves.len() - 1 - i);
                }
                ui.add_space(SAVE_SLOT_PADDING);
            }
            if let Some(index) = delete_index {
                let (json_path, _) = saves.remove(index);
                delete_save(&json_path);
            }
            app.loaded_saves.saves = saves;
            if app.loaded_saves.displaying_index + 1 <= app.loaded_saves.saves.len() {
                if fade_t >= 1.0 {
                    app.loaded_saves.displaying_index += 1;
                    app.loaded_saves.displaying_timestamp_ms = now_ms;
                }
                ui.ctx().request_repaint();
            }
        });
    });
}

fn load_saves(app: &mut FvnrsApp, open_close_time: u64, now_ms: u64) {
    // Read all filenames
    let save_files = list_files(saves_dir(), "json").unwrap_or_default();
    let mut loaded_saves = Vec::with_capacity(save_files.len());
    // Load each file
    for save_file in save_files {
        if let Ok(save) = read_file::<SaveSchema, &str>(save_file.as_str()) {
            loaded_saves.push((save_file, save));
        }
    }
    app.loaded_saves = LoadedSaves {
        saves: loaded_saves,
        loaded_timestamp_ms: open_close_time,
        displaying_index: 0,
        displaying_timestamp_ms: now_ms,
    };
}

/// Allocates a save slot, paints its background, and runs `add_contents` in the padded interior.
fn draw_save_slot(ui: &mut Ui, opacity: f32, add_contents: impl FnOnce(&mut Ui)) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ui.available_width(), SAVE_SLOT_HEIGHT),
        egui::Sense::click(),
    );

    let mut fill = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().faint_bg_color
    };
    fill = fill.gamma_multiply(opacity);
    ui.painter()
        .rect_filled(rect, SAVE_SLOT_CORNER_RADIUS, fill);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(rect.shrink(SAVE_SLOT_PADDING)),
        add_contents,
    );
    response
}

fn draw_save_button(ui: &mut Ui, frame: &mut eframe::Frame, app: &mut FvnrsApp) {
    let response = draw_save_slot(ui, 1.0, |ui| {
        let (rect, _) = ui.allocate_exact_size(SAVE_SLOT_THUMB_SIZE, egui::Sense::hover());
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            FontId::proportional(64.0),
            ui.visuals().text_color(),
        );
    });

    if response.clicked() {
        let saved = app.engine_core.write().save();
        if let Some((name, save_schema)) = saved {
            let screenshot_path = saves_dir().join(format!("{name}.webp"));
            app.save_screenshot(frame, ui.ctx(), &screenshot_path, SCREENSHOT_HEIGHT);
            let json_path = saves_dir().join(format!("{name}.json"));
            app.loaded_saves
                .saves
                .push((json_path.to_string_lossy().into_owned(), save_schema));
            log::warn!("Saved the game!");
        }
    }
}

/// Returns true when the delete button was clicked.
fn draw_save_item(
    ui: &mut Ui,
    app: &mut FvnrsApp,
    json_path: &str,
    save: &SaveSchema,
    opacity: f32,
) -> bool {
    let mut delete_clicked = false;
    let response = draw_save_slot(ui, opacity, |ui| {
        ui.horizontal(|ui| {
            if let Some(handle) = app.screenshot_tex_ids.get(&screenshot_path_for(json_path)) {
                ui.add(
                    egui::Image::new((handle.id(), SAVE_SLOT_THUMB_SIZE))
                        .corner_radius(SAVE_SLOT_CORNER_RADIUS),
                );
            } else {
                ui.add_sized(SAVE_SLOT_THUMB_SIZE, egui::Label::new(""));
            }

            ui.vertical(|ui| {
                ui.set_width(SAVE_ITEM_DETAILS_WIDTH);
                ui.label(
                    RichText::new(&save.name).font(FontId::proportional(SAVE_ITEM_TITLE_TEXT_SIZE)),
                );
                ui.label(
                    RichText::new(format_saved_at(save.timestamp_ms))
                        .font(FontId::proportional(SAVE_ITEM_DETAILS_TEXT_SIZE)),
                );
                ui.label(
                    RichText::new(format_playtime(save.total_playtime_sec))
                        .font(FontId::proportional(SAVE_ITEM_DETAILS_TEXT_SIZE)),
                );
            });

            delete_clicked = ui
                .button(
                    RichText::new("\u{1F5D1}")
                        .font(FontId::proportional(SAVE_ITEM_DELETE_TEXT_SIZE)),
                )
                .clicked();
        });
    });

    if delete_clicked {
        true
    } else if response.clicked() {
        let mut core_write = app.engine_core.write();
        core_write.load_save(save);
        core_write.screens.close_menu();
        log::warn!("Loaded the game!");
        false
    } else {
        false
    }
}

fn screenshot_path_for(json_path: &str) -> String {
    let mut path = PathBuf::from(json_path);
    path.set_extension("webp");
    path.to_string_lossy().into_owned()
}

fn delete_save(json_path: &str) {
    let screenshot_path = screenshot_path_for(json_path);
    if let Err(e) = remove_file(json_path) {
        log::error!("Failed to delete save {json_path}: {e}");
    }
    if let Err(e) = remove_file(&screenshot_path) {
        log::error!("Failed to delete screenshot {screenshot_path}: {e}");
    }
}

fn format_saved_at(timestamp_ms: u64) -> String {
    let Some(dt) = chrono::DateTime::from_timestamp_millis(timestamp_ms as i64) else {
        return String::new();
    };
    dt.with_timezone(&chrono::Local)
        .format("%b %-d, %-I:%M %p")
        .to_string()
}

fn format_playtime(total_sec: u64) -> String {
    let hours = total_sec / 3600;
    let minutes = (total_sec % 3600) / 60;
    let seconds = total_sec % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}
