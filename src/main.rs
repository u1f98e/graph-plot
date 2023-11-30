use bevy::{
    prelude::*,
    render::{RenderPlugin, settings::{WgpuSettings, PowerPreference, RenderCreation}},
};
use bevy_egui::EguiPlugin;

mod graph;
mod input;
mod materials;
mod ui;
pub mod types;
// pub mod smatrix;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct TestMesh;

fn setup(
    mut commands: Commands,
) {
    commands.init_resource::<input::CursorInfo>();

    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(RenderPlugin {
            render_creation: RenderCreation::Automatic(WgpuSettings {
                power_preference: PowerPreference::LowPower,
                ..Default::default()
            })
        }))
        .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .add_plugins(EguiPlugin)
        .add_plugins(graph::plugin::GraphPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                input::key_input_sys,
                input::mouse_movement_sys,
                input::mouse_button_sys,
                input::mouse_scroll_input,
            ),
        )
        .run();
}
