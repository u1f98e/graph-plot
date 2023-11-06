use bevy::{prelude::*, render::render_resource::{AsBindGroup, ShaderRef}, reflect::{TypePath, TypeUuid}, sprite::Material2d};

#[derive(Default, AsBindGroup, TypePath, TypeUuid, Debug, Clone)]
#[uuid = "172eee85-2e56-4e77-972a-6c040d366ccb"]
pub struct CurveMaterial {
	#[uniform(0)]
	pub color: Color,
	#[uniform(0)]
	pub thickness: f32,
}

impl Material2d for CurveMaterial {
	fn fragment_shader() -> ShaderRef {
		"shaders/edge.wgsl".into()
	}
}