pub enum WidgetSchema {
    /// A widget that displays the route options in order.
    RouteSelect { routes: Vec<RouteSelectSchema> },
}

pub struct RouteSelectSchema {
    pub scene_id: usize,
    pub description: String,
}
