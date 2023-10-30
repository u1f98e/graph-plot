use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

#[derive(Component)]
struct Grabbable {
	radius: u32
}

impl Default for Grabbable {
    fn default() -> Self {
        Self { radius: 1 }
    }
}

#[derive(Bundle)]
struct NodeBundle {
    node: Node,
    grab: Grabbable,
    transform: Transform,
    sprite: Sprite,
    img: Handle<Image>,
}

#[derive(Component)]
struct Edge {
    id: usize,
    start: usize,
    end: usize,
    color: Color,
    weight: i32,
}

#[derive(Default, Bundle)]
struct EdgeHandle {
    grab: Grabbable,
	mesh: MaterialMesh2dBundle<ColorMaterial>
}

#[derive(Bundle)]
struct EdgeBundle {
    edge: Edge,

    handle: EdgeHandle,
}

#[derive(Component)]
struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl Graph {
    pub fn create_node(&mut self) {
        
    }
}