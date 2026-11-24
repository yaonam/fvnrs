use eframe::egui::{self, Checkbox, FontId, RichText, Ui, UiBuilder};
use state::schema::widget_schema::{MultipleChoiceSchema, WidgetSchema};

use crate::{
    constants::{
        CONTROLS_ROW_HEIGHT, HUD_HEIGHT_RATIO, LARGE_BUTTON_FONT_SIZE, ROUTE_SLOT_HEIGHT,
        ROUTE_SLOT_PADDING, ROUTE_SLOT_TEXT_SIZE, ROUTE_SLOT_WIDTH,
    },
    inputs::handle_jump,
    ui::FvnrsApp,
};

pub fn draw_multiple_choice(app: &mut FvnrsApp, ui: &mut Ui) -> bool {
    let mut core_write = app.engine_core.write();
    let Some(game_state) = core_write.screens.get_game_state_mut() else {
        return false;
    };
    let Some(widget) = &mut game_state.widget else {
        return false;
    };
    let WidgetSchema::MultipleChoice {
        choices,
        max_weight,
        points_threshold,
        pass_scene_id,
        fail_scene_id,
    } = widget
    else {
        return false;
    };

    // TODO: Add submit button to sizing
    let viewport = ui.max_rect();
    let pos = egui::pos2(
        viewport.center().x,
        (viewport.height() * (1.0 - HUD_HEIGHT_RATIO) - CONTROLS_ROW_HEIGHT) / 2.0,
    );
    let spacing_total = choices.len().saturating_sub(1) as f32 * ROUTE_SLOT_PADDING;
    let size = egui::vec2(
        ROUTE_SLOT_WIDTH,
        ROUTE_SLOT_HEIGHT * choices.len() as f32 + spacing_total,
    );
    let rect = egui::Rect::from_center_size(pos, size);

    let mut needs_render = false;
    let mut jump_to_scene: Option<usize> = None;

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = ROUTE_SLOT_PADDING;

        // Check if at max weight
        let weight_sum = choices
            .iter()
            .map(|c| if c.selected { c.weight } else { 0 })
            .sum::<usize>();
        let at_max_weight = weight_sum >= *max_weight;

        for choice in choices.iter_mut() {
            needs_render |= draw_choice_button(ui, choice, at_max_weight)
        }

        if draw_submit_button(ui) {
            let points_sum = choices
                .iter()
                .map(|c| if c.selected { c.points } else { 0 })
                .sum::<usize>();
            let at_points_threshold = points_sum >= *points_threshold;
            if at_points_threshold {
                jump_to_scene = Some(*pass_scene_id);
            } else {
                jump_to_scene = Some(*fail_scene_id);
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
fn draw_choice_button(ui: &mut Ui, choice: &mut MultipleChoiceSchema, at_max_weight: bool) -> bool {
    if at_max_weight && !choice.selected {
        ui.add_enabled(
            false,
            Checkbox::new(
                &mut choice.selected,
                RichText::new(&choice.description)
                    .font(egui::FontId::proportional(ROUTE_SLOT_TEXT_SIZE)),
            ),
        );
        false
    } else {
        ui.checkbox(
            &mut choice.selected,
            RichText::new(&choice.description)
                .font(egui::FontId::proportional(ROUTE_SLOT_TEXT_SIZE)),
        )
        .clicked()
    }
}

fn draw_submit_button(ui: &mut Ui) -> bool {
    ui.button(RichText::new("Submit").font(FontId::proportional(LARGE_BUTTON_FONT_SIZE)))
        .clicked()
}
