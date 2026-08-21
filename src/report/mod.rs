use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Issue {
    file: String,
    line: Option<usize>,
    col: Option<usize>,
    severity: String,
    message: String,
}

pub fn generate_report(
    format: Option<&str>,
    output: Option<&str>,
    open: bool,
    _all: bool,
) -> Result<()> {
    let fmt = format.unwrap_or("html");
    let out = output.unwrap_or("report.html");

    let files = collect_files_for_report()?;
    let issues = collect_issues(&files);

    let content = match fmt {
        "json" => generate_json_report(&issues),
        "markdown" => generate_markdown_report(&issues),
        "csv" => generate_csv_report(&issues),
        _ => generate_html_report(&issues),
    };

    fs::write(&out, content)?;
    println!("Report saved to {}", out);

    if open {
        if cfg!(target_os = "linux") || cfg!(target_os = "android") {
            Command::new("xdg-open").arg(&out).output()?;
        } else if cfg!(target_os = "macos") {
            Command::new("open").arg(&out).output()?;
        } else if cfg!(target_os = "windows") {
            Command::new("start").arg(&out).output()?;
        }
    }

    Ok(())
}

pub fn summary(verbose: bool) -> Result<()> {
    let files = collect_files_for_report()?;
    let issues = collect_issues(&files);

    let errors: Vec<_> = issues.iter().filter(|i| i.severity == "error").collect();
    let warnings: Vec<_> = issues.iter().filter(|i| i.severity == "warning").collect();
    let infos: Vec<_> = issues.iter().filter(|i| i.severity == "info").collect();

    println!("Repository Summary:");
    println!("  Files scanned: {}", files.len());
    println!("  Errors: {}", errors.len());
    println!("  Warnings: {}", warnings.len());
    println!("  Infos: {}", infos.len());

    if verbose {
        if !errors.is_empty() {
            println!("\nErrors:");
            for e in errors {
                println!("  {}", e.message);
            }
        }
        if !warnings.is_empty() {
            println!("\nWarnings:");
            for w in warnings {
                println!("  {}", w.message);
            }
        }
    }

    Ok(())
}

pub fn stats(top: Option<usize>) -> Result<()> {
    let files = collect_files_for_report()?;
    let issues = collect_issues(&files);

    let top_n = top.unwrap_or(10);

    let mut file_counts = std::collections::HashMap::new();
    for issue in &issues {
        *file_counts.entry(&issue.file).or_insert(0) += 1;
    }

    let mut sorted: Vec<_> = file_counts.into_iter().collect();
    sorted.sort_by_key(|a| std::cmp::Reverse(a.1));

    println!("Top {} files with most issues:", top_n);
    for (file, count) in sorted.iter().take(top_n) {
        println!("  {}: {} issues", file, count);
    }

    println!("\nTotal issues: {}", issues.len());
    Ok(())
}

pub fn export_issues(format: Option<&str>, output: Option<&str>) -> Result<()> {
    let fmt = format.unwrap_or("csv");
    let out = output.unwrap_or("issues.csv");

    let files = collect_files_for_report()?;
    let issues = collect_issues(&files);

    let content = match fmt {
        "json" => generate_json_report(&issues),
        "csv" => generate_csv_report(&issues),
        _ => generate_csv_report(&issues),
    };

    fs::write(&out, content)?;
    println!("Exported issues to {}", out);
    Ok(())
}

pub fn import_issues(file: &str) -> Result<()> {
    println!("Importing issues from {} (not yet fully implemented)", file);
    let content = fs::read_to_string(file)?;
    if let Ok(data) = serde_json::from_str::<serde_json::Value>(&content) {
        println!(
            "Imported {} issues",
            data.as_array().map(|a| a.len()).unwrap_or(0)
        );
    } else {
        println!("Failed to parse JSON. Only JSON import is supported.");
    }
    Ok(())
}

