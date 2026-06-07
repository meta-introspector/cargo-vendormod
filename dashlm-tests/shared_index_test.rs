use dashlm_shared_index::SharedIndex;
use std::fs;

#[test]
fn test_create_and_open() {
    let dir = std::env::temp_dir().join("dashlm_shared_index_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let idx = SharedIndex::open_or_create(dir.join("test.shm")).expect("open_or_create");
    assert!(idx.is_open());
    let _ = fs::remove_dir_all(&dir);
}
