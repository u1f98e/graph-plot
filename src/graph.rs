pub mod event;
pub mod plugin;

use std::collections::HashMap;

use bevy::{prelude::*, sprite::Mesh2dHandle};

use self::plugin::ImageCache;

static EDGE_VERTEX_OFFSET: f32 = 5.0;

#[derive(Component)]
pub enum Grabbable {
    Circle { radius: f32 },
    Rect { width: f32, height: f32 },
}

impl Default for Grabbable {
    fn default() -> Self {
        Self::Circle { radius: 15.0 }
    }
}

impl Grabbable {
    pub fn new_circle(radius: f32) -> Self {
        Self::Circle { radius }
    }

    pub fn new_rect(width: f32, height: f32) -> Self {
        Self::Rect { width, height }
    }

    pub fn contains(&self, pos: Vec3, point: Vec3) -> bool {
        match *self {
            Grabbable::Circle { radius } => pos.distance(point) <= radius,
            Grabbable::Rect { width, height } => {
                let width_2 = width / 2.0;
                let min_x = pos.x - width_2;
                let max_x = pos.x + width_2;
                let height_2 = width / 2.0;
                let min_y = pos.x - height_2;
                let max_y = pos.x + height_2;

                point.x >= min_x && point.x <= max_x && point.y >= min_y && point.y <= max_y
            }
        }
    }
}

#[derive(Component)]
pub struct NodeLabel;

#[derive(Bundle)]
pub struct NodeLabelBundle {
    pub label: NodeLabel,
    pub grab: Grabbable,
    pub text: Text2dBundle,
}

// impl Default for NodeLabelBundle {
//     fn default() -> Self {
//         Self {
//             label: NodeLabel(),
//             grab: Grabbable::,
//         }
//     }
// }

enum GNodeSide {
    Start,
    End,
    Loop,
}

#[derive(Component, Default)]
pub struct GNode;

#[derive(Bundle)]
struct GNodeBundle {
    node: GNode,
    sprite: SpriteBundle,
    grab: Grabbable,
}

#[derive(Component)]
pub struct GEdge {
    start: Entity,
    end: Entity,
    weight: i32,
    offset: Option<usize>,
}

impl GEdge {
    pub fn is_loop(&self) -> bool {
        self.start == self.end
    }

    pub fn size_in_mesh(&self) -> usize {
        if self.is_loop() {
            4
        } else {
            3
        }
    }
}

#[derive(Default, Bundle)]
pub struct GEdgeHandle {
    grab: Grabbable,
    sprite: SpriteBundle,
}

#[derive(Bundle)]
struct GEdgeBundle {
    edge: GEdge,
    handle: GEdgeHandle,
}

#[derive(Resource)]
pub struct Graph {
    pub adjacencies: HashMap<Entity, Vec<Entity>>, // <Node, Vec<Edge>>
    pub degree: usize,
    pub edge_mesh_handle: Mesh2dHandle,
    pub edge_mesh: Entity,

    pub directed: bool,
    pub components: u32,
    pub last_node_num: u32,
    pub last_edge_num: u32,

    pub show_labels: bool
}

impl Graph {
    pub fn node_count(&self) -> usize {
        self.adjacencies.len()
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    fn add_node(&mut self, node: Entity) {
        self.last_node_num += 1;
        self.adjacencies.insert(node, Vec::new());
    }

    fn remove_node(&mut self, node: &Entity) {
        self.adjacencies.remove(&node);
    }

    fn add_edge(&mut self, edge: Entity, start: &Entity, end: &Entity) {
        self.last_edge_num += 1;
        self.adjacencies.get_mut(start).unwrap().push(edge);
        if start != end {
            self.adjacencies.get_mut(end).unwrap().push(edge);
        }
        self.degree += 2;
    }

    fn remove_edge(&mut self, edge: Entity, start: &Entity, end: &Entity) {
        if let Some(a) = self.adjacencies.get_mut(start) {
            a.retain(|&x| x != edge);
        }
        if start != end {
            if let Some(a) = self.adjacencies.get_mut(end) {
                a.retain(|&x| x != edge);
            }
        }
        self.degree -= 2;
    }
}
