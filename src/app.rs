use anyhow::{bail, Context, Result};
use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use tempfile::TempDir;

use crate::cli::Cli;
use crate::parser::{parse_patch_blocks, PatchBlock};

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        crate::cli::Commands::Test(args) => run_test(&args.files, cli.verbose),
        crate::cli::Commands::Apply(args) => run_apply(&args.files, cli.verbose),
        crate::cli::Commands::Show(args) => run_show(&args.files, cli.verbose),
    }
}

pub fn resolve_and_sort_files(files: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut resolved = if files.is_empty() {
        let codelog_dir = PathBuf::from("codelog");
        if !codelog_dir.exists() || !codelog_dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut found = Vec::new();
        for entry in fs::read_dir(codelog_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                found.push(path);
            }
        }
        found
    } else {
        files.to_vec()
    };

    resolved.sort_by(|a, b| a.to_string_lossy().cmp(&b.to_string_lossy()));
    Ok(resolved)
}

fn apply_patch_to_dir(repo_dir: &Path, patch: &PatchBlock) -> Result<(), String> {
    let mut temp_patch_file = tempfile::NamedTempFile::new().map_err(|e| e.to_string())?;
    temp_patch_file
        .write_all(patch.content.as_bytes())
        .map_err(|e| e.to_string())?;
    let patch_path = temp_patch_file.path();

    let output = Command::new("git")
        .current_dir(repo_dir)
        .arg("apply")
        .arg(patch_path)
        .output()
        .map_err(|e| format!("Failed to execute git apply: {}", e))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let combined = if stderr.is_empty() {
            stdout
        } else if stdout.is_empty() {
            stderr
        } else {
            format!("{}\n{}", stdout, stderr)
        };
        Err(combined.trim().to_string())
    }
}

fn create_empty_temp_repo() -> Result<(PathBuf, Option<TempDir>)> {
    let custom_tmpdir = env::var("CODELOG_TMPDIR").ok().filter(|s| !s.is_empty());

    let (target_dir, temp_guard) = if let Some(tmpdir_str) = custom_tmpdir {
        let custom_path = PathBuf::from(&tmpdir_str);
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let target_dir = custom_path.join(format!("codelog-repo-{}", timestamp));
        fs::create_dir_all(&target_dir)?;
        (target_dir, None)
    } else {
        let temp_dir = tempfile::Builder::new()
            .prefix("codelog-repo-")
            .tempdir()
            .context("Failed to create temporary directory")?;
        let target_dir = temp_dir.path().to_path_buf();
        (target_dir, Some(temp_dir))
    };

    let init_output = Command::new("git")
        .current_dir(&target_dir)
        .arg("init")
        .output()
        .context("Failed to execute git init in temporary directory")?;

    if !init_output.status.success() {
        let err = String::from_utf8_lossy(&init_output.stderr);
        bail!("Failed to initialize empty git repo in temp dir: {}", err);
    }

    Ok((target_dir, temp_guard))
}

