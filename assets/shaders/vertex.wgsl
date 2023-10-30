struct VertexIn {
	@location(0) pos: vec2<f32>,
	@location(1) color: vec4<f32>,
	@location(2) tex_coord: vec2<f32>,
}

struct VertexOut {
	@builtin(position) clip_pos: vec4<f32>,
	@location(0) color: vec4<f32>,
	@location(1) tex_coord: vec2<f32>,
}

@vertex
fn vs_main(
	model: VertexIn
) -> VertexOut {
	var out: VertexOut;
	out.clip_pos = vec4<f32>(model.pos, 0.0, 1.0);
	out.color = model.color;
	out.tex_coord = model.tex_coord;
	return out;
}

@vertex
fn vs_line(
	model: VertexIn
) -> VertexOut {
	var out: VertexOut;
	out.clip_pos = vec4<f32>(model.pos, 0.0, 1.0);
	out.color = model.color;
	out.tex_coord = model.tex_coord;
	return out;
}

@vertex
fn vs_point(
	model: VertexIn
) -> VertexOut {
	var out: VertexOut;
	out.clip_pos = vec4<f32>(model.pos, 0.0, 1.0);
	out.color = model.color;
	out.tex_coord = model.tex_coord;
	return out;
}