use state::{
    game_state::{GameState, GameStateInputMode},
    schema::widget_schema::{RouteSelectSchema, WidgetSchema},
};

pub fn route_select(game_state: &mut GameState, routes: Vec<RouteSelectSchema>) {
    game_state.widget = Some(WidgetSchema::RouteSelect { routes });
    game_state.input_mode = GameStateInputMode::AdvanceBlocked;
}
