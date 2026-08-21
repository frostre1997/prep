// src/core/mod.rs
#![allow(clippy::too_many_arguments)]

use crate::checks::{fix_file, run_checks_on_file, CheckResult, is_binary};
use anyhow::Result;
use colored::*;
use ignore::WalkBuilder;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use regex::Regex;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::PathBuf;

pub fn audit(
    changed: bool,
    verbose: bool,
    exclude: Option<&str>,
    include: Option<&str>,
    follow_symlinks: bool,
    max_depth: Option<usize>,
    _since: Option<&str>,
    _until: Option<&str>,
    no_ignore: bool,
) -> Result<()> {
    let files = collect_files(
        changed,
        exclude,
        include,
        follow_symlinks,
        max_depth,
        no_ignore,
    )?;

    if files.is_empty() {
        println!("No files to scan.");
        return Ok(());
    }

    println!("Scanning {} files...", files.len());

    let pb = ProgressBar::new(files.len() as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    let results: Vec<(PathBuf, Vec<CheckResult>)> = files
        .par_iter()
        .map(|path| {
            let issues = run_checks_on_file(path);
            pb.inc(1);
            (path.clone(), issues)
        })
        .collect();

    pb.finish_and_clear();

    let mut total_errors = 0;
    let mut total_warnings = 0;

    for (path, issues) in results {
        if issues.is_empty() {
            continue;
        }
        println!("\n{}", path.display());
        for issue in issues {
            match issue.severity {
                crate::checks::Severity::Error => {
                    println!("  [ERROR] {}", issue.message.red());
                    total_errors += 1;
                }
                crate::checks::Severity::Warning => {
                    println!("  [WARN] {}", issue.message.yellow());
                    total_warnings += 1;
                }
                crate::checks::Severity::Info => {
                    if verbose {
                        println!("  [INFO] {}", issue.message.cyan());
                    }
                }
            }
        }
    }

    println!(
        "\nDone. Errors: {}, Warnings: {}",
        total_errors, total_warnings
    );
    Ok(())
}

pub fn fix(
    dry_run: bool,
    all: bool,
    trim: bool,
    eof: bool,
    crlf: bool,
    bom: bool,
    _interactive: bool,
    changed: bool,
) -> Result<()> {
    let files = collect_files(changed, None, None, false, None, false)?;

    if files.is_empty() {
        println!("No files to fix.");
        return Ok(());
    }

    println!("Fixing {} files...", files.len());

    let mut fixed_files = 0;
    for path in files {
        let (fixed, issues) = fix_file(&path, dry_run, all, trim, eof, crlf, bom)?;
        if fixed {
            fixed_files += 1;
            if !dry_run {
                println!("[FIXED] {}", path.display());
            } else {
                println!("[DRY RUN] Would fix {}", path.display());
                for issue in issues {
                    println!("  - {}", issue.message);
                }
            }
        }
    }

    if dry_run {
        println!("\nDry run complete. Would fix {} file(s).", fixed_files);
    } else {
        println!("\nFixed {} file(s).", fixed_files);
    }

    Ok(())
}

pub fn search(
    pattern: &str,
    case_insensitive: bool,
    count: bool,
    files_only: bool,
    line_numbers: bool,
    _after: Option<usize>,
    _before: Option<usize>,
    _context: Option<usize>,
    _full_context: bool,
    _replace: Option<&str>,
    changed: bool,
) -> Result<()> {
    let files = collect_files(changed, None, None, false, None, false)?;
    if files.is_empty() {
        println!("No files to search.");
        return Ok(());
    }

    let re = if case_insensitive {
        Regex::new(&format!("(?i){}", pattern))?
    } else {
        Regex::new(pattern)?
    };

    let mut total_matches = 0;

    for path in files {
        if is_binary(&path) {
            continue;
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let lines: Vec<&str> = content.lines().collect();
        let mut matches = Vec::new();

        for (i, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                matches.push((i + 1, line));
            }
        }

        if matches.is_empty() {
            continue;
        }

        if files_only {
            println!("{}", path.display());
            continue;
        }

        if count {
            println!("{}: {}", path.display(), matches.len());
            continue;
        }

        println!("{}", path.display());
        for (line_num, line) in matches {
            if line_numbers {
                println!("  {}: {}", line_num, line);
            } else {
                println!("  {}", line);
            }
            total_matches += 1;
        }
    }

    if !files_only && !count {
        println!("\nTotal matches: {}", total_matches);
    }

    Ok(())
}

pub fn manifest(verify: bool, out: Option<&str>) -> Result<()> {
    use std::collections::HashMap;

    let files = collect_files(false, None, None, false, None, false)?;
    let mut manifest = HashMap::new();

    for path in files {
        if is_binary(&path) {
            continue;
        }
        let mut file = fs::File::open(&path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }
        let hash = hex::encode(hasher.finalize());
        manifest.insert(path.display().to_string(), hash);
    }

    if verify {
        println!("Verifying manifest... (not yet fully implemented)");
        return Ok(());
    }

    let content = manifest
        .iter()
        .map(|(path, hash)| format!("{}  {}", hash, path))
        .collect::<Vec<_>>()
        .join("\n");

    if let Some(output_path) = out {
        fs::write(output_path, content)?;
        println!("Manifest saved to {}", output_path);
    } else {
        println!("{}", content);
    }

    Ok(())
}

pub fn ci_mode(fail_on_warning: bool, threshold: Option<usize>) -> Result<()> {
    let files = collect_files(false, None, None, false, None, false)?;
    let mut errors = 0;
    let mut warnings = 0;

    for path in files {
        let issues = crate::checks::run_checks_on_file(&path);
        for issue in issues {
            match issue.severity {
                crate::checks::Severity::Error => errors += 1,
                crate::checks::Severity::Warning => warnings += 1,
                _ => {}
            }
        }
    }

    println!("Errors: {}, Warnings: {}", errors, warnings);

    if errors > 0 {
        std::process::exit(1);
    }

    if fail_on_warning && warnings > 0 {
        std::process::exit(1);
    }

    if let Some(limit) = threshold {
        if warnings > limit {
            std::process::exit(1);
        }
    }

    Ok(())
}

pub fn repo_info(detailed: bool) -> Result<()> {
    let mut file_count = 0;
    let mut total_size = 0;
    let mut languages = std::collections::HashMap::new();

    for entry in WalkBuilder::new(".")
        .git_ignore(true)
        .add_custom_ignore_filename(".prepignore")
        .build()
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            file_count += 1;
            if let Ok(metadata) = fs::metadata(path) {
                total_size += metadata.len();
            }
            if let Some(ext) = path.extension() {
                let ext = ext.to_string_lossy().to_lowercase();
                *languages.entry(ext).or_insert(0) += 1;
            }
        }
    }

    println!("Repository Info:");
    println!("  Files: {}", file_count);
    println!("  Total size: {} MB", total_size / (1024 * 1024));
    println!("  Top extensions:");

    let mut sorted: Vec<_> = languages.into_iter().collect();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.1));
    for (ext, count) in sorted.iter().take(5) {
        println!("    .{}: {}", ext, count);
    }

    if detailed {
        let output = std::process::Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output();
        if let Ok(out) = output {
            let hash = String::from_utf8_lossy(&out.stdout);
            println!("  Git commit: {}", hash.trim());
        }
    }

    Ok(())
}

