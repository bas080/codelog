# Dependencies & Architecture

This document specifies the required Rust dependencies, architectural design, and evaluated markdown parsing alternatives for rendering Markdown directly to the terminal and delegating ` ```patch ` codeblock pretty-printing to `delta`.

---

## Required Dependencies

| Crate | Purpose |
| --- | --- |
| **`pulldown-cmark`** | Event-driven pull-parser for CommonMark. Used to stream Markdown events, identify ` ```patch ` fenced codeblocks, track line numbers, and stream content on-the-fly. |
| **`clap`** (with `derive`) | Subcommand parsing (`test`, `apply`, `show`) and option flags (`--help`, `--version`, `--verbose`). |
| **`tempfile`** | Creation and management of temporary directory clones for safe dry-run execution (`codelog test`). |
| **`anstream` / `anstyle`** | ANSI terminal coloring and style control, adhering to terminal color flags and the `CODELOG_COLOR` environment variable. |
| **`termimad`** (or `pulldown-cmark` terminal event writer) | Direct Markdown-to-terminal ANSI rendering (headers, lists, bold/italic text) without HTML intermediate formats. |
| **`anyhow` / `thiserror`** | Idiomatic CLI error handling and context reporting. |

---

## Markdown Parsing Alternatives Evaluated

| Alternative | Architecture | Pros | Cons / Trade-offs | Selected? |
| --- | --- | --- | --- | --- |
| **`pulldown-cmark` (+ `anstyle` / `termimad`)** | Event-driven pull parser iterator (`Parser::into_offset_iter()`) | • Zero-copy, stream-oriented.<br>• Intercepts ` ```patch ` blocks immediately on event stream to pipe into `delta`.<br>• Accurate line offset tracking. | Requires small custom event handler to render non-patch Markdown to ANSI. | **YES (Primary choice)** |
| **`termimad`** | Markdown terminal renderer built on custom parsing | • Out-of-the-box rich terminal styling (headers, tables, lists, ANSI skins). | Less suited for custom event stream interception (e.g. piping specific codeblocks to sub-processes like `delta`). | **YES (As rendering complement)** |
| **`comrak`** | POSIX-compliant AST arena parser | • Feature-rich GFM support.<br>• Strongly typed AST nodes. | Builds full in-memory AST before traversal rather than streaming event-by-event. | NO |
| **`markdown`** (wooorm `mdast` Rust port) | Full `mdast` syntax tree parser | • Comprehensive AST representation. | Heavier memory model; not event-stream oriented. | NO |

---

## Markdown Parsing & Terminal Delta Integration Architecture

1. **Markdown Event Stream**:
   - `pulldown-cmark` parses Markdown files in a pull-stream manner using `Parser::into_offset_iter()`.
   - Non-patch elements (headers, paragraphs, lists) are formatted directly for terminal output using ANSI terminal styling (via `anstyle` / `termimad`). No HTML output is produced.

2. **Patch Codeblock Detection & `delta` Streaming**:
   - When `pulldown-cmark` yields a `Tag::CodeBlock(CodeBlockKind::Fenced(lang))` where `lang == "patch"`, `codelog` collects the patch text and line offset.
   - For `codelog show`:
     - If `delta` is installed (or enabled via `CODELOG_USE_DELTA`), `codelog` spawns `delta` via `std::process::Command` with standard input redirected (`Stdio::piped()`).
     - The patch block text is piped directly to `delta` for terminal pretty-printing and syntax-highlighted diff rendering.
     - If `delta` is absent or disabled, `codelog` renders syntax-highlighted or ANSI-colored diff lines directly to the terminal standard output.
