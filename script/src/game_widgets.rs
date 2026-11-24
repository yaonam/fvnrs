use state::{
    game_state::{GameState, GameStateInputMode},
    schema::widget_schema::{MultipleChoiceSchema, RouteSelectSchema, WidgetSchema},
};

pub fn route_select(game_state: &mut GameState, routes: Vec<RouteSelectSchema>) {
    game_state.widget = Some(WidgetSchema::RouteSelect { routes });
    game_state.input_mode = GameStateInputMode::AdvanceBlocked;
}

pub fn multiple_choice(
    game_state: &mut GameState,
    choices: Vec<MultipleChoiceSchema>,
    max_weight: usize,
    points_threshold: usize,
    pass_scene_id: usize,
    fail_scene_id: usize,
) {
    game_state.widget = Some(WidgetSchema::MultipleChoice {
        choices,
        max_weight,
        points_threshold,
        pass_scene_id,
        fail_scene_id,
    });
    game_state.input_mode = GameStateInputMode::AdvanceBlocked;
}
