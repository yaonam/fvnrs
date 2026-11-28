use std::ops::Sub;

use crate::{
    constants::{
        ACTIVATED_BUTTON_COLOR, CHARACTER_NAME_SIZE, CONTROLS_INTERACT_SIZE, CONTROLS_ROW_HEIGHT,
        CONTROLS_V_PADDING, HUD_BLUR_FILL, HUD_BORDER_STROKE, HUD_HEIGHT_RATIO, HUD_MARGIN_RATIO,
        HUD_WIDTH_RATIO, NORMAL_BUTTON_COLOR, NVL_FILL, NVL_SPACING, NVL_WIDTH_RATIO, TEXT_SIZE,
    },
    render::{
        buttons::{button_text, col_button_click},
        rich_text::rich_text_label,
    },
    ui::FvnrsApp,
};
use eframe::egui::{self, Align, Color32, Frame, Layout, Pos2, ScrollArea, Ui, epaint};
use state::schema::{
    session_schema::{TextScrollMode, UserInputOverride},
    text_schema::TextModifiers,
};

/// Returns whether a repaint is needed.
pub fn draw_dialogue_hud(app: &mut FvnrsApp, ui: &mut Ui) -> bool {
    // Reuse the viewport to prevent the cursor from moving
    let viewport = ui.max_rect();

    ui.style_mut().spacing.interact_size = CONTROLS_INTERACT_SIZE;

    if app
        .engine_core
        .read()
        .screens
        .get_game_state()
        .map_or(false, |gs| gs.textbox.text_history.is_some())
    {
        draw_nvl_dialogue(app, ui, viewport);
    } else {
        draw_dialogue(app, ui, viewport);
    }
    draw_controls(app, ui, viewport)
}

fn draw_nvl_dialogue(app: &mut FvnrsApp, ui: &mut Ui, viewport: egui::Rect) {
    let core_read = app.engine_core.read();
    let Some(game_state) = core_read.screens.get_game_state() else {
        return;
    };

    let Some(text_history) = &game_state.textbox.text_history else {
        log::error!("NVL mode is enabled but text_history is None.");
        return;
    };

    let raw_size = &game_state
        .textbox
        .size
        .unwrap_or([viewport.width() * NVL_WIDTH_RATIO, viewport.height()]);
    let size: egui::Vec2 = raw_size.into();
    let raw_pos = &game_state
        .textbox
        .position
        .unwrap_or([viewport.center().x - size.x / 2.0, viewport.top()]);
    let pos: Pos2 = raw_pos.into();
    let rect = egui::Rect::from_min_size(pos, size);

    let mut should_end_text_scroll = false;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let scaled_margin = HUD_MARGIN_RATIO * viewport.width();

        // Draw the blurred game target
        let uv = egui::Rect::from_min_max(
            egui::pos2(
                (rect.min.x - viewport.min.x) / viewport.width(),
                (rect.min.y - viewport.min.y) / viewport.height(),
            ),
            egui::pos2(
                (rect.max.x - viewport.min.x) / viewport.width(),
                (rect.max.y - viewport.min.y) / viewport.height(),
            ),
        );
        egui::Image::new((app.egui_blurred_tex_id, rect.size()))
            .uv(uv)
            .corner_radius(15.0) // match frame
            .paint_at(ui, rect);
        let hud_frame = Frame::default()
            .fill(NVL_FILL)
            .inner_margin(scaled_margin)
            .shadow(epaint::Shadow {
                color: NVL_FILL,
                offset: [0, 0],
                blur: 20,
                spread: 17,
            });

        // Apply the styling frame
        hud_frame.show(ui, |ui| {
            ScrollArea::vertical()
                .auto_shrink(false)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                // .stick_to_bottom(true)
                .scroll_source(egui::scroll_area::ScrollSource::NONE)
                .max_height(ui.available_height() - CONTROLS_ROW_HEIGHT)
                .show(ui, |ui| {
                    let base_modifiers = TextModifiers {
                        color: game_state.textbox.text_color.clone(),
                        size: Some(TEXT_SIZE),
                        ..Default::default()
                    };
                    // TODO: Display name if set
                    text_history.iter().for_each(|text| {
                        rich_text_label(
                            ui,
                            &TextScrollMode::ScrollEnd(0), // Hardcode to disable scrolling
                            &core_read.persistent.settings,
                            text,
                            &base_modifiers,
                        );
                        ui.add_space(NVL_SPACING);
                    });

                    ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                        should_end_text_scroll = rich_text_label(
                            ui,
                            &core_read.session.text_scroll_mode,
                            &core_read.persistent.settings,
                            &game_state.textbox.text,
                            &base_modifiers,
                        );
                    });

                    ui.scroll_to_cursor(Some(Align::BOTTOM));
                });
            ui.add_space(CONTROLS_ROW_HEIGHT);
        });
    });

    // Need to drop core_read before this!
    drop(core_read);
    if should_end_text_scroll {
        app.engine_core.write().session.end_text_scroll();
    }
}

