use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn blame(author: Option<&str>, since: Option<&str>, short: bool) -> Result<()> {
    let mut args = vec!["blame"];
    if let Some(s) = since {
        args.push("--since");
        args.push(s);
    }
    if short {
        args.push("-s");
    }
    args.push(".");

    let output = Command::new("git").args(&args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    if let Some(auth) = author {
        for line in stdout.lines() {
            if line.contains(auth) {
                println!("{}", line);
            }
        }
    } else {
        print!("{}", stdout);
    }

    Ok(())
}

pub fn diff(staged: bool, uncommitted: bool, base: Option<&str>) -> Result<()> {
    let mut args = vec!["diff"];
    if staged {
        args.push("--staged");
    }
    if let Some(b) = base {
        args.push(b);
    }
    if uncommitted {
        // `git diff` without arguments shows uncommitted changes
        // already, so we just run without extra flags
    }

    let output = Command::new("git").args(&args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);

    Ok(())
}

pub fn manage_hooks(action: &crate::cli::commands::HooksAction) -> Result<()> {
    match action {
        crate::cli::commands::HooksAction::Install { force, all } => install_hook(*force, *all),
        crate::cli::commands::HooksAction::Uninstall => uninstall_hook(),
        crate::cli::commands::HooksAction::Status => hook_status(),
        crate::cli::commands::HooksAction::Run { fix } => run_hook(*fix),
    }
}

fn install_hook(force: bool, all: bool) -> Result<()> {
    let hook_dir = ".git/hooks";
    if !Path::new(hook_dir).exists() {
        println!("Not a git repository. Run `git init` first.");
        return Ok(());
    }

    let hook_path = format!("{}/pre-commit", hook_dir);

    if Path::new(&hook_path).exists() && !force {
        println!("pre-commit hook already exists. Use --force to overwrite.");
        return Ok(());
    }

    // Get absolute path to the current prep binary
    let prep_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "prep".to_string());

    let script = format!(
        r#"#!/bin/sh
# prep pre-commit hook
# Using absolute path: {}

echo "Running prep audit..."
{} audit
if [ $? -ne 0 ]; then
    echo "Aborting commit due to prep audit errors."
    exit 1
fi

echo "Running prep fix --dry-run..."
{} fix --dry-run
if [ $? -ne 0 ]; then
    echo "Aborting commit due to prep fix issues."
    exit 1
fi

echo "prep checks passed."
"#,
        prep_path, prep_path, prep_path
    );

    fs::write(&hook_path, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    println!("pre-commit hook installed at {}", hook_path);
    if all {
        println!("Installing other hooks... (not yet implemented)");
    }
    Ok(())
}

fn uninstall_hook() -> Result<()> {
    let hook_path = ".git/hooks/pre-commit";
    if Path::new(hook_path).exists() {
        fs::remove_file(hook_path)?;
        println!("pre-commit hook removed.");
    } else {
        println!("No pre-commit hook found.");
    }
    Ok(())
}

fn hook_status() -> Result<()> {
    let hook_path = ".git/hooks/pre-commit";
    if Path::new(hook_path).exists() {
        println!("pre-commit hook is installed.");
        let content = fs::read_to_string(hook_path)?;
        println!("Hook content:\n{}", content);
    } else {
        println!("No pre-commit hook installed.");
    }
    Ok(())
}

fn run_hook(fix: bool) -> Result<()> {
    let prep_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "prep".to_string());

    if fix {
        println!("Running prep fix...");
        let status = Command::new(&prep_path).args(["fix"]).status()?;
        if !status.success() {
            println!("prep fix failed.");
        }
    } else {
        println!("Running prep audit...");
        let status = Command::new(&prep_path).args(["audit"]).status()?;
        if !status.success() {
            println!("prep audit failed.");
        }
    }
    Ok(())
}

pub fn compare_commits(first: &str, second: &str) -> Result<()> {
    let output = Command::new("git").args(["diff", first, second]).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{}", stdout);
    Ok(())
}
