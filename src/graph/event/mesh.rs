use bevy::{prelude::*, render::mesh::{VertexAttributeValues, Indices}};

use crate::{types::{GNodeExclusive, GEdgeExclusive}, graph::{GEdge, Graph, NodeE, GNode}};

use super::{ItemMovedEvent, RegenEdgeMesh};

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

pub(crate) fn move_item_event(
    mut events: EventReader<ItemMovedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    graph: Res<Graph>,
    mut q_nodes: Query<(Entity, &mut Transform), GNodeExclusive>,
    mut q_edges: Query<(&GEdge, &mut Transform), GEdgeExclusive>,
) {
    let mesh: &mut Mesh = meshes.get_mut(&graph.edge_mesh_handle.0).unwrap();
    let positions = match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(positions) => positions,
        _ => panic!("Wrong type for position attribute"),
    };

    for ItemMovedEvent(entity, delta) in events.read() {
        // Do the transformation separately to get around borrow restrictions
        if let Ok((_, mut node_t)) = q_nodes.get_mut(*entity) {
            node_t.translation += *delta;
        }

        if let Ok((node_e, node_t)) = q_nodes.get(*entity) {
            for edge_e in graph.node_edges.get(&NodeE(node_e)).unwrap() {
                let (edge, mut edge_t) = q_edges.get_mut(**edge_e).unwrap();
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

                        if *edge.start == node_e {
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
                if node_e == *edge.start {
                    // The node being moved is the start node for this edge
                    let (_, end_node_t) = q_nodes.get(*edge.end).unwrap();
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
                    let (_, start_node_t) = q_nodes.get(*edge.start).unwrap();
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
        } else if let Ok((edge, mut edge_t)) = q_edges.get_mut(*entity) {
            // Do the transformation
            edge_t.translation += *delta;

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

fn color_lerp(a: Color, b: Color, t: f32) -> Color {
    let a: Vec4 = a.into();
    let b: Vec4 = b.into();
    let c = a.lerp(b, t);
    Color::rgba(c.x, c.y, c.z, c.w)
}

pub(crate) fn regen_edge_mesh(
    mut events: EventReader<RegenEdgeMesh>,
    graph: Res<Graph>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_node: Query<(Entity, &mut GNode, &Transform, &Sprite), With<GNode>>,
    mut q_edge: Query<(&mut GEdge, &Transform, &Sprite)>,
) {
    if let Some(_) = events.read().last() {
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

        for (mut edge, edge_t, handle_sprite) in q_edge.iter_mut() {
            let (_, _, start_t, start_sprite) = q_node.get(*edge.start).unwrap();
            let (_, _, end_t, end_sprite) = q_node.get(*edge.end).unwrap();
            let start = start_t.translation;
            let handle = edge_t.translation;
            let end = end_t.translation;

            if edge.is_loop() {
                edge.offset = Some(positions.len());
                positions.extend_from_slice(&loop_vertices(start, handle));
                tex_coords.extend_from_slice(&[[0.0, 0.0], [0.5, 0.0], [1.0, 1.0], [0.5, 0.0]]);

                let start_color = start_sprite.color.into();
                let handle_color = handle_sprite.color.into();
                let midpoint_color = color_lerp(start_color, handle_color, 0.5);

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

                let start_color = color_lerp(start_sprite.color, handle_sprite.color, 0.5);
                let handle_color = handle_sprite.color;
                let end_color = color_lerp(end_sprite.color, handle_sprite.color, 0.5);

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