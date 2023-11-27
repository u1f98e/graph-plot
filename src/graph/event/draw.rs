use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiContexts, egui};

use crate::{graph::Graph, types::{GEdgeExclusive, GNodeExclusive}};

use super::{AnalyzeGraphEvent, RegenEdgeMesh};

pub(crate) fn draw_spanning_tree(
	mut events: EventReader<AnalyzeGraphEvent>,
	graph: Res<Graph>,
	mut q_node: Query<&mut Sprite, GNodeExclusive>,
	mut q_edge: Query<&mut Sprite, GEdgeExclusive>,
	mut ev_regen: EventWriter<RegenEdgeMesh>,
) {
	for event in events.iter() {
		if let AnalyzeGraphEvent::SpanningTree(node_e) = event {
			graph.spanning_tree(node_e, |node_e, edge_e| {
				if let Ok(mut sprite) = q_node.get_mut(**node_e) {
					sprite.color = Color::rgb(0.0, 1.0, 0.0);
				}
				if let Some(edge_e) = edge_e {
					if let Ok(mut sprite) = q_edge.get_mut(**edge_e) {
						sprite.color = Color::rgb(0.0, 1.0, 0.0);
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
	mut egui: EguiContexts
) {
	for event in events.iter() {
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