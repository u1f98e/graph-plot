use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseButtonInput, MouseScrollUnit, MouseWheel},
    },
    prelude::*,
};
use bevy_egui::EguiContext;

use crate::{graph::event::*, types::*};
use crate::{graph::Grabbable, MainCamera};

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub enum CursorMode {
    #[default]
    Normal,
    CreateNode,
    CreateEdge,
    Remove,
    Paint,
    Info
}

impl core::fmt::Display for CursorMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CursorMode::Normal => write!(f, "Drag/Pan"),
            CursorMode::Info => write!(f, "Node Info"),
            CursorMode::CreateNode => write!(f, "Create Node"),
            CursorMode::CreateEdge => write!(f, "Create Edge"),
            CursorMode::Remove => write!(f, "Erase"),
            CursorMode::Paint => write!(f, "Paint"),
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
    pub paint_color: Color,
}

impl CursorInfo {
    pub fn set_mode(&mut self, mode: &CursorMode, ev_selected: &mut EventWriter<ItemSelectedEvent>) {
        self.mode = *mode;

        if *mode != CursorMode::Normal {
            ev_selected.send(ItemSelectedEvent::Deselected);
        }
    }
}

fn egui_has_pointer(query: &mut Query<&mut EguiContext>) -> bool {
    query.iter_mut().any(|mut ctx| {
        let ctx = ctx.get_mut();
        ctx.wants_pointer_input() || ctx.is_pointer_over_area()
    })
}

fn egui_has_keyboard(query: &mut Query<&mut EguiContext>) -> bool {
    query
        .iter_mut()
        .any(|mut ctx| ctx.get_mut().wants_keyboard_input())
}

pub(crate) fn key_input_sys(
    mut key_evr: EventReader<KeyboardInput>,
    mut cursor: ResMut<CursorInfo>,
    mut q_egui: Query<&mut EguiContext>,
    mut ev_regen: EventWriter<RegenEdgeMesh>,
    mut ev_selected: EventWriter<ItemSelectedEvent>,
) {
    for KeyboardInput { key_code, state, ..  } in key_evr.iter()
    {
        if egui_has_keyboard(&mut q_egui) {
            continue;
        }

        if state.is_pressed() {
            if let Some(key) = key_code {
                match key {
                    KeyCode::S => cursor.set_mode(&CursorMode::CreateNode, &mut ev_selected),
                    KeyCode::E => cursor.set_mode(&CursorMode::CreateEdge, &mut ev_selected),
                    KeyCode::W => cursor.set_mode(&CursorMode::Normal, &mut ev_selected),
                    KeyCode::D => cursor.set_mode(&CursorMode::Remove, &mut ev_selected),
                    KeyCode::A => cursor.set_mode(&CursorMode::Paint, &mut ev_selected),
                    KeyCode::R => {
                        ev_regen.send(RegenEdgeMesh());
                    }
                    _ => (),
                }
            }
        }
    }
}

pub fn mouse_movement_sys(
    mut cursor_evr: EventReader<CursorMoved>,
    mut cursor: ResMut<CursorInfo>,
    mut q_camera: Query<
        (Entity, &Camera, &mut Transform, &OrthographicProjection, &GlobalTransform),
        With<crate::MainCamera>,
    >,
    mut q_grab: Query<(Entity, &mut Transform), (With<Grabbable>, Without<Camera>)>,
    mut ev_move_item: EventWriter<ItemMovedEvent>,
) {
    let (camera_e, camera, mut camera_tf, proj, camera_global_tf) = q_camera.single_mut();

    for CursorMoved { position, .. } in cursor_evr.iter() {
        let ray = camera.viewport_to_world(camera_global_tf, *position);
        let mut cursor_delta = Vec2::ZERO;
        if let Some(ray) = ray {
            cursor_delta = (*position - cursor.screen_pos) * proj.scale;
            cursor.screen_pos = *position;
            cursor.world_pos = ray.origin.truncate();
        }

        if let Some(entity) = cursor.grabbed {
            if camera_e == entity {
                camera_tf.translation.x -= cursor_delta.x;
                camera_tf.translation.y += cursor_delta.y;
            } else {
                let mut transform = q_grab.get_mut(entity).unwrap().1;
                transform.translation = Vec3::new(cursor.world_pos.x, cursor.world_pos.y, 0.0);
                ev_move_item.send(ItemMovedEvent(
                    entity,
                    Vec3::new(cursor_delta.x, -cursor_delta.y, 0.0),
                ));
            }
        }
    }
}