fn draw_dialogue(app: &mut FvnrsApp, ui: &mut Ui, viewport: egui::Rect) {
    let core_read = app.engine_core.read();
    let Some(game_state) = core_read.screens.get_game_state() else {
        return;
    };

    let size = egui::vec2(
        viewport.width() * HUD_WIDTH_RATIO,
        viewport.height() * HUD_HEIGHT_RATIO,
    );
    let raw_pos = if let Some(textbox_pos) = game_state.textbox.position {
        egui::pos2(
            viewport.width() * textbox_pos[0] + viewport.left(),
            viewport.height() * textbox_pos[1] + viewport.top(),
        )
    } else {
        egui::pos2(viewport.center().x, viewport.bottom())
    };
    // Adjust pos by textbox size and botton controls bar
    let pos: Pos2 = raw_pos
        .sub(egui::pos2(size.x / 2.0, size.y + CONTROLS_ROW_HEIGHT))
        .to_pos2();
    let rect = egui::Rect::from_min_size(pos, size);

    let mut should_end_text_scroll = false;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let scaled_margin = HUD_MARGIN_RATIO * viewport.width();

        // Draw the hud if TextBoxSchema.position is not set
        // Define the styling for the HUD
        let hud_frame = if game_state.textbox.visible {
            // Draw the blurred game target
            let uv = egui::Rect::from_min_max(
                egui::pos2(
                    (rect.min.x - viewport.min.x) / viewport.width(),
                    (rect.min.y - viewport.min.y) / viewport.height(),
                ),
                egui::pos2(
                    (rect.max.x - viewport.min.x) / viewport.width(),
                    (rect.max.y - viewport.min.y) / viewport.height(),
                ),
            );
            egui::Image::new((app.egui_blurred_tex_id, rect.size()))
                .uv(uv)
                .corner_radius(15.0) // match frame
                .paint_at(ui, rect);
            // Overlay hud frame
            Frame::default()
                .fill(HUD_BLUR_FILL)
                .corner_radius(15.0)
                .stroke(HUD_BORDER_STROKE)
                .inner_margin(scaled_margin)
                .shadow(epaint::Shadow {
                    color: Color32::from_black_alpha(30),
                    offset: [0, 4],
                    blur: 20,
                    spread: 17,
                })
        } else {
            Frame::default()
        };

        // Apply the styling frame
        hud_frame.show(ui, |ui| {
            ui.set_width(viewport.width() * HUD_WIDTH_RATIO - scaled_margin * 2.0);
            ui.set_height(viewport.height() * HUD_HEIGHT_RATIO - scaled_margin * 2.0);
            {
                ui.with_layout(Layout::top_down(Align::LEFT), |ui| {
                    rich_text_label(
                        ui,
                        &TextScrollMode::ScrollEnd(0), // Hardcode to disable scrolling
                        &core_read.persistent.settings,
                        &game_state.textbox.character_name,
                        &TextModifiers {
                            color: game_state.textbox.character_name_color.clone(),
                            size: Some(CHARACTER_NAME_SIZE),
                            ..Default::default()
                        },
                    );
                    should_end_text_scroll = rich_text_label(
                        ui,
                        &core_read.session.text_scroll_mode,
                        &core_read.persistent.settings,
                        &game_state.textbox.text,
                        &TextModifiers {
                            color: game_state.textbox.text_color.clone(),
                            size: Some(TEXT_SIZE),
                            ..Default::default()
                        },
                    );
                });
            }
        });
    });

    // Need to drop core_read before this!
    drop(core_read);
    if should_end_text_scroll {
        app.engine_core.write().session.end_text_scroll();
    }
}

/// Draws the controls at the bottom of the screen.
fn draw_controls(app: &mut FvnrsApp, ui: &mut Ui, viewport: egui::Rect) -> bool {
    let mut render = false;
    let rect = egui::Rect::from_min_size(
        egui::pos2(
            viewport.left() + viewport.width() * 0.25,
            viewport.bottom() - CONTROLS_INTERACT_SIZE.y - CONTROLS_V_PADDING,
        ),
        egui::vec2(viewport.width() * 0.5, ui.spacing().interact_size.y),
    );

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        ui.columns(5, |cols| {
            if col_button_click(cols, 0, "Back") {
                let mut core_write = app.engine_core.write();
                core_write.back();
                // Render next screen
                if core_write.attr_changed_rev(|s| &s.playable.images) {
                    render = true;
                }
            }
            if cols[1]
                .centered_and_justified(|ui| {
                    ui.button(button_text("Auto").color(
                        match app.engine_core.read().session.user_input_override {
                            Some(UserInputOverride::Auto) => ACTIVATED_BUTTON_COLOR,
                            _ => NORMAL_BUTTON_COLOR,
                        },
                    ))
                })
                .inner
                .clicked()
            {
                let mut core_write = app.engine_core.write();
                core_write.session.user_input_override =
                    match core_write.session.user_input_override.is_some() {
                        true => None,
                        false => Some(UserInputOverride::Auto),
                    };
            }
            if cols[2]
                .centered_and_justified(|ui| {
                    ui.button(button_text("Skip").color(
                        match app.engine_core.read().session.user_input_override {
                            Some(UserInputOverride::Skip) => ACTIVATED_BUTTON_COLOR,
                            _ => NORMAL_BUTTON_COLOR,
                        },
                    ))
                })
                .inner
                .clicked()
            {
                let mut core_write = app.engine_core.write();
                core_write.session.user_input_override =
                    match core_write.session.user_input_override.is_some() {
                        true => None,
                        false => Some(UserInputOverride::Skip),
                    };
            }
            if col_button_click(cols, 3, "Hide") {
                let mut core_write = app.engine_core.write();

                if core_write.session.user_input_override != Some(UserInputOverride::Hide) {
                    core_write.session.user_input_override = Some(UserInputOverride::Hide);
                } else {
                    core_write.session.user_input_override = None;
                }
            }
            if col_button_click(cols, 4, "Menu") {
                let mut core_write = app.engine_core.write();
                core_write.screens.open_menu();
                log::warn!("Opened menu");
            }
        });
    });
    render
}
