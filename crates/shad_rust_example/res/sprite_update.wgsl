struct Sprite {
    position: vec2<f32>,
    size: vec2<f32>,
    rotation: f32,
    color: vec4<f32>,
};

@group(0)
@binding(0)
var<storage, read_write> sprites: array<Sprite>;

@group(1)
@binding(0)
var<uniform> delta: f32;

@compute
@workgroup_size(256, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    sprites[i].position += vec2<f32>(0.1 * delta, 0.1 * delta);
}
