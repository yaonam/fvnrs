use eframe::egui::{self, Checkbox, RichText, Ui, UiBuilder};
use state::schema::widget_schema::{MultipleChoiceSchema, WidgetSchema};

use crate::{
    constants::{
        CONTROLS_ROW_HEIGHT, HUD_HEIGHT_RATIO, ROUTE_SLOT_HEIGHT, ROUTE_SLOT_PADDING,
        ROUTE_SLOT_TEXT_SIZE, ROUTE_SLOT_WIDTH,
    },
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

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = ROUTE_SLOT_PADDING;

        // Check if at max weight
        let at_max_weight = choices
            .iter()
            .map(|c| if c.selected { c.weight } else { 0 })
            .sum::<usize>()
            >= *max_weight;

        for mut choice in choices {
            if draw_choice_button(ui, &mut choice, at_max_weight) {
                needs_render = true;
            }
        }

        // TODO: Add submit button
    });

    needs_render
}

/// Returns true if the route button was clicked.
fn draw_choice_button(ui: &mut Ui, choice: &mut MultipleChoiceSchema, at_max_weight: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ROUTE_SLOT_WIDTH, ROUTE_SLOT_HEIGHT),
        egui::Sense::click(),
    );

    if at_max_weight && !choice.selected {
        ui.add_enabled(
            false,
            Checkbox::new(
                &mut choice.selected,
                RichText::new(&choice.description)
                    .font(egui::FontId::proportional(ROUTE_SLOT_TEXT_SIZE)),
            ),
        );
    } else {
        ui.checkbox(
            &mut choice.selected,
            RichText::new(&choice.description)
                .font(egui::FontId::proportional(ROUTE_SLOT_TEXT_SIZE)),
        );
    }

    response.clicked()
}
