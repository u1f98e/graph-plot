use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    graph::{event::{GraphEvent, get_visibility}, plugin::ImageCache, GEdge, GNode, Graph, NodeE, EdgeE},
    input::{CursorInfo, CursorMode},
    types::{GEdgeExclusive, GNodeExclusive},
};

#[derive(Default, Resource)]
pub enum UiItemInfo {
    #[default]
    None,
    Node {
        node_e: Entity,
        label: Option<Entity>
    },
    Edge {
        edge: GEdge,
        edge_e: Entity,
        is_bridge: bool,
        label: Option<Entity>
    },
}

pub(crate) fn egui_sys(
    mut contexts: EguiContexts,
    mut graph_ev: EventWriter<GraphEvent>,
    resources: (
        ResMut<Graph>,
        ResMut<CursorInfo>,
        ResMut<UiItemInfo>,
        Res<ImageCache>,
    ),
    queries: (
        Query<(&GNode, &Children), GNodeExclusive>,
        Query<(&GEdge, &mut Handle<Image>, &Children), GEdgeExclusive>,
        Query<(&mut Text, &mut Visibility), With<Parent>>,
    ),
) {
    let (q_node, mut q_edge, mut q_labels) = queries;
    let (mut graph, mut cursor, mut info_item, img_cache) = resources;

    egui::Window::new("Graph Plotter").show(contexts.ctx_mut(), |ui| {
        ui.label(format!("Vertices: {}", graph.node_count()));
        ui.label(format!("Edges: {}", graph.degree() / 2));
        ui.label(format!("Total Degree: {}", graph.degree()));
        ui.label(format!("Components: {}", graph.components()));

        let mut directed = graph.directed;
        if ui.checkbox(&mut directed, "Directed").changed() {
            graph.directed = directed;
            graph_ev.send(GraphEvent::ItemDeselected);
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

        if ui.checkbox(&mut graph.do_physics, "Physics").changed() {
            graph_ev.send(GraphEvent::PhysicsInit);
        }

        if ui.button("Reset Colors").clicked() {
            graph_ev.send(GraphEvent::ResetColors);
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
                ui.selectable_value(
                    &mut mode,
                    CursorMode::SpanningTree,
                    format!("{}", CursorMode::SpanningTree),
                );
            });
        if mode != cursor.mode {
            cursor.set_mode(&mode, &mut graph_ev);
        }

        if mode == CursorMode::Paint {
            let mut color: [u8; 3] = cursor.paint_color.as_rgba_u8()[0..3].try_into().unwrap();
            egui::color_picker::color_edit_button_srgb(ui, &mut color);
            cursor.paint_color = Color::rgb_u8(color[0], color[1], color[2]);
        } 
        else if mode == CursorMode::Info {
            match &*info_item {
                UiItemInfo::None => if let Some(entity) = cursor.selected {
                    if let Ok((_node, children)) = q_node.get(entity) {
                        let label = children.iter().find(|&child| {
                            q_labels.get(*child).is_ok()
                        }).copied();

                        *info_item = UiItemInfo::Node {
                            node_e: entity,
                            label
                        };
                    } else if let Ok((edge, _, children)) = q_edge.get_mut(entity) {
                        let label = children.iter().find(|&child| {
                            q_labels.get(*child).is_ok()
                        }).copied();

                        *info_item = UiItemInfo::Edge {
                            edge: edge.clone(),
                            edge_e: entity,
                            label,
                            is_bridge: graph.is_bridge(&EdgeE(entity)),
                        };
                    }
                },
                UiItemInfo::Node { node_e, label } => {
                    ui.label(format!("Node: ID = {}", node_e.index()));

                    if let Some(label) = label {
                        if let Ok((mut label, _)) = q_labels.get_mut(*label) {
                            ui.text_edit_singleline(&mut label.sections[0].value);
                        }
                    }

                    ui.label(format!(
                        "Degree: {}",
                        graph.node_edges.get(&NodeE(*node_e)).unwrap().len()
                    ));
                },
                UiItemInfo::Edge { edge, edge_e, is_bridge, label } => {
                    ui.label(format!("Edge: ID = {}", edge_e.index()));

                    if let Some(label) = label {
                        if let Ok((mut label, _)) = q_labels.get_mut(*label) {
                            ui.text_edit_singleline(&mut label.sections[0].value);
                        }
                    }

                    ui.label(format!("Weight: {}", edge.weight));
                    ui.label(format!("Is Bridge: {}", is_bridge));
                }
            }
        }
    });
}

fn egui_matrix(ui: &mut egui::Ui, data: Vec<Vec<f32>>) {
    for row in data.iter() {
        ui.horizontal(|ui| {
            for col in row.iter() {
                ui.label(format!("{:.2}", col));
            }
        });
    }
}

fn egui_graph_info(ui: &mut egui::Ui, graph: &Graph) {
    ui.label(format!("Vertices: {}", graph.node_count()));
    ui.label(format!("Edges: {}", graph.degree() / 2));
    ui.label(format!("Total Degree: {}", graph.degree()));
    ui.label(format!("Components: {}", graph.components));
}