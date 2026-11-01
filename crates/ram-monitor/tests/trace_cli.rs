use std::process::Command;

fn path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("ram-monitor-{}-{name}.trace", std::process::id()))
}

#[test]
fn uploaded_trace_cli_trips_and_malformed_trace_fails_closed() {
    let good = path("good");
    let bad = path("bad");
    let mut text = String::from("# time_ns,address\n");
    for index in 0..130_u64 {
        text.push_str(&format!("{},0x{:x}\n", index * 50, (3 << 6) + index % 64));
    }
    std::fs::write(&good, text).unwrap();
    std::fs::write(&bad, "10,0x1000\n9,0x1040\n").unwrap();

    let binary = env!("CARGO_BIN_EXE_ram-monitor");
    let output = Command::new(binary)
        .args(["--source", "trace", "--trace-file"])
        .arg(&good)
        .args(["--samples", "1000"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("samples=130"), "{stdout}");
    assert!(stdout.contains("trips="), "{stdout}");
    assert!(!stdout.contains("trips=0"), "{stdout}");

    let rejected = Command::new(binary)
        .args(["--source", "trace", "--trace-file"])
        .arg(&bad)
        .output()
        .unwrap();
    assert_eq!(rejected.status.code(), Some(3));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("timestamp moved backward"));

    std::fs::remove_file(good).unwrap();
    std::fs::remove_file(bad).unwrap();
}
