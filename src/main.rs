use bevy::{
    asset::ChangeWatcher,
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_egui::EguiPlugin;

mod graph;
mod input;
mod materials;
mod ui;

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
pub struct TestMesh;

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

    commands.spawn((
        Camera2dBundle {
            // transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        MainCamera,
    ));

    commands.spawn(Text2dBundle {
        text: Text::from_section("Test", text_style.clone()).with_alignment(TextAlignment::Center),
        ..Default::default()
    });

    let mut mesh_test = Mesh::new(PrimitiveTopology::TriangleList);
    mesh_test.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]]
    );
    mesh_test.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0], [1.0, 1.0], [0.5, 0.0]]
    );
    mesh_test.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.5], [0.0, 0.0, 1.0, 1.0]]
    );
    mesh_test.set_indices(Some(Indices::U32(vec![0, 1, 2])));

    let mesh_test_handle = meshes.add(mesh_test);

    let mat_test = curve_mat.add(materials::CurveMaterial {
        color: Color::RED,
        thickness: 2.0,
    });

    commands.spawn((
        MaterialMesh2dBundle {
            material: mat_test,
            mesh: mesh_test_handle.into(),
            transform: Transform::default().with_scale(Vec3::splat(128.0)),
            ..Default::default()
        },
        TestMesh,
    ));
}

fn update(
    time: Res<Time>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    q_mesh: Query<&Mesh2dHandle, With<TestMesh>>,
) {
    let mesh_e = q_mesh.iter().next().unwrap();
    let mesh: &mut Mesh = meshes.get_mut(&mesh_e.0).unwrap();

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0],
        [-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, -1.0, 0.0]],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        vec![[0.0, 0.0], [1.0, 1.0], [0.5, 0.0],
        [0.0, 0.0], [1.0, 1.0], [0.5, 0.0]],
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_COLOR,
        vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.5], [0.0, 0.0, 1.0, 1.0],
        [1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.5], [0.0, 0.0, 1.0, 1.0]]
    );
    mesh.set_indices(Some(Indices::U32(vec![0, 1, 2, 3, 4, 5])));

    // mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap().as_float3().unwrap().clone_into(&mut positions);
    // positions[0][1] += time.delta_seconds() * 0.1;
    // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions)
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_secs(2)),
            ..default()
        }))
        // .insert_resource(bevy::winit::WinitSettings::desktop_app())
        .insert_resource(bevy::winit::WinitSettings {
            focused_mode: bevy::winit::UpdateMode::Continuous,
            unfocused_mode: bevy::winit::UpdateMode::ReactiveLowPower {
                max_wait: std::time::Duration::from_millis(10),
            },
            ..default()
        })
        .add_plugins(EguiPlugin)
        .add_plugins(graph::plugin::GraphPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                update,
                ui::egui_sys,
                input::key_input_sys,
                input::mouse_movement_sys,
                input::mouse_button_sys,
            ),
        )
        .run();
}
