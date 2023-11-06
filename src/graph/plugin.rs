use std::collections::HashMap;

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
            .add_systems(Startup, GraphPlugin::init)
            .init_resource::<ImageCache>();

        app.add_event::<AddNodeEvent>()
            .add_event::<AddEdgeEvent>()
            .add_event::<RemoveItemEvent>()
            .add_event::<ItemMovedEvent>()
            .add_systems(
                Update,
                (
                    event::add_node_event,
                    event::add_edge_event,
                    event::remove_item_event,
                    event::move_item_event,
                ),
            );
    }
}

impl GraphPlugin {
    fn init(
        mut commands: Commands,
        assets: ResMut<AssetServer>,
        mut img_cache: ResMut<ImageCache>,
        mut meshes: ResMut<Assets<Mesh>>,
    ) {
        img_cache
            .0
            .insert("node".into(), assets.load("sprites/node30.png"));
        img_cache
            .0
            .insert("handle".into(), assets.load("sprites/handle.png"));

		commands.insert_resource(Graph::new(&mut meshes));
    }
}
