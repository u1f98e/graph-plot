pub mod event;
pub mod plugin;

use std::collections::HashMap;

use bevy::{prelude::*, sprite::Mesh2dHandle};

use self::plugin::ImageCache;

static EDGE_VERTEX_OFFSET: f32 = 5.0;

#[derive(Component)]
pub enum Grabbable {
    Circle { radius: f32 },
}

impl Default for Grabbable {
    fn default() -> Self {
        Self::Circle { radius: 15.0 }
    }
}

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
    offset: Option<usize>,
    pub weight: i32,
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
