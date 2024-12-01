struct Sprite {
    position: vec2<f32>,
    size: vec2<f32>,
    rotation: atomic<f32>,
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
    var a: atomic<i32>;
    atomicStore(&a, 0);
    let i = id.x;
    sprites[i].position += vec2<f32>(0.05 * delta, 0.05 * delta);
    sprites[i].rotation = wrap_angle(sprites[i].rotation + 0.5 * delta);
}

fn wrap_angle(radians: f32) -> f32 {
    return atan2(sin(radians),cos(radians));
}
