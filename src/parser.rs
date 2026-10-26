use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchBlock {
    pub file_path: PathBuf,
    pub index: usize,
    pub start_line: usize,
    pub content: String,
}

pub fn byte_offset_to_line_number(text: &str, offset: usize) -> usize {
    let mut line = 1;
    for (i, b) in text.bytes().enumerate() {
        if i >= offset {
            break;
        }
        if b == b'\n' {
            line += 1;
        }
    }
    line
}

pub fn parse_patch_blocks(path: &Path) -> anyhow::Result<Vec<PatchBlock>> {
    let content = fs::read_to_string(path)?;
    let parser = Parser::new_ext(&content, Options::all()).into_offset_iter();

    let mut patch_blocks = Vec::new();
    let mut current_patch: Option<(usize, usize, String)> = None; // (index, start_line, content)
    let mut patch_count = 0;

    for (event, range) in parser {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                if lang.trim().eq_ignore_ascii_case("patch") {
                    patch_count += 1;
                    let line_no = byte_offset_to_line_number(&content, range.start);
                    current_patch = Some((patch_count, line_no, String::new()));
                }
            }
            Event::Text(text) => {
                if let Some((_, _, ref mut block_content)) = current_patch {
                    block_content.push_str(&text);
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((idx, line_no, block_content)) = current_patch.take() {
                    patch_blocks.push(PatchBlock {
                        file_path: path.to_path_buf(),
                        index: idx,
                        start_line: line_no,
                        content: block_content,
                    });
                }
            }
            _ => {}
        }
    }

    Ok(patch_blocks)
}
