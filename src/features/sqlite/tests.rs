use super::*;
use rusqlite::Connection;
use std::path::Path;

#[test]
fn test_supported_extensions() {
    let parser = SqliteParser;
    let exts = parser.supported_extensions();
    assert!(exts.contains(&"sqlite"));
    assert!(exts.contains(&"sqlite3"));
    assert!(exts.contains(&"db"));
    assert!(exts.contains(&"db3"));
    assert!(exts.contains(&"s3db"));
    assert!(exts.contains(&"sl3"));
}

#[test]
fn test_nonexistent_file() {
    let parser = SqliteParser;
    let res = parser.parse(Path::new("/nonexistent/file_does_not_exist.sqlite"));
    assert!(matches!(res, Err(ParseError::FileNotFound)));
}

#[test]
fn test_invalid_magic_header_rejects() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let fake_db = temp_dir.path().join("fake.db");
    std::fs::write(&fake_db, b"This is not a sqlite database file!").expect("write fake db");

    let parser = SqliteParser;
    let res = parser.parse(&fake_db);
    assert!(matches!(res, Err(ParseError::UnsupportedFormat)));
}

#[test]
fn test_zero_byte_database_file() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("empty_0bytes.sqlite");
    std::fs::write(&db_path, b"").expect("create 0 byte file");

    let parser = SqliteParser;
    let res = parser.parse(&db_path).expect("parse 0-byte db");

    match res {
        ParsedContent::Spreadsheet { sheets } => {
            assert_eq!(sheets.len(), 1);
            assert_eq!(sheets[0].name, "Database Info");
            assert_eq!(sheets[0].headers, vec!["Status".to_string()]);
            assert_eq!(
                sheets[0].rows,
                vec![vec!["Database file is empty (0 bytes)".to_string()]]
            );
        }
        other => panic!("expected Spreadsheet, got {other:?}"),
    }
}

#[test]
fn test_empty_database_no_tables() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("empty_no_tables.sqlite");

    // Create an initialized sqlite db with 0 tables
    {
        let conn = Connection::open(&db_path).expect("create db");
        conn.execute_batch("PRAGMA user_version = 0;")
            .expect("init db header");
    }

    let parser = SqliteParser;
    let res = parser.parse(&db_path).expect("parse empty sqlite db");

    match res {
        ParsedContent::Spreadsheet { sheets } => {
            assert_eq!(sheets.len(), 1);
            assert_eq!(sheets[0].name, "Database Info");
            assert_eq!(sheets[0].headers, vec!["Status".to_string()]);
            assert_eq!(
                sheets[0].rows,
                vec![vec!["Database contains no tables or views".to_string()]]
            );
        }
        other => panic!("expected Spreadsheet, got {other:?}"),
    }
}

#[test]
fn test_sqlite_datatypes_and_views() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test_datatypes.db");

    {
        let conn = Connection::open(&db_path).expect("create db");
        conn.execute_batch(
            r#"
            CREATE TABLE "users" (
                "id" INTEGER PRIMARY KEY,
                "name" TEXT NOT NULL,
                "score" REAL,
                "avatar" BLOB,
                "bio" TEXT,
                "notes" TEXT
            );

            INSERT INTO "users" ("id", "name", "score", "avatar", "bio", "notes")
            VALUES (
                1,
                'Nguyễn Văn A',
                95.5,
                X'0102030405',
                'Lập trình viên Rust & KDE Plasma',
                NULL
            );

            INSERT INTO "users" ("id", "name", "score", "avatar", "bio", "notes")
            VALUES (
                2,
                'Integer Score User',
                100.0,
                X'',
                'Short bio',
                'Sample notes'
            );

            CREATE VIEW "active_users" AS
            SELECT "id", "name", "score" FROM "users" WHERE "score" >= 90.0;
            "#,
        )
        .expect("populate test database");
    }

    let parser = SqliteParser;
    let res = parser.parse(&db_path).expect("parse sqlite db");

    match res {
        ParsedContent::Spreadsheet { sheets } => {
            assert_eq!(sheets.len(), 2);

            // View sheet: active_users
            let active_view = &sheets[0];
            assert_eq!(active_view.name, "active_users");
            assert_eq!(active_view.headers, vec!["id", "name", "score"]);
            assert_eq!(active_view.rows.len(), 2);
            assert_eq!(active_view.rows[0], vec!["1", "Nguyễn Văn A", "95.5"]);
            assert_eq!(active_view.rows[1], vec!["2", "Integer Score User", "100"]);

            // Table sheet: users
            let users_table = &sheets[1];
            assert_eq!(users_table.name, "users");
            assert_eq!(
                users_table.headers,
                vec!["id", "name", "score", "avatar", "bio", "notes"]
            );
            assert_eq!(users_table.rows.len(), 2);

            // Check row 1
            assert_eq!(users_table.rows[0][0], "1");
            assert_eq!(users_table.rows[0][1], "Nguyễn Văn A");
            assert_eq!(users_table.rows[0][2], "95.5");
            assert_eq!(users_table.rows[0][3], "[BLOB 5 bytes]");
            assert_eq!(users_table.rows[0][4], "Lập trình viên Rust & KDE Plasma");
            assert_eq!(users_table.rows[0][5], "[NULL]");

            // Check row 2
            assert_eq!(users_table.rows[1][0], "2");
            assert_eq!(users_table.rows[1][1], "Integer Score User");
            assert_eq!(users_table.rows[1][2], "100");
            assert_eq!(users_table.rows[1][3], "[BLOB 0 bytes]");
            assert_eq!(users_table.rows[1][4], "Short bio");
            assert_eq!(users_table.rows[1][5], "Sample notes");
        }
        other => panic!("expected Spreadsheet, got {other:?}"),
    }
}

