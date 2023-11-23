pub mod event;
pub mod plugin;

use std::collections::{HashMap, HashSet};

use bevy::{core_pipeline::core_2d::graph::node, prelude::*, sprite::Mesh2dHandle};

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

#[derive(Deref, Eq, PartialEq, Hash, Clone, Copy)]
pub struct NodeE(pub Entity);
impl From<Entity> for NodeE {
    fn from(e: Entity) -> Self {
        Self(e)
    }
}

#[derive(Deref, Eq, PartialEq, Hash, Clone, Copy)]
pub struct EdgeE(pub Entity);
impl From<Entity> for EdgeE {
    fn from(e: Entity) -> Self {
        Self(e)
    }
}

#[derive(Resource)]
pub struct Graph {
    pub node_edges: HashMap<NodeE, Vec<EdgeE>>, // <Node, Vec<Edge>>
    pub edge_nodes: HashMap<EdgeE, (NodeE, NodeE)>, // <Edge, (Start, End)>
    pub degree: usize,
    pub edge_mesh_handle: Mesh2dHandle,
    pub edge_mesh: Entity,

    pub directed: bool,
    pub components: u32,
    pub last_node_num: u32,
    pub last_edge_num: u32,

    pub show_labels: bool,
}

impl Graph {
    pub fn node_count(&self) -> usize {
        self.node_edges.len()
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    fn add_node(&mut self, node: NodeE) {
        self.last_node_num += 1;
        self.components += 1;
        self.node_edges.insert(node, Vec::new());
    }

    fn remove_node(&mut self, node: &NodeE) {
        self.node_edges.remove(&node);
    }

    fn add_edge(&mut self, edge: EdgeE, start: &NodeE, end: &NodeE) {
        self.last_edge_num += 1;
        self.node_edges.get_mut(start).unwrap().push(edge);
        if start != end {
            self.node_edges.get_mut(end).unwrap().push(edge);
        }

        self.edge_nodes.insert(edge, (*start, *end));
        self.degree += 2;
    }

    fn remove_edge(&mut self, edge: EdgeE, start: &NodeE, end: &NodeE) {
        if let Some(a) = self.node_edges.get_mut(start) {
            a.retain(|&x| x != edge);
        }
        if start != end {
            if let Some(a) = self.node_edges.get_mut(end) {
                a.retain(|&x| x != edge);
            }
        }
        self.edge_nodes.remove(&edge);
        self.degree -= 2;
    }

    pub fn adjacent_node(&self, node: &NodeE, edge: &EdgeE) -> Option<NodeE> {
        let (start, end) = self.edge_nodes.get(edge)?;
        if start == end {
            None
        } else if node == start {
            Some(*end)
        } else if !self.directed {
            Some(*start)
        } else {
            None
        }
    }

    pub fn adjacent_nodes(&self, node: &NodeE) -> HashSet<NodeE> {
        let mut nodes = HashSet::new();
        for edge in self.node_edges.get(node).unwrap() {
            if let Some(adj) = self.adjacent_node(node, edge) {
                nodes.insert(adj);
            }
        }

        nodes
    }

    pub fn spanning_tree<F>(&self, start: &NodeE, mut visit: F)
    where
        F: FnMut(&NodeE, Option<&EdgeE>), // Node, Edge
    {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        stack.push(*start);
        visit(start, None);
        visited.insert(*start);

        while let Some(node) = stack.pop() {
            for edge in self.node_edges.get(&node).unwrap() {
                if let Some(adj) = self.adjacent_node(&node, edge) {
                    if !visited.contains(&adj) {
                        visit(&adj, Some(edge));
                        visited.insert(adj);
                        stack.push(adj);
                    }
                }
            }
        }
    }

    pub fn shortest_path(&self, start: &NodeE, end: &NodeE) -> Option<Vec<(NodeE, EdgeE)>> {
        None
    }
}
