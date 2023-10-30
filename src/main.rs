use std::sync::Arc;

use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{Material2dPlugin, MaterialMesh2dBundle}, asset::ChangeWatcher,
};
use edge::CurveMaterial;

mod edge;
mod graph;
mod input;

fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut curve_mat: ResMut<Assets<edge::CurveMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let text_style = TextStyle {
        font: assets.load("fonts/FiraSans-Regular.ttf"),
        font_size: 60.0,
        color: Color::WHITE,
    };

    commands.spawn(Camera2dBundle::default());

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

    let mat_test = curve_mat.add(edge::CurveMaterial { color: Color::RED, thickness: 2.0 });
    // let mat_test = materials.add(ColorMaterial::from(Color::PURPLE));

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
        .add_plugins(Material2dPlugin::<CurveMaterial>::default())
        .add_asset::<edge::CurveMaterial>()
        .add_systems(Startup, setup)
        .add_systems(Update, (update, input::key_input))
        .run();
}
