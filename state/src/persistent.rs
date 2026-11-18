use serde::{Deserialize, Serialize};

use crate::{schema::schema::SceneSchema, settings::Settings};

// Global state that is always read from and saved to disk
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Persistent {
    pub settings: Settings,

    pub global_max_scene: SceneSchema,
}

impl Default for Persistent {
    fn default() -> Self {
        Self {
            settings: Settings::default(),
            global_max_scene: SceneSchema::default(),
        }
    }
}
