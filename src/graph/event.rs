mod add;
mod remove;
mod mesh;
pub(crate) mod draw;
pub(crate) mod phys;

pub(crate) use {add::*, remove::*, mesh::*};

use bevy::prelude::*;

use crate::{input::CursorInfo, ui::UiItemInfo};
use crate::types::*;

use super::{Graph, plugin::ImageCache, NodeE};

pub(crate) fn get_visibility(is_visible: bool) -> Visibility {
    if is_visible {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    }
}

#[derive(Event)]
pub enum GraphEvent {
    AddNode(Vec2),
    AddEdge(NodeE, NodeE),
    RemoveItem(Entity),
    ItemSelected(Entity),
    ItemDeselected,
    ResetColors,
    PhysicsInit,
}

#[derive(Event)]
pub struct ItemMovedEvent(pub Entity, pub Vec3);

#[derive(Event)]
pub(crate) struct RegenEdgeMesh();

#[derive(Event)]
pub enum AnalyzeGraphEvent {
    SpanningTree(NodeE),
    Bipartite(NodeE),
    Dijkstra(NodeE, NodeE),
}

pub(crate) fn item_selected_event(
    mut events: EventReader<GraphEvent>,
    mut cursor: ResMut<CursorInfo>,
    mut q_node: Query<&mut Handle<Image>, GNodeExclusive>,
    mut q_edge: Query<&mut Handle<Image>, GEdgeExclusive>,
    graph: Res<Graph>,
    cache: Res<ImageCache>,
    mut ui_info: ResMut<UiItemInfo>,
) {
    for event in events.read() {
        match event {
            GraphEvent::ItemSelected(entity) => {
                if let Some(entity) = cursor.selected {
                    if let Ok(mut texture) = q_node.get_mut(entity) {
                        *texture = cache.get("node").unwrap().clone();
                    } else if let Ok(mut texture) = q_edge.get_mut(entity) {
                        *texture = if graph.directed {
                            cache.get("handle-dir")
                        } else {
                            cache.get("handle")
                        }
                        .unwrap()
                        .clone();
                    }
                }
                if let Ok(mut texture) = q_node.get_mut(*entity) {
                    *texture = cache.get("node-sel").unwrap().clone();
                } else if let Ok(mut texture) = q_edge.get_mut(*entity) {
                    *texture = if graph.directed {
                        cache.get("handle-dir-sel")
                    } else {
                        cache.get("handle-sel")
                    }
                    .unwrap()
                    .clone();
                }
                cursor.selected = Some(*entity);
            }
            GraphEvent::ItemDeselected => {
                if let Some(entity) = cursor.selected {
                    if let Ok(mut texture) = q_node.get_mut(entity) {
                        *texture = cache.get("node").unwrap().clone();
                    } else if let Ok(mut texture) = q_edge.get_mut(entity) {
                        *texture = if graph.directed {
                            cache.get("handle-dir")
                        } else {
                            cache.get("handle")
                        }
                        .unwrap()
                        .clone();
                    }
                }
                cursor.selected = None;
            }
            _ => ()
        }

        *ui_info = UiItemInfo::None;
    }
}

pub(crate) fn reset_colors_event(
    mut events: EventReader<GraphEvent>,
    mut q_node: Query<&mut Sprite, GNodeExclusive>,
    mut q_edge: Query<&mut Sprite, GEdgeExclusive>,
    mut regen_edge_mesh: EventWriter<RegenEdgeMesh>,
) {
    for event in events.read() {
        match event {
            GraphEvent::ResetColors => {
                for mut sprite in q_node.iter_mut() {
                    sprite.color = Color::WHITE;
                }
                for mut sprite in q_edge.iter_mut() {
                    sprite.color = Color::WHITE;
                }
                regen_edge_mesh.send(RegenEdgeMesh());
            }
            _ => ()
        }
    }
}

// fn edge_vertices(start_pos: Vec3, handle_pos: Vec3, end_pos: Vec3) -> [[f32; 3]; 4] {
//     let start = Vec3::from_array(positions[offset - 1]);
//     let end = Vec3::from_array(positions[offset + 1]);
//     let start_end_mid = start.lerp(end, 0.5);
//     let pos = 2.0 * edge_t.translation - start_end_mid;
//     positions[offset] = pos.to_array();
// }
