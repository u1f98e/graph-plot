use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::{input::{CursorMode, CursorInfo}, graph::Graph};

pub(crate) fn egui_sys(mut contexts: EguiContexts, mut cursor: ResMut<CursorInfo>, graph: Res<Graph>) {
    egui::Window::new("Graph Plotter").show(contexts.ctx_mut(), |ui| {
		ui.label(format!("Vertices: {}", graph.node_count()));
		ui.label(format!("Degree: {}", graph.degree()));

		let mode = &mut cursor.mode;
		egui::ComboBox::from_label("Mode")
			.selected_text(format!("{mode}"))
			.show_ui(ui, |ui| {
				ui.selectable_value(mode, CursorMode::Normal, format!("{}", CursorMode::Normal));
				ui.selectable_value(mode, CursorMode::CreateNode, format!("{}", CursorMode::CreateNode));
				ui.selectable_value(mode, CursorMode::CreateEdge, format!("{}", CursorMode::CreateEdge));
				ui.selectable_value(mode, CursorMode::Remove, format!("{}", CursorMode::Remove));
			});
    });
}
