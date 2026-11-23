use log::warn;
use state::{
    game_state::{GameState, GameStateInputMode},
    schema::{
        animation_schema::{
            AnimationMode, AnimationSchema, AnimationStep, AnimationTweenSchema, EasingFn,
        },
        image_schema::{ImageSchema, ImageTransformSchema, opacity, rotation},
    },
};

use crate::layered_image::LayeredImage;

pub struct AnimationBuilder<'a> {
    pub animation_id: &'static str,
    pub game_state: &'a mut GameState,
    /// Images with transform from last animation step
    pub last_images: Option<Vec<ImageSchema>>, // TODO: Make this not an option?
    pub last_transform: ImageTransformSchema,
}

impl<'a> AnimationBuilder<'a> {
    pub fn rotation(self, radians: f32) -> Self {
        self.apply_transform_and_show(rotation(radians))
    }

    pub fn opacity(self, opacity_factor: f32) -> Self {
        self.apply_transform_and_show(opacity(opacity_factor))
    }

    // TODO: Other transforms
    // TODO: Absolute vs relative transforms

    pub fn tween(mut self, duration_seconds: f32) -> Self {
        let base_images = self.last_images.as_ref().expect("images exist").clone();
        // Create a tween step from the last images and transform
        let tween = AnimationTweenSchema {
            base_images,
            start_transform: self.last_transform.clone(),
            end_transform: self.last_transform.clone(),
            duration: duration_seconds,
            easing: EasingFn::Linear,
        };
        // Insert to game state
        self.push_step(AnimationStep::Tween(tween));

        self
    }

    pub fn wait(mut self, wait_seconds: f32) -> Self {
        self.push_step(AnimationStep::Wait(wait_seconds));

        self
    }

    /// Sets the easing for the last tween step.
    pub fn easing(mut self, easing: EasingFn) -> Self {
        let animation_step = self
            .get_animation_mut()
            .steps
            .last_mut()
            .expect("animation step exists");
        match animation_step {
            AnimationStep::Tween(tween) => tween.easing = easing,
            _ => warn!("Cannot set easing for non-tween animation step"),
        };

        self
    }

    pub fn linear(self) -> Self {
        self.easing(EasingFn::Linear)
    }

    pub fn quadratic_in(self) -> Self {
        self.easing(EasingFn::QuadraticIn)
    }

    pub fn quadratic_out(self) -> Self {
        self.easing(EasingFn::QuadraticOut)
    }

    pub fn quadratic_in_out(self) -> Self {
        self.easing(EasingFn::QuadraticInOut)
    }

    pub fn cubic_in(self) -> Self {
        self.easing(EasingFn::CubicIn)
    }

    pub fn cubic_out(self) -> Self {
        self.easing(EasingFn::CubicOut)
    }

    pub fn cubic_in_out(self) -> Self {
        self.easing(EasingFn::CubicInOut)
    }

    pub fn sin_in(self) -> Self {
        self.easing(EasingFn::SinIn)
    }

    pub fn sin_out(self) -> Self {
        self.easing(EasingFn::SinOut)
    }

    pub fn sin_in_out(self) -> Self {
        self.easing(EasingFn::SinInOut)
    }

    pub fn circular_in(self) -> Self {
        self.easing(EasingFn::CircularIn)
    }

    pub fn circular_out(self) -> Self {
        self.easing(EasingFn::CircularOut)
    }

    pub fn circular_in_out(self) -> Self {
        self.easing(EasingFn::CircularInOut)
    }

    pub fn exponential_in(self) -> Self {
        self.easing(EasingFn::ExponentialIn)
    }

    pub fn exponential_out(self) -> Self {
        self.easing(EasingFn::ExponentialOut)
    }

    pub fn exponential_in_out(self) -> Self {
        self.easing(EasingFn::ExponentialInOut)
    }

    pub fn back_in(self) -> Self {
        self.easing(EasingFn::BackIn)
    }

    pub fn back_out(self) -> Self {
        self.easing(EasingFn::BackOut)
    }

    pub fn back_in_out(self) -> Self {
        self.easing(EasingFn::BackInOut)
    }

    pub fn bounce_in(self) -> Self {
        self.easing(EasingFn::BounceIn)
    }

    pub fn bounce_out(self) -> Self {
        self.easing(EasingFn::BounceOut)
    }

    pub fn bounce_in_out(self) -> Self {
        self.easing(EasingFn::BounceInOut)
    }

    pub fn show(mut self, layered_image: impl LayeredImage) -> Self {
        let images = layered_image.images();
        self.last_images = Some(images.clone());
        self.last_transform = ImageTransformSchema::default();
        self.push_step(AnimationStep::Show(images));

        self
    }

    pub fn hide(mut self) -> Self {
        self.push_step(AnimationStep::Hide);

        self
    }

    /// Sets the animation mode to repeat.
    pub fn repeat(mut self) -> Self {
        self.get_animation_mut().mode = AnimationMode::Repeat;

        self
    }

    /// Makes the animation block user inputs.
    pub fn blocking(mut self) -> Self {
        self.get_animation_mut().mode = AnimationMode::Blocking;

        self
    }

    pub fn auto_advance(mut self) -> Self {
        self.get_animation_mut().auto_advance = true;

        self
    }

    pub fn and(self) -> Self {
        self.game_state.input_mode = GameStateInputMode::Continue;
        self
    }

    fn get_animation_mut(&mut self) -> &mut AnimationSchema {
        self.game_state
            .playable
            .animations
            .get_mut(self.animation_id)
            .expect("animation exists")
    }

    fn push_step(&mut self, step: AnimationStep) {
        let animation = self.get_animation_mut();
        animation.steps.push(step);
    }

    fn apply_transform_and_show(mut self, transform: ImageTransformSchema) -> Self {
        // Update last transform
        self.last_transform = self.last_transform.merge(&transform);

        // Apply transform to the last tween if exists
        if let Some(AnimationStep::Tween(tween)) = self.get_animation_mut().steps.last_mut() {
            tween.end_transform = tween.end_transform.merge(&transform);
            return self;
        }

        let images = self.last_images.as_mut().expect("images exist");
        images
            .iter_mut()
            .for_each(|image| image.transform = image.transform.merge(&transform));
        let images = images.clone();

        let animation = self.get_animation_mut();

        match animation.steps.last_mut() {
            Some(AnimationStep::Show(last_images)) => {
                // Replace the last Show
                *last_images = images;
            }
            _ => {
                animation.steps.push(AnimationStep::Show(images));
            }
        }

        self
    }
}

pub fn hide_animation(game_state: &mut GameState, animation_id: &'static str) {
    game_state.playable.animations.remove(animation_id);
    // Also remove the images
    game_state.playable.images.remove(animation_id);
}
