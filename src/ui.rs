use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    graph::{event::{ItemSelectedEvent, get_visibility}, plugin::ImageCache, GEdge, GNode, Graph},
    input::{CursorInfo, CursorMode},
    types::{GEdgeExclusive, GNodeExclusive},
};

pub(crate) fn egui_sys(
    mut contexts: EguiContexts,
    mut cursor: ResMut<CursorInfo>,
    mut graph: ResMut<Graph>,
    q_node: Query<(&GNode, &Children), GNodeExclusive>,
    mut q_edge: Query<(&GEdge, &mut Handle<Image>, &Children), GEdgeExclusive>,
    mut q_labels: Query<(&mut Text, &mut Visibility), With<Parent>>,
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
            for (_, mut handle, _) in q_edge.iter_mut() {
                *handle = if directed {
                    img_cache.get("handle-dir").unwrap().clone()
                } else {
                    img_cache.get("handle").unwrap().clone()
                };
            }
        }

        if ui.checkbox(&mut graph.show_labels, "Show Labels").changed() {
            for (_, mut label_vis) in q_labels.iter_mut() {
                *label_vis = get_visibility(graph.show_labels);
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
                ui.selectable_value(&mut mode, CursorMode::Info, format!("{}", CursorMode::Info));
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
        } else if mode == CursorMode::Info {
            if let Some(entity) = cursor.selected {
                if let Ok((node, children)) = q_node.get(entity) {
                    ui.label(format!("Node: ID = {}", entity.index()));

                    for &child in children.iter() {
                        if let Ok((mut label, _)) = q_labels.get_mut(child) {
                            ui.text_edit_singleline(&mut label.sections[0].value);
                        }
                    }

                    ui.label(format!(
                        "Degree: {}",
                        graph.adjacencies.get(&entity).unwrap().len()
                    ));
                } else if let Ok((edge, _, children)) = q_edge.get_mut(entity) {
                    ui.label(format!("Edge: ID = {}", entity.index()));

                    for &child in children.iter() {
                        if let Ok((mut label, _)) = q_labels.get_mut(child) {
                            ui.text_edit_singleline(&mut label.sections[0].value);
                        }
                    }

                    ui.label(format!("Weight: {}", edge.weight));
                }
            }
        }
    });
}
