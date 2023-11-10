pub mod event;
pub mod plugin;

use bevy::{prelude::*, render::{render_resource::PrimitiveTopology, mesh::Indices}, sprite::Mesh2dHandle};

use crate::materials;

use self::plugin::ImageCache;

#[derive(Component)]
pub struct Grabbable {
    pub radius: f32,
}

impl Default for Grabbable {
    fn default() -> Self {
        Self { radius: 15.0 }
    }
}

#[derive(Component)]
pub struct GNode {
    id: u64,
}

#[derive(Bundle)]
struct GNodeBundle {
    node: GNode,
    sprite: SpriteBundle,
    grab: Grabbable,
}

#[derive(Component)]
pub struct GEdge {
    start: u64,
    end: u64,
    start_node: Entity,
    end_node: Entity,
    weight: i32,
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

#[derive(Default)]
struct AdjacencyList {
    data: Vec<Vec<u64>>,
    degree: usize,
}

impl AdjacencyList {
    pub fn adjacenies(&self, node: u64) -> &[u64] {
        &self.data[node as usize]
    }

    pub fn is_adjacent_to(&self, a: u64, b: u64) -> bool {
        self.data[a as usize].contains(&b)
    }

    pub fn add_edge(&mut self, a: u64, b: u64) {
        self.data[a as usize].push(b);
        self.degree += 1;

        if a != b {
            self.data[b as usize].push(a);
            self.degree += 1;
        }
    }

    pub fn remove_edge(&mut self, a: u64, b: u64) {
        self.data[a as usize].retain(|&x| x != b);
        self.degree -= 1;

        if a != b {
            self.data[b as usize].retain(|&x| x != a);
            self.degree -= 1;
        }
    }

    pub fn insert_node(&mut self, node: u64) {
        let last_node = self.data.len() as u64 - 1;
        if node < last_node {
            for row in &mut self.data {
                for x in row {
                    if *x >= node {
                        *x += 1;
                    }
                }
            }
        }

        self.data.insert(node as usize, Vec::new());
    }

    pub fn append_node(&mut self) -> u64 {
        self.data.push(Vec::new());
        self.data.len() as u64 - 1
    }

    pub fn remove_node(&mut self, node: u64) {
        for row in &mut self.data {
            *row = row
                .iter()
                .filter_map(|x| {
                    if *x == node {
                        None
                    } else if *x > node {
                        Some(*x - 1)
                    } else {
                        Some(*x)
                    }
                })
                .collect();
        }

        self.data.remove(node as usize);
    }

    pub fn node_count(&self) -> usize {
        self.data.len()
    }

    pub fn degree(&self) -> usize {
        self.degree
    }
}

#[derive(Resource, Default)]
pub struct Graph {
    adjacencies: AdjacencyList,
    node_edges: Vec<Vec<Entity>>,
    edge_mesh_handle: Mesh2dHandle,
}

impl Graph {
    pub fn node_count(&self) -> usize {
        self.adjacencies.node_count()
    }

    pub fn degree(&self) -> usize {
        self.adjacencies.degree()
    }

    fn new_node(&mut self) -> u64 {
        self.node_edges.push(Vec::new());
        self.adjacencies.append_node()
    }

    fn remove_node(&mut self, id: u64) {
        self.node_edges.remove(id as usize);
        self.adjacencies.remove_node(id);
    }

    fn add_edge(&mut self, edge_e: Entity, start: u64, end: u64) {
        self.node_edges
            .get_mut(start as usize)
            .unwrap()
            .push(edge_e);
        self.node_edges.get_mut(end as usize).unwrap().push(edge_e);
        self.adjacencies.add_edge(start, end);
    }

    fn remove_edge(&mut self, edge_e: Entity, start: u64, end: u64) {
        for edge_set in &mut self.node_edges {
            edge_set.retain(|&x| x != edge_e);
        }
        self.adjacencies.remove_edge(start, end);
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
