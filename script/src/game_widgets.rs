use state::{
    game_state::{GameState, GameStateInputMode},
    schema::widget_schema::{
        CoffeeDilutionSchema, MultipleChoiceSchema, RouteSelectSchema, WidgetSchema,
    },
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

pub fn coffee_dilution(game_state: &mut GameState, pass_scene_id: usize, fail_scene_id: usize) {
    game_state.widget = Some(WidgetSchema::CoffeeDilution(CoffeeDilutionSchema {
        c_target: 1,
        s_target: 1,
        pass_scene_id,
        fail_scene_id,
        c1: 0,
        s1: 0,
        c2: 0,
        s2: 0,
        r1: 0,
        r2: 0,
    }))
}
