use std::sync::Arc;

use bevy::{
    asset::ChangeWatcher,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{Material2dPlugin, MaterialMesh2dBundle},
};
use bevy_egui::{EguiContexts, EguiPlugin, egui};
use materials::CurveMaterial;

mod materials;
mod graph;
mod input;
mod ui;

#[derive(Component)]
pub struct MainCamera;

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut curve_mat: ResMut<Assets<materials::CurveMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.init_resource::<input::CursorInfo>();

    let text_style = TextStyle {
        font: assets.load("fonts/FiraSans-Regular.ttf"),
        font_size: 60.0,
        color: Color::WHITE,
    };

    commands.spawn((Camera2dBundle::default(), MainCamera));

    commands.spawn(Text2dBundle {
        text: Text::from_section("Test", text_style.clone()).with_alignment(TextAlignment::Center),
        ..Default::default()
    });

    let mut mesh_test = Mesh::new(PrimitiveTopology::TriangleList);
    mesh_test.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]],
    );
    mesh_test.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0], [1.0, 1.0], [0.5, 0.0]],
    );
    mesh_test.set_indices(Some(Indices::U32(vec![0, 1, 2])));

    let mesh_test_handle = meshes.add(mesh_test);

    let mat_test = curve_mat.add(materials::CurveMaterial {
        color: Color::RED,
        thickness: 2.0,
    });

    commands.spawn(MaterialMesh2dBundle {
        material: mat_test,
        mesh: mesh_test_handle.into(),
        transform: Transform::default().with_scale(Vec3::splat(128.0)),
        ..Default::default()
    });
}

fn update() {}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_secs(2)),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        .add_plugins(graph::plugin::GraphPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (update, ui::egui_sys, input::key_input_sys, input::mouse_movement_sys, input::mouse_button_sys),
        )
        .run();
}
