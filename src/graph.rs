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

    pub fn endpoints(&self) -> (NodeE, NodeE) {
        (self.start, self.end)
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

impl From<NodeE> for Entity {
    fn from(e: NodeE) -> Self {
        e.0
    }
}

#[derive(Deref, Eq, PartialEq, Hash, Clone, Copy)]
pub struct EdgeE(pub Entity);
impl From<Entity> for EdgeE {
    fn from(e: Entity) -> Self {
        Self(e)
    }
}

impl From<EdgeE> for Entity {
    fn from(e: EdgeE) -> Self {
        e.0
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
    pub do_physics: bool,
}

pub enum OppositeNode {
    Adjacent(NodeE),
    CounterAdjacent(NodeE),
    Loop,
    None
}

#[derive(Copy, Clone)]
pub struct PathPart {
    node: NodeE,
    edge: Option<EdgeE>
}

impl PathPart {
    pub fn with_edge(node: NodeE, edge: EdgeE) -> Self {
        PathPart { node, edge: Some(edge) }
    }

    pub fn without_edge(node: NodeE) -> Self {
        PathPart { node, edge: None }
    }
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
            do_physics: false,
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

    pub fn is_adjacent(&self, a: &NodeE, b: &NodeE) -> bool {
        self.node_edges.get(a).unwrap().iter().any(|&x| {
            let (start, end) = self.edge_nodes.get(&x).unwrap();
            start == b || end == b
        })
    }

    /// Get the node opposite the given node across the given edge (if the 
    /// direction of the edge moves to the opposite node, and the edge is not a loop)
    pub fn opposite(&self, node: &NodeE, edge: &EdgeE) -> OppositeNode {
        let (start, end) = match self.edge_nodes.get(edge) {
            Some(tuple) => tuple,
            None => return OppositeNode::None
        };

        if start == end {
            OppositeNode::Loop
        } else if node == start {
            OppositeNode::Adjacent(*end)
        } else {
            if self.directed {
                OppositeNode::CounterAdjacent(*start)
            }
            else {
                OppositeNode::Adjacent(*start)
            }
        }
    }

    /// Perform a depth-first search of the graph, starting at the given node
    /// and calling the predicate on each node and edge visited.
    pub fn spanning_tree<F>(&self, start: &NodeE, mut visit: F)
    where
        F: FnMut(&PathPart) -> bool,
    {
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        stack.push(*start);
        visit(&PathPart::without_edge(*start));
        visited.insert(*start);

        while let Some(node) = stack.pop() {
            for edge in self.node_edges.get(&node).unwrap() {
                if let OppositeNode::Adjacent(adj) = self.opposite(&node, edge) {
                    if !visited.contains(&adj) {
                        if visit(&PathPart::with_edge(adj, *edge)) {
                            return; // If visit returns true, return early
                        }
                        visited.insert(adj);
                        stack.push(adj);
                    }
                }
            }
        }
    }

    pub fn bipartite_walk<F>(&self, start: &NodeE, mut visit: F) -> bool
    where
        F: FnMut(&NodeE, Option<&EdgeE>, usize),
    {
        let mut sets = Vec::new();
        sets.push(HashSet::new());
        sets.push(HashSet::new());

        // Visit the start node and add it to set 0
        sets[0].insert(*start);
        visit(start, None, 0);
        let mut stack = Vec::new();
        stack.push((0, *start));

        while let Some((current_set, node)) = stack.pop() {
            let next_set = (current_set + 1) % sets.len();
            for edge in self.node_edges.get(&node).unwrap() {
                let adj = match self.opposite(&node, edge) {
                    OppositeNode::Adjacent(adj) => adj,
                    OppositeNode::CounterAdjacent(adj) => adj,
                    OppositeNode::Loop => return false, // No loops in bipartite graphs
                    OppositeNode::None => continue
                };
                if sets[current_set].contains(&adj) {
                    return false;
                }
                else {
                    visit(&adj, Some(edge), next_set);
                    if !sets[next_set].contains(&adj) {
                        sets[next_set].insert(adj);
                        stack.push((next_set, adj));
                    }
                }
            }
        }

        return true;
    }

    /// Determine whether two nodes are connected in the graph
    pub fn connected(&self, a: &NodeE, b: &NodeE) -> bool {
        let mut connected = false;
        self.spanning_tree(a, |part| {
            if part.node == *b {
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
                let adj = match self.opposite(&node, edge) {
                    OppositeNode::Adjacent(adj) => adj,
                    OppositeNode::CounterAdjacent(adj) => adj,
                    _ => continue, // No loops in bipartite graphs
                };
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

        // No alternate path found, this is a bridge
        return true;
    }

    pub fn dijkstra_path(&self, start: &NodeE, end: &NodeE, q_edge: Option<&Query<&GEdge>>) -> Option<Vec<PathPart>> {
        let mut unvisited = HashSet::new();
        let mut distances = HashMap::new();
        let mut previous: HashMap<NodeE, PathPart> = HashMap::new();

        for node in self.node_edges.keys() {
            distances.insert(*node, std::f32::INFINITY);
            unvisited.insert(*node);
        }
        distances.insert(*start, 0.0);

        while !unvisited.is_empty() {
            let n = match unvisited.iter().min_by(|a, b| {
                distances[a].partial_cmp(&distances[b]).unwrap()
            }) {
                Some(node) => if node == end {
                    break
                }
                else {
                    *node
                },
                None => break
            };

            unvisited.remove(&n);

            // For each edge adjacent to n that is still unvisited
            for edge_e in self.node_edges.get(&n).unwrap().iter() {
                let adj = match self.opposite(&n, &edge_e) {
                    OppositeNode::Adjacent(adj) => {
                        if unvisited.contains(&adj) {
                            adj
                        }
                        else {
                            continue;
                        }
                    },
                    _ => continue
                };

                let weight = match q_edge {
                    Some(q) => q.get(**edge_e).unwrap().weight,
                    None => 1
                };

                let dist = distances[&n] + weight as f32;
                if dist < distances[&adj] {
                    distances.insert(adj, dist);
                    previous.insert(adj, PathPart::with_edge(n, *edge_e));
                }
            }
        }

        if previous.get(end).is_some() || start == end {
            let mut path = Vec::new();
            let mut cursor = Some(PathPart::without_edge(*end));
            while let Some(part) = cursor {
                cursor = previous.get(&part.node).map(|part| *part);
                path.push(part);
            }

            path.reverse();
            Some(path)
        }
        else {
            None // No path to the end node from start
        }
    }

    pub fn adjacency_matrix(&self, q_node: &Query<(Entity, &Children), With<GNode>>, q_text: &Query<&Text>) -> LabeledMatrix {
        let mut matrix = LabeledMatrix::default();
        matrix.data = vec![vec![0.0; self.node_edges.len()]; self.node_edges.len()];

        let mut labels = Vec::new();
        q_node.iter().for_each(|(node_e, children)| {
            for child in children.iter() {
                q_text.get(*child).ok().map(|text| {
                    labels.push((node_e, text.sections[0].value.clone()));
                });
            }
        });
        labels.sort_by(|(_, a), (_, b)| a.cmp(b));

        let mut indicies = HashMap::new();
        for (i, (node, _)) in labels.iter().enumerate() {
            indicies.insert(*node, i);
            matrix.v_headers.push(labels[i].1.clone());
            matrix.h_headers.push(labels[i].1.clone());
        }

        for (node, edges) in self.node_edges.iter() {
            for edge in edges.iter() {
                let adj = match self.opposite(node, edge) {
                    OppositeNode::Adjacent(adj) => adj,
                    OppositeNode::CounterAdjacent(_) => continue,
                    OppositeNode::Loop => *node,
                    _ => continue,
                };
                matrix.data[indicies[node]][indicies[&adj]] = 1.0;
            }
        }

        matrix
    }
}

#[derive(Default)]
pub struct LabeledMatrix {
    pub data: Vec<Vec<f32>>,
    pub h_headers: Vec<String>,
    pub v_headers: Vec<String>,
}

impl LabeledMatrix {
}