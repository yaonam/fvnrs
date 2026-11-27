use state::{
    game_state::{GameState, GameStateInputMode},
    schema::{
        animation_schema::AnimationSchema,
        image_schema::{
            ImageSchema, ImageTransformSchema, opacity, pivot, position, rotation, scale, scale_xy,
            z_index,
        },
    },
};

use crate::animation::{AnimationBuilder, hide_animation};

const BACKGROUND_ID: &str = "background";

/// A wrapper trait that helps group and manage images.
/// Allows for various permutations of a set images and shared transforms.
pub trait LayeredImage {
    const ID: &'static str;

    fn images(&self) -> Vec<ImageSchema>;

    fn show(self, game_state: &mut GameState) -> ShowBuilder<'_>
    where
        Self: Sized,
    {
        game_state
            .playable
            .images
            .insert(Self::ID.to_string(), self.images());
        ShowBuilder {
            layered_image_id: Self::ID,
            game_state,
            transform: ImageTransformSchema::default(),
        }
    }

    fn hide(game_state: &mut GameState) {
        game_state.playable.images.remove(Self::ID);
    }
}

/// An ephemeral builder that allows for chaining methods to apply transforms to a layered image.
pub struct ShowBuilder<'a> {
    pub layered_image_id: &'static str,
    pub game_state: &'a mut GameState,
    pub transform: ImageTransformSchema,
}

impl<'a> ShowBuilder<'a> {
    pub fn animate(self) -> AnimationBuilder<'a> {
        let layered_image_id = self.layered_image_id;
        let last_images = self
            .game_state
            .playable
            .images
            .get(layered_image_id)
            .cloned();
        self.game_state
            .playable
            .animations
            .insert(layered_image_id.to_string(), AnimationSchema::default());
        AnimationBuilder {
            animation_id: layered_image_id,
            game_state: self.game_state,
            last_images,
            last_transform: self.transform,
        }
    }

    pub fn animate_with(
        self,
        predefined_animation: impl Fn(AnimationBuilder<'a>) -> AnimationBuilder<'a>,
    ) -> AnimationBuilder<'a> {
        predefined_animation(self.animate())
    }

    /// Consumes the builder to set the input mode
    pub fn and(self) -> Self {
        self.game_state.input_mode = GameStateInputMode::Continue;
        self
    }

    pub fn position(self, pos: [f32; 2]) -> Self {
        self.apply_transform(position(pos))
    }

    pub fn scale(self, scale_factor: f32) -> Self {
        self.apply_transform(scale(scale_factor))
    }

    pub fn scale_xy(self, scale_factor: [f32; 2]) -> Self {
        self.apply_transform(scale_xy(scale_factor))
    }

    pub fn flip_x(self) -> Self {
        self.scale_xy([-1.0, 1.0])
    }

    pub fn rotation(self, radians: f32) -> Self {
        self.apply_transform(rotation(radians))
    }

    pub fn pivot(self, pivot_point: [f32; 2]) -> Self {
        self.apply_transform(pivot(pivot_point))
    }

    pub fn opacity(self, opacity_factor: f32) -> Self {
        self.apply_transform(opacity(opacity_factor))
    }

    pub fn z_index(self, z: i32) -> Self {
        self.apply_transform(z_index(z))
    }

    fn apply_transform(self, transform: ImageTransformSchema) -> Self {
        if let Some(images) = self
            .game_state
            .playable
            .images
            .get_mut(self.layered_image_id)
        {
            images.into_iter().for_each(|image| {
                image.transform = image.transform.merge(&transform);
                println!("Merged transform: {:?}", image.transform);
            });
        }

        self
    }
}

// TODO: Refactor with showable generic?
pub fn show<L: LayeredImage>(game_state: &mut GameState, layered_image: L) -> ShowBuilder<'_> {
    game_state
        .playable
        .images
        .insert(L::ID.to_string(), layered_image.images());
    ShowBuilder {
        layered_image_id: L::ID,
        game_state,
        transform: ImageTransformSchema::default(),
    }
}

/// Immediately hides the layered image.
/// (For animation purposes, use `show().animate().hide()` instead.)
pub fn hide<'a>(game_state: &'a mut GameState, layered_image_id: &'static str) -> ShowBuilder<'a> {
    game_state.playable.images.remove(layered_image_id);
    ShowBuilder {
        layered_image_id,
        game_state,
        transform: ImageTransformSchema::default(),
    }
}

pub fn show_bg<L: LayeredImage>(game_state: &mut GameState, layered_image: L) -> ShowBuilder<'_> {
    game_state
        .playable
        .images
        .insert(BACKGROUND_ID.to_string(), layered_image.images());
    ShowBuilder {
        layered_image_id: BACKGROUND_ID,
        game_state,
        transform: ImageTransformSchema::default(),
    }
    .z_index(-1)
}

/// Also hides any bg animation
pub fn hide_bg(game_state: &mut GameState) -> ShowBuilder<'_> {
    game_state.playable.images.remove(BACKGROUND_ID);
    hide_animation(game_state, BACKGROUND_ID);
    ShowBuilder {
        layered_image_id: BACKGROUND_ID,
        game_state,
        transform: ImageTransformSchema::default(),
    }
    .z_index(-1)
}
