use anyhow::Result;
use colored::*;
use regex::Regex;
use std::collections::HashSet;
use std::io::{BufRead, BufReader, IsTerminal};
use std::process::Command;

pub fn parse_build(_run: bool, _watch: bool, _tool: Option<&str>, _save: bool) -> Result<()> {
    if _run {
        return run_and_parse_build(_tool);
    }

    if _watch {
        eprintln!("--watch is not yet implemented.");
        return Ok(());
    }

    if std::io::stdin().is_terminal() {
        eprintln!("Please pipe build output into prep build, or use --run");
        eprintln!("Example: ./gradlew assembleDebug 2>&1 | prep build");
        return Ok(());
    }

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let lines: Vec<String> = reader.lines().filter_map(Result::ok).collect();

    let (errors, warnings, task_failure, status) = parse_build_output(&lines);

    if errors.is_empty() && warnings.is_empty() && task_failure.is_none() && status.is_none() {
        println!("No build output detected.");
        return Ok(());
    }

    format_and_print_output(errors, warnings, task_failure, status);

    Ok(())
}

fn run_and_parse_build(tool: Option<&str>) -> Result<()> {
    let default_tool = "gradle";
    let tool_name = tool.unwrap_or(default_tool);

    let (cmd, args) = match tool_name {
        "gradle" => ("gradle", vec!["assembleDebug"]),
        "maven" => ("mvn", vec!["compile"]),
        "cargo" => ("cargo", vec!["build"]),
        _ => {
            eprintln!(
                "Unsupported tool: {}. Use gradle, maven, or cargo.",
                tool_name
            );
            return Ok(());
        }
    };

    let output = Command::new(cmd).args(&args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);
    let lines: Vec<String> = combined.lines().map(String::from).collect();

    let (errors, warnings, task_failure, status) = parse_build_output(&lines);
    format_and_print_output(errors, warnings, task_failure, status);

    Ok(())
}

#[derive(Debug, Clone)]
pub struct BuildIssue {
    pub file: Option<String>,
    pub line: Option<usize>,
    pub col: Option<usize>,
    pub message: String,
    pub raw: String,
}

