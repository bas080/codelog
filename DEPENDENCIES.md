# Dependencies & Architecture

This document specifies the required Rust dependencies and architectural design for rendering Markdown directly to the terminal and delegating ` ```patch ` codeblock pretty-printing to `delta`.

---

## Required Dependencies

| Crate | Purpose |
| --- | --- |
| **`pulldown-cmark`** | Pull-parser for parsing CommonMark events from Markdown files. Used to identify ` ```patch ` fenced codeblocks, track line numbers, and stream Markdown elements. |
| **`clap`** (with `derive`) | Subcommand parsing (`test`, `apply`, `show`) and option flags (`--help`, `--version`, `--verbose`). |
| **`tempfile`** | Creation and management of temporary directory clones for safe dry-run execution (`codelog test`). |
| **`anstream` / `anstyle`** | ANSI terminal coloring and style control, adhering to terminal color flags and the `CODELOG_COLOR` environment variable. |
| **`termimad`** (or custom `pulldown-cmark` terminal renderer) | Direct Markdown-to-terminal ANSI rendering (headers, lists, bold/italic text) without HTML intermediate formats. |
| **`anyhow` / `thiserror`** | Idiomatic CLI error handling and context reporting. |

---

## Markdown Parsing & Terminal Delta Integration Architecture

1. **Markdown Event Stream**:
   - `pulldown-cmark` parses Markdown files in a pull-stream manner using `Parser::into_offset_iter()`.
   - Non-patch elements (headers, paragraphs, lists) are formatted directly for terminal output using ANSI terminal styling (e.g. `termimad` / `anstyle`). No HTML output is produced.

2. **Patch Codeblock Detection & `delta` Streaming**:
   - When `pulldown-cmark` yields a `Tag::CodeBlock(CodeBlockKind::Fenced(lang))` where `lang == "patch"`, `codelog` collects the patch text and line offset.
   - For `codelog show`:
     - If `delta` is installed (or enabled via `CODELOG_USE_DELTA`), `codelog` spawns `delta` via `std::process::Command` with standard input redirected (`Stdio::piped()`).
     - The patch block text is piped directly to `delta` for terminal pretty-printing and syntax-highlighted diff rendering.
     - If `delta` is absent or disabled, `codelog` renders syntax-highlighted or ANSI-colored diff lines directly to the terminal standard output.
