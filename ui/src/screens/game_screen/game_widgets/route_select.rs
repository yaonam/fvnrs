use eframe::egui::{self, FontId, Ui, UiBuilder};
use state::schema::widget_schema::WidgetSchema;

use crate::{
    constants::{
        CONTROLS_ROW_HEIGHT, HUD_HEIGHT_RATIO, ROUTE_SLOT_CORNER_RADIUS, ROUTE_SLOT_HEIGHT,
        ROUTE_SLOT_PADDING, ROUTE_SLOT_TEXT_SIZE, ROUTE_SLOT_WIDTH,
    },
    inputs::handle_jump,
    ui::FvnrsApp,
};

pub fn draw_route_select(app: &mut FvnrsApp, ui: &mut Ui) -> bool {
    let mut core_write = app.engine_core.write();
    let Some(game_state) = core_write.screens.get_game_state_mut() else {
        return false;
    };
    let Some(widget) = &game_state.widget else {
        return false;
    };
    let WidgetSchema::RouteSelect { routes } = widget else {
        return false;
    };

    let viewport = ui.max_rect();
    let pos = egui::pos2(
        viewport.center().x,
        (viewport.height() * (1.0 - HUD_HEIGHT_RATIO) - CONTROLS_ROW_HEIGHT) / 2.0,
    );
    let spacing_total = routes.len().saturating_sub(1) as f32 * ROUTE_SLOT_PADDING;
    let size = egui::vec2(
        ROUTE_SLOT_WIDTH,
        ROUTE_SLOT_HEIGHT * routes.len() as f32 + spacing_total,
    );
    let rect = egui::Rect::from_center_size(pos, size);

    let mut jump_to_scene: Option<usize> = None;

    ui.scope_builder(UiBuilder::new().max_rect(rect), |ui| {
        ui.spacing_mut().item_spacing.y = ROUTE_SLOT_PADDING;

        for route in routes {
            if draw_route_button(ui, &route.description) {
                jump_to_scene = Some(route.scene_id);
            }
        }
    });

    if let Some(scene_id) = jump_to_scene {
        drop(core_write);
        handle_jump(app, scene_id);

        return true;
    }

    false
}

/// Returns true if the route button was clicked.
fn draw_route_button(ui: &mut Ui, route_description: &str) -> bool {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(ROUTE_SLOT_WIDTH, ROUTE_SLOT_HEIGHT),
        egui::Sense::click(),
    );

    let fill = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().faint_bg_color
    };
    ui.painter()
        .rect_filled(rect, ROUTE_SLOT_CORNER_RADIUS, fill);

    ui.painter().text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        route_description,
        FontId::proportional(ROUTE_SLOT_TEXT_SIZE),
        ui.visuals().text_color(),
    );

    response.clicked()
}
