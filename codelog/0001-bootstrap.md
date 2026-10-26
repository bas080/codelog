# Codelog: Initial Repository Bootstrap

## Step 1: Create `.gitignore`

```patch
diff --git a/.gitignore b/.gitignore
new file mode 100644
index 0000000..451d8a1
--- /dev/null
+++ b/.gitignore
@@ -0,0 +1,2 @@
+.local*
+target
```

## Step 2: Create `Cargo.toml`

```patch
diff --git a/Cargo.toml b/Cargo.toml
new file mode 100644
index 0000000..a3d487e
--- /dev/null
+++ b/Cargo.toml
@@ -0,0 +1,16 @@
+[package]
+name = "codelog"
+version = "0.1.0"
+edition = "2021"
+license = "AGPL-3.0-only"
+description = "A Rust CLI tool designed to test, apply, and review patch-based Markdown logs (codelogs)."
+
+[dependencies]
+clap = { version = "4.4", features = ["derive", "env"] }
+pulldown-cmark = "0.10"
+tempfile = "3.8"
+anstream = "0.6"
+anstyle = "0.3"
+termimad = "0.30"
+anyhow = "1.0"
+thiserror = "1.0"
```

## Step 3: Create `src/lib.rs`

```patch
diff --git a/src/lib.rs b/src/lib.rs
new file mode 100644
index 0000000..3b75a55
--- /dev/null
+++ b/src/lib.rs
@@ -0,0 +1,3 @@
+pub mod app;
+pub mod cli;
+pub mod parser;
```

## Step 4: Create `src/cli.rs`

```patch
diff --git a/src/cli.rs b/src/cli.rs
new file mode 100644
index 0000000..bd3aa51
--- /dev/null
+++ b/src/cli.rs
@@ -0,0 +1,39 @@
+use std::path::PathBuf;
+use clap::{Parser, Subcommand};
+
+#[derive(Parser, Debug)]
+#[command(
+    name = "codelog",
+    author,
+    version,
+    about = "A CLI tool to test, apply, and review patch-based Markdown logs",
+    long_about = None
+)]
+pub struct Cli {
+    #[arg(short, long, global = true, env = "CODELOG_VERBOSE", help = "Enables verbose log output")]
+    pub verbose: bool,
+
+    #[arg(long, global = true, env = "CODELOG_COLOR", value_name = "WHEN", help = "Control ANSI color output: auto, always, never")]
+    pub color: Option<String>,
+
+    #[command(subcommand)]
+    pub command: Commands,
+}
+
+#[derive(Subcommand, Debug)]
+pub enum Commands {
+    #[command(about = "Tests applying patch codeblock(s) against a fresh empty repository in alphanumeric order")]
+    Test(FileArgs),
+
+    #[command(about = "Applies patch codeblock(s) directly to the working directory repository in alphanumeric order")]
+    Apply(FileArgs),
+
+    #[command(about = "Pretty-prints/reviews patch codeblocks from Markdown file(s) in alphanumeric order")]
+    Show(FileArgs),
+}
+
+#[derive(clap::Args, Debug)]
+pub struct FileArgs {
+    #[arg(help = "Markdown file(s) containing patch codeblocks. Defaults to all .md files in codelog/ in alphanumeric order if empty.")]
+    pub files: Vec<PathBuf>,
+}
```

## Step 5: Create `src/parser.rs`

