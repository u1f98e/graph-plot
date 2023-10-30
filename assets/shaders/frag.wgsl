struct VertexOut {
	@builtin(position) clip_pos: vec4<f32>,
	@location(0) color: vec4<f32>,
	@location(1) tex_coord: vec2<f32>,
}

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
	return quadratic_curve(in.tex_coord, in.color);
}

@fragment
fn fs_line(in: VertexOut) -> @location(0) vec4<f32> {
	return vec4<f32>(1.0, 0.0, 1.0, 1.0);
}

@fragment
fn fs_point(in: VertexOut) -> @location(0) vec4<f32> {
	return vec4<f32>(0.0, 1.0, 0.0, 1.0);
}