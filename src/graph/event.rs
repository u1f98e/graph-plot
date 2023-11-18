use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};

use crate::input::CursorInfo;
use crate::types::*;

use super::*;

#[derive(Event)]
pub struct AddNodeEvent(pub Vec2);

#[derive(Event)]
pub struct RemoveItemEvent(pub Entity);

#[derive(Event)]
pub struct AddEdgeEvent(pub Entity, pub Entity);

#[derive(Event)]
pub struct ItemMovedEvent(pub Entity, pub Vec3);

#[derive(Event)]
pub(crate) struct RegenEdgeMesh();

#[derive(Event)]
pub enum ItemSelectedEvent {
    Selected(Entity),
    Deselected,
}

pub(crate) fn item_selected_event(
    mut events: EventReader<ItemSelectedEvent>,
    mut cursor: ResMut<CursorInfo>,
    mut q_node: Query<&mut Handle<Image>, GNodeExclusive>,
    mut q_edge: Query<&mut Handle<Image>, GEdgeExclusive>,
    cache: Res<ImageCache>,
) {
    for event in events.iter() {
        match event {
            ItemSelectedEvent::Selected(entity) => {
                if let Ok(mut texture) = q_node.get_mut(*entity) {
                    *texture = cache.get("nodeSelected").unwrap().clone();
                } else if let Ok(mut texture) = q_edge.get_mut(*entity) {
                    *texture = cache.get("edgeSelected").unwrap().clone();
                }
                cursor.selected = Some(*entity);
            }
            ItemSelectedEvent::Deselected => {
                if let Some(entity) = cursor.selected {
                    if let Ok(mut texture) = q_node.get_mut(entity) {
                        *texture = cache.get("node").unwrap().clone();
                    } else if let Ok(mut texture) = q_edge.get_mut(entity) {
                        *texture = cache.get("edge").unwrap().clone();
                    }
                }
                cursor.selected = None;
            }
        }
    }
}

pub(super) fn add_node_event(
    mut events: EventReader<AddNodeEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    img_cache: Res<ImageCache>,
) {
    for AddNodeEvent(pos) in events.iter() {
        let transform = Transform::default().with_translation(Vec3::new(pos.x, pos.y, 0.0));
        let node = commands
            .spawn(GNodeBundle {
                node: GNode::default(),
                sprite: SpriteBundle {
                    texture: img_cache.get("node").unwrap().clone(),
                    transform,
                    ..Default::default()
                },
                grab: Grabbable::default(),
            })
            .id();

        graph.add_node(node);
        regen_ev.send(RegenEdgeMesh());
    }
}

pub(super) fn add_edge_event(
    mut events: EventReader<AddEdgeEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    q_nodes: Query<(Entity, &Transform), With<GNode>>,
    cache: Res<ImageCache>,
) {
    for AddEdgeEvent(a, b) in events.iter() {
        let (start, start_t) = q_nodes.get(*a).unwrap();
        let (end, end_t) = q_nodes.get(*b).unwrap();

        let transform = if start == end {
            // This edge is a loop
            Transform::default()
                .with_translation(start_t.translation + Vec3::new(0.0, 50.0, 0.0))
                .with_scale(Vec3::splat(0.5))
        } else {
            // Place the handle between the two nodes
            let midpoint = start_t.translation.lerp(end_t.translation, 0.5);
            let diff = end_t.translation - start_t.translation;
            let offset = (Vec3::new(diff.y, -diff.x, 0.0).normalize() * diff.length() * 0.3)
                .clamp_length(5.0, 25.0);
            let sign = if start_t.translation.y < end_t.translation.y {
                1.0
            } else {
                -1.0
            };
            Transform::default()
                .with_translation(midpoint + offset)
                .with_rotation(Quat::from_axis_angle(
                    Vec3::Z,
                    (end_t.translation - start_t.translation).angle_between(Vec3::X) * sign,
                ))
                .with_scale(Vec3::splat(0.5))
        };

        let texture = if graph.directed {
            cache.get("handle_directed").unwrap().clone()
        } else {
            cache.get("handle").unwrap().clone()
        };

        let edge = commands
            .spawn(GEdgeBundle {
                edge: GEdge {
                    start: *a,
                    end: *b,
                    weight: 1,
                    offset: None,
                },
                handle: GEdgeHandle {
                    grab: Grabbable::default(),
                    sprite: SpriteBundle {
                        texture,
                        transform,
                        ..Default::default()
                    },
                },
            })
            .id();

        graph.add_edge(edge, &start, &end);
        regen_ev.send(RegenEdgeMesh());
    }
}