```patch
diff --git a/src/parser.rs b/src/parser.rs
new file mode 100644
index 0000000..4ecedd4
--- /dev/null
+++ b/src/parser.rs
@@ -0,0 +1,63 @@
+use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
+use std::fs;
+use std::path::{Path, PathBuf};
+
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct PatchBlock {
+    pub file_path: PathBuf,
+    pub index: usize,
+    pub start_line: usize,
+    pub content: String,
+}
+
+pub fn byte_offset_to_line_number(text: &str, offset: usize) -> usize {
+    let mut line = 1;
+    for (i, b) in text.bytes().enumerate() {
+        if i >= offset {
+            break;
+        }
+        if b == b'\n' {
+            line += 1;
+        }
+    }
+    line
+}
+
+pub fn parse_patch_blocks(path: &Path) -> anyhow::Result<Vec<PatchBlock>> {
+    let content = fs::read_to_string(path)?;
+    let parser = Parser::new_ext(&content, Options::all()).into_offset_iter();
+
+    let mut patch_blocks = Vec::new();
+    let mut current_patch: Option<(usize, usize, String)> = None; // (index, start_line, content)
+    let mut patch_count = 0;
+
+    for (event, range) in parser {
+        match event {
+            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
+                if lang.trim().eq_ignore_ascii_case("patch") {
+                    patch_count += 1;
+                    let line_no = byte_offset_to_line_number(&content, range.start);
+                    current_patch = Some((patch_count, line_no, String::new()));
+                }
+            }
+            Event::Text(text) => {
+                if let Some((_, _, ref mut block_content)) = current_patch {
+                    block_content.push_str(&text);
+                }
+            }
+            Event::End(TagEnd::CodeBlock) => {
+                if let Some((idx, line_no, block_content)) = current_patch.take() {
+                    patch_blocks.push(PatchBlock {
+                        file_path: path.to_path_buf(),
+                        index: idx,
+                        start_line: line_no,
+                        content: block_content,
+                    });
+                }
+            }
+            _ => {}
+        }
+    }
+
+    Ok(patch_blocks)
+}
```

## Step 6: Create `src/app.rs`

