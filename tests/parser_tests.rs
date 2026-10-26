use codelog::app::extract_commit_hash_from_filename;
use codelog::parser::{byte_offset_to_line_number, parse_patch_blocks};
use std::fs;
use std::path::Path;
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
fn test_extract_commit_hash_from_filename() {
    let valid = Path::new("codelog/3682e00f9d19673e4a394269f91a3ef646811c3c.md");
    assert_eq!(
        extract_commit_hash_from_filename(valid),
        Some("3682e00f9d19673e4a394269f91a3ef646811c3c".to_string())
    );

    let invalid = Path::new("codelog/feature-greeting.md");
    assert_eq!(extract_commit_hash_from_filename(invalid), None);
}

#[test]
fn test_parse_patch_blocks_single() {
    let md = r#"
# Heading

Some text.

```patch
--- a/file.txt
+++ b/file.txt
@@ -1 +1 @@
-old
+new
```
"#;
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), md).unwrap();

    let blocks = parse_patch_blocks(temp_file.path()).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].index, 1);
    assert!(blocks[0].content.contains("--- a/file.txt"));
    assert!(blocks[0].content.contains("+new"));
}

#[test]
fn test_parse_patch_blocks_ignores_non_patch() {
    let md = r#"
```rust
fn test() {}
```

```patch
--- a/test
+++ b/test
```
"#;
    let temp_file = NamedTempFile::new().unwrap();
    fs::write(temp_file.path(), md).unwrap();

    let blocks = parse_patch_blocks(temp_file.path()).unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].index, 1);
}
