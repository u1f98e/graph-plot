use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    graph::{event::ItemSelectedEvent, Graph, GEdge, plugin::ImageCache},
    input::{CursorInfo, CursorMode},
};

pub(crate) fn egui_sys(
    mut contexts: EguiContexts,
    mut cursor: ResMut<CursorInfo>,
    mut graph: ResMut<Graph>,
    mut q_edge_img: Query<&mut Handle<Image>, With<GEdge>>,
    mut ev_selected: EventWriter<ItemSelectedEvent>,
	img_cache: Res<ImageCache>,
) {
    egui::Window::new("Graph Plotter").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Vertices: {}", graph.node_count()));
        ui.label(format!("Edges: {}", graph.degree() / 2));
        ui.label(format!("Total Degree: {}", graph.degree()));

        let mut directed = graph.directed;
        if ui.checkbox(&mut directed, "Directed").changed() {
			graph.directed = directed;
			ev_selected.send(ItemSelectedEvent::Deselected);
			for mut handle in q_edge_img.iter_mut() {
				*handle = if directed {
					img_cache.get("handle_directed").unwrap().clone()
				} else {
					img_cache.get("handle").unwrap().clone()
				};
			}
		}

        let mut mode = cursor.mode;
        egui::ComboBox::from_label("Mode")
            .selected_text(format!("{mode}"))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut mode,
                    CursorMode::Normal,
                    format!("{}", CursorMode::Normal),
                );
                ui.selectable_value(
                    &mut mode,
                    CursorMode::CreateNode,
                    format!("{}", CursorMode::CreateNode),
                );
                ui.selectable_value(
                    &mut mode,
                    CursorMode::CreateEdge,
                    format!("{}", CursorMode::CreateEdge),
                );
                ui.selectable_value(
                    &mut mode,
                    CursorMode::Remove,
                    format!("{}", CursorMode::Remove),
                );
                ui.selectable_value(
                    &mut mode,
                    CursorMode::Paint,
                    format!("{}", CursorMode::Paint),
                );
            });
        if mode != cursor.mode {
            cursor.set_mode(&mode, &mut ev_selected);
        }

        if mode == CursorMode::Paint {
            let mut color: [u8; 3] = cursor.paint_color.as_rgba_u8()[0..3].try_into().unwrap();
            egui::color_picker::color_edit_button_srgb(ui, &mut color);
            cursor.paint_color = Color::rgb_u8(color[0], color[1], color[2]);
        }
    });
}
