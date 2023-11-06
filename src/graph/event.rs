use bevy::prelude::*;

use super::*;

#[derive(Event)]
pub struct AddNodeEvent(pub Vec2);

#[derive(Event)]
pub struct RemoveItemEvent(pub Entity);

#[derive(Event)]
pub struct AddEdgeEvent(pub Entity, pub Entity);

#[derive(Event)]
pub struct ItemMovedEvent(pub Entity);

pub(super) fn add_node_event(
    mut events: EventReader<AddNodeEvent>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    cache: Res<ImageCache>,
) {
    for AddNodeEvent(pos) in events.iter() {
        let id = graph.new_node();
		let transform = Transform::default()
			.with_translation(Vec3::new(pos.x, pos.y, 0.0));

        commands.spawn(GNodeBundle {
            node: super::GNode { id },
            sprite: SpriteBundle {
                texture: cache.0.get("node").unwrap().clone(),
                transform,
                ..Default::default()
            },
            grab: Grabbable::default(),
        });
    }
}

pub(super) fn add_edge_event(
    mut events: EventReader<AddEdgeEvent>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    q_nodes: Query<&GNode>,
    cache: Res<ImageCache>,
) {
    for AddEdgeEvent(a, b) in events.iter() {
        let start = q_nodes.get(*a).unwrap().id;
        let end = q_nodes.get(*b).unwrap().id;
        commands.spawn(GEdgeBundle {
            edge: GEdge {
                start,
                end,
                color: Color::WHITE,
                weight: 1,
            },
            handle: GEdgeHandle {
                grab: Grabbable::default(),
                sprite: SpriteBundle {
                    texture: cache.0.get("handle").unwrap().clone(),
                    ..Default::default()
                },
            },
        });

        graph.add_edge(start, end);
    }
}

pub(super) fn remove_item_event(
    mut events: EventReader<RemoveItemEvent>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    mut q_nodes: Query<&mut GNode>,
    q_edges: Query<&GEdge>,
) {
    for RemoveItemEvent(entity) in events.iter() {
        let node_id = q_nodes.get(*entity).ok().map(|n| n.id);
        if let Some(id) = node_id {
            graph.remove_node(id);

            for mut n in q_nodes.iter_mut() {
                if n.id > id {
                    n.id -= 1;
                }
            }
        } else if let Ok(edge) = q_edges.get(*entity) {
            graph.remove_edge(edge.start, edge.end);
        }

        commands.entity(*entity).despawn();
    }
}

pub(super) fn move_item_event(
    mut events: EventReader<ItemMovedEvent>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    q_nodes: Query<(&GNode, &Transform)>,
    q_edges: Query<(&GEdge, &Transform)>,
) {
    for ItemMovedEvent(entity) in events.iter() {
        if let Ok((node, transform)) = q_nodes.get(*entity) {
            graph.node_moved(node.id, transform.translation);
        } else if let Ok((edge, transform)) = q_edges.get(*entity) {
            graph.edge_handle_moved(&edge, transform.translation);
        }
    }
}
