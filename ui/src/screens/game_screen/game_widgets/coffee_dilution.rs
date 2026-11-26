use eframe::egui::{self, Checkbox, FontId, Grid, RichText, Slider, Ui, UiBuilder};
use state::schema::widget_schema::{CoffeeDilutionSchema, MultipleChoiceSchema, WidgetSchema};

use crate::{
    constants::{
        CONTROLS_ROW_HEIGHT, HUD_HEIGHT_RATIO, LARGE_BUTTON_FONT_SIZE, ROUTE_SLOT_HEIGHT,
        ROUTE_SLOT_PADDING, ROUTE_SLOT_TEXT_SIZE, ROUTE_SLOT_WIDTH,
    },
    inputs::handle_jump,
    ui::FvnrsApp,
};

pub fn draw_coffee_dilution(app: &mut FvnrsApp, ui: &mut Ui) -> bool {
    let mut core_write = app.engine_core.write();
    let Some(game_state) = core_write.screens.get_game_state_mut() else {
        return false;
    };
    let Some(widget) = &mut game_state.widget else {
        return false;
    };
    let WidgetSchema::CoffeeDilution(d) = widget else {
        return false;
    };

    let viewport = ui.max_rect();
    let pos = egui::pos2(
        viewport.center().x,
        (viewport.height() * (1.0 - HUD_HEIGHT_RATIO) - CONTROLS_ROW_HEIGHT) / 2.0,
    );
    let spacing_total = 2.0 * ROUTE_SLOT_PADDING;
    let size = egui::vec2(ROUTE_SLOT_WIDTH, ROUTE_SLOT_HEIGHT * 2.0 + spacing_total);
    let rect = egui::Rect::from_center_size(pos, size);

    let mut needs_render = false;
    let mut jump_to_scene: Option<usize> = None;

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = ROUTE_SLOT_PADDING;

        needs_render |= draw_dilution_form(ui, d);

        if draw_submit_button(ui) {
            let pass = {
                if d.r1 == 0 && d.r2 == 0 {
                    false
                } else {
                    let c_diluted = (d.c1 * d.r1 + d.c2 * d.r2) / (d.r1 + d.r2);
                    let s_diluted = (d.s1 * d.r1 + d.s2 * d.r2) / (d.r1 + d.r2);

                    c_diluted == d.c_target && s_diluted == d.s_target
                }
            };
            if pass {
                jump_to_scene = Some(d.pass_scene_id);
            } else {
                jump_to_scene = Some(d.fail_scene_id);
            }
        }
    });

    if let Some(scene_id) = jump_to_scene {
        drop(core_write);
        handle_jump(app, scene_id);

        return true;
    }

    needs_render
}

/// Returns true if the route button was clicked.
fn draw_dilution_form(ui: &mut Ui, d: &mut CoffeeDilutionSchema) -> bool {
    Grid::new("game_widget_coffee_dilution_grid").show(ui, |ui| {
        ui.add(Slider::new(&mut d.c1, 0..=5).step_by(1.0));
        ui.add(Slider::new(&mut d.s1, 0..=5).step_by(1.0));

        ui.add(Slider::new(&mut d.c2, 0..=5).step_by(1.0));
        ui.add(Slider::new(&mut d.s2, 0..=5).step_by(1.0));

        ui.add(Slider::new(&mut d.r1, 0..=5).step_by(1.0));
        ui.add(Slider::new(&mut d.r2, 0..=5).step_by(1.0));
    });

    false
}

fn draw_submit_button(ui: &mut Ui) -> bool {
    ui.button(RichText::new("Submit").font(FontId::proportional(LARGE_BUTTON_FONT_SIZE)))
        .clicked()
}
