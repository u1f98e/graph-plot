use bevy::prelude::*;

use crate::graph::{Graph, GNode, GEdge, EdgeE, NodeE};

use super::{GraphEvent, RegenEdgeMesh};

pub(crate) fn remove_item_event(
    mut events: EventReader<GraphEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    q_nodes: Query<&mut GNode>,
    mut q_edges: Query<(Entity, &mut GEdge)>,
) {
    let mut removed_edges = Vec::new();
    for event in events.read() {
		if let GraphEvent::RemoveItem(entity) = event {
			// Collect any edges that will be destroyed by this removal
			if let Ok(_) = q_nodes.get(*entity) {
				for edge_e in graph.node_edges.get(&NodeE(*entity)).unwrap() {
					removed_edges.push(*edge_e);
				}
			} else if let Ok((edge_e, _)) = q_edges.get(*entity) {
				removed_edges.push(EdgeE(edge_e));
			}

			for edge_e in removed_edges.iter() {
				let offset_removed: Option<usize> = if let Ok((_, edge)) = q_edges.get(**edge_e) {
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
				commands.entity(**edge_e).despawn_recursive();
			}
			removed_edges.clear();

			// Actually remove the node after its edges are removed
			if let Ok(_) = q_nodes.get(*entity) {
				graph.remove_node(&(*entity).into());
				commands.entity(*entity).despawn_recursive();
			}

			regen_ev.send(RegenEdgeMesh());
		}
	}
}
