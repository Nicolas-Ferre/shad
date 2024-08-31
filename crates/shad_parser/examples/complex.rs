// Entrypoint of the program.
// All possible sections of this method are run in compute shaders, and the rest is interpreted on CPU side.
fn main() {
    // Initialisation
    let rects = create_rects();
    // "Game" loop
    loop {
        // Only `range<int>` is supported in `for` loops for the moment.
        // Builtin `range()` method accepts only a `uint` type as input for the moment.
        // Builtin `len()` method returns a `uint` (as we don't expect more than 4B items in an array).
        for i in range(len(rects)) {
            update_rect(rect[i]); // Will be run in parallel on GPU side.
        }
        update_rect(rects[0]);
        render(rects); // For the moment, it is only possible to render rects.
    }
}

fn create_rects() -> array<Rect> {
    // Type is always inferred without ambiguity, so no syntax to specify the variable type.
    let rects = [rect(); 2];
    rects[0] = create_rect(vec2(0., 0.));
    rects[1] = create_rect(vec2(0.5, -0.5));
    return rects;
}

fn create_rect(position: vec2f) -> Rect {
    let rect = rect();
    rect.position = position;
    rect.size = vec2(0.5, 0.5);
    rect.rotation = 0.;
    rect.color = vec3(1., 1., 1.);
    return rect;
}

// Method params and return value use reference semantic (references are always valid).
// For variables, copy semantics is always used (compiler may optimize in some cases).
fn update_rect(rect: Rect) -> Rect {
    let speed = 0.1; // unambiguous decimal type: `float`, use `0.1d` for `double` type
    rect.position = rect.position + vec2(1., 1.) * speed;
    return rect;
}

// cpu fn render(rects: array<Rect>);
// cpu fn show();
// gpu fn vec2(x: float, y: float) -> vec2f;
// gpu fn vec3(x: float, y: float, z: float) -> vec3f;
