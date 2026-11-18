use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::schema::image_schema::{ImageSchema, ImageTransformSchema};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct AnimationSchema {
    pub steps: Vec<AnimationStep>,
    pub mode: AnimationMode,
    pub auto_advance: bool,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AnimationMode {
    OneTime,
    Repeat,
    Blocking,
}

impl Default for AnimationSchema {
    fn default() -> Self {
        Self {
            steps: vec![],
            mode: AnimationMode::OneTime,
            auto_advance: false,
        }
    }
}

impl AnimationSchema {
    /// Returns the animations keys to remove and to reset.
    pub fn get_update_keys(
        old: &HashMap<String, AnimationSchema>,
        new: &HashMap<String, AnimationSchema>,
    ) -> (Vec<String>, Vec<String>) {
        // Old to remove
        let old_to_remove = old
            .iter()
            .filter(|(key, _)| !new.contains_key(*key))
            .map(|(key, _)| key.clone())
            .collect();

        // Updated to reset
        let updated_to_reset = new
            .iter()
            .filter(|(key, animation)| {
                old.get(*key)
                    .map(|old_animation| old_animation != *animation)
                    .unwrap_or(true)
            })
            .map(|(key, _)| key.clone())
            .collect();

        (old_to_remove, updated_to_reset)
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum AnimationStep {
    Show(Vec<ImageSchema>),
    Tween(AnimationTweenSchema),
    Wait(f32),
    Hide,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct AnimationTweenSchema {
    pub base_images: Vec<ImageSchema>,
    pub duration: f32, // in seconds
    pub start_transform: ImageTransformSchema,
    pub end_transform: ImageTransformSchema,
    /// Takes in the time elapsed / duration and returns a value between 0 and 1.
    pub easing: EasingFn,
}

impl Default for AnimationTweenSchema {
    fn default() -> Self {
        Self {
            base_images: Vec::new(),
            duration: 1.0,
            start_transform: ImageTransformSchema::default(),
            end_transform: ImageTransformSchema::default(),
            easing: EasingFn::Linear,
        }
    }
}

/// A function that takes in a value between 0 and 1 and returns a value between 0 and 1.
/// (Back curves can return values outside that range.)
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum EasingFn {
    Linear,
    QuadraticIn,
    QuadraticOut,
    QuadraticInOut,
    CubicIn,
    CubicOut,
    CubicInOut,
    SinIn,
    SinOut,
    SinInOut,
    CircularIn,
    CircularOut,
    CircularInOut,
    ExponentialIn,
    ExponentialOut,
    ExponentialInOut,
    BackIn,
    BackOut,
    BackInOut,
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl EasingFn {
    /// Takes in the time elapsed / duration.
    pub fn ease(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            EasingFn::Linear => emath::easing::linear(t),
            EasingFn::QuadraticIn => emath::easing::quadratic_in(t),
            EasingFn::QuadraticOut => emath::easing::quadratic_out(t),
            EasingFn::QuadraticInOut => emath::easing::quadratic_in_out(t),
            EasingFn::CubicIn => emath::easing::cubic_in(t),
            EasingFn::CubicOut => emath::easing::cubic_out(t),
            EasingFn::CubicInOut => emath::easing::cubic_in_out(t),
            EasingFn::SinIn => emath::easing::sin_in(t),
            EasingFn::SinOut => emath::easing::sin_out(t),
            EasingFn::SinInOut => emath::easing::sin_in_out(t),
            EasingFn::CircularIn => emath::easing::circular_in(t),
            EasingFn::CircularOut => emath::easing::circular_out(t),
            EasingFn::CircularInOut => emath::easing::circular_in_out(t),
            EasingFn::ExponentialIn => emath::easing::exponential_in(t),
            EasingFn::ExponentialOut => emath::easing::exponential_out(t),
            EasingFn::ExponentialInOut => emath::easing::exponential_in_out(t),
            EasingFn::BackIn => emath::easing::back_in(t),
            EasingFn::BackOut => emath::easing::back_out(t),
            EasingFn::BackInOut => emath::easing::back_in_out(t),
            EasingFn::BounceIn => emath::easing::bounce_in(t),
            EasingFn::BounceOut => emath::easing::bounce_out(t),
            EasingFn::BounceInOut => emath::easing::bounce_in_out(t),
        }
    }
}
