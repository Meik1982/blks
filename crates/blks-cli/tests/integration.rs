use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn get_bin_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // drop test binary name
    if path.ends_with("deps") {
        path.pop();
    }
    path.push("blks");
    path.to_str().unwrap().to_string()
}

#[test]
fn test_cli_stdin_and_exact_64_chars() {
    let bin = get_bin_path();
    let mut child = Command::new(&bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("failed to spawn blks");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        stdin.write_all(b"Hello, blks-384 tree hash!").unwrap();
    }

    let output = child.wait_with_output().expect("failed to read stdout");
    assert!(output.status.success());

    let stdout_str = String::from_utf8(output.stdout).unwrap();
    let parts: Vec<&str> = stdout_str.split_whitespace().collect();
    assert_eq!(parts.len(), 2);
    let hash = parts[0];
    let file = parts[1];

    assert_eq!(file, "-");
    assert_eq!(hash.len(), 64, "Base64 hash must be exactly 64 characters");
    assert!(!hash.contains('='), "Must have zero padding");
}

#[test]
fn test_cli_file_hash_and_hex() {
    let bin = get_bin_path();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("blks_test_sample.bin");

    let sample_data = vec![0x42u8; 100_000]; // 100 KB
    fs::write(&test_file, &sample_data).unwrap();

    // Standard Base64
    let output_b64 = Command::new(&bin)
        .arg(&test_file)
        .output()
        .expect("execute blks");
    assert!(output_b64.status.success());
    let b64_str = String::from_utf8(output_b64.stdout).unwrap();
    let parts: Vec<&str> = b64_str.split_whitespace().collect();
    assert_eq!(parts[0].len(), 64);

    // Hexadecimal
    let output_hex = Command::new(&bin)
        .arg("--hex")
        .arg(&test_file)
        .output()
        .expect("execute blks --hex");
    assert!(output_hex.status.success());
    let hex_str = String::from_utf8(output_hex.stdout).unwrap();
    let hex_parts: Vec<&str> = hex_str.split_whitespace().collect();
    assert_eq!(
        hex_parts[0].len(),
        96,
        "Hex digest must be exactly 96 chars"
    );

    // Clean up
    let _ = fs::remove_file(test_file);
}

#[test]
fn test_cli_check_verification() {
    let bin = get_bin_path();
    let temp_dir = std::env::temp_dir();
    let file_a = temp_dir.join("blks_check_a.txt");
    let file_b = temp_dir.join("blks_check_b.txt");
    let check_file = temp_dir.join("blks_checksums.txt");

    fs::write(&file_a, b"File A content").unwrap();
    fs::write(&file_b, b"File B content").unwrap();

    // Compute checksums
    let out = Command::new(&bin)
        .arg(&file_a)
        .arg(&file_b)
        .output()
        .unwrap();
    assert!(out.status.success());
    fs::write(&check_file, &out.stdout).unwrap();

    // Verify: should succeed
    let check_out = Command::new(&bin)
        .arg("-c")
        .arg(&check_file)
        .output()
        .unwrap();
    assert!(check_out.status.success());
    let check_str = String::from_utf8(check_out.stdout).unwrap();
    assert!(check_str.contains("blks_check_a.txt: OK"));
    assert!(check_str.contains("blks_check_b.txt: OK"));

    // Corrupt File B
    fs::write(&file_b, b"Corrupted content!").unwrap();
    let fail_out = Command::new(&bin)
        .arg("-c")
        .arg(&check_file)
        .output()
        .unwrap();
    assert!(!fail_out.status.success());
    let fail_str = String::from_utf8(fail_out.stdout).unwrap();
    assert!(fail_str.contains("blks_check_a.txt: OK"));
    assert!(fail_str.contains("blks_check_b.txt: FAILED"));

    // Clean up
    let _ = fs::remove_file(file_a);
    let _ = fs::remove_file(file_b);
    let _ = fs::remove_file(check_file);
}

#[test]
fn test_cli_json_telemetry() {
    let bin = get_bin_path();
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("blks_json_sample.bin");
    fs::write(&test_file, b"JSON telemetry test data").unwrap();

    let output = Command::new(&bin)
        .arg("--json")
        .arg(&test_file)
        .output()
        .unwrap();
    assert!(output.status.success());

    let json_str = String::from_utf8(output.stdout).unwrap();
    assert!(json_str.contains(r#""algo":"blks-384""#));
    assert!(json_str.contains(r#""bits":384"#));
    assert!(json_str.contains(r#""collision_bits":192"#));

    let _ = fs::remove_file(test_file);
}
