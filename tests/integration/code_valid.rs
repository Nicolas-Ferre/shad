use shad_analyzer::TypeResolving;
use shad_runner::Runner;
use std::fs;

#[test]
fn run_valid_code() {
    let mut should_rerun = false;
    for entry in fs::read_dir("./cases_valid/code").unwrap() {
        let code_path = entry.unwrap().path();
        let mut runner = Runner::new(&code_path).unwrap();
        runner.run_step();
        let asg = runner.asg();
        let mut buffers = asg
            .buffers
            .iter()
            .map(|(buffer_name, buffer)| {
                match buffer
                    .expr
                    .as_ref()
                    .unwrap()
                    .type_(asg)
                    .unwrap()
                    .buf_final_name
                    .as_str()
                {
                    "i32" => format!("{}={}", buffer_name, to_i32(&runner.buffer(buffer_name))),
                    "u32" => format!("{}={}", buffer_name, to_u32(&runner.buffer(buffer_name))),
                    "f32" => format!("{}={}", buffer_name, to_f32(&runner.buffer(buffer_name))),
                    _ => format!("{}={:?}", buffer_name, runner.buffer(buffer_name)),
                }
            })
            .collect::<Vec<_>>();
        buffers.sort_unstable();
        let expected = buffers.join("\n");
        let case_name = code_path.file_stem().unwrap();
        let buffers_path = code_path
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("expected/")
            .join(case_name);
        if buffers_path.exists() {
            assert_eq!(
                fs::read_to_string(buffers_path).unwrap(),
                expected,
                "mismatching result for valid {case_name:?} case",
            );
        } else {
            fs::write(buffers_path, expected).unwrap();
            should_rerun = true;
        }
    }
    assert!(
        !should_rerun,
        "expected buffers saved on disk, please check and rerun the tests"
    );
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
