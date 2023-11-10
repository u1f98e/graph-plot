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
				ui.selectable_value(mode, CursorMode::Paint, format!("{}", CursorMode::Paint));
			});

		if *mode == CursorMode::Paint {
			let mut color: [f32; 3] = cursor.paint_color.as_rgba_f32()[0..3].try_into().unwrap();
			egui::color_picker::color_edit_button_rgb(ui, &mut color);
			cursor.paint_color = color.into(); // Color::rgb(color[0], color[1], color[2]);
		}
    });
}
