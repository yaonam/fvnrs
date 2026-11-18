use state::game_state::{GameState, GameStateInputMode};

pub fn hide_textbox(game_state: &mut GameState) {
    game_state.textbox.visible = false;

    // Make it advance by default
    game_state.input_mode = GameStateInputMode::Continue;
}

pub fn show_textbox(game_state: &mut GameState) {
    game_state.textbox.visible = true;

    // Make it advance by default
    game_state.input_mode = GameStateInputMode::Continue;
}

pub fn set_textbox_pos(game_state: &mut GameState, pos: [f32; 2]) {
    game_state.textbox.position = Some(pos);

    // Make it advance by default
    game_state.input_mode = GameStateInputMode::Continue;
}

pub fn reset_textbox_pos(game_state: &mut GameState) {
    game_state.textbox.position = None;

    // Make it advance by default
    game_state.input_mode = GameStateInputMode::Continue;
}

pub fn reset_textbox(game_state: &mut GameState) {
    game_state.textbox.position = None;
    game_state.textbox.visible = true;

    // Make it advance by default
    game_state.input_mode = GameStateInputMode::Continue;
}
