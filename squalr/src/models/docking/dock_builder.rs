use crate::models::docking::dock_split_direction::DockSplitDirection;
use crate::models::docking::docked_window_node::DockedWindowNode;

#[derive(Default)]
pub struct DockBuilder {
    id: String,
    direction: DockSplitDirection,
    ratio: f32,
    children: Vec<DockBuilder>,
}

impl DockBuilder {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            direction: DockSplitDirection::Horizontal,
            ratio: 1.0,
            children: Vec::new(),
        }
    }

    pub fn build(self) -> DockedWindowNode {
        DockedWindowNode {
            window_identifier: self.id,
            direction: self.direction,
            ratio: self.ratio,
            children: self.children.into_iter().map(|b| b.build()).collect(),
        }
    }

    pub fn direction(
        mut self,
        direction: DockSplitDirection,
    ) -> Self {
        self.direction = direction;
        self
    }

    pub fn split(
        mut self,
        ratio: f32,
        builder: DockBuilder,
    ) -> Self {
        if self.children.is_empty() {
            // If this is the first split, the builder becomes the first child
            self.children.push(builder.ratio(ratio));
            self.id = format!("{}_container", self.id);
        } else {
            // For subsequent splits, just add the new child with its ratio
            self.children.push(builder.ratio(ratio));
        }
        self
    }

    pub fn ratio(
        mut self,
        ratio: f32,
    ) -> Self {
        self.ratio = ratio;
        self
    }
}
