use serde::{Deserialize, Serialize};

use crate::{game_state::GameState, schema::schema::SceneSchema};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct SaveSchema {
    pub name: String,
    pub timestamp_ms: u64,
    // Session schema
    pub total_playtime_sec: u64,
    pub max_scene_schema: SceneSchema,
    // Game state
    pub game_state: GameState,
}
