use ram_monitor::file_object::{identity_token, parse_identity_token, read_object};
use ram_monitor::integrity::compare_files;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

fn sandbox() -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/file-object/runtime");
    fs::create_dir_all(&root).expect("Disk2 experiment root");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time")
        .as_nanos();
    let path = root.join(format!("cli-{}-{now}", std::process::id()));
    fs::create_dir(&path).expect("unique sandbox");
    path
}

fn capture(path: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ram-monitor"))
        .args(["--source", "object-capture", "--observed-file"])
        .arg(path)
        .output()
        .expect("launch CLI")
}

fn check(path: &Path, token: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ram-monitor"))
        .args(["--source", "object-check", "--observed-file"])
        .arg(path)
        .args(["--identity-token", token])
        .output()
        .expect("launch CLI")
}

fn success_text(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 CLI output")
}

#[test]
fn real_file_object_continuity_complements_identical_content() {
    let root = sandbox();
    let watched = root.join("watched");
    let moved = root.join("moved");
    let copied = root.join("copied");
    let staged = root.join("staged");
    let reference = root.join("reference");
    let alias = root.join("alias");
    let bytes = [0x3c_u8; 160];
    fs::write(&watched, bytes).expect("write watched fixture");
    fs::write(&reference, bytes).expect("write trusted fixture");
    let direct = read_object(&watched).expect("baseline metadata");
    let output = success_text(capture(&watched));
    let token = output
        .split_whitespace()
        .next()
        .and_then(|word| word.strip_prefix("object_identity="))
        .expect("identity token")
        .to_string();
    assert_eq!(
        parse_identity_token(&token).expect("parse token"),
        direct.identity
    );
    assert_eq!(token, identity_token(direct.identity));
    assert!(success_text(check(&watched, &token)).contains("object_continuity=same"));

    fs::rename(&watched, &moved).expect("rename own fixture");
    assert!(success_text(check(&moved, &token)).contains("object_continuity=same"));
    fs::hard_link(&moved, &watched).expect("hardlink own fixture");
    assert!(success_text(check(&watched, &token)).contains("object_continuity=same"));

    fs::copy(&watched, &copied).expect("copy identical bytes");
    assert!(success_text(check(&copied, &token)).contains("object_continuity=different"));
    let material_before = compare_files(&reference, &watched).expect("same-content Atom baseline");
    assert_eq!(material_before.different_bytes, 0);

    fs::write(&staged, bytes).expect("stage identical replacement bytes");
    fs::rename(&staged, &watched).expect("atomic replacement at watched path");
    let material_after = compare_files(&reference, &watched).expect("same-content Atom after");
    assert_eq!(material_after, material_before);
    assert!(success_text(check(&watched, &token)).contains("object_continuity=different"));
    assert!(success_text(check(&moved, &token)).contains("object_continuity=same"));
    symlink(&watched, &alias).expect("symlink current watched path");
    assert!(success_text(check(&alias, &token)).contains("object_continuity=different"));

    for bad in [
        "",
        "1:2",
        "1:2:3:4",
        "1:x:3",
        " 1:2:3",
        "1:2:-3",
        "18446744073709551616:2:3",
    ] {
        let output = check(&watched, bad);
        assert_eq!(output.status.code(), Some(3), "{bad}");
        assert!(output.stdout.is_empty());
    }
    assert_eq!(capture(&root).status.code(), Some(3));
    assert_eq!(capture(&root.join("missing")).status.code(), Some(3));
    fs::remove_dir_all(&root).expect("remove only generated sandbox");
}

#[test]
fn object_modes_require_explicit_and_compatible_arguments() {
    let executable = env!("CARGO_BIN_EXE_ram-monitor");
    for args in [
        vec!["--source", "object-capture"],
        vec!["--source", "object-check"],
        vec!["--source", "replay", "--identity-token", "1:2:3"],
    ] {
        let output = Command::new(executable)
            .args(args)
            .output()
            .expect("launch CLI");
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}
