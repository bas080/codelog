use codelog::app::resolve_and_sort_files;
use codelog::parser::{byte_offset_to_line_number, parse_patch_blocks};
use std::fs;
use std::path::PathBuf;
use tempfile::NamedTempFile;

#[test]
fn test_byte_offset_to_line_number() {
    let text = "line 1\nline 2\nline 3\n";
    assert_eq!(byte_offset_to_line_number(text, 0), 1);
    assert_eq!(byte_offset_to_line_number(text, 6), 1);
    assert_eq!(byte_offset_to_line_number(text, 7), 2);
    assert_eq!(byte_offset_to_line_number(text, 14), 3);
}

#[test]
fn test_resolve_and_sort_files() {
    let files = vec![
        PathBuf::from("codelog/0002-second.md"),
        PathBuf::from("codelog/0001-first.md"),
    ];
    let sorted = resolve_and_sort_files(&files).unwrap();
    assert_eq!(
        sorted,
        vec![
            PathBuf::from("codelog/0001-first.md"),
            PathBuf::from("codelog/0002-second.md"),
        ]
    );
}

#[test]
fn test_parse_patch_blocks_single() {
    let md = r#"
# Heading

Some text.

```patch
--- /dev/null
+++ b/file.txt
@@ -0,0 +1 @@
+new content
```
"#;
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), md).unwrap();

    let blocks = parse_patch_blocks(temp_file.path()).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].index, 1);
    assert!(blocks[0].content.contains("+++ b/file.txt"));
    assert!(blocks[0].content.contains("+new content"));
}

#[test]
fn test_parse_patch_blocks_ignores_non_patch() {
    let md = r#"
```rust
fn test() {}
```

```patch
--- /dev/null
+++ b/test
@@ -0,0 +1 @@
+test
```
"#;
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), md).unwrap();

    let blocks = parse_patch_blocks(temp_file.path()).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].index, 1);
}
