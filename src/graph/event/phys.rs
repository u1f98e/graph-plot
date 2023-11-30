use bevy::prelude::*;

use crate::{graph::{GEdge, GNode, Graph, NodeE}, types::{GNodeExclusive, GEdgeExclusive}, input::CursorInfo};

use super::{GraphEvent, ItemMovedEvent, offset_midpoint};

static EDGE_SPRING_CONSTANT: f32 = 5.0;
static EDGE_SPRING_LENGTH: f32 = 125.0;
static NODE_GRAVITY: f32 = 10000.0;
static NODE_REPEL_RADIUS: f32 = 350.0;

pub(crate) fn physics_init_event(
	mut events: EventReader<GraphEvent>,
	mut move_ev: EventWriter<ItemMovedEvent>,
	mut q_edges: Query<(Entity, &GEdge, &mut Transform), GEdgeExclusive>,
	q_nodes: Query<&mut Transform, GNodeExclusive>
) {
	match events.read().last() {
		Some(GraphEvent::PhysicsInit) => {
			for (edge_e, edge, edge_t) in q_edges.iter_mut() {
				if edge.is_loop() {
					continue;
				}
				let start_t = q_nodes.get(*edge.start).unwrap();
				let end_t = q_nodes.get(*edge.end).unwrap();
				let midpoint = offset_midpoint(start_t.translation, end_t.translation, 20.0);
				let diff = midpoint - edge_t.translation;
				move_ev.send(ItemMovedEvent(edge_e, diff));
			}
		}
		_ => ()
	}
}

pub(crate) fn physics_sim_system(
	time: Res<Time>,
	graph: Res<Graph>,
	cursor: Res<CursorInfo>,
	q_nodes: Query<(Entity, &GNode, &Transform)>,
	mut move_ev: EventWriter<ItemMovedEvent>,
) {
	if !graph.do_physics {
		return;
	}

	let dt = time.delta_seconds();
	if dt > 0.3 {
		return; // Don't do simulation if dt is too large, otherwise things explode
	}

	// for current_node_e in graph.node_edges.keys().filter(|node_e| cursor.grabbed != Some(***node_e)) {
	for (current_node_e, _, _) in q_nodes.iter().filter(|node| cursor.grabbed != Some(node.0)) {
		let current_node_t = q_nodes.get(current_node_e).unwrap().2.clone();
		let mut translation = current_node_t.translation;
		for (node_e, _, node_t) in q_nodes.iter().filter(|(node_e, _, _)| *node_e != current_node_e) {
			let diff = node_t.translation - translation;
			let dist = diff.length();
			let dir = diff.normalize();
			let gravity = NODE_GRAVITY / dist;
			let repel = (dist.recip() * NODE_REPEL_RADIUS - 1.0).clamp(0.0, 10000.0) * 200.0;
			let mut force = dir * (gravity - repel);
			if graph.is_adjacent(&NodeE(current_node_e), &NodeE(node_e)) {
				force += dir * (EDGE_SPRING_CONSTANT * (dist - EDGE_SPRING_LENGTH));
			}

			translation += force * dt;
		}

		let diff = translation - current_node_t.translation;
		if diff.length() < 0.1 {
			continue;
		}
		move_ev.send(ItemMovedEvent(current_node_e, translation - current_node_t.translation));
	}
}