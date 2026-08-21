// src/checks/mod.rs
use anyhow::Result;
use regex::Regex;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub severity: Severity,
    pub message: String,
}

// ------------------------------------------------------------------------
// Run all checks on a file
// ------------------------------------------------------------------------

pub fn run_checks_on_file(path: &Path) -> Vec<CheckResult> {
    let mut results = Vec::new();

    if is_binary(path) {
        return results;
    }

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => {
            results.push(CheckResult {
                severity: Severity::Error,
                message: "Failed to read file (possibly binary or encoding issue)".to_string(),
            });
            return results;
        }
    };

    let lines: Vec<&str> = content.lines().collect();

    // 1. Secrets
    let secret_patterns = [
        (r"AKIA[0-9A-Z]{16}", "AWS Access Key"),
        (r"-----BEGIN (RSA|DSA|EC) PRIVATE KEY-----", "Private Key"),
        (
            r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
            "JWT Token",
        ),
    ];
    for (i, line) in lines.iter().enumerate() {
        for (pat, desc) in &secret_patterns {
            if let Ok(re) = Regex::new(pat) {
                if re.is_match(line) {
                    results.push(CheckResult {
                        severity: Severity::Error,
                        message: format!("Line {}: Possible {} found", i + 1, desc),
                    });
                }
            }
        }
    }

    // 2. Merge conflicts
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("<<<<<<< HEAD")
            || line.starts_with(">>>>>>>")
            || line.starts_with("=======")
        {
            results.push(CheckResult {
                severity: Severity::Error,
                message: format!("Line {}: Unresolved merge conflict marker", i + 1),
            });
        }
    }

    // 3. Trailing whitespace
    for (i, line) in lines.iter().enumerate() {
        if line.ends_with(' ') || line.ends_with('\t') {
            results.push(CheckResult {
                severity: Severity::Warning,
                message: format!("Line {}: Trailing whitespace", i + 1),
            });
        }
    }

    // 4. Missing newline at EOF
    if !lines.is_empty() && !content.ends_with('\n') {
        results.push(CheckResult {
            severity: Severity::Warning,
            message: "No newline at end of file".to_string(),
        });
    }

    // 5. CRLF line endings
    for (i, line) in lines.iter().enumerate() {
        if line.contains('\r') {
            results.push(CheckResult {
                severity: Severity::Info,
                message: format!("Line {}: CRLF line ending (should be LF)", i + 1),
            });
        }
    }

    // 6. BOM (Byte Order Mark)
    if content.starts_with('\u{feff}') {
        results.push(CheckResult {
            severity: Severity::Info,
            message: "File contains UTF-8 BOM (should be removed)".to_string(),
        });
    }

    results
}

// ------------------------------------------------------------------------
// Fix file
// ------------------------------------------------------------------------

pub fn fix_file(
    path: &Path,
    dry_run: bool,
    all: bool,
    trim: bool,
    eof: bool,
    crlf: bool,
    bom: bool,
) -> Result<(bool, Vec<CheckResult>)> {
    let mut fixed = false;
    let mut issues = Vec::new();

    if is_binary(path) {
        return Ok((false, issues));
    }

    let content = fs::read_to_string(path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Default: fix trailing whitespace and EOF if not overriding
    let fix_trim = trim || !all;
    let fix_eof = eof || !all;
    let fix_crlf = crlf || all;
    let fix_bom = bom || all;

    // 1. Trim trailing whitespace
    if fix_trim {
        let mut changed = false;
        for line in &mut lines {
            if line.ends_with(' ') || line.ends_with('\t') {
                *line = line.trim_end().to_string();
                changed = true;
            }
        }
        if changed {
            fixed = true;
            issues.push(CheckResult {
                severity: Severity::Info,
                message: "Removed trailing whitespace".to_string(),
            });
        }
    }

    // 2. Fix EOF newline
    if fix_eof {
        if !lines.is_empty() {
            let last = lines.last().unwrap();
            if !last.is_empty() {
                lines.push("".to_string());
                fixed = true;
                issues.push(CheckResult {
                    severity: Severity::Info,
                    message: "Added missing newline at EOF".to_string(),
                });
            }
        }
    }

    // 3. Fix CRLF -> LF
    if fix_crlf {
        let mut changed = false;
        for line in &mut lines {
            if line.contains('\r') {
                *line = line.replace('\r', "");
                changed = true;
            }
        }
        if changed {
            fixed = true;
            issues.push(CheckResult {
                severity: Severity::Info,
                message: "Converted CRLF to LF".to_string(),
            });
        }
    }

    // 4. Remove BOM
    if fix_bom {
        let mut content_bytes = fs::read(path)?;
        if content_bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
            content_bytes.drain(0..3);
            if !dry_run {
                fs::write(path, content_bytes)?;
            }
            fixed = true;
            issues.push(CheckResult {
                severity: Severity::Info,
                message: "Removed UTF-8 BOM".to_string(),
            });
            return Ok((fixed, issues));
        }
    }

    // Write the modified content (if any changes and not dry-run)
    if fixed && !dry_run {
        let new_content = lines.join("\n");
        fs::write(path, new_content)?;
    }

    Ok((fixed, issues))
}

// ------------------------------------------------------------------------
// Stubs for missing check commands
// ------------------------------------------------------------------------

pub fn license_check(_action: &crate::cli::commands::LicenseAction) -> Result<()> {
    println!("License check (not yet implemented)");
    Ok(())
}

pub fn deps_check(_action: &crate::cli::commands::DepsAction) -> Result<()> {
    println!("Dependency check (not yet implemented)");
    Ok(())
}

pub fn find_duplicates(_delete: bool, _move_to_trash: bool) -> Result<()> {
    println!("Duplicate finder (not yet implemented)");
    Ok(())
}

// ------------------------------------------------------------------------
// Helper: check if file is binary (by looking for null bytes)
// ------------------------------------------------------------------------

pub fn is_binary(path: &Path) -> bool {
    if let Ok(mut file) = fs::File::open(path) {
        let mut buffer = [0; 8192];
        if let Ok(n) = file.read(&mut buffer) {
            return buffer[..n].contains(&0);
        }
    }
    true
}
