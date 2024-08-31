fn main() {
    let initial = 21.;
    let value = double(initial);
    loop {
        print(value);
    }
}

fn double(value: f32) -> f32 {
    value *= 2;
    return value;
}
