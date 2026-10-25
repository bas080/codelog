# codelog

`codelog` is a Rust CLI tool designed to test, apply, and review patch-based Markdown logs ("codelogs").

The core philosophy behind `codelog` is that codebase changes should be accompanied by clear human explanations. Instead of raw git diffs, a codelog combines rich narrative context detailing **why** a patch is necessary and **what** it achieves alongside executable ` ```patch ` codeblocks.

`codelog` parses ` ```patch ` blocks, ignores non-patch text and code, tests them safely inside temporary repository clones, applies them to working repositories, and pretty-prints diffs for review.

---

## Features

- **Safe Dry-Run Testing (`codelog test`)**: Clones the current repository into an isolated temporary directory to safely test whether patches apply cleanly.
- **Direct Application (`codelog apply`)**: Applies patch codeblocks directly to your active repository.
- **Interactive Review (`codelog show`)**: Pretty-prints patch codeblocks. Integrates automatically with [`delta`](https://github.com/dandavison/delta) for enhanced diff syntax highlighting if installed.
- **Selective Parsing**: Only extracts fenced codeblocks tagged with ` ```patch `. All other markdown sections and codeblocks (such as ` ```rust `, ` ```bash `, or un-tagged blocks) are ignored during application while preserving full context for readers.
- **Detailed Error Context**: On patch failure, `codelog` highlights the exact Markdown filename and line number of the failing patch block alongside the exact raw output from `git apply`.

---

## Installation

### From Source (Cargo)

Ensure you have Rust and `cargo` installed on your machine.

```bash
cargo install --path .
```

---

## Usage

```bash
codelog <SUBCOMMAND> [OPTIONS] <FILE>...
```

### Subcommands

#### 1. `codelog test <file>...`
Tests applying the patch codeblock(s) in the specified Markdown file(s) against a temporary clone of the current repository.

```bash
codelog test examples/feature-greeting.md
```

#### 2. `codelog apply <file>...`
Applies the patch codeblock(s) in the specified Markdown file(s) directly to the working directory repository.

```bash
codelog apply examples/feature-greeting.md
```

#### 3. `codelog show <file>...`
Pretty-prints/reviews the patch codeblocks from the specified Markdown file(s). Uses [`delta`](https://github.com/dandavison/delta) for syntax highlighting when available.

```bash
codelog show examples/feature-greeting.md
```

---

## Options & Flags

- `-h, --help`: Prints help information.
- `-V, --version`: Prints version information.
- `-v, --verbose`: Enables verbose log output.

---

## Environment Variables

`codelog` can be configured using the following environment variables:

| Variable | Description | Options / Default |
| --- | --- | --- |
| `CODELOG_USE_DELTA` | Enable or disable `delta` integration for `codelog show`. | `true`/`1` or `false`/`0` (default: auto-detected) |
| `CODELOG_TMPDIR` | Custom temporary directory path for temporary repo clones during `codelog test`. | Path string (default: system temp dir) |
| `CODELOG_COLOR` | Control ANSI color output in terminal. | `auto`, `always`, `never` (default: `auto`) |
| `CODELOG_VERBOSE` | Enable verbose logging output. | `1`/`true` or `0`/`false` (default: `false`) |

---

## Codelog Markdown Format

A codelog is a Markdown file designed to explain the rationale behind changes while containing executable patch codeblocks.

An example codelog file is provided at [`examples/feature-greeting.md`](examples/feature-greeting.md):

````markdown
# Codelog: Add User Greeting Feature

## Context & Motivation

Currently, the application prints a generic `"Hello, world!"` message on startup.
Users have requested a personalized greeting function so that the application can address specific users by name.

This codelog documents the step-by-step patch modifications required.

---

## Step 1: Add `greet` Helper Function and Update `main`

### Why this patch is needed:
We need a reusable function `greet(name: &str)` in `src/main.rs` that encapsulates
personalized greeting logic, and we need to update `main()` to invoke `greet("Alice")`.

```patch
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,7 @@
+fn greet(name: &str) {
+    println!("Hello, {}!", name);
+}
+
 fn main() {
-    println!("Hello, world!");
+    greet("Alice");
 }
```

---

## Technical Notes (Ignored by `codelog`)

Non-patch codeblocks are ignored during execution:

```rust
// Note: Future async wrapper idea (ignored during codelog apply/test)
async fn greet_async(name: String) {
    tokio::task::spawn_blocking(move || greet(&name)).await.ok();
}
```
````

---

## Execution Workflow Examples

### 1. Successful Test Run

```text
$ codelog test examples/feature-greeting.md
[codelog] Cloned repository to temporary directory: /tmp/codelog-repo-a1b2c3
[codelog] Found 2 patch codeblocks in examples/feature-greeting.md

[1/2] Applying patch block #1 (examples/feature-greeting.md:20)... SUCCESS
[2/2] Applying patch block #2 (examples/feature-greeting.md:41)... SUCCESS

✔ All patches applied successfully in temporary repository!

Follow-up steps:
  • Inspect changes in temp repo:  cd /tmp/codelog-repo-a1b2c3 && git diff
  • Apply patches to main repo:     codelog apply examples/feature-greeting.md
  • Clean up temporary clone:       rm -rf /tmp/codelog-repo-a1b2c3
```

### 2. Failure Handling

When a patch fails to apply, `codelog` outputs the filename and line number in the Markdown file where the patch block starts, along with raw output from `git apply`:

```text
$ codelog test examples/feature-greeting.md
[codelog] Cloned repository to temporary directory: /tmp/codelog-repo-x9y8z7
[codelog] Found 2 patch codeblocks in examples/feature-greeting.md

[1/2] Applying patch block #1 (examples/feature-greeting.md:20)... SUCCESS
[2/2] Applying patch block #2 (examples/feature-greeting.md:41)... FAILED

✖ Error: Patch block #2 in examples/feature-greeting.md:41 failed to apply.

--- Raw 'git apply' output ---
error: patch failed: src/main.rs:15
error: src/main.rs: patch does not apply
------------------------------

Follow-up steps:
  • Edit and fix patch block #2 in examples/feature-greeting.md at line 41.
  • Re-run 'codelog test examples/feature-greeting.md' to test the updated patches.
```

---

## License

This project is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**. See the `LICENSE` file for details.