fn run_test(files: &[PathBuf], verbose: bool) -> Result<()> {
    let target_files = resolve_and_sort_files(files)?;
    if target_files.is_empty() {
        println!("[codelog] No codelog Markdown files found.");
        return Ok(());
    }

    let (temp_repo_dir, _temp_dir_guard) = create_empty_temp_repo()?;

    println!(
        "[codelog] Initialized empty repository in temporary directory: {}",
        temp_repo_dir.display()
    );

    let mut total_blocks = Vec::new();
    for file in &target_files {
        let blocks = parse_patch_blocks(file)
            .with_context(|| format!("Failed to parse markdown file: {}", file.display()))?;
        total_blocks.extend(blocks);
    }

    let file_names_str = target_files
        .iter()
        .map(|f| f.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    println!(
        "[codelog] Found {} patch codeblock(s) across sorted file(s): {}",
        total_blocks.len(),
        file_names_str
    );
    println!();

    let total_count = total_blocks.len();
    for (i, patch) in total_blocks.iter().enumerate() {
        let idx = i + 1;
        let file_str = patch.file_path.display();
        print!(
            "[{}/{}] Applying patch block #{} ({}:{})... ",
            idx, total_count, patch.index, file_str, patch.start_line
        );
        std::io::stdout().flush().ok();

        match apply_patch_to_dir(&temp_repo_dir, patch) {
            Ok(()) => {
                println!("SUCCESS");
            }
            Err(err_msg) => {
                println!("FAILED\n");
                println!(
                    "✖ Error: Patch block #{} in {}:{} failed to apply.\n",
                    patch.index, file_str, patch.start_line
                );
                println!("--- Raw 'git apply' output ---");
                println!("{}", err_msg);
                println!("------------------------------\n");
                bail!("Patch execution failed.");
            }
        }
    }

    println!();
    println!("✔ All patches applied successfully in temporary empty repository!\n");
    if verbose {
        println!("Follow-up steps:");
        println!(
            "  • Inspect generated source in temp repo: cd {}",
            temp_repo_dir.display()
        );
        println!(
            "  • Clean up temporary repo:              rm -rf {}",
            temp_repo_dir.display()
        );
    }

    Ok(())
}

fn run_apply(files: &[PathBuf], verbose: bool) -> Result<()> {
    let current_dir = env::current_dir().context("Failed to get current working directory")?;
    let target_files = resolve_and_sort_files(files)?;

    if target_files.is_empty() {
        println!("[codelog] No codelog Markdown files found.");
        return Ok(());
    }

    let mut total_blocks = Vec::new();
    for file in &target_files {
        let blocks = parse_patch_blocks(file)
            .with_context(|| format!("Failed to parse markdown file: {}", file.display()))?;
        total_blocks.extend(blocks);
    }

    if verbose {
        println!(
            "[codelog] Applying {} patch block(s) in alphanumeric order directly to working directory",
            total_blocks.len()
        );
    }

    for patch in &total_blocks {
        let file_str = patch.file_path.display();
        match apply_patch_to_dir(&current_dir, patch) {
            Ok(()) => {
                println!(
                    "Applied patch block #{} ({}:{}) successfully.",
                    patch.index, file_str, patch.start_line
                );
            }
            Err(err_msg) => {
                println!(
                    "✖ Error: Failed to apply patch block #{} in {}:{}",
                    patch.index, file_str, patch.start_line
                );
                println!("--- Raw 'git apply' output ---");
                println!("{}", err_msg);
                println!("------------------------------");
                bail!("Patch execution failed.");
            }
        }
    }

    println!("✔ All patches applied successfully.");
    Ok(())
}

fn is_delta_available() -> bool {
    let env_setting = env::var("CODELOG_USE_DELTA").unwrap_or_default();
    if env_setting == "false" || env_setting == "0" {
        return false;
    }
    if env_setting == "true" || env_setting == "1" {
        return true;
    }

    Command::new("delta")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn run_show(files: &[PathBuf], _verbose: bool) -> Result<()> {
    let use_delta = is_delta_available();
    let target_files = resolve_and_sort_files(files)?;

    if target_files.is_empty() {
        println!("[codelog] No codelog Markdown files found.");
        return Ok(());
    }

    for file in &target_files {
        let blocks = parse_patch_blocks(file)
            .with_context(|| format!("Failed to parse markdown file: {}", file.display()))?;

        println!("=== File: {} ===", file.display());
        for patch in blocks {
            println!(
                "--- Patch Block #{} ({}:{}) ---",
                patch.index,
                patch.file_path.display(),
                patch.start_line
            );

            if use_delta {
                if let Ok(mut child) = Command::new("delta")
                    .stdin(Stdio::piped())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn()
                {
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(patch.content.as_bytes());
                    }
                    let _ = child.wait();
                    continue;
                }
            }

            for line in patch.content.lines() {
                if line.starts_with('+') && !line.starts_with("+++") {
                    println!("\x1b[32m{}\x1b[0m", line);
                } else if line.starts_with('-') && !line.starts_with("---") {
                    println!("\x1b[31m{}\x1b[0m", line);
                } else if line.starts_with('@') {
                    println!("\x1b[36m{}\x1b[0m", line);
                } else {
                    println!("{}", line);
                }
            }
            println!();
        }
    }

    Ok(())
}
