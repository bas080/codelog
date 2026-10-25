# Glossary

This document defines key terms, concepts, and architectural components used in `codelog`.

---

## Terms and Definitions

### Codelog
A Markdown document that combines human-readable context and explanations (e.g., motivation, rationale, technical notes) with executable git patch codeblocks.

### Patch Codeblock
A fenced codeblock tagged specifically with ` ```patch ` in a Codelog Markdown file. These codeblocks contain standard `git diff` / `git patch` formatting and are parsed by `codelog` to be tested or applied via `git apply`.

### Non-Patch Codeblock
Fenced codeblocks in Markdown tagged with languages other than `patch` (e.g. ` ```rust `, ` ```bash `, ` ```json `). `codelog` ignores non-patch codeblocks during `test` and `apply` operations.

### Dry-Run Testing (`codelog test`)
The execution mode where `codelog` creates a temporary clone of the current repository in a separate directory (`/tmp`) and attempts to apply all ` ```patch ` codeblocks sequentially to verify they apply cleanly without mutating the user's active working directory.

### Direct Application (`codelog apply`)
The execution mode where `codelog` extracts and applies ` ```patch ` codeblocks directly to the current active git repository working tree.

### Pretty-Printing & Review (`codelog show`)
The command used to inspect and display the patch codeblocks from a Codelog Markdown file.

### Delta Integration
Automatic integration with [`delta`](https://github.com/dandavison/delta), a terminal diff viewer. When `delta` is installed (or enabled via `CODELOG_USE_DELTA`), `codelog show` pipes diff output through `delta` for syntax highlighting and formatting.

### Temporary Repository Clone
A temporary copy of the local git repository created during `codelog test` (by default under the system temp folder or `CODELOG_TMPDIR`). Used as an isolated sandbox for testing patches.

### AGPLv3 (GNU Affero General Public License v3.0)
The open-source software license under which `codelog` is distributed.
