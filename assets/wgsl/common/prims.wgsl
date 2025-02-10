#define_import_path prims

@export struct Primitive {
    color: vec4<f32>,
    translate: vec2<f32>,
    z_index: i32,
	_pad: u32,
};