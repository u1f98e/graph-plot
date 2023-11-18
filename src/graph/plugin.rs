use std::collections::HashMap;

use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::view::NoFrustumCulling;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::{prelude::*, sprite::Material2dPlugin};

use crate::materials;

use super::event;
use super::event::*;
use super::Graph;

#[derive(Resource, Default, Deref)]
pub struct ImageCache(HashMap<String, Handle<Image>>);

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
            .add_event::<ItemSelectedEvent>()
            .add_systems(
                Update,
                (
                    event::add_node_event,
                    event::add_edge_event,
                    event::remove_item_event,
                    event::move_item_event,
                    event::item_selected_event,
                ),
            )
            .add_systems(PostUpdate, event::regen_edge_mesh);
    }
}

impl GraphPlugin {
    #[rustfmt::skip]
    fn init(
        assets: ResMut<AssetServer>,
        mut img_cache: ResMut<ImageCache>,
    ) {
        img_cache.0.insert("node".into(), assets.load("sprites/node30.png"));
        img_cache.0.insert("handle".into(), assets.load("sprites/handle30.png"));
        img_cache.0.insert("handle_directed".into(), assets.load("sprites/handle_directed30.png"));
        img_cache.0.insert("nodeSelected".into(), assets.load("sprites/node_selected30.png"));
        img_cache.0.insert("handleSelected".into(), assets.load("sprites/handle_selected30.png"));
    }

    fn init_graph(
        mut commands: Commands,
        mut meshes: ResMut<Assets<Mesh>>,
        mut curve_mats: ResMut<Assets<materials::CurveMaterial>>,
    ) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

        // The mesh refuses to render if we don't put placeholder values here
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
            vec![
                [1.0, 0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0, 0.5],
                [0.0, 0.0, 1.0, 1.0],
            ],
        );
        mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));

        let mat_test = curve_mats.add(materials::CurveMaterial { thickness: 2.0 });

        let edge_mesh_handle: Mesh2dHandle = meshes.add(mesh).into();
        let edge_mesh = commands
            .spawn((MaterialMesh2dBundle {
                material: mat_test,
                mesh: edge_mesh_handle.clone(),
                transform: Transform::default().with_translation(Vec3::new(0.0, 0.0, -1.0)),
                ..Default::default()
            }, NoFrustumCulling))
            .id();

        let graph = Graph {
            edge_mesh_handle,
            edge_mesh,
            adjacencies: HashMap::new(),
            degree: 0,
            directed: false
        };

        commands.insert_resource(graph);
    }
}
