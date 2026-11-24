use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ram_monitor::file_object::{read_object, ObjectReadError};

fn sandbox() -> std::path::PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/file-object/runtime");
    fs::create_dir_all(&root).expect("create Disk 2 test root");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("current time")
        .as_nanos();
    let path = root.join(format!("object-{}-{now}", std::process::id()));
    fs::create_dir(&path).expect("create isolated test directory");
    path
}

#[test]
fn real_rename_hardlink_copy_and_atomic_replacement_have_distinct_readings() {
    let root = sandbox();
    let original = root.join("original");
    let moved = root.join("moved");
    let linked = root.join("linked");
    let copied = root.join("copied");
    let alias = root.join("alias");
    let staged = root.join("staged");
    let bytes = b"same bytes across file-object operations";
    fs::write(&original, bytes).expect("write own fixture");
    let before = read_object(&original).expect("read original identity");
    assert_eq!(before.size, bytes.len() as u64);

    fs::rename(&original, &moved).expect("rename fixture");
    assert_eq!(
        read_object(&moved).expect("moved identity").identity,
        before.identity
    );
    fs::hard_link(&moved, &linked).expect("make hardlink");
    let hardlink = read_object(&linked).expect("hardlink identity");
    assert_eq!(hardlink.identity, before.identity);
    assert_eq!(hardlink.links, 2);

    fs::copy(&moved, &copied).expect("copy fixture");
    let copy = read_object(&copied).expect("copy identity");
    assert_ne!(copy.identity, before.identity);
    assert_eq!(fs::read(&copied).expect("copy bytes"), bytes);

    symlink(&moved, &alias).expect("make path alias");
    assert_eq!(
        read_object(&alias).expect("alias target").identity,
        before.identity
    );
    fs::write(&staged, bytes).expect("stage same-content replacement");
    fs::rename(&staged, &moved).expect("replace through rename");
    let replaced = read_object(&moved).expect("new object at old path");
    assert_ne!(replaced.identity, before.identity);
    assert_eq!(fs::read(&moved).expect("replacement bytes"), bytes);
    assert_eq!(
        read_object(&linked).expect("old hardlink").identity,
        before.identity
    );
    assert_eq!(
        read_object(&alias)
            .expect("alias now follows replacement")
            .identity,
        replaced.identity
    );

    assert_eq!(read_object(&root), Err(ObjectReadError::NotRegularFile));
    assert!(matches!(
        read_object(&original),
        Err(ObjectReadError::Io(_))
    ));
    fs::remove_dir_all(&root).expect("remove only this generated sandbox");
}
