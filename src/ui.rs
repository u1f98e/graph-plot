use bevy::prelude::*;
use bevy_egui::{EguiContexts, egui};

use crate::input::CursorMode;

pub(crate) fn egui_sys(mut contexts: EguiContexts) {
    // egui::Window::new("Graph Plotter").show(contexts.ctx_mut(), |ui| {
	// 	let mut mode_selected = 
	// 	egui::ComboBox::from_label("Mode")
	// 		.selected_text("Create Node")
	// 		.show_ui(ui, |ui| {
	// 			ui.selectable_value(&mut selected, CursorMode::Normal, "Drag/Pan");
	// 		});
    // });
}
