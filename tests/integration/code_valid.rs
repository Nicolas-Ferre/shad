use shad_runner::Runner;
use std::fs;
use std::path::PathBuf;

#[rstest::rstest]
fn run_valid_code(#[files("./cases_valid/code/*")] path: PathBuf) {
    let mut runner = Runner::new(&path).unwrap();
    runner.run_step();
    let asg = runner.analysis();
    let mut buffers = asg
        .buffers
        .keys()
        .map(|buffer| {
            format!(
                "{}.{}={}",
                buffer.module,
                buffer.name,
                match runner.analysis().buffer_type(buffer).buffer_name.as_str() {
                    "i32" => format!("{}", to_i32(&runner.buffer(buffer))),
                    "u32" => format!("{}", to_u32(&runner.buffer(buffer))),
                    "f32" => format!("{}", to_f32(&runner.buffer(buffer))),
                    _ => format!("{:?}", runner.buffer(buffer)),
                }
            )
        })
        .collect::<Vec<_>>();
    buffers.sort_unstable();
    let actual = buffers.join("\n");
    let case_name = path.file_stem().unwrap();
    let buffers_path = path
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("expected/")
        .join(case_name);
    if buffers_path.exists() {
        assert_eq!(
            fs::read_to_string(buffers_path).unwrap(),
            actual,
            "mismatching result for valid {case_name:?} case",
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
