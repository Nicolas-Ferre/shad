use shad::Runner;
use std::fs;
use std::path::{Path, PathBuf};

#[rstest::rstest]
fn run_valid_code(
    #[dirs]
    #[files("./cases_valid/*")]
    path: PathBuf,
) {
    let program = shad::compile(Path::new(&path)).unwrap();
    let mut runner = Runner::new(program, None, Some((4, 3)));
    runner.run_step();
    let mut buffers = runner
        .program()
        .buffers
        .iter()
        .map(|(name, props)| {
            format!(
                "{}={}",
                name,
                match props.type_name.as_str() {
                    "i32" => format!("{}", to_i32(&runner.read(name))),
                    "u32" | "bool" => format!("{}", to_u32(&runner.read(name))),
                    "f32" => format!("{}", to_f32(&runner.read(name))),
                    _ => format!("{:?}", runner.read(name)),
                }
            )
        })
        .collect::<Vec<_>>();
    buffers.sort_unstable();
    let actual = buffers.join("\n");
    let buffers_path = path.join(".expected");
    if buffers_path.exists() {
        assert_eq!(
            fs::read_to_string(buffers_path).unwrap(),
            actual,
            "mismatching result for valid {:?} case",
            path.file_stem().unwrap(),
        );
    } else {
        fs::write(buffers_path, actual).unwrap();
        panic!("expected buffers saved on disk, please check and rerun the tests");
    }
}

fn to_f32(buffer: &[u8]) -> f32 {
    assert_eq!(buffer.len(), 4);
    let bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
    f32::from_ne_bytes(bytes)
}

fn to_u32(buffer: &[u8]) -> u32 {
    assert_eq!(buffer.len(), 4);
    let bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
    u32::from_ne_bytes(bytes)
}

fn to_i32(buffer: &[u8]) -> i32 {
    assert_eq!(buffer.len(), 4);
    let bytes = [buffer[0], buffer[1], buffer[2], buffer[3]];
    i32::from_ne_bytes(bytes)
}
