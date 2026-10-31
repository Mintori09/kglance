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
        original_index: 0,
    };
    let item2 = TrashedItem {
        original_path: PathBuf::from("/tmp/file2.txt"),
        original_index: 1,
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
    let result = move_to_trash(non_existent, 0);
    assert!(result.is_err());
}

#[test]
fn test_move_to_trash_and_restore_real() {
    let dir = tempdir().expect("create temp dir");
    let file_path = dir.path().join("test_real_trash.txt");
    File::create(&file_path).expect("create file");
    assert!(file_path.exists());

    let trashed_res = move_to_trash(&file_path, 0);
    let trashed = match trashed_res {
        Ok(item) => item,
        Err(err) => {
            // In headless CI or minimal containers without FreeDesktop trash support, skip gracefully
            eprintln!("Skipping test_move_to_trash_and_restore_real (trash unsupported): {err}");
            return;
        }
    };

    assert_eq!(trashed.original_path, file_path);
    assert_eq!(trashed.original_index, 0);
    assert!(!file_path.exists());

    let restore_res = restore_trashed_item(&trashed);
    if let Err(err) = restore_res {
        // In headless CI or minimal containers, FreeDesktop trash listing across mounts may fail
        if std::env::var("CI").is_ok() {
            eprintln!(
                "Skipping restore assertion in CI (trash listing unsupported or desynced): {err}"
            );
            return;
        }
        panic!("Restore failed: {err}");
    }
    assert!(file_path.exists());
}