#[test]
fn test_long_text_cell_truncation() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test_truncation.sqlite3");

    let long_string = "A".repeat(500);

    {
        let conn = Connection::open(&db_path).expect("create db");
        conn.execute(
            "CREATE TABLE docs (id INTEGER PRIMARY KEY, content TEXT);",
            [],
        )
        .expect("create table");
        conn.execute(
            "INSERT INTO docs (id, content) VALUES (1, ?1);",
            [&long_string],
        )
        .expect("insert long text");
    }

    let parser = SqliteParser;
    let res = parser.parse(&db_path).expect("parse sqlite db");

    match res {
        ParsedContent::Spreadsheet { sheets } => {
            assert_eq!(sheets.len(), 1);
            let row = &sheets[0].rows[0];
            let cell = &row[1];
            assert_eq!(cell.chars().count(), 300);
            assert!(cell.ends_with("..."));
            assert_eq!(&cell[..297], &"A".repeat(297));
        }
        other => panic!("expected Spreadsheet, got {other:?}"),
    }
}

#[test]
fn test_table_row_limit_100() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test_limit.db");

    {
        let mut conn = Connection::open(&db_path).expect("create db");
        conn.execute(
            "CREATE TABLE large_table (id INTEGER PRIMARY KEY, value INTEGER);",
            [],
        )
        .expect("create table");

        let tx = conn.transaction().expect("start transaction");
        {
            let mut stmt = tx
                .prepare("INSERT INTO large_table (id, value) VALUES (?1, ?2);")
                .expect("prepare stmt");
            for i in 1..=250 {
                stmt.execute([i, i * 10]).expect("insert row");
            }
        }
        tx.commit().expect("commit");
    }

    let parser = SqliteParser;
    let res = parser.parse(&db_path).expect("parse sqlite db");

    match res {
        ParsedContent::Spreadsheet { sheets } => {
            assert_eq!(sheets.len(), 1);
            assert_eq!(sheets[0].rows.len(), 100);
            assert_eq!(sheets[0].rows[0], vec!["1", "10"]);
            assert_eq!(sheets[0].rows[99], vec!["100", "1000"]);
        }
        other => panic!("expected Spreadsheet, got {other:?}"),
    }
}

#[test]
fn test_special_characters_in_table_and_column_names() {
    let temp_dir = tempfile::tempdir().expect("create temp dir");
    let db_path = temp_dir.path().join("test_special.sl3");

    {
        let conn = Connection::open(&db_path).expect("create db");
        conn.execute_batch(
            r#"
            CREATE TABLE "bảng dữ liệu (2026)" (
                "mã số #1" INTEGER,
                "tên/thông tin" TEXT
            );
            INSERT INTO "bảng dữ liệu (2026)" VALUES (42, 'Dữ liệu kiểm tra tiếng Việt');
            "#,
        )
        .expect("create table with special names");
    }

    let parser = SqliteParser;
    let res = parser.parse(&db_path).expect("parse sqlite db");

    match res {
        ParsedContent::Spreadsheet { sheets } => {
            assert_eq!(sheets.len(), 1);
            assert_eq!(sheets[0].name, "bảng dữ liệu (2026)");
            assert_eq!(sheets[0].headers, vec!["mã số #1", "tên/thông tin"]);
            assert_eq!(
                sheets[0].rows,
                vec![vec![
                    "42".to_string(),
                    "Dữ liệu kiểm tra tiếng Việt".to_string()
                ]]
            );
        }
        other => panic!("expected Spreadsheet, got {other:?}"),
    }
}
