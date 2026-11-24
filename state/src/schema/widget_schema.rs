use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum WidgetSchema {
    /// A widget that displays the route options in order.
    RouteSelect { routes: Vec<RouteSelectSchema> },
    /// Displays multiple weighted choices, with a max weight.
    MultipleChoice {
        choices: Vec<MultipleChoiceSchema>,
        max_weight: usize,
        points_threshold: usize,
        pass_scene_id: usize,
        fail_scene_id: usize,
    },
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RouteSelectSchema {
    pub scene_id: usize,
    pub description: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct MultipleChoiceSchema {
    pub description: String,
    pub weight: usize,
    pub points: usize,
    pub selected: bool,
}

impl MultipleChoiceSchema {
    pub fn new(description: &str, weight: usize, points: usize) -> Self {
        MultipleChoiceSchema {
            description: description.to_string(),
            weight,
            points,
            selected: false,
        }
    }
}
