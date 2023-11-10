use bevy::{prelude::*, render::mesh::Indices};

use super::*;

#[derive(Event)]
pub struct AddNodeEvent(pub Vec2);

#[derive(Event)]
pub struct RemoveItemEvent(pub Entity);

#[derive(Event)]
pub struct AddEdgeEvent(pub Entity, pub Entity);

#[derive(Event)]
pub struct ItemMovedEvent(pub Entity);

#[derive(Event)]
pub(crate) struct RegenEdgeMesh();

pub(super) fn add_node_event(
    mut events: EventReader<AddNodeEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    cache: Res<ImageCache>,
) {
    for AddNodeEvent(pos) in events.iter() {
        let transform = Transform::default().with_translation(Vec3::new(pos.x, pos.y, 0.0));

        println!("Position: {:?}", transform.translation);
        let node = commands.spawn(GNodeBundle {
            node: super::GNode{},
            sprite: SpriteBundle {
                texture: cache.0.get("node").unwrap().clone(),
                transform,
                ..Default::default()
            },
            grab: Grabbable::default(),
        }).id();

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
    mut q_nodes: Query<Entity, With<GNode>>,
    q_edges: Query<(Entity, &GEdge)>,
) {
    for RemoveItemEvent(entity) in events.iter() {
        if let Ok(node) = q_nodes.get(*entity) {
            graph.remove_node(&node);

            for (edge_e, edge) in q_edges.iter() {
                if edge.start== node || edge.end== node {
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
    q_nodes: Query<(&GNode, &Transform)>,
    q_edges: Query<(&GEdge, &Transform)>,
) {
    // let mesh: &mut Mesh = meshes.get_mut(&graph.edge_mesh_handle.0).unwrap();
    for ItemMovedEvent(entity) in events.iter() {
        if let Ok((node, transform)) = q_nodes.get(*entity) {

        } else if let Ok((edge, transform)) = q_edges.get(*entity) {

        }
    }
}

pub(super) fn regen_edge_mesh(
    mut events: EventReader<RegenEdgeMesh>,
    graph: Res<Graph>,
    mut meshes: ResMut<Assets<Mesh>>,
    q_node: Query<(Entity, &Transform, &Sprite), With<GNode>>,
    q_edge: Query<(&GEdge, &Transform, &Sprite)>,
) {
    for _ in events.iter() {
        println!("Regen edge mesh");
        let mesh: &mut Mesh = meshes.get_mut(&graph.edge_mesh_handle.0).unwrap();

        // There needs to be some initial values here or the mesh gets optimized
        // out
        let mut positions: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [0.0, 0.0, 0.0]];
        let mut tex_coords: Vec<[f32; 2]> = vec![[0.0, 0.0], [1.0, 1.0], [0.5, 0.0]];
        let mut colors: Vec<[f32; 4]> = vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0], [0.0, 0.0, 0.0, 1.0]];
        let mut indices: Vec<u32> = vec![0, 1, 2];

        for (mut index, (edge, edge_t, handle_sprite)) in q_edge.iter().enumerate() {
            index += 1;
            let (_, start_t, start_sprite) = q_node.get(edge.start).unwrap();
            let (_, end_t, end_sprite) = q_node.get(edge.end).unwrap();

            let start_pos = start_t.translation;
            let handle_pos = edge_t.translation;
            let end_pos = end_t.translation;

            positions.extend_from_slice(&[
                [start_pos.x, start_pos.y, 0.0],
                [handle_pos.x, handle_pos.y, 0.0],
                [end_pos.x, end_pos.y, 0.0],
            ]);

            tex_coords.extend_from_slice(&[[0.0, 0.0], [0.5, 0.0], [1.0, 1.0]]);

            let start_color = start_sprite.color;
            let handle_color = handle_sprite.color;
            let end_color = end_sprite.color;

            colors.extend_from_slice(&[start_color.into(), handle_color.into(), end_color.into()]);

            indices.extend_from_slice(&[
                index as u32 * 3,
                index as u32 * 3 + 1,
                index as u32 * 3 + 2,
            ]);
        }

        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, tex_coords);
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.set_indices(Some(Indices::U32(indices)));
    }
}