fn get_closest_grab<'a, I>(cursor: &CursorInfo, q_grab_iter: I) -> Option<Entity>
where
    I: Iterator<Item = (Entity, &'a Grabbable, &'a Transform)>,
{
    let mut closest_distance = f32::INFINITY;
    let mut closest_grab: Option<Entity> = None;

    for (entity, grab, transform) in q_grab_iter {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);
        let distance = cursor.world_pos.distance(pos);
        match grab {
            Grabbable::Circle { radius } => {
                if distance < *radius && distance < closest_distance {
                    closest_distance = distance;
                    closest_grab = Some(entity);
                }
            }
            _ => {
                if closest_grab.is_none() {
                    // Only grab rect if no other options
                    closest_grab = Some(entity)
                }
            }
        }
    }

    closest_grab
}

pub(crate) fn mouse_button_sys(
    mut ev_mouse_button: EventReader<MouseButtonInput>,
    mut cursor: ResMut<CursorInfo>,
    query: (
        Query<&mut EguiContext>,
        Query<(Entity, &crate::graph::Grabbable, &mut Transform), GNodeExclusive>,
        Query<(Entity, &crate::graph::Grabbable, &mut Transform), GEdgeExclusive>,
        Query<(Entity, &mut Sprite)>,
        Query<Entity, With<MainCamera>>,
    ),
    mut ev_add_node: EventWriter<AddNodeEvent>,
    mut ev_add_edge: EventWriter<AddEdgeEvent>,
    mut ev_remove_graph_item: EventWriter<RemoveItemEvent>,
    mut ev_regen_mesh: EventWriter<RegenEdgeMesh>,
    mut ev_selected: EventWriter<ItemSelectedEvent>,
) {
    let (mut q_egui, q_node, q_handle, mut q_sprite, q_camera) = query;

    for MouseButtonInput { button, state, .. } in ev_mouse_button.iter() {
        if *button == MouseButton::Left && !state.is_pressed() {
            cursor.grabbed = None;
        }

        if egui_has_pointer(&mut q_egui) {
            continue;
        }

        let q_grab_combined = q_node.iter().chain(q_handle.iter());
        if *button == MouseButton::Left && state.is_pressed() {
            match cursor.mode {
                CursorMode::Normal => {
                    if let Some(entity) = get_closest_grab(&cursor, q_grab_combined) {
                        cursor.grabbed = Some(entity);
                    } else {
                        cursor.grabbed = Some(q_camera.single());
                    }
                }
                CursorMode::CreateNode => {
                    ev_add_node.send(AddNodeEvent(cursor.world_pos));
                }
                CursorMode::CreateEdge => {
                    if let Some(entity) = get_closest_grab(&cursor, q_node.iter()) {
                        if let Some(selected_entity) = cursor.selected {
                            ev_add_edge.send(AddEdgeEvent(selected_entity, entity));
                            ev_selected.send(ItemSelectedEvent::Deselected);
                        } else {
                            cursor.selected = Some(entity);
                            ev_selected.send(ItemSelectedEvent::Selected(entity));
                        }
                    }
                }
                CursorMode::Remove => {
                    if let Some(entity) = get_closest_grab(&cursor, q_grab_combined) {
                        ev_remove_graph_item.send(RemoveItemEvent(entity));
                    }
                }
                CursorMode::Paint => {
                    if let Some(entity) = get_closest_grab(&cursor, q_grab_combined) {
                        if let Ok((_, mut sprite)) = q_sprite.get_mut(entity) {
                            sprite.color = cursor.paint_color;
                            ev_regen_mesh.send(RegenEdgeMesh());
                        }
                    }
                }
                CursorMode::Info => {
                    if let Some(entity) = get_closest_grab(&cursor, q_grab_combined) {
                        ev_selected.send(ItemSelectedEvent::Selected(entity));
                    }
                }
            }
        }
    }
}

pub fn mouse_scroll_input(
    mut scroll_evr: EventReader<MouseWheel>,
    mut q_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    for MouseWheel { unit, y, .. } in scroll_evr.iter() {
        let mut proj = q_camera.single_mut();
        match unit {
            MouseScrollUnit::Line => {
                proj.scale -= y * 0.1;
            }
            MouseScrollUnit::Pixel => {
                proj.scale -= y * 0.001;
            }
        }

        if proj.scale < 0.8 {
            proj.scale = 0.8;
        }
        else if proj.scale > 3.0 {
            proj.scale = 3.0;
        }
    }
}
