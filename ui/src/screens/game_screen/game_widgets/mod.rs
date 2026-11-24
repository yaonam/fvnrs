use eframe::egui;
use state::schema::widget_schema::WidgetSchema;

use crate::ui::FvnrsApp;

mod multiple_choice;
mod route_select;

#[derive(Clone, Copy)]
enum WidgetKind {
    RouteSelect,
    MultipleChoice,
}

pub(crate) fn draw_game_widgets(app: &mut FvnrsApp, ui: &mut egui::Ui) -> bool {
    let widget = {
        let core_read = app.engine_core.read();
        let Some(game_state) = core_read.screens.get_game_state() else {
            return false;
        };
        let Some(widget) = &game_state.widget else {
            return false;
        };
        match widget {
            WidgetSchema::RouteSelect { .. } => WidgetKind::RouteSelect,
            WidgetSchema::MultipleChoice { .. } => WidgetKind::MultipleChoice,
        }
    };

    match widget {
        WidgetKind::RouteSelect => route_select::draw_route_select(app, ui),
        WidgetKind::MultipleChoice => multiple_choice::draw_multiple_choice(app, ui),
    }
}
