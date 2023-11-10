use std::collections::HashMap;

use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{prelude::*, sprite::Material2dPlugin};

use crate::materials;

use super::event;
use super::event::*;
use super::Graph;

#[derive(Resource, Default)]
pub struct ImageCache(pub(super) HashMap<String, Handle<Image>>);

pub struct GraphPlugin;
impl Plugin for GraphPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<materials::CurveMaterial>::default())
            .add_asset::<materials::CurveMaterial>()
            .add_systems(Startup, (GraphPlugin::init, GraphPlugin::init_graph))
            .init_resource::<ImageCache>();

        app.add_event::<AddNodeEvent>()
            .add_event::<AddEdgeEvent>()
            .add_event::<RemoveItemEvent>()
            .add_event::<ItemMovedEvent>()
            .add_event::<RegenEdgeMesh>()
            .add_systems(
                Update,
                (
                    event::add_node_event,
                    event::add_edge_event,
                    event::remove_item_event,
                    event::move_item_event,
					event::regen_edge_mesh,
                ),
            );
    }
}

impl GraphPlugin {
    fn init(
        assets: ResMut<AssetServer>,
        mut img_cache: ResMut<ImageCache>,
    ) {
        img_cache
            .0
            .insert("node".into(), assets.load("sprites/node30.png"));
        img_cache
            .0
            .insert("handle".into(), assets.load("sprites/handle30.png"));
    }

	fn init_graph(
		mut commands: Commands,
		mut meshes: ResMut<Assets<Mesh>>,
		mut curve_mats: ResMut<Assets<materials::CurveMaterial>>,
	) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        // mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, Vec::<[f32; 3]>::new());
        // mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, Vec::<[f32; 2]>::new());
        // // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, Vec::<[f32; 4]>::new());
        // mesh.set_indices(None);

        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![[-0.5, -0.5, 0.0], [0.5, -0.5, 0.0], [0.0, 0.5, 0.0]],
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[0.0, 0.0], [1.0, 1.0], [0.5, 0.0]],
        );
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_COLOR,
            vec![[1.0, 0.0, 0.0, 1.0], [0.0, 1.0, 0.0, 0.5], [0.0, 0.0, 1.0, 1.0]]
        );
        mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));

        let mat_test = curve_mats.add(materials::CurveMaterial {
            color: Color::BLUE,
            thickness: 2.0,
        });

        let edge_mesh_handle: Mesh2dHandle = meshes.add(mesh).into();
        commands.spawn(MaterialMesh2dBundle {
            material: mat_test,
            mesh: edge_mesh_handle.clone(),
            transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, -1.0)),
            ..Default::default()
        });

        let graph = Graph {
            edge_mesh_handle,
			..Default::default()
        };

		commands.insert_resource(graph);
	}
}