pub(super) fn remove_item_event(
    mut events: EventReader<RemoveItemEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    q_nodes: Query<&mut GNode>,
    mut q_edges: Query<(Entity, &mut GEdge)>,
) {
    let mut removed_edges = Vec::new();
    for RemoveItemEvent(entity) in events.iter() {
        if let Ok(_) = q_nodes.get(*entity) {
            graph.remove_node(&entity);

            for (edge_e, edge) in q_edges.iter() {
                if edge.start == *entity || edge.end == *entity {
                    removed_edges.push(edge_e);
                }
            }
        } else if let Ok((edge_e, _)) = q_edges.get(*entity) {
            removed_edges.push(edge_e);
        }

        for edge_e in removed_edges.iter() {
            let offset_removed: Option<usize> = if let Ok((_, edge)) = q_edges.get(*edge_e) {
                graph.remove_edge(*edge_e, &edge.start, &edge.end);
                edge.offset
            } else {
                println!("Tried to remove an edge that doesn't exist");
                None
            };

            if let Some(offset_removed) = offset_removed {
                for (_, mut edge) in q_edges.iter_mut() {
                    if edge.offset.is_some() && edge.offset.unwrap() > offset_removed {
                        let removed_size = edge.size_in_mesh();
                        edge.offset = edge.offset.map(|o| o - removed_size);
                    }
                }
            }
            commands.entity(*edge_e).despawn();
        }
        removed_edges.clear();

        commands.entity(*entity).despawn();
        regen_ev.send(RegenEdgeMesh());
    }
}

// fn edge_vertices(start_pos: Vec3, handle_pos: Vec3, end_pos: Vec3) -> [[f32; 3]; 4] {
//     let start = Vec3::from_array(positions[offset - 1]);
//     let end = Vec3::from_array(positions[offset + 1]);
//     let start_end_mid = start.lerp(end, 0.5);
//     let pos = 2.0 * edge_t.translation - start_end_mid;
//     positions[offset] = pos.to_array();
// }

fn loop_vertices(node_pos: Vec3, handle_pos: Vec3) -> [[f32; 3]; 4] {
    let midpoint = node_pos.lerp(handle_pos, 0.5);
    let start_handle = node_pos - handle_pos;
    let orthagonal = Vec3::new(start_handle.y, -start_handle.x, 0.0).normalize();
    let lh = midpoint + orthagonal * 50.0;
    let rh = midpoint - orthagonal * 50.0;

    [
        [node_pos.x, node_pos.y, 0.0],
        [lh.x, lh.y, 0.0],
        [handle_pos.x, handle_pos.y, 0.0],
        [rh.x, rh.y, 0.0],
    ]
}

pub(super) fn move_item_event(
    mut events: EventReader<ItemMovedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    graph: Res<Graph>,
    q_nodes: Query<(Entity, &Transform), GNodeExclusive>,
    mut q_edges: Query<(&GEdge, &mut Transform), GEdgeExclusive>,
) {
    let mesh: &mut Mesh = meshes.get_mut(&graph.edge_mesh_handle.0).unwrap();
    let positions = match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(positions) => positions,
        _ => panic!("Wrong type for position attribute"),
    };

    for ItemMovedEvent(entity, delta) in events.iter() {
        if let Ok((node_e, node_t)) = q_nodes.get(*entity) {
            for edge_e in graph.adjacencies.get(entity).unwrap() {
                let (edge, mut edge_t) = q_edges.get_mut(*edge_e).unwrap();
                if let Some(offset) = edge.offset {
                    if edge.is_loop() {
                        edge_t.translation += *delta;

                        positions[(offset)..(offset + 4)].clone_from_slice(&loop_vertices(
                            node_t.translation,
                            edge_t.translation,
                        ));

                        continue;
                    } else {
                        edge_t.translation += *delta / 2.0;

                        if edge.start == node_e {
                            positions[offset] = node_t.translation.to_array();
                        } else {
                            positions[offset + 2] = node_t.translation.to_array();
                        }
                        let start = Vec3::from_array(positions[offset]);
                        let end = Vec3::from_array(positions[offset + 2]);
                        let start_end_mid = start.lerp(end, 0.5);
                        let pos = 2.0 * edge_t.translation - start_end_mid;
                        positions[offset + 1] = pos.to_array();
                    }
                }

                // Rotate the edge to point towards the other node
                if node_e == edge.start {
                    // The node being moved is the start node for this edge
                    let (_, end_node_t) = q_nodes.get(edge.end).unwrap();
                    let sign = if node_t.translation.y < end_node_t.translation.y {
                        1.0
                    } else {
                        -1.0
                    };
                    edge_t.rotation = Quat::from_rotation_z(
                        (end_node_t.translation - node_t.translation).angle_between(Vec3::X) * sign,
                    );
                } else {
                    // The node being moved is the end node for this edge
                    let (_, start_node_t) = q_nodes.get(edge.start).unwrap();
                    let sign = if node_t.translation.y < start_node_t.translation.y {
                        -1.0
                    } else {
                        1.0
                    };
                    edge_t.rotation = Quat::from_rotation_z(
                        (node_t.translation - start_node_t.translation).angle_between(Vec3::X)
                            * sign,
                    );
                }
            }
        } else if let Ok((edge, edge_t)) = q_edges.get(*entity) {
            if let Some(offset) = edge.offset {
                // Normal edge
                if edge.is_loop() {
                    let start = Vec3::from_array(positions[offset]);
                    positions[(offset)..(offset + 4)]
                        .clone_from_slice(&loop_vertices(start, edge_t.translation));
                } else {
                    let start = Vec3::from_array(positions[offset]);
                    let end = Vec3::from_array(positions[offset + 2]);
                    let start_end_mid = start.lerp(end, 0.5);
                    let pos = 2.0 * edge_t.translation - start_end_mid;
                    positions[offset + 1] = pos.to_array();
                }
            }
        }
    }
}