pub fn parse_build_output(
    lines: &[String],
) -> (
    Vec<BuildIssue>,
    Vec<BuildIssue>,
    Option<String>,
    Option<String>,
) {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut task_failure = None;
    let mut status = None;
    let mut in_what_went_wrong = false;
    let mut what_lines = Vec::new();
    let mut captured = HashSet::new();

    let error_patterns = [
        Regex::new(r"^e:\s+file://([^:]+):(\d+):(\d+)\s+(.*)").unwrap(),
        Regex::new(r"^([^:]+):(\d+):\s+error:\s+(.*)").unwrap(),
        Regex::new(r"^(.*?):(\d+):\s+error:\s+(.*)").unwrap(),
        Regex::new(r"^(.*?):\s+error:\s+(.*)").unwrap(),
    ];
    let warning_patterns = [
        Regex::new(r"^w:\s+file://([^:]+):(\d+):(\d+)\s+(.*)").unwrap(),
        Regex::new(r"^([^:]+):(\d+):\s+warning:\s+(.*)").unwrap(),
        Regex::new(r"^(.*?):(\d+):\s+warning:\s+(.*)").unwrap(),
        Regex::new(r"^(.*?):\s+warning:\s+(.*)").unwrap(),
    ];

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("BUILD SUCCESSFUL") || line.starts_with("BUILD FAILED") {
            status = Some(line.to_string());
            continue;
        }

        if line.starts_with("* What went wrong:") {
            in_what_went_wrong = true;
            what_lines.clear();
            continue;
        }
        if in_what_went_wrong && line.starts_with("* Try:") {
            in_what_went_wrong = false;
            if !what_lines.is_empty() {
                let msg = what_lines.join(" ");
                let key = format!("config:{}", msg);
                if !captured.contains(&key) {
                    errors.push(BuildIssue {
                        file: None,
                        line: None,
                        col: None,
                        message: msg.clone(),
                        raw: msg,
                    });
                    captured.insert(key);
                }
            }
            continue;
        }
        if in_what_went_wrong {
            what_lines.push(line.to_string());
            continue;
        }

        if line.starts_with("FAILURE:") && !line.starts_with("BUILD FAILED") {
            continue;
        }

        if line.contains("Execution failed for task") {
            task_failure = Some(line.to_string());
            continue;
        }

        let mut matched = false;
        for pat in &error_patterns {
            if let Some(caps) = pat.captures(line) {
                let groups: Vec<&str> = caps.iter().map(|m| m.unwrap().as_str()).collect();
                let (file, line_num, col, msg) = if groups.len() == 5 {
                    (
                        Some(groups[1].to_string()),
                        Some(groups[2].parse().unwrap_or(0)),
                        Some(groups[3].parse().unwrap_or(0)),
                        groups[4].to_string(),
                    )
                } else if groups.len() == 4 {
                    (
                        Some(groups[1].to_string()),
                        Some(groups[2].parse().unwrap_or(0)),
                        None,
                        groups[3].to_string(),
                    )
                } else {
                    (None, None, None, line.to_string())
                };
                let key = format!("err:{:?}:{:?}:{:?}:{}", file, line_num, col, msg);
                if !captured.contains(&key) {
                    errors.push(BuildIssue {
                        file,
                        line: line_num,
                        col,
                        message: msg,
                        raw: line.to_string(),
                    });
                    captured.insert(key);
                }
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        for pat in &warning_patterns {
            if let Some(caps) = pat.captures(line) {
                let groups: Vec<&str> = caps.iter().map(|m| m.unwrap().as_str()).collect();
                let (file, line_num, col, msg) = if groups.len() == 5 {
                    (
                        Some(groups[1].to_string()),
                        Some(groups[2].parse().unwrap_or(0)),
                        Some(groups[3].parse().unwrap_or(0)),
                        groups[4].to_string(),
                    )
                } else if groups.len() == 4 {
                    (
                        Some(groups[1].to_string()),
                        Some(groups[2].parse().unwrap_or(0)),
                        None,
                        groups[3].to_string(),
                    )
                } else {
                    (None, None, None, line.to_string())
                };
                let key = format!("warn:{:?}:{:?}:{:?}:{}", file, line_num, col, msg);
                if !captured.contains(&key) {
                    warnings.push(BuildIssue {
                        file,
                        line: line_num,
                        col,
                        message: msg,
                        raw: line.to_string(),
                    });
                    captured.insert(key);
                }
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        let lower = line.to_lowercase();
        if (lower.contains("error") || lower.contains("exception"))
            && !line.starts_with("FAILURE:")
            && !line.starts_with("BUILD FAILED")
        {
            let key = format!("raw_err:{}", line);
            if !captured.contains(&key) {
                errors.push(BuildIssue {
                    file: None,
                    line: None,
                    col: None,
                    message: line.to_string(),
                    raw: line.to_string(),
                });
                captured.insert(key);
            }
        } else if lower.contains("warning") {
            let key = format!("raw_warn:{}", line);
            if !captured.contains(&key) {
                warnings.push(BuildIssue {
                    file: None,
                    line: None,
                    col: None,
                    message: line.to_string(),
                    raw: line.to_string(),
                });
                captured.insert(key);
            }
        }
    }

    if status.is_none() {
        status = if !errors.is_empty() || task_failure.is_some() {
            Some("BUILD FAILED".to_string())
        } else {
            Some("BUILD SUCCESSFUL".to_string())
        };
    }

    (errors, warnings, task_failure, status)
}

pub fn format_and_print_output(
    errors: Vec<BuildIssue>,
    warnings: Vec<BuildIssue>,
    task_failure: Option<String>,
    status: Option<String>,
) {
    let mut output = Vec::new();

    for err in errors {
        if let (Some(file), Some(line)) = (&err.file, &err.line) {
            output.push(format!("[error] - {}", file));
            if let Some(col) = &err.col {
                output.push(format!("- [line] - {}:{}", line, col));
            } else {
                output.push(format!("- [line] - {}", line));
            }
            output.push("*reason of the error".to_string());
            output.push(format!(" > {}", err.message));
            output.push("".to_string());
        } else {
            let raw = &err.raw;
            if raw.contains(" > ") {
                let parts: Vec<&str> = raw.splitn(2, " > ").collect();
                output.push(format!("[error] - {}", parts[0]));
                output.push("*reason of the error".to_string());
                output.push(format!(" > {}", parts[1]));
            } else {
                output.push(format!("[error] - {}", raw));
                output.push("*reason of the error".to_string());
                output.push(format!(" > {}", err.message));
            }
            output.push("".to_string());
        }
    }

    for warn in warnings {
        if let (Some(file), Some(line)) = (&warn.file, &warn.line) {
            output.push(format!("[warning] - {}", file));
            if let Some(col) = &warn.col {
                output.push(format!("- [line] - {}:{}", line, col));
            } else {
                output.push(format!("- [line] - {}", line));
            }
            output.push("*reason of the warning".to_string());
            output.push(format!(" > {}", warn.message));
            output.push("".to_string());
        } else {
            output.push(format!("[warning] - {}", warn.raw));
            output.push("*reason of the warning".to_string());
            output.push(format!(" > {}", warn.message));
            output.push("".to_string());
        }
    }

    if let Some(tf) = task_failure {
        output.push(tf);
    }

    if let Some(st) = status {
        output.push(st);
    }

    for line in output {
        if line.starts_with("[error]") {
            println!("{}", line.red());
        } else if line.starts_with("[warning]") {
            println!("{}", line.yellow());
        } else {
            println!("{}", line);
        }
    }
}
