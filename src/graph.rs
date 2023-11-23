pub mod event;
pub mod plugin;

use std::collections::{HashMap, HashSet};

use bevy::{prelude::*, sprite::Mesh2dHandle};

#[derive(Component)]
pub enum Grabbable {
    Circle { radius: f32 },
}

impl Default for Grabbable {
    fn default() -> Self {
        Self::Circle { radius: 15.0 }
    }
}

#[derive(Clone, Component, Default)]
pub struct GNode;

#[derive(Bundle)]
struct GNodeBundle {
    node: GNode,
    sprite: SpriteBundle,
    grab: Grabbable,
}

#[derive(Clone, Component)]
pub struct GEdge {
    start: NodeE,
    end: NodeE,
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
    pub last_component_num: u32,

    pub show_labels: bool,
}

impl Graph {
    pub fn new(edge_mesh_handle: Mesh2dHandle, edge_mesh: Entity) -> Self {
        Graph {
            node_edges: HashMap::new(),
            edge_nodes: HashMap::new(),
            degree: 0,
            edge_mesh_handle,
            edge_mesh,
            directed: false,
            components: 0,
            last_node_num: 0,
            last_edge_num: 0,
            last_component_num: 0,
            show_labels: false,
        }
    }

    pub fn node_count(&self) -> usize {
        self.node_edges.len()
    }

    pub fn degree(&self) -> usize {
        self.degree
    }

    pub fn components(&self) -> u32 {
        self.components
    }

    fn add_node(&mut self, node: NodeE) {
        self.last_node_num += 1;
        self.components += 1;
        self.node_edges.insert(node, Vec::new());
    }

    fn remove_node(&mut self, node: &NodeE) {
        if self.node_edges.get(node).unwrap().is_empty() {
            self.components -= 1;
        }
        self.node_edges.remove(&node);
    }

    fn add_edge(&mut self, edge: EdgeE, start: &NodeE, end: &NodeE) {
        if !self.connected(start, end) {
            self.components -= 1;
        }

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

        if !self.connected(start, end) {
            self.components += 1;
        }
    }

    /// Determine whether two nodes are adjacent in the graph
    pub fn is_adjacent(&self, node: &NodeE, edge: &EdgeE) -> Option<NodeE> {
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

    /// Get the nodes adjacent to the given node
    pub fn adjacent_nodes(&self, node: &NodeE) -> HashSet<NodeE> {
        let mut nodes = HashSet::new();
        for edge in self.node_edges.get(node).unwrap() {
            if let Some(adj) = self.is_adjacent(node, edge) {
                nodes.insert(adj);
            }
        }

        nodes
    }

    /// Perform a depth-first search of the graph, starting at the given node
    /// and calling the predicate on each node and edge visited.
    pub fn spanning_tree<F>(&self, start: &NodeE, mut visit: F)
    where
        F: FnMut(&NodeE, Option<&EdgeE>) -> bool, // Node, Edge
    {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        stack.push(*start);
        visit(start, None);
        visited.insert(*start);

        while let Some(node) = stack.pop() {
            for edge in self.node_edges.get(&node).unwrap() {
                if let Some(adj) = self.is_adjacent(&node, edge) {
                    if !visited.contains(&adj) {
                        if visit(&adj, Some(edge)) {
                            return; // If visit returns true, return early
                        }
                        visited.insert(adj);
                        stack.push(adj);
                    }
                }
            }
        }
    }

    /// Determine whether two nodes are connected in the graph
    pub fn connected(&self, a: &NodeE, b: &NodeE) -> bool {
        let mut connected = false;
        self.spanning_tree(a, |node, _| {
            if node == b {
                connected = true;
                return true;
            }
            return false;
        });
        connected
    }

    /// Determine whether a given edge is a bridge in the graph
    pub fn is_bridge(&self, target_edge: &EdgeE) -> bool {
        let (start, end) = self.edge_nodes.get(target_edge).unwrap();

        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        stack.push(*start);
        visited.insert(*start);

        while let Some(node) = stack.pop() {
            for edge in self.node_edges.get(&node).unwrap() {
                if let Some(adj) = self.is_adjacent(&node, edge) {
                    if edge == target_edge {
                        continue; // Skip the target edge, we want to see if there are any connections other than through it
                    }
                    if adj == *end {
                        return false; // Found an alternate path, this is a link, not a bridge
                    }
                    if !visited.contains(&adj) {
                        visited.insert(adj);
                        stack.push(adj);
                    }
                }
            }
        }

        // No alternate path found, this is a bridge
        return true;
    }

    pub fn shortest_path(&self, start: &NodeE, end: &NodeE) -> Option<Vec<(NodeE, EdgeE)>> {
        None
    }
}
