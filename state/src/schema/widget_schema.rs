use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum WidgetSchema {
    /// A widget that displays the route options in order.
    RouteSelect { routes: Vec<RouteSelectSchema> },
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RouteSelectSchema {
    pub scene_id: usize,
    pub description: String,
}
