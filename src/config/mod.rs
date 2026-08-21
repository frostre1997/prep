use crate::cli::commands::IgnoreAction;
use anyhow::Result;
use std::fs;
use std::path::Path;

pub fn init_config(force: bool) -> Result<()> {
    let ignore_path = ".prepignore";
    let config_path = ".preprc";

    if Path::new(ignore_path).exists() && !force {
        println!(".prepignore already exists. Use --force to overwrite.");
    } else {
        let default_ignore = r#"# .prepignore - files to ignore when scanning
*.log
*.tmp
*.cache
*.pyc
__pycache__/
node_modules/
target/
build/
dist/
*.lock
"#;
        fs::write(ignore_path, default_ignore)?;
        println!("Created .prepignore");
    }

    if Path::new(config_path).exists() && !force {
        println!(".preprc already exists. Use --force to overwrite.");
    } else {
        let default_config = r#"# prep configuration
# Add custom rules here
[rules]
"TODO: fix" = "warning"
"console.log" = "warning"
"#;
        fs::write(config_path, default_config)?;
        println!("Created .preprc");
    }

    Ok(())
}

pub fn manage_config(show: bool, edit: bool, set: Option<&str>) -> Result<()> {
    let config_path = ".preprc";

    if show {
        if Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)?;
            println!("{}", content);
        } else {
            println!("No configuration file found. Run `prep init` to create one.");
        }
    }

    if edit {
        if Path::new(config_path).exists() {
            let editor = std::env::var("EDITOR").unwrap_or_else(|_| "nano".to_string());
            let status = std::process::Command::new(editor)
                .arg(config_path)
                .status()?;
            if !status.success() {
                println!("Editor exited with error.");
            }
        } else {
            println!("No configuration file found. Run `prep init` to create one.");
        }
    }

    if let Some(key_value) = set {
        let parts: Vec<&str> = key_value.splitn(2, '=').collect();
        if parts.len() != 2 {
            println!("Invalid format. Use key=value");
            return Ok(());
        }
        println!("Setting {} = {} (not yet implemented)", parts[0], parts[1]);
    }

    Ok(())
}

pub fn manage_ignore(action: &IgnoreAction) -> Result<()> {
    let ignore_path = ".prepignore";

    match action {
        IgnoreAction::Add { pattern } => {
            let content = if Path::new(ignore_path).exists() {
                fs::read_to_string(ignore_path)?
            } else {
                String::new()
            };
            let new_content = format!("{}\n{}", content, pattern);
            fs::write(ignore_path, new_content)?;
            println!("Added pattern: {}", pattern);
        }
        IgnoreAction::Remove { pattern } => {
            if !Path::new(ignore_path).exists() {
                println!("No .prepignore file found.");
                return Ok(());
            }
            let content = fs::read_to_string(ignore_path)?;
            let new_content: Vec<String> = content
                .lines()
                .filter(|line| line.trim() != pattern && !line.trim().is_empty())
                .map(String::from)
                .collect();
            fs::write(ignore_path, new_content.join("\n"))?;
            println!("Removed pattern: {}", pattern);
        }
        IgnoreAction::List => {
            if Path::new(ignore_path).exists() {
                let content = fs::read_to_string(ignore_path)?;
                println!("Current ignored patterns:\n{}", content);
            } else {
                println!("No .prepignore file found.");
            }
        }
        IgnoreAction::Check { file } => {
            if Path::new(ignore_path).exists() {
                let content = fs::read_to_string(ignore_path)?;
                let is_ignored = content.lines().any(|line| {
                    let line = line.trim();
                    !line.is_empty() && !line.starts_with('#') && file.contains(line)
                });
                if is_ignored {
                    println!("{} is ignored", file);
                } else {
                    println!("{} is NOT ignored", file);
                }
            } else {
                println!("No .prepignore file found.");
            }
        }
        IgnoreAction::Reset => {
            let default_content =
                "# .prepignore\n*.log\n*.tmp\n*.cache\nnode_modules/\ntarget/\nbuild/\n";
            fs::write(ignore_path, default_content)?;
            println!("Reset .prepignore to defaults.");
        }
    }

    Ok(())
}
