use super::*;
use std::fs::File;
use tempfile::tempdir;

#[test]
fn test_trash_history_lifecycle() {
    let mut history = TrashHistory::new();
    assert!(history.is_empty());
    assert_eq!(history.last(), None);

    let item1 = TrashedItem {
        original_path: PathBuf::from("/tmp/file1.txt"),
    };
    let item2 = TrashedItem {
        original_path: PathBuf::from("/tmp/file2.txt"),
    };

    history.push(item1.clone());
    assert!(!history.is_empty());
    assert_eq!(history.last(), Some(&item1));

    history.push(item2.clone());
    assert_eq!(history.last(), Some(&item2));

    let popped = history.pop();
    assert_eq!(popped, Some(item2));
    assert_eq!(history.last(), Some(&item1));

    history.clear();
    assert!(history.is_empty());
}

#[test]
fn test_move_to_trash_non_existent_file() {
    let non_existent = Path::new("/path/that/definitely/does/not/exist/xyz_kglance.txt");
    let result = move_to_trash(non_existent);
    assert!(result.is_err());
}

#[test]
fn test_move_to_trash_and_restore_real() {
    let dir = tempdir().expect("create temp dir");
    let file_path = dir.path().join("test_real_trash.txt");
    File::create(&file_path).expect("create file");
    assert!(file_path.exists());

    let trashed = move_to_trash(&file_path).expect("move to trash");
    assert_eq!(trashed.original_path, file_path);
    assert!(!file_path.exists());

    let restore_res = restore_trashed_item(&trashed);
    assert!(
        restore_res.is_ok(),
        "Restore failed: {:?}",
        restore_res.err()
    );
    assert!(file_path.exists());
}
