pub mod plugin;
pub mod event;

use std::collections::HashMap;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::Material2dPlugin, log,
};

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
struct GNode {
    id: u64
}

#[derive(Bundle)]
struct GNodeBundle {
    node: GNode,
    sprite: SpriteBundle,
    grab: Grabbable,
}

#[derive(Component)]
struct GEdge {
    start: u64,
    end: u64,
    color: Color,
    weight: i32,
}

#[derive(Default, Bundle)]
struct GEdgeHandle {
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
        self.data[b as usize].push(a);
    }

    pub fn remove_edge(&mut self, a: u64, b: u64) {
        self.data[a as usize].retain(|&x| x != b);
        self.data[b as usize].retain(|&x| x != a);
    }

    pub fn insert_node(&mut self, node: u64) {
        let last_node = self.data.len() as u64 - 1;
        if node < last_node {
            for row in &mut self.data {
                for x in row {
                    if *x >= node {
                        *x += 1;
                    }
                };
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
            *row = row.iter().filter_map(|x| {
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
}

#[derive(Resource, Default)]
pub struct Graph {
    adjacencies: AdjacencyList,
    edge_mesh_handle: Handle<Mesh>,
}

impl Graph {
    pub fn new(assets: &mut Assets<Mesh>) -> Self {
        let edge_mesh_handle = assets.add(Mesh::new(PrimitiveTopology::TriangleList));

        Self {
            edge_mesh_handle,
            ..Default::default()
        }
    }

    fn new_node(&mut self) -> u64 {
        self.adjacencies.append_node()
    }

    fn remove_node(&mut self, id: u64) {
        self.adjacencies.remove_node(id);
    }

    fn add_edge(&mut self, start: u64, end: u64) {
        self.adjacencies.add_edge(start, end);
    }

    fn remove_edge(&mut self, start: u64, end: u64) {
        self.adjacencies.remove_edge(start, end);
    }

    fn node_moved(&mut self, id: u64, new_pos: Vec3) {

    }

    fn edge_handle_moved(&mut self, edge: &GEdge, new_pos: Vec3) {

    }

    // pub fn regen_edge_mesh(&self, assets: &mut Assets<Mesh>) {
    //     let mut mesh = assets.get_mut(&self.edge_mesh_handle).unwrap();

    //     let positions: Vec<[f32; 3]> = self.edges.iter().flat_map(|edge| edge.vertices()).collect();

    //     let colors: Vec<[f32; 4]> = self.edges.iter().map(|edge| edge.color_vec()).collect();

    //     let tex_coords: Vec<[f32; 3]> = self
    //         .edges
    //         .iter()
    //         .flat_map(|edge| edge.tex_coords())
    //         .collect();

    //     let indices = self
    //         .edges
    //         .iter()
    //         .enumerate()
    //         .flat_map(|(idx, edge)| edge.indices(idx as u32 * 4))
    //         .collect();

    //     mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    //     mesh.set_indices(Some(Indices::U32(indices)));
    // }

    // pub fn update_edge_mesh_pos(&self, assets: &mut Assets<Mesh>) {
    //     let mut mesh = assets.get_mut(&self.node_mesh_handle);
    // }

    // pub fn update_node_mesh(&self, assets: &mut Assets<Mesh>) {
    //     let mut mesh = assets.get_mut(&self.node_mesh_handle);
    //     // TODO: Use attribute_mut to only modify specific verticies (probably doesn't matter)
    // }

    // pub fn update_edge_mesh() {}
}
