use state::game_state::GameState;

use crate::layered_image::{LayeredImage, show_bg};

pub fn fade_in<L: LayeredImage>(game_state: &mut GameState, layered_image: L, tween_duration: f32) {
    show_bg(game_state, layered_image)
        .opacity(0.0)
        .animate()
        .opacity(0.0)
        .tween(tween_duration)
        .opacity(1.0)
        .wait(1.0)
        .auto_advance()
        .and();
}

pub fn fade_out<L: LayeredImage>(
    game_state: &mut GameState,
    layered_image: L,
    tween_duration: f32,
) {
    show_bg(game_state, layered_image)
        .opacity(1.0)
        .animate()
        .opacity(1.0)
        .tween(tween_duration)
        .opacity(0.0)
        .wait(1.0)
        .auto_advance()
        .and();
}