pub(super) fn regen_edge_mesh(
    mut events: EventReader<RegenEdgeMesh>,
    graph: Res<Graph>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut q_node: Query<(Entity, &mut GNode, &Transform, &Sprite), With<GNode>>,
    mut q_edge: Query<(&mut GEdge, &Transform, &Sprite)>,
) {
    if let Some(_) = events.iter().last() {
        let mesh: &mut Mesh = meshes.get_mut(&graph.edge_mesh_handle.0).unwrap();

        // There needs to be some initial values here or the mesh gets optimized
        // out
        let mut positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let mut tex_coords: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 1.0], [0.5, 0.0]];
        let mut colors: Vec<[f32; 4]> = vec![
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0, 1.0],
        ];
        let mut indices: Vec<u32> = vec![0, 1, 2];
        let mut index = 3;

        for (mut edge, edge_t, handle_sprite) in q_edge.iter_mut() {
            let (_, _, start_t, start_sprite) = q_node.get(edge.start).unwrap();
            let (_, _, end_t, end_sprite) = q_node.get(edge.end).unwrap();
            let start = start_t.translation;
            let handle = edge_t.translation;
            let end = end_t.translation;

            if edge.is_loop() {
                edge.offset = Some(positions.len());
                positions.extend_from_slice(&loop_vertices(start, handle));
                tex_coords.extend_from_slice(&[[0.0, 0.0], [0.5, 0.0], [1.0, 1.0], [0.5, 0.0]]);

                let start_color: Vec4 = start_sprite.color.into();
                let handle_color: Vec4 = handle_sprite.color.into();
                let midpoint_color = start_color.lerp(handle_color, 0.5);

                colors.extend_from_slice(&[
                    start_color.into(),
                    midpoint_color.into(),
                    handle_color.into(),
                    midpoint_color.into(),
                ]);

                let i = edge.offset.unwrap() as u32;
                indices.extend_from_slice(&[i, i + 1, i + 2, i + 2, i + 3, i]);
            }
            // Regular edge
            else {
                let start_end_mid = start.lerp(end, 0.5);
                // Project the handle's translation out 2x further away from the
                // midpoint between the start and end nodes
                let handle = 2.0 * handle - start_end_mid;

                edge.offset = Some(positions.len());
                positions.extend_from_slice(&[
                    [start.x, start.y, 0.0],
                    [handle.x, handle.y, 0.0],
                    [end.x, end.y, 0.0],
                ]);

                tex_coords.extend_from_slice(&[[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]]);

                let start_color = start_sprite.color.with_a(0.2);
                let handle_color = handle_sprite.color;
                let end_color = end_sprite.color.with_a(0.2);

                colors.extend_from_slice(&[
                    start_color.into(),
                    handle_color.into(),
                    end_color.into(),
                ]);

                let i = edge.offset.unwrap() as u32;
                indices.extend_from_slice(&[i, i + 1, i + 2]);
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(indices)));
    }
}
