use bevy::prelude::*;

use crate::{graph::{Graph, GEdge}, types::{GEdgeExclusive, GNodeExclusive}, input::CursorInfo};

use super::{AnalyzeGraphEvent, RegenEdgeMesh};

pub(crate) fn draw_spanning_tree(
	mut events: EventReader<AnalyzeGraphEvent>,
	graph: Res<Graph>,
	cursor: Res<CursorInfo>,
	mut q_node: Query<&mut Sprite, GNodeExclusive>,
	mut q_edge: Query<&mut Sprite, GEdgeExclusive>,
	mut ev_regen: EventWriter<RegenEdgeMesh>,
) {
	for event in events.read() {
		if let AnalyzeGraphEvent::SpanningTree(node_e) = event {
			let color = if cursor.paint_color != Color::WHITE {
				cursor.paint_color
			}
			else {
				Color::GREEN
			};

			graph.spanning_tree(node_e, |part| {
				if let Ok(mut sprite) = q_node.get_mut(*part.node) {
					sprite.color = color;
				}
				if let Some(edge_e) = part.edge {
					if let Ok(mut sprite) = q_edge.get_mut(*edge_e) {
						sprite.color = color;
					}
				}
				false
			});
			ev_regen.send(RegenEdgeMesh());
		}
	}
}

pub(crate) fn draw_bipartite(
	mut events: EventReader<AnalyzeGraphEvent>,
	graph: Res<Graph>,
	mut alerts: ResMut<crate::ui::Alerts>,
	mut q_node: Query<&mut Sprite, GNodeExclusive>,
	mut q_edge: Query<&mut Sprite, GEdgeExclusive>,
	mut ev_regen: EventWriter<RegenEdgeMesh>,
) {
	for event in events.read() {
		if let AnalyzeGraphEvent::Bipartite(node_e) = event {
			let result = graph.bipartite_walk(node_e, |node_e, edge_e, set| {
				let color = if set == 0 {
					Color::rgb(1.0, 0.0, 0.0)
				} else {
					Color::rgb(0.0, 0.0, 1.0)
				};

				if let Ok(mut sprite) = q_node.get_mut(**node_e) {
					sprite.color = color;
				}
				if let Some(edge_e) = edge_e {
					if let Ok(mut sprite) = q_edge.get_mut(**edge_e) {
						sprite.color = color;
					}
				}
			});
			ev_regen.send(RegenEdgeMesh());

			if !result {
				alerts.0.push("Graph is not bipartite".to_string());
			}
		}
	}
}

pub(crate) fn draw_shortest_path(
	mut events: EventReader<AnalyzeGraphEvent>,
	graph: Res<Graph>,
	cursor: Res<CursorInfo>,
	mut q_node: Query<&mut Sprite, GNodeExclusive>,
	mut p_edge: ParamSet<(Query<&mut Sprite, GEdgeExclusive>, Query<&GEdge>)>,
	mut ev_regen: EventWriter<RegenEdgeMesh>,
) {
	for event in events.read() {
		if let AnalyzeGraphEvent::Dijkstra(a, b) = event {
			let color = if cursor.paint_color != Color::WHITE {
				cursor.paint_color
			}
			else {
				Color::GREEN
			};

			if let Some(path) = graph.dijkstra_path(&a, &b, Some(&p_edge.p1())) {
				for part in path {
					if let Ok(mut sprite) = q_node.get_mut(*part.node) {
						sprite.color = color;
					}
					if let Some(edge_e) = part.edge {
						if let Ok(mut sprite) = p_edge.p0().get_mut(*edge_e) {
							sprite.color = color;
						}
					}
				}

				ev_regen.send(RegenEdgeMesh());
			}
		}
	}
}