pub fn watch_files(interval: Option<u64>, fix: bool) -> Result<()> {
    use std::time::Duration;
    use std::thread;

    let interval = interval.unwrap_or(5);
    println!("Watching for changes (interval: {}s)...", interval);

    loop {
        println!("\n[Watch] Scanning...");
        let files = collect_files(false, None, None, false, None, false)?;
        let mut issues_found = false;

        for path in files {
            let issues = crate::checks::run_checks_on_file(&path);
            if !issues.is_empty() {
                issues_found = true;
                println!("{}:", path.display());
                for issue in issues {
                    println!("  {}", issue.message);
                }
                if fix {
                    let _ = crate::checks::fix_file(&path, false, false, true, true, false, false);
                    println!("  [FIXED]");
                }
            }
        }

        if !issues_found {
            println!("No issues found.");
        }

        thread::sleep(Duration::from_secs(interval));
    }
}

pub fn trim_whitespace(dry_run: bool) -> Result<()> {
    let files = collect_files(false, None, None, false, None, false)?;
    let mut trimmed = 0;

    for path in files {
        if crate::checks::is_binary(&path) {
            continue;
        }
        let content = fs::read_to_string(&path)?;
        let lines: Vec<String> = content.lines().map(|s| s.trim_end().to_string()).collect();
        let new_content = lines.join("\n");
        if new_content != content {
            trimmed += 1;
            if !dry_run {
                fs::write(&path, new_content)?;
                println!("Trimmed: {}", path.display());
            } else {
                println!("Would trim: {}", path.display());
            }
        }
    }

    if dry_run {
        println!("\nWould trim {} file(s).", trimmed);
    } else {
        println!("\nTrimmed {} file(s).", trimmed);
    }

    Ok(())
}

