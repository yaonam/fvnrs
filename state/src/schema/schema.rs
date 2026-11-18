use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct SceneSchema {
    /// serde serialized scene name (enum)
    pub current_scene_id: usize,
    /// Index of the current action in the scene
    pub index: usize,
}

impl SceneSchema {
    pub fn new(current_scene_id: usize) -> Self {
        Self {
            current_scene_id,
            index: 0,
        }
    }

    pub fn update_if_bigger(&mut self, other: &Self) -> bool {
        if other.current_scene_id > self.current_scene_id {
            self.current_scene_id = other.current_scene_id;
            self.index = other.index;
            true
        } else if other.current_scene_id == self.current_scene_id && other.index > self.index {
            self.index = other.index;
            true
        } else {
            false
        }
    }

    pub fn is_later_than(&self, other: &Self) -> bool {
        self.current_scene_id > other.current_scene_id
            || (self.current_scene_id == other.current_scene_id && self.index > other.index)
    }
}
