use bevy::{prelude::*, render::mesh::{Indices, VertexAttributeValues}};

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
                    *texture = cache.0.get("nodeSelected").unwrap().clone();
                }
                else if let Ok(mut texture) = q_edge.get_mut(*entity) {
                    *texture = cache.0.get("edgeSelected").unwrap().clone();
                }
                cursor.selected = Some(*entity);
            }
            ItemSelectedEvent::Deselected => {
                if let Some(entity) = cursor.selected {
                    if let Ok(mut texture) = q_node.get_mut(entity) {
                        *texture = cache.0.get("node").unwrap().clone();
                    }
                    else if let Ok(mut texture) = q_edge.get_mut(entity) {
                        *texture = cache.0.get("edge").unwrap().clone();
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
    cache: Res<ImageCache>,
) {
    for AddNodeEvent(pos) in events.iter() {
        let transform = Transform::default().with_translation(Vec3::new(pos.x, pos.y, 0.0));
        let node = commands
            .spawn(GNodeBundle {
                node: GNode::default(),
                sprite: SpriteBundle {
                    texture: cache.0.get("node").unwrap().clone(),
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
            Transform::default()
                .with_translation(midpoint + offset)
                .with_scale(Vec3::splat(0.5))
        };

        let edge = commands
            .spawn(GEdgeBundle {
                edge: GEdge {
                    start: *a,
                    end: *b,
                    weight: 1,
                    offset: 0,
                },
                handle: GEdgeHandle {
                    grab: Grabbable::default(),
                    sprite: SpriteBundle {
                        texture: cache.0.get("handle").unwrap().clone(),
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
    q_nodes: Query<Entity, With<GNode>>,
    q_edges: Query<(Entity, &GEdge)>,
) {
    for RemoveItemEvent(entity) in events.iter() {
        if let Ok(node) = q_nodes.get(*entity) {
            graph.remove_node(&node);

            for (edge_e, edge) in q_edges.iter() {
                if edge.start == node || edge.end == node {
                    commands.entity(edge_e).despawn();
                }
            }
        } else if let Ok((_, edge)) = q_edges.get(*entity) {
            graph.remove_edge(*entity, &edge.start, &edge.end);
        }

        commands.entity(*entity).despawn();
        regen_ev.send(RegenEdgeMesh());
    }
}

pub(super) fn move_item_event(
    mut events: EventReader<ItemMovedEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    graph: Res<Graph>,
    q_nodes: Query<(&GNode, &Transform), Without<GEdge>>,
    mut q_edges: Query<(&GEdge, &mut Transform), Without<GNode>>,
) {
    let mesh: &mut Mesh = meshes.get_mut(&graph.edge_mesh_handle.0).unwrap();
    let positions = match mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap() {
        VertexAttributeValues::Float32x3(positions) => positions,
        _ => panic!("Wrong type for position attribute"),
    };

    for ItemMovedEvent(entity, delta) in events.iter() {
        if let Ok((node, transform)) = q_nodes.get(*entity) {
            for (offset, _side) in node.offsets.iter() {
                positions[*offset] = transform.translation.to_array();
            }
            for edge_e in graph.adjacencies.get(entity).unwrap() {
                let (edge, mut edge_t) = q_edges.get_mut(*edge_e).unwrap();
                edge_t.translation += *delta / 2.0;

                let start = Vec3::from_array(positions[edge.offset - 1]);
                let end = Vec3::from_array(positions[edge.offset + 1]);
                let start_end_mid = start.lerp(end, 0.5);
                let pos = 2.0 * edge_t.translation - start_end_mid;
                positions[edge.offset] = pos.to_array();
            }
        } else if let Ok((edge, transform)) = q_edges.get(*entity) {
            if edge.offset == 0 || edge.offset == positions.len() - 1 {
                continue;
            }
            let start = Vec3::from_array(positions[edge.offset - 1]);
            let end = Vec3::from_array(positions[edge.offset + 1]);
            let start_end_mid = start.lerp(end, 0.5);
            let pos = 2.0 * transform.translation - start_end_mid;
            positions[edge.offset] = pos.to_array();
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
    for _ in events.iter() {
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

        for (mut index, (mut edge, edge_t, handle_sprite)) in q_edge.iter_mut().enumerate() {
            index += 1;
            {
                let (_, _, start_t, start_sprite) = q_node.get(edge.start).unwrap();
                let (_, _, end_t, end_sprite) = q_node.get(edge.end).unwrap();
                let start = start_t.translation;
                let handle = edge_t.translation;
                let end = end_t.translation;

                // let start_end = (start_t.translation - end_t.translation).normalize();
                // let start_mid = (start_t.translation - edge_t.translation).normalize();
                // let end_mid = (end_t.translation - edge_t.translation).normalize();

                let start_end_mid = start.lerp(end, 0.5);
                let handle = 2.0 * handle - start_end_mid;

                positions.extend_from_slice(&[
                    [start.x, start.y, 0.0],
                    [handle.x, handle.y, 0.0],
                    [end.x, end.y, 0.0],
                ]);

                tex_coords.extend_from_slice(&[[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]]);

                let start_color = start_sprite.color.with_a(0.2);
                let handle_color = handle_sprite.color;
                let end_color = end_sprite.color.with_a(0.2);

                colors.extend_from_slice(&[start_color.into(), handle_color.into(), end_color.into()]);

                indices.extend_from_slice(&[
                    index as u32 * 3,
                    index as u32 * 3 + 1,
                    index as u32 * 3 + 2,
                ]);
            }

            edge.offset = index * 3 + 1;
            {
                let mut start_node = q_node.get_mut(edge.start).unwrap().1;
                start_node.offsets.push((index * 3, GNodeSide::Start));
            }
            {
                let mut end_node = q_node.get_mut(edge.end).unwrap().1;
                end_node.offsets.push((index * 3 + 2, GNodeSide::End));
            }
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(indices)));
    }
}