```patch
diff --git a/src/app.rs b/src/app.rs
new file mode 100644
index 0000000..314ace6
--- /dev/null
+++ b/src/app.rs
@@ -0,0 +1,306 @@
+use anyhow::{bail, Context, Result};
+use std::env;
+use std::fs;
+use std::io::Write;
+use std::path::{Path, PathBuf};
+use std::process::{Command, Stdio};
+use tempfile::TempDir;
+
+use crate::cli::Cli;
+use crate::parser::{parse_patch_blocks, PatchBlock};
+
+pub fn run(cli: Cli) -> Result<()> {
+    match cli.command {
+        crate::cli::Commands::Test(args) => run_test(&args.files, cli.verbose),
+        crate::cli::Commands::Apply(args) => run_apply(&args.files, cli.verbose),
+        crate::cli::Commands::Show(args) => run_show(&args.files, cli.verbose),
+    }
+}
+
+pub fn resolve_and_sort_files(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
+    let mut resolved = if files.is_empty() {
+        let codelog_dir = PathBuf::from("codelog");
+        if !codelog_dir.exists() || !codelog_dir.is_dir() {
+            return Ok(Vec::new());
+        }
+        let mut found = Vec::new();
+        for entry in fs::read_dir(codelog_dir)? {
+            let entry = entry?;
+            let path = entry.path();
+            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
+                found.push(path);
+            }
+        }
+        found
+    } else {
+        files.to_vec()
+    };
+
+    resolved.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
+    Ok(resolved)
+}
+
+fn apply_patch_to_dir(repo_dir: &Path, patch: &PatchBlock) -> Result<(), String> {
+    let mut temp_patch_file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
+    temp_patch_file
+        .write_all(patch.content.as_bytes())
+        .map_err(|e| e.to_string())?;
+    let patch_path = temp_patch_file.path();
+
+    let output = Command::new("git")
+        .current_dir(repo_dir)
+        .arg("apply")
+        .arg(patch_path)
+        .output()
+        .map_err(|e| format!("Failed to execute git apply: {}", e))?;
+
+    if output.status.success() {
+        Ok(())
+    } else {
+        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
+        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
+        let combined = if stderr.is_empty() {
+            stdout
+        } else if stdout.is_empty() {
+            stderr
+        } else {
+            format!("{}\n{}", stdout, stderr)
+        };
+        Err(combined.trim().to_string())
+    }
+}
+
+fn create_empty_temp_repo() -> Result<(PathBuf, Option<TempDir>)> {
+    let custom_tmpdir = env::var("CODELOG_TMPDIR").ok().filter(|s| !s.is_empty());
+
+    let (target_dir, temp_guard) = if let Some(tmpdir_str) = custom_tmpdir {
+        let custom_path = PathBuf::from(&tmpdir_str);
+        let timestamp = std::time::SystemTime::now()
+            .duration_since(std::time::UNIX_EPOCH)
+            .map(|d| d.as_secs())
+            .unwrap_or(0);
+        let target_dir = custom_path.join(format!("codelog-repo-{}", timestamp));
+        fs::create_dir_all(&target_dir)?;
+        (target_dir, None)
+    } else {
+        let temp_dir = tempfile::Builder::new()
+            .prefix("codelog-repo-")
+            .tempdir()
+            .context("Failed to create temporary directory")?;
+        let target_dir = temp_dir.path().to_path_buf();
+        (target_dir, Some(temp_dir))
+    };
+
+    let init_output = Command::new("git")
+        .current_dir(&target_dir)
+        .arg("init")
+        .output()
+        .context("Failed to execute git init in temporary directory")?;
+
+    if !init_output.status.success() {
+        let err = String::from_utf8_lossy(&init_output.stderr);
+        bail!("Failed to initialize empty git repo in temp dir: {}", err);
+    }
+
+    Ok((target_dir, temp_guard))
+}
+
+fn run_test(files: &[PathBuf], verbose: bool) -> Result<()> {
+    let target_files = resolve_and_sort_files(files)?;
+    if target_files.is_empty() {
+        println!("[codelog] No codelog Markdown files found.");
+        return Ok(());
+    }
+
+    let (temp_repo_dir, _temp_dir_guard) = create_empty_temp_repo()?;
+
+    println!(
+        "[codelog] Initialized empty repository in temporary directory: {}",
+        temp_repo_dir.display()
+    );
+
+    let mut total_blocks = Vec::new();
+    for file in &target_files {
+        let blocks = parse_patch_blocks(file)
+            .with_context(|| format!("Failed to parse markdown file: {}", file.display()))?;
+        total_blocks.extend(blocks);
+    }
+
+    let file_names_str = target_files
+        .iter()
+        .map(|f| f.display().to_string())
+        .collect::<Vec<_>>()
+        .join(", ");
+    println!(
+        "[codelog] Found {} patch codeblock(s) across sorted file(s): {}",
+        total_blocks.len(),
+        file_names_str
+    );
+    println!();
+
+    let total_count = total_blocks.len();
+    for (i, patch) in total_blocks.iter().enumerate() {
+        let idx = i + 1;
+        let file_str = patch.file_path.display();
+        print!(
+            "[{}/{}] Applying patch block #{} ({}:{})... ",
+            idx, total_count, patch.index, file_str, patch.start_line
+        );
+        std::io::stdout().flush().ok();
+
+        match apply_patch_to_dir(&temp_repo_dir, patch) {
+            Ok(()) => {
+                println!("SUCCESS");
+            }
+            Err(err_msg) => {
+                println!("FAILED\n");
+                println!(
+                    "✖ Error: Patch block #{} in {}:{} failed to apply.\n",
+                    patch.index, file_str, patch.start_line
+                );
+                println!("--- Raw 'git apply' output ---");
+                println!("{}", err_msg);
+                println!("------------------------------\n");
+                bail!("Patch execution failed.");
+            }
+        }
+    }
+
+    println!();
+    println!("✔ All patches applied successfully in temporary empty repository!\n");
+    if verbose {
+        println!("Follow-up steps:");
+        println!(
+            "  • Inspect generated source in temp repo: cd {}",
+            temp_repo_dir.display()
+        );
+        println!(
+            "  • Clean up temporary repo:              rm -rf {}",
+            temp_repo_dir.display()
+        );
+    }
+
+    Ok(())
+}
+
+fn run_apply(files: &[PathBuf], verbose: bool) -> Result<()> {
+    let current_dir = env::current_dir().context("Failed to get current working directory")?;
+    let target_files = resolve_and_sort_files(files)?;
+
+    if target_files.is_empty() {
+        println!("[codelog] No codelog Markdown files found.");
+        return Ok(());
+    }
+
+    let mut total_blocks = Vec::new();
+    for file in &target_files {
+        let blocks = parse_patch_blocks(file)
+            .with_context(|| format!("Failed to parse markdown file: {}", file.display()))?;
+        total_blocks.extend(blocks);
+    }
+
+    if verbose {
+        println!(
+            "[codelog] Applying {} patch block(s) in alphanumeric order directly to working directory",
+            total_blocks.len()
+        );
+    }
+
+    for patch in &total_blocks {
+        let file_str = patch.file_path.display();
+        match apply_patch_to_dir(&current_dir, patch) {
+            Ok(()) => {
+                println!(
+                    "Applied patch block #{} ({}:{}) successfully.",
+                    patch.index, file_str, patch.start_line
+                );
+            }
+            Err(err_msg) => {
+                println!(
+                    "✖ Error: Failed to apply patch block #{} in {}:{}",
+                    patch.index, file_str, patch.start_line
+                );
+                println!("--- Raw 'git apply' output ---");
+                println!("{}", err_msg);
+                println!("------------------------------");
+                bail!("Patch execution failed.");
+            }
+        }
+    }
+
+    println!("✔ All patches applied successfully.");
+    Ok(())
+}
+
+fn is_delta_available() -> bool {
+    let env_setting = env::var("CODELOG_USE_DELTA").unwrap_or_default();
+    if env_setting == "false" || env_setting == "0" {
+        return false;
+    }
+    if env_setting == "true" || env_setting == "1" {
+        return true;
+    }
+
+    Command::new("delta")
+        .arg("--version")
+        .stdout(Stdio::null())
+        .stderr(Stdio::null())
+        .status()
+        .map(|s| s.success())
+        .unwrap_or(false)
+}
+
+fn run_show(files: &[PathBuf], _verbose: bool) -> Result<()> {
+    let use_delta = is_delta_available();
+    let target_files = resolve_and_sort_files(files)?;
+
+    if target_files.is_empty() {
+        println!("[codelog] No codelog Markdown files found.");
+        return Ok(());
+    }
+
+    for file in &target_files {
+        let blocks = parse_patch_blocks(file)
+            .with_context(|| format!("Failed to parse markdown file: {}", file.display()))?;
+
+        println!("=== File: {} ===", file.display());
+        for patch in blocks {
+            println!(
+                "--- Patch Block #{} ({}:{}) ---",
+                patch.index,
+                patch.file_path.display(),
+                patch.start_line
+            );
+
+            if use_delta {
+                if let Ok(mut child) = Command::new("delta")
+                    .stdin(Stdio::piped())
+                    .stdout(Stdio::inherit())
+                    .stderr(Stdio::inherit())
+                    .spawn()
+                {
+                    if let Some(mut stdin) = child.stdin.take() {
+                        let _ = stdin.write_all(patch.content.as_bytes());
+                    }
+                    let _ = child.wait();
+                    continue;
+                }
+            }
+
+            for line in patch.content.lines() {
+                if line.starts_with('+') && !line.starts_with("+++") {
+                    println!("\x1b[32m{}\x1b[0m", line);
+                } else if line.starts_with('-') && !line.starts_with("---") {
+                    println!("\x1b[31m{}\x1b[0m", line);
+                } else if line.starts_with('@') {
+                    println!("\x1b[36m{}\x1b[0m", line);
+                } else {
+                    println!("{}", line);
+                }
+            }
+            println!();
+        }
+    }
+
+    Ok(())
+}
```

