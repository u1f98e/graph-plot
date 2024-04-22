use bevy::{
    prelude::*,
    render::{RenderPlugin, settings::{WgpuSettings, PowerPreference, RenderCreation}},
};
use bevy_egui::{EguiPlugin, EguiSettings};

mod graph;
mod input;
mod materials;
mod ui;
pub mod types;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct TestMesh;

fn setup(
    mut commands: Commands,
    mut egui_settings: ResMut<EguiSettings>,
) {
    commands.init_resource::<input::CursorInfo>();

    commands.spawn((
        Camera2dBundle::default(),
        MainCamera,
    ));

    egui_settings.scale_factor = 1.25;
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
        .insert_resource(ClearColor(Color::rgb(0.3, 0.3, 0.3)))
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
