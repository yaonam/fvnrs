use eframe::{egui::{self, FontId, Frame, RichText, Ui}, epaint};
use state::schema::widget_schema::{WidgetSchema};

use crate::{constants::{HUD_BLUR_FILL, ROUTE_SLOT_CORNER_RADIUS, ROUTE_SLOT_HEIGHT, ROUTE_SLOT_PADDING, ROUTE_SLOT_TEXT_SIZE}, inputs::handle_jump, ui::FvnrsApp};

pub fn draw_route_select(app: &mut FvnrsApp, ui: &mut Ui) -> bool {
    let mut core_write = app.engine_core.write();
    let Some(game_state) = core_write.screens.get_game_state_mut() else {
        return false;
    };
    let Some(widget) = &game_state.widget else {
        return false;
    };
    let WidgetSchema::RouteSelect { routes } = widget;

    let viewport = ui.max_rect();
    let size = egui::vec2(viewport.width(), viewport.height());
    let pos = egui::pos2(viewport.center().x - size.x / 2.0, viewport.top());
    let rect = egui::Rect::from_min_size(pos, size);

    let mut jump_to_scene: Option<usize> = None;

    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
        let hud_frame = 
            // Overlay hud frame
            Frame::default()
                .shadow(epaint::Shadow {
                    color: HUD_BLUR_FILL,
                    offset: [0, 4],
                    blur: 20,
                    spread: 17,
                });

        hud_frame.show(ui, |ui| {
            ui.with_layout(
                egui::Layout::top_down(egui::Align::Center)
                .with_main_align(egui::Align::Center), |ui| {
                for route in routes {
                    if draw_route_button(ui, &route.description) {
                        jump_to_scene = Some(route.scene_id);
                    }
                }
            });
        });
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
        egui::vec2(ui.available_width(), ROUTE_SLOT_HEIGHT),
        egui::Sense::click(),
    );

    let fill = if response.hovered() {
        ui.visuals().widgets.hovered.bg_fill
    } else {
        ui.visuals().faint_bg_color
    };
    ui.painter()
        .rect_filled(rect, ROUTE_SLOT_CORNER_RADIUS, fill);

    ui.scope_builder(
        egui::UiBuilder::new().max_rect(rect.shrink(ROUTE_SLOT_PADDING)),
        |ui| {
            ui.label(
                RichText::new(route_description).font(FontId::proportional(ROUTE_SLOT_TEXT_SIZE))
            );
        },
    );
    response.clicked()
}
