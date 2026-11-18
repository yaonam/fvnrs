use eframe::egui::{Align, Layout, Pos2, Rect, Ui, UiBuilder};

use crate::{
    constants::{BUTTON_PADDING, EDGE_PADDING, MENUS_TOP_OFFSET, OVERLAY_H_DIVIDER},
    ui::FvnrsApp,
};

pub(crate) fn padded_ui(ui: &mut Ui) -> Ui {
    let mut padded_ui = ui.new_child(UiBuilder::new().max_rect(Rect::from_center_size(
        ui.max_rect().center(),
        ui.max_rect().size() * (1.0 - 2.0 * EDGE_PADDING),
    )));
    padded_ui.set_clip_rect(padded_ui.max_rect());

    padded_ui
}

pub(crate) fn controls_rect(padded_ui: &Ui) -> Rect {
    let clip_rect = padded_ui.clip_rect();
    let div_x = clip_rect.width() * OVERLAY_H_DIVIDER + clip_rect.left();
    let div_bot: Pos2 = [div_x, clip_rect.bottom()].into();
    Rect::from_two_pos(clip_rect.left_top(), div_bot)
}

pub(crate) fn draw_menu_controls(
    app: &mut FvnrsApp,
    ui: &mut Ui,
    viewport: Rect,
    ctrl_fn: fn(&mut FvnrsApp, &mut Ui),
) {
    ui.style_mut().spacing.button_padding = BUTTON_PADDING;

    ui.scope_builder(
        UiBuilder::new().layout(
            Layout::top_down(Align::LEFT).with_cross_justify(true), // Expand buttons horizontally
        ),
        |ui| {
            ui.set_min_size(viewport.size());
            ui.set_max_size(viewport.size());

            ui.add_space(MENUS_TOP_OFFSET);

            ctrl_fn(app, ui);
        },
    );
}