fn collect_files_for_report() -> Result<Vec<std::path::PathBuf>> {
    use ignore::WalkBuilder;
    let mut files = Vec::new();
    for entry in WalkBuilder::new(".")
        .git_ignore(true)
        .add_custom_ignore_filename(".prepignore")
        .build()
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && path
                .extension()
                .map(|e| {
                    e != "png"
                        && e != "jpg"
                        && e != "jpeg"
                        && e != "gif"
                        && e != "ico"
                        && e != "bin"
                })
                .unwrap_or(true)
        {
            files.push(path.to_path_buf());
        }
    }
    Ok(files)
}

fn collect_issues(files: &[std::path::PathBuf]) -> Vec<Issue> {
    use crate::checks::run_checks_on_file;
    let mut issues = Vec::new();
    for path in files {
        let results = run_checks_on_file(path);
        for r in results {
            let severity = match r.severity {
                crate::checks::Severity::Error => "error",
                crate::checks::Severity::Warning => "warning",
                crate::checks::Severity::Info => "info",
            };
            let file = path.display().to_string();
            let (line, _col) = if r.message.contains("Line ") {
                let parts: Vec<&str> = r.message.splitn(2, ':').collect();
                if parts.len() == 2 {
                    let line_part = parts[0].replace("Line ", "");
                    if let Ok(line_num) = line_part.trim().parse::<usize>() {
                        (Some(line_num), None::<usize>)
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };
            issues.push(Issue {
                file,
                line,
                col: None,
                severity: severity.to_string(),
                message: r.message,
            });
        }
    }
    issues
}

fn generate_html_report(issues: &[Issue]) -> String {
    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html><head><title>prep Report</title>");
    html.push_str("<style>body{font-family:sans-serif;margin:20px;background:#f5f5f5;}");
    html.push_str(".error{color:#d32f2f;}.warning{color:#f57c00;}.info{color:#1976d2;}");
    html.push_str("table{width:100%;border-collapse:collapse;background:#fff;box-shadow:0 2px 4px rgba(0,0,0,0.1);}");
    html.push_str("th,td{padding:10px;text-align:left;border-bottom:1px solid #ddd;}");
    html.push_str("th{background:#2c3e50;color:#fff;}");
    html.push_str(".severity{font-weight:bold;}</style></head><body>");
    html.push_str("<h1>prep Report</h1>");
    html.push_str(&format!("<p>Total issues: {}</p>", issues.len()));
    html.push_str("<table><tr><th>File</th><th>Line</th><th>Severity</th><th>Message</th></tr>");
    for issue in issues {
        let sev_class = &issue.severity;
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td class='severity {}'>{}</td><td>{}</td></tr>",
            issue.file,
            issue
                .line
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string()),
            sev_class,
            sev_class.to_uppercase(),
            issue.message
        ));
    }
    html.push_str("</table></body></html>");
    html
}

fn generate_json_report(issues: &[Issue]) -> String {
    let json = json!({
        "total": issues.len(),
        "issues": issues
    });
    serde_json::to_string_pretty(&json).unwrap_or_else(|_| "[]".to_string())
}

fn generate_markdown_report(issues: &[Issue]) -> String {
    let mut md = String::new();
    md.push_str("# prep Report\n\n");
    md.push_str(&format!("Total issues: {}\n\n", issues.len()));
    md.push_str("| File | Line | Severity | Message |\n");
    md.push_str("|------|------|----------|---------|\n");
    for issue in issues {
        md.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            issue.file,
            issue
                .line
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string()),
            issue.severity.to_uppercase(),
            issue.message
        ));
    }
    md
}

fn generate_csv_report(issues: &[Issue]) -> String {
    let mut csv = String::new();
    csv.push_str("File,Line,Severity,Message\n");
    for issue in issues {
        csv.push_str(&format!(
            "{},{},{},{}\n",
            issue.file,
            issue
                .line
                .map(|l| l.to_string())
                .unwrap_or_else(|| "-".to_string()),
            issue.severity,
            issue.message
        ));
    }
    csv
}
