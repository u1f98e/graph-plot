use std::fmt::format;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::{
    graph::{
        event::{get_visibility, GraphEvent},
        plugin::ImageCache,
        EdgeE, GEdge, GNode, Graph, LabeledMatrix, NodeE,
    },
    input::{CursorInfo, CursorMode},
    smatrix::SMatrix,
    types::{GEdgeExclusive, GNodeExclusive},
};

#[derive(Default, Resource)]
pub(crate) struct Alerts(pub Vec<String>);

#[derive(Default, Resource)]
pub(crate) enum UiItemInfo {
    #[default]
    None,
    Node {
        node_e: Entity,
        label: Option<Entity>,
    },
    Edge {
        edge: GEdge,
        edge_e: Entity,
        is_bridge: bool,
        label: Option<Entity>,
    },
}

#[derive(Default, Resource)]
pub(crate) struct GraphInfoWindow {
    pub open: bool,
    adj_matrix: LabeledMatrix,
    max_col_width: usize,
    eigen: Option<nalgebra::SymmetricEigen<f32, nalgebra::Dyn>>,
}

pub(crate) fn egui_sys(
    mut contexts: EguiContexts,
    mut graph_ev: EventWriter<GraphEvent>,
    resources: (
        ResMut<Graph>,
        ResMut<CursorInfo>,
        ResMut<UiItemInfo>,
        ResMut<Alerts>,
        ResMut<GraphInfoWindow>,
        Res<ImageCache>,
    ),
    queries: (
        Query<(&GNode, &Children), GNodeExclusive>,
        Query<(&GEdge, &mut Handle<Image>, &Children), GEdgeExclusive>,
        Query<(&mut Text, &mut Visibility), With<Parent>>,
    ),
) {
    let (q_node, mut q_edge, mut q_labels) = queries;
    let (mut graph, mut cursor, mut info_item, mut alerts, mut info_win, img_cache) = resources;

    show_alerts(contexts.ctx_mut(), &mut alerts.0);

    egui::Window::new("Graph Plotter").show(contexts.ctx_mut(), |ui| {
        egui_graph_info(ui, &graph);

        if ui.button("Graph Info").clicked() {
            info_win.open = true;
        }

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
            .width(125.0)
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
                ui.selectable_value(
                    &mut mode,
                    CursorMode::Bipartite,
                    format!("{}", CursorMode::Bipartite),
                );
                ui.selectable_value(
                    &mut mode,
                    CursorMode::Dijkstra,
                    format!("{}", CursorMode::Dijkstra),
                );
            });
        if mode != cursor.mode {
            cursor.set_mode(&mode, &mut graph_ev);
        }

        if mode == CursorMode::Paint {
            let mut color: [u8; 3] = cursor.paint_color.as_rgba_u8()[0..3].try_into().unwrap();
            egui::color_picker::color_edit_button_srgb(ui, &mut color);
            cursor.paint_color = Color::rgb_u8(color[0], color[1], color[2]);
        } else if mode == CursorMode::Info {
            match &*info_item {
                UiItemInfo::None => {
                    if let Some(entity) = cursor.selected {
                        if let Ok((_node, children)) = q_node.get(entity) {
                            let label = children
                                .iter()
                                .find(|&child| q_labels.get(*child).is_ok())
                                .copied();

                            *info_item = UiItemInfo::Node {
                                node_e: entity,
                                label,
                            };
                        } else if let Ok((edge, _, children)) = q_edge.get_mut(entity) {
                            let label = children
                                .iter()
                                .find(|&child| q_labels.get(*child).is_ok())
                                .copied();

                            *info_item = UiItemInfo::Edge {
                                edge: edge.clone(),
                                edge_e: entity,
                                label,
                                is_bridge: graph.is_bridge(&EdgeE(entity)),
                            };
                        }
                    }
                }
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
                }
                UiItemInfo::Edge {
                    edge,
                    edge_e,
                    is_bridge,
                    label,
                } => {
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

fn egui_matrix(ui: &mut egui::Ui, matrix: &LabeledMatrix, max_col_width: usize) {
    ui.horizontal(|ui| {
        ui.monospace(format!("{1:0$}", max_col_width, ""));
        for col in matrix.h_headers.iter() {
            ui.monospace(format!("{1:0$}", max_col_width, col));
        }
    });

    for (row, cols) in matrix.data.iter().enumerate() {
        ui.horizontal(|ui| {
            ui.monospace(format!("{1:0$}", max_col_width, matrix.v_headers[row]));
            for col in cols.iter() {
                ui.monospace(format!("{1:0$}", max_col_width, col));
            }
        });
    }
}

fn egui_graph_info(ui: &mut egui::Ui, graph: &Graph) {
    ui.label(format!("Vertices: {}", graph.node_count()));
    ui.label(format!("Edges: {}", graph.degree() / 2));
    ui.label(format!("Total Degree: {}", graph.degree()));
    ui.label(format!("Components: {}", graph.components()));
}

pub(crate) fn egui_show_graph_info(
    mut contexts: EguiContexts,
    mut info_win: ResMut<GraphInfoWindow>,
    q_nodes: Query<(Entity, &Children), With<GNode>>,
    q_text: Query<&Text>,
    graph: Res<Graph>,
) {
    let mut open = info_win.open;
    egui::Window::new("Graph Info")
        .open(&mut open)
        .show(contexts.ctx_mut(), |ui| {
            egui::ScrollArea::both()
                .max_width(200.0)
                .max_width(200.0)
                .show(ui, |ui| {
                    if ui.button("Refresh").clicked() || info_win.adj_matrix.data.is_empty() {
                        info_win.adj_matrix = graph.adjacency_matrix(&q_nodes, &q_text);

                        let size = info_win.adj_matrix.data.len();
                        let matrix = nalgebra::DMatrix::from_vec(
                            size,
                            size,
                            info_win.adj_matrix.data.iter().flatten().cloned().collect(),
                        );
                        info_win.eigen = Some(nalgebra::linalg::SymmetricEigen::new(matrix));

                        // Get the maximum column width for the matrix
                        let mut max_width = 0;
                        for header in info_win.adj_matrix.h_headers.iter() {
                            max_width = max_width.max(header.len());
                        }
                        info_win.max_col_width = max_width;
                    }

                    ui.label("Adjacency Matrix");
                    egui_matrix(ui, &info_win.adj_matrix, info_win.max_col_width);
                    ui.separator();

                    ui.label("Eigenvalues:");
                    ui.label(format!(
                        "{}",
                        info_win
                            .eigen
                            .as_ref()
                            .unwrap()
                            .eigenvalues
                            .iter()
                            .map(|e| format!("{:.4}", e))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                    ui.separator();

                    ui.label("Eigenvectors (iterative estimation):");
                    for (i, vec) in info_win
                        .eigen
                        .as_ref()
                        .unwrap()
                        .eigenvectors
                        .column_iter()
                        .enumerate()
                    {
                        ui.label(format!("{}: [{}]", i, vec.iter().map(|e| format!("{:.4}", e)).collect::<Vec<_>>().join(", ")));
                    }
                });
        });

    info_win.open = open;
}

fn show_alerts(ctx: &mut egui::Context, alerts: &mut Vec<String>) {
    let mut closed = Vec::new();
    for alert in alerts.iter() {
        egui::Window::new("Alert").show(ctx, |ui| {
            ui.label(alert);
            if ui.button("Ok").clicked() {
                closed.push(alert.clone());
            }
        });
    }

    alerts.retain(|a| !closed.contains(a));
}