pub fn show_version(check: bool) -> Result<()> {
    println!("prep version 0.100.0");
    if check {
        println!("Checking for updates...");
        let output = std::process::Command::new("curl")
            .args(["-s", "https://api.github.com/repos/frostre1997/prep/releases/latest"])
            .output();
        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            if let Some(tag) = stdout
                .lines()
                .find(|l| l.contains("\"tag_name\""))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().trim_matches('"').trim_matches(','))
            {
                println!("Latest release: {}", tag);
            }
        }
    }
    Ok(())
}

pub fn show_examples() -> Result<()> {
    println!("prep Examples:");
    println!("  prep audit                        Scan repository for issues");
    println!("  prep audit -r                     Scan only changed files");
    println!("  prep fix --dry-run                Show what would be fixed");
    println!("  prep fix                          Auto-fix issues");
    println!("  prep search \"TODO\"                Find TODO comments");
    println!("  prep search -i \"error\"            Case-insensitive search");
    println!("  prep manifest                     Generate SHA-256 manifest");
    println!("  prep manifest --out manifest.txt  Save manifest to file");
    println!("  ./gradlew build 2>&1 | prep build  Parse build output");
    println!("  prep build --run --tool gradle    Run and parse build");
    println!("  prep report --format html         Generate HTML report");
    println!("  prep report --format json         Generate JSON report");
    println!("  prep hooks install                Install pre-commit hook");
    println!("  prep hooks status                 Check hook status");
    println!("  prep blame                        Show git blame for issues");
    println!("  prep diff                         Scan changed files");
    println!("  prep info                         Show repository info");
    println!("  prep clean                        Clean temporary files");
    println!("  prep init                         Create .prepignore and config");
    println!("  prep version                      Show version");
    Ok(())
}

fn collect_files(
    changed: bool,
    exclude: Option<&str>,
    include: Option<&str>,
    follow_symlinks: bool,
    max_depth: Option<usize>,
    no_ignore: bool,
) -> Result<Vec<PathBuf>> {
    if changed {
        return get_changed_files();
    }

    let mut builder = WalkBuilder::new(".");
    if !no_ignore {
        builder.add_custom_ignore_filename(".prepignore");
        builder.git_ignore(true);
        builder.git_global(true);
        builder.git_exclude(true);
    }
    builder.follow_links(follow_symlinks);
    if let Some(depth) = max_depth {
        builder.max_depth(Some(depth));
    }

    let exclude_glob = exclude.map(glob::Pattern::new).transpose()?;
    let include_glob = include.map(glob::Pattern::new).transpose()?;

    let mut files = Vec::new();
    for entry in builder.build() {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let path_str = path.to_string_lossy();
            if let Some(excl) = &exclude_glob {
                if excl.matches(&path_str) {
                    continue;
                }
            }
            if let Some(incl) = &include_glob {
                if !incl.matches(&path_str) {
                    continue;
                }
            }
            files.push(path.to_path_buf());
        }
    }

    Ok(files)
}

fn get_changed_files() -> Result<Vec<PathBuf>> {
    let output = std::process::Command::new("git")
        .args(["diff", "--name-only", "HEAD~1"])
        .output()?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let stdout = String::from_utf8(output.stdout)?;
    let files: Vec<PathBuf> = stdout
        .lines()
        .map(PathBuf::from)
        .filter(|p| p.exists() && p.is_file())
        .collect();
    Ok(files)
}
