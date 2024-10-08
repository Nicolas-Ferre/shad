struct Vertex {
    @location(0)
    position: vec3<f32>,
};

struct Sprite {
    @location(1)
    position: vec2<f32>,
    @location(2)
    size: vec2<f32>,
    @location(3)
    rotation: f32,
    @location(4)
    color: vec4<f32>,
};

struct Fragment {
    @builtin(position)
    position: vec4<f32>,
    @location(0)
    color: vec4<f32>,
};

@vertex
fn vs_main(vertex: Vertex, sprite: Sprite) -> Fragment {
    let pos = vec2(
        vertex.position.x * sprite.size.x,
        vertex.position.y * sprite.size.y,
    );
    return Fragment(
        vec4<f32>(
            pos.x * cos(sprite.rotation) - pos.y * sin(sprite.rotation) + sprite.position.x,
            pos.x * sin(sprite.rotation) + pos.y * cos(sprite.rotation) + sprite.position.y,
            vertex.position.z,
            1.
        ),
        sprite.color
    );
}

@fragment
fn fs_main(fragment: Fragment) -> @location(0) vec4<f32> {
    return fragment.color;
}
