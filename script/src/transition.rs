use state::game_state::GameState;

use crate::{
    animation::AnimationBuilder,
    layered_image::{LayeredImage, show_bg},
};

pub fn fade_in<'a>(animation_builder: AnimationBuilder<'a>) -> AnimationBuilder<'a> {
    animation_builder
        .opacity(0.0)
        .tween(1.0)
        .opacity(1.0)
        .wait(1.0)
        .and()
}

pub fn fade_out<'a>(animation_builder: AnimationBuilder<'a>) -> AnimationBuilder<'a> {
    animation_builder
        .opacity(1.0)
        .tween(1.0)
        .opacity(0.0)
        .wait(1.0)
        .and()
}

pub fn fade_in_bg<L: LayeredImage>(
    game_state: &mut GameState,
    layered_image: L,
    tween_duration: f32,
) {
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

pub fn fade_out_bg<L: LayeredImage>(
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

/// Fades out the hide_image and then fades in the show_image.
pub fn fade_in_out_bg<HL: LayeredImage, SL: LayeredImage>(
    game_state: &mut GameState,
    hide_image: HL,
    show_image: SL,
    tween_duration: f32,
) {
    show_bg(game_state, hide_image)
        .opacity(1.0)
        .animate()
        .opacity(1.0)
        .tween(tween_duration)
        .opacity(0.0)
        .wait(1.0)
        // Fade in next image
        .show_bg(show_image)
        .opacity(0.0)
        .tween(tween_duration)
        .opacity(1.0)
        .wait(1.0)
        .auto_advance()
        .and();
}
