use bevy::{input::{mouse::{MouseWheel, MouseButtonInput}, ButtonState}, prelude::*, window::PrimaryWindow};
use bevy_egui::{EguiContext};

use crate::{graph::{Graph, Grabbable, plugin::ImageCache}, MainCamera};
use crate::graph::event::*;

#[derive(Default, PartialEq, Eq)]
pub enum CursorMode {
    #[default]
    Normal,
    CreateNode,
    CreateEdge,
    Remove,
    Paint,
}

impl core::fmt::Display for CursorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CursorMode::Normal => write!(f, "Drag/Pan"),
            CursorMode::CreateNode => write!(f, "Create Node"),
            CursorMode::CreateEdge => write!(f, "Create Edge"),
            CursorMode::Remove => write!(f, "Erase"),
            CursorMode::Paint => write!(f, "Paint")
        }
    }
}

#[derive(Resource, Default)]
pub struct CursorInfo {
    pub screen_pos: Vec2,
    pub world_pos: Vec2,
    pub mode: CursorMode,
    pub grabbed: Option<Entity>,
    pub selected: Option<Entity>,
}

pub fn key_input_sys(
    key: Res<Input<KeyCode>>,
    mut cursor: ResMut<CursorInfo>,
    mut egui_ctx: Query<&mut EguiContext>,
) {
    if egui_ctx
        .iter_mut()
        .all(|mut ctx| ctx.get_mut().wants_keyboard_input())
    {
        return;
    }

    if key.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
    if key.just_pressed(KeyCode::N) {
        cursor.mode = CursorMode::CreateNode;
    }
    if key.just_pressed(KeyCode::S) {
        cursor.mode = CursorMode::Normal;
    }
}

pub fn mouse_movement_sys(
    mut cursor_evr: EventReader<CursorMoved>,
    mut cursor: ResMut<CursorInfo>,
    mut q_camera: Query<(Entity, &Camera, &mut Transform, &GlobalTransform), With<crate::MainCamera>>,
    mut q_grab: Query<(Entity, &mut Transform), (With<Grabbable>, Without<Camera>)>,
    mut ev_move_item: EventWriter<ItemMovedEvent>,
) {
    let (camera_e, camera, mut camera_tf, camera_global_tf) = q_camera.single_mut();

    for CursorMoved { position, .. } in cursor_evr.iter() {
        let ray = camera.viewport_to_world(camera_global_tf, *position);
        let mut cursor_delta = Vec2::ZERO;
        if let Some(ray) = ray {
            cursor_delta = *position - cursor.screen_pos;
            cursor.screen_pos = *position;
            cursor.world_pos = ray.origin.truncate();
        }

        if let Some(entity) = cursor.grabbed {
            if camera_e == entity {
                camera_tf.translation.x -= cursor_delta.x;
                camera_tf.translation.y += cursor_delta.y;
            }
            else {
                let mut transform = q_grab.get_mut(entity).unwrap().1;
                transform.translation = Vec3::new(cursor.world_pos.x, cursor.world_pos.y, 0.0);
                ev_move_item.send(ItemMovedEvent(entity));
            }
        }
    }
}

fn get_closest_grab(
    cursor: &CursorInfo,
    q_grab: &Query<(Entity, &Grabbable, &mut Transform)>,
) -> Option<Entity> {
    let mut closest_distance = f32::INFINITY;
    let mut closest_grab: Option<Entity> = None;

    for (entity, grab, transform) in q_grab.iter() {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = cursor.world_pos.distance(pos);
        if distance < grab.radius && distance < closest_distance {
            closest_distance = distance;
            closest_grab = Some(entity);
        }
    }

    closest_grab
}

pub fn mouse_button_sys(
    mut ev_mouse_button: EventReader<MouseButtonInput>,
    mut cursor: ResMut<CursorInfo>,
    query: (Query<&mut EguiContext>, Query<(Entity, &crate::graph::Grabbable, &mut Transform)>, Query<(Entity, With<MainCamera>)>),
    mut ev_add_node: EventWriter<AddNodeEvent>,
    mut ev_add_edge: EventWriter<AddEdgeEvent>,
    mut ev_remove_graph_item: EventWriter<RemoveItemEvent>,
) {
    let (mut q_egui, q_grab, q_camera) = query;

    for MouseButtonInput { button, state, .. } in ev_mouse_button.iter() {
        if *button == MouseButton::Left && !state.is_pressed() {
            cursor.grabbed = None;
        }

        if q_egui
            .iter_mut()
            .any(|mut ctx| ctx.get_mut().wants_pointer_input())
        {
            continue;
        }

        if *button == MouseButton::Left && state.is_pressed() {
            match cursor.mode {
                CursorMode::Normal => {
                    if let Some(entity) = get_closest_grab(&cursor, &q_grab) {
                        cursor.grabbed = Some(entity);
                    }
                    else {
                        cursor.grabbed = Some(q_camera.single().0);
                    }
                }
                CursorMode::CreateNode => {
                    ev_add_node.send(AddNodeEvent(cursor.world_pos));
                }
                CursorMode::CreateEdge => {
                    if let Some(entity) = get_closest_grab(&cursor, &q_grab) {
                        if let Some(selected_entity) = cursor.selected {
                            ev_add_edge.send(AddEdgeEvent(selected_entity, entity));
                            cursor.selected = None;
                        }
                        else {
                            cursor.selected = Some(entity);
                        }
                    }
                }
                CursorMode::Remove => {
                    if let Some(entity) = get_closest_grab(&cursor, &q_grab) {
                        ev_remove_graph_item.send(RemoveItemEvent(entity));
                    }
                }
                CursorMode::Paint => {
                    if let Some(entity) = get_closest_grab(&cursor, &q_grab) {
                        // graph.paint(&mut commands, );
                    }
                }
            }
        }
    }
}

pub fn mouse_scroll_input(mut scroll_evr: EventReader<MouseWheel>) {}
