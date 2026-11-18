use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::schema::{
    animation_schema::AnimationSchema, audio_schema::ChannelsSchema, image_schema::ImageSchema,
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct PlayableSchema {
    pub images: HashMap<String, Vec<ImageSchema>>,
    pub animations: HashMap<String, AnimationSchema>,
    pub audio: ChannelsSchema,
}

impl Default for PlayableSchema {
    fn default() -> Self {
        Self {
            images: HashMap::new(),
            animations: HashMap::new(),
            audio: ChannelsSchema::default(),
        }
    }
}

impl PlayableSchema {
    pub fn iter_images(&self) -> impl Iterator<Item = &Vec<ImageSchema>> {
        self.images.values()
    }

    pub fn iter_animations(&self) -> impl Iterator<Item = (&String, &AnimationSchema)> {
        self.animations.iter()
    }
}