## Step 7: Create `src/main.rs`

```patch
diff --git a/src/main.rs b/src/main.rs
new file mode 100644
index 0000000..c70a604
--- /dev/null
+++ b/src/main.rs
@@ -0,0 +1,12 @@
+use codelog::cli::Cli;
+use clap::Parser;
+use std::process::exit;
+
+fn main() {
+    let cli = Cli::parse();
+
+    if let Err(err) = codelog::app::run(cli) {
+        eprintln!("{}", err);
+        exit(1);
+    }
+}
```

## Step 8: Create `tests/parser_tests.rs`

```patch
diff --git a/tests/parser_tests.rs b/tests/parser_tests.rs
new file mode 100644
index 0000000..1ebbcf5
--- /dev/null
+++ b/tests/parser_tests.rs
@@ -0,0 +1,76 @@
+use codelog::app::resolve_and_sort_files;
+use codelog::parser::{byte_offset_to_line_number, parse_patch_blocks};
+use std::fs;
+use std::path::PathBuf;
+use tempfile::NamedTempFile;
+
+#[test]
+fn test_byte_offset_to_line_number() {
+    let text = "line 1\nline 2\nline 3\n";
+    assert_eq!(byte_offset_to_line_number(text, 0), 1);
+    assert_eq!(byte_offset_to_line_number(text, 6), 1);
+    assert_eq!(byte_offset_to_line_number(text, 7), 2);
+    assert_eq!(byte_offset_to_line_number(text, 14), 3);
+}
+
+#[test]
+fn test_resolve_and_sort_files() {
+    let files = vec![
+        PathBuf::from("codelog/0002-second.md"),
+        PathBuf::from("codelog/0001-first.md"),
+    ];
+    let sorted = resolve_and_sort_files(&files).unwrap();
+    assert_eq!(
+        sorted,
+        vec![
+            PathBuf::from("codelog/0001-first.md"),
+            PathBuf::from("codelog/0002-second.md"),
+        ]
+    );
+}
+
+#[test]
+fn test_parse_patch_blocks_single() {
+    let md = r#"
+# Heading
+
+Some text.
+
+```patch
+--- /dev/null
++++ b/file.txt
+@@ -0,0 +1 @@
++new content
+```
+"#;
+    let temp_file = NamedTempFile::new().unwrap();
+    fs::write(temp_file.path(), md).unwrap();
+
+    let blocks = parse_patch_blocks(temp_file.path()).unwrap();
+    assert_eq!(blocks.len(), 1);
+    assert_eq!(blocks[0].index, 1);
+    assert!(blocks[0].content.contains("+++ b/file.txt"));
+    assert!(blocks[0].content.contains("+new content"));
+}
+
+#[test]
+fn test_parse_patch_blocks_ignores_non_patch() {
+    let md = r#"
+```rust
+fn test() {}
+```
+
+```patch
+--- /dev/null
++++ b/test
+@@ -0,0 +1 @@
++test
+```
+"#;
+    let temp_file = NamedTempFile::new().unwrap();
+    fs::write(temp_file.path(), md).unwrap();
+
+    let blocks = parse_patch_blocks(temp_file.path()).unwrap();
+    assert_eq!(blocks.len(), 1);
+    assert_eq!(blocks[0].index, 1);
+}
```

## Step 9: Create `.github/workflows/test.yml`

```patch
diff --git a/.github/workflows/test.yml b/.github/workflows/test.yml
new file mode 100644
index 0000000..234feaf
--- /dev/null
+++ b/.github/workflows/test.yml
@@ -0,0 +1,27 @@
+name: Test
+
+on:
+  push:
+    branches: [ main, master ]
+  pull_request:
+    branches: [ main, master ]
+
+jobs:
+  test:
+    runs-on: ubuntu-latest
+
+    steps:
+      - name: Checkout repository
+        uses: actions/checkout@v4
+
+      - name: Set up Rust
+        uses: dtolnay/rust-toolchain@stable
+
+      - name: Rust Cache
+        uses: Swatinem/rust-cache@v2
+
+      - name: Run unit and integration tests
+        run: cargo test --verbose
+
+      - name: Run codelog test using prebuilt binary
+        run: ./bin/codelog test
```
