use bevy::prelude::*;

use crate::graph::{
    plugin::{DefaultTextStyle, ImageCache},
    EdgeE, GEdge, GEdgeBundle, GEdgeHandle, GNode, GNodeBundle, Grabbable, Graph, NodeE,
};

use super::{get_visibility, GraphEvent, RegenEdgeMesh};

pub(crate) fn add_node_event(
    mut events: EventReader<GraphEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    img_cache: Res<ImageCache>,
    text_style: Res<DefaultTextStyle>,
) {
    for event in events.iter() {
        if let GraphEvent::AddNode(pos) = event {
            let transform = Transform::default().with_translation(Vec3::new(pos.x, pos.y, 0.0));
            let node = commands
                .spawn(GNodeBundle {
                    node: GNode::default(),
                    sprite: SpriteBundle {
                        texture: img_cache.get("node").unwrap().clone(),
                        transform,
                        ..Default::default()
                    },
                    grab: Grabbable::default(),
                })
                .with_children(|p| {
                    p.spawn(Text2dBundle {
                        text: Text::from_section(
                            format!("v{}", graph.last_node_num),
                            text_style.clone(),
                        ),
                        transform: Transform::from_translation(Vec3::new(0.0, 30.0, 1.0)),
                        visibility: get_visibility(graph.show_labels),
                        ..Default::default()
                    });
                })
                .id();

            graph.add_node(NodeE(node));
            regen_ev.send(RegenEdgeMesh());
        }
    }
}

pub(crate) fn offset_midpoint(start: Vec3, end: Vec3, offset_len: f32) -> Vec3 {
    let midpoint = start.lerp(end, 0.5);
    let diff = end - start;
    let offset = Vec3::new(diff.y, -diff.x, 0.0).normalize() * offset_len;
    midpoint + offset
}

pub(crate) fn add_edge_event(
    mut events: EventReader<GraphEvent>,
    mut regen_ev: EventWriter<RegenEdgeMesh>,
    mut graph: ResMut<Graph>,
    mut commands: Commands,
    q_nodes: Query<(Entity, &Transform), With<GNode>>,
    cache: Res<ImageCache>,
    text_style: Res<DefaultTextStyle>,
) {
    for event in events.iter() {
        if let GraphEvent::AddEdge(a, b) = event {
            let (start, start_t) = q_nodes.get(**a).unwrap();
            let (end, end_t) = q_nodes.get(**b).unwrap();

            let transform = if start == end {
                // This edge is a loop
                Transform::default()
                    .with_translation(start_t.translation + Vec3::new(0.0, 50.0, 0.0))
                    .with_scale(Vec3::splat(0.5))
            } else {
                // Place the handle between the two nodes
                let (s, e) = (start_t.translation, end_t.translation);
                let point = offset_midpoint(s, e, 20.0);
                let sign = if s.y < e.y {
                    1.0
                } else {
                    -1.0
                };
                Transform::default()
                    .with_translation(point)
                    .with_rotation(Quat::from_axis_angle(
                        Vec3::Z,
                        (end_t.translation - start_t.translation).angle_between(Vec3::X) * sign,
                    ))
                    .with_scale(Vec3::splat(0.75))
            };

            let texture = if graph.directed {
                cache.get("handle-dir").unwrap().clone()
            } else {
                cache.get("handle").unwrap().clone()
            };

            let edge = commands
                .spawn(GEdgeBundle {
                    edge: GEdge {
                        start: *a,
                        end: *b,
                        weight: 1,
                        offset: None,
                    },
                    handle: GEdgeHandle {
                        grab: Grabbable::default(),
                        sprite: SpriteBundle {
                            texture,
                            transform,
                            ..Default::default()
                        },
                    },
                })
                .with_children(|p| {
                    p.spawn(Text2dBundle {
                        text: Text::from_section(
                            format!("e{}", graph.last_edge_num),
                            text_style.clone(),
                        ),
                        transform: Transform::from_translation(Vec3::new(0.0, 30.0, 1.0)),
                        visibility: get_visibility(graph.show_labels),
                        ..Default::default()
                    });
                })
                .id();

            graph.add_edge(EdgeE(edge), &NodeE(start), &NodeE(end));
            regen_ev.send(RegenEdgeMesh());
        }
    }
}
