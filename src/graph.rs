pub mod event;
pub mod plugin;

use std::collections::HashMap;

use bevy::{prelude::*, sprite::Mesh2dHandle};

use self::plugin::ImageCache;

static EDGE_VERTEX_OFFSET: f32 = 5.0;

#[derive(Component)]
pub struct Grabbable {
    pub radius: f32,
}

impl Default for Grabbable {
    fn default() -> Self {
        Self { radius: 15.0 }
    }
}

enum GNodeSide {
    Start,
    End
}

#[derive(Component, Default)]
pub struct GNode {
    offsets: Vec<(usize, GNodeSide)>,
}

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
    offset: usize
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

#[derive(Resource, Default)]
pub struct Graph {
    adjacencies: HashMap<Entity, Vec<Entity>>, // <Node, Vec<Edge>>
    degree: usize,
    edge_mesh_handle: Mesh2dHandle,
}

impl Graph {
    pub fn node_count(&self) -> usize {
        self.adjacencies.len()
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    fn add_node(&mut self, node: Entity) {
        self.adjacencies.insert(node, Vec::new());
    }

    fn remove_node(&mut self, node: &Entity) {
        self.degree -= self.adjacencies[&node].len();
        self.adjacencies.remove(&node);
    }

    fn add_edge(&mut self, edge: Entity, start: &Entity, end: &Entity) {
        self.adjacencies.get_mut(start).unwrap().push(edge);
        if start != end {
            self.adjacencies.get_mut(end).unwrap().push(edge);
        }
        self.degree += 2;
    }

    fn remove_edge(&mut self, edge: Entity, start: &Entity, end: &Entity) {
        self.adjacencies.get_mut(end).unwrap().retain(|&x| x != edge);
        if start != end {
            self.adjacencies.get_mut(end).unwrap().retain(|&x| x != edge);
        }
        self.degree -= 2;
    }

    fn node_moved(&mut self, id: u64, new_pos: Vec3) {}

    fn edge_handle_moved(&mut self, edge: &GEdge, new_pos: Vec3) {}

    // pub fn update_edge_mesh_pos(&self, assets: &mut Assets<Mesh>) {
    //     let mut mesh = assets.get_mut(&self.node_mesh_handle);
    // }

    // pub fn update_node_mesh(&self, assets: &mut Assets<Mesh>) {
    //     let mut mesh = assets.get_mut(&self.node_mesh_handle);
    //     // TODO: Use attribute_mut to only modify specific verticies (probably doesn't matter)
    // }

    // pub fn update_edge_mesh() {}
}
