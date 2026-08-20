use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(name = "prep")]
#[command(about = "A repository health auditor and build error parser")]
#[command(version = "0.100.0")]
#[command(author = "frostre1997 <n9043395@gmail.com>")]
#[command(arg_required_else_help = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Audit repository for issues
    Audit {
        /// Scan only files changed in last commit
        #[arg(short = 'r', long)]
        changed: bool,
        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Exclude files matching pattern
        #[arg(short = 'e', long)]
        exclude: Option<String>,
        /// Include only files matching pattern
        #[arg(short = 'i', long)]
        include: Option<String>,
        /// Follow symbolic links
        #[arg(long)]
        follow_symlinks: bool,
        /// Maximum directory depth
        #[arg(long)]
        max_depth: Option<usize>,
        /// Only files modified after date
        #[arg(long)]
        since: Option<String>,
        /// Only files modified before date
        #[arg(long)]
        until: Option<String>,
        /// Ignore .prepignore rules
        #[arg(long)]
        no_ignore: bool,
    },
    /// Auto-fix formatting issues
    Fix {
        /// Show what would be fixed without changing
        #[arg(long)]
        dry_run: bool,
        /// Fix all fixable issues (including CRLF)
        #[arg(long)]
        all: bool,
        /// Only trim trailing whitespace
        #[arg(long)]
        trim: bool,
        /// Only add EOF newline
        #[arg(long)]
        eof: bool,
        /// Convert CRLF to LF
        #[arg(long)]
        crlf: bool,
        /// Remove BOM from UTF-8 files
        #[arg(long)]
        bom: bool,
        /// Confirm each fix
        #[arg(short = 'i', long)]
        interactive: bool,
        /// Restrict to changed files
        #[arg(short = 'r', long)]
        changed: bool,
    },
    /// Parse build output from stdin or run build
    Build {
        /// Run build and parse output
        #[arg(long)]
        run: bool,
        /// Watch for build changes and re-parse
        #[arg(long)]
        watch: bool,
        /// Build tool (gradle, maven, cargo)
        #[arg(long)]
        tool: Option<String>,
        /// Save build output for later analysis
        #[arg(long)]
        save: bool,
    },
    /// Search for regex pattern
    Search {
        /// Pattern to search for
        pattern: String,
        /// Case-insensitive search
        #[arg(short = 'i', long)]
        case_insensitive: bool,
        /// Show count of matches
        #[arg(short = 'c', long)]
        count: bool,
        /// List filenames only
        #[arg(short = 'l', long)]
        files_only: bool,
        /// Show line numbers
        #[arg(short = 'n', long)]
        line_numbers: bool,
        /// Show lines after match
        #[arg(short = 'A', long)]
        after: Option<usize>,
        /// Show lines before match
        #[arg(short = 'B', long)]
        before: Option<usize>,
        /// Show lines both before and after
        #[arg(short = 'C', long)]
        context: Option<usize>,
        /// Full context (whole function/block)
        #[arg(long)]
        full_context: bool,
        /// Replace matches
        #[arg(long)]
        replace: Option<String>,
        /// Restrict to changed files
        #[arg(short = 'r', long)]
        changed: bool,
    },
    /// Generate SHA-256 manifest
    Manifest {
        /// Verify against previous manifest
        #[arg(long)]
        verify: bool,
        /// Save manifest to file
        #[arg(long)]
        out: Option<String>,
    },
    /// Generate report
    Report {
        /// Report format (html, json, markdown, csv)
        #[arg(long)]
        format: Option<String>,
        /// Save report to file
        #[arg(long)]
        output: Option<String>,
        /// Open report in browser
        #[arg(long)]
        open: bool,
        /// Include all issues
        #[arg(long)]
        all: bool,
    },
    /// Show quick summary of issues
    Summary {
        /// Verbose summary
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Show detailed statistics
    Stats {
        /// Show top N
        #[arg(long)]
        top: Option<usize>,
    },
    /// Show git blame for each issue
    Blame {
        /// Filter by author
        #[arg(long)]
        author: Option<String>,
        /// Filter by date
        #[arg(long)]
        since: Option<String>,
        /// Short format
        #[arg(long)]
        short: bool,
    },
    /// Scan only changed files
    Diff {
        /// Scan only staged files
        #[arg(long)]
        staged: bool,
        /// Scan uncommitted changes
        #[arg(long)]
        uncommitted: bool,
        /// Compare against a branch
        #[arg(long)]
        base: Option<String>,
    },
    /// Show scan history
    History {
        /// Clear history
        #[arg(long)]
        clear: bool,
        /// Show history details
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Initialize .prepignore and config
    Init {
        /// Overwrite existing config
        #[arg(long)]
        force: bool,
    },
    /// Show or edit configuration
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        /// Open config in editor
        #[arg(long)]
        edit: bool,
        /// Set config value (key=value)
        #[arg(long)]
        set: Option<String>,
    },
    /// Manage ignore patterns
    Ignore {
        #[command(subcommand)]
        action: IgnoreAction,
    },
    /// Install/uninstall Git hooks
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
    /// Run in CI mode
    Ci {
        /// Fail on warnings
        #[arg(long)]
        fail_on_warning: bool,
        /// Fail if more than N warnings
        #[arg(long)]
        threshold: Option<usize>,
    },
    /// Check license headers
    License {
        #[command(subcommand)]
        action: LicenseAction,
    },
    /// Check or update dependencies
    Deps {
        #[command(subcommand)]
        action: DepsAction,
    },
    /// Find duplicate files
    Duplicates {
        /// Delete duplicates (interactive)
        #[arg(long)]
        delete: bool,
        /// Move duplicates to trash
        #[arg(long)]
        move_to_trash: bool,
    },
    /// Remove temporary files and cache
    Clean {
        /// Remove all caches and temp files
        #[arg(long)]
        all: bool,
        /// Show what would be removed
        #[arg(long)]
        dry_run: bool,
    },
    /// Show repository information
    Info {
        /// Show detailed info
        #[arg(long)]
        detailed: bool,
    },
    /// Watch files and auto-scan on change
    Watch {
        /// Check interval in seconds
        #[arg(long)]
        interval: Option<u64>,
        /// Auto-fix on change
        #[arg(long)]
        fix: bool,
    },
    /// Compare two commits/branches
    Compare {
        /// First reference (commit/branch)
        first: String,
        /// Second reference (commit/branch)
        second: String,
    },
    /// Export issues to file
    Export {
        /// Output format (csv, json, xml)
        #[arg(long)]
        format: Option<String>,
        /// Output file
        #[arg(long)]
        output: Option<String>,
    },
    /// Import issues from file
    Import {
        /// Input file
        file: String,
    },
    /// Run performance benchmark
    Benchmark {
        /// Verbose output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    /// Remove trailing whitespace in all files
    Trim {
        /// Show what would be trimmed
        #[arg(long)]
        dry_run: bool,
    },
    /// Show version
    Version {
        /// Check for updates
        #[arg(long)]
        check: bool,
    },
    /// Show usage examples
    Examples,
}

#[derive(Subcommand)]
pub enum IgnoreAction {
    /// Add pattern to .prepignore
    Add { pattern: String },
    /// Remove pattern from .prepignore
    Remove { pattern: String },
    /// List all ignored patterns
    List,
    /// Check if file is ignored
    Check { file: String },
    /// Reset to default ignored patterns
    Reset,
}

#[derive(Subcommand)]
pub enum HooksAction {
    /// Install pre-commit hook
    Install {
        /// Overwrite existing hook
        #[arg(long)]
        force: bool,
        /// Install all hooks
        #[arg(long)]
        all: bool,
    },
    /// Remove pre-commit hook
    Uninstall,
    /// Show hook status
    Status,
    /// Run hooks manually
    Run {
        /// Run with auto-fix
        #[arg(long)]
        fix: bool,
    },
}

#[derive(Subcommand)]
pub enum LicenseAction {
    /// Check license headers
    Check,
    /// Add license header
    Add {
        /// Overwrite existing headers
        #[arg(long)]
        force: bool,
    },
}

#[derive(Subcommand)]
pub enum DepsAction {
    /// Check outdated dependencies
    Check,
    /// Update dependencies (interactive)
    Update {
        /// Update without confirmation
        #[arg(long)]
        yes: bool,
    },
    /// List all dependencies
    List,
}

impl Cli {
    pub fn execute(&self) -> Result<()> {
        match &self.command {
            // For brevity, we'll call stub functions for each command.
            // You can replace these with actual implementations later.
            Commands::Audit { changed, verbose, exclude, include, follow_symlinks, max_depth, since, until, no_ignore } => {
                crate::core::audit(
                    *changed, *verbose,
                    exclude.as_deref(),
                    include.as_deref(),
                    *follow_symlinks,
                    *max_depth,
                    since.as_deref(),
                    until.as_deref(),
                    *no_ignore,
                )
            }
            Commands::Fix { dry_run, all, trim, eof, crlf, bom, interactive, changed } => {
                crate::core::fix(
                    *dry_run, *all, *trim, *eof, *crlf, *bom, *interactive, *changed,
                )
            }
            Commands::Build { run, watch, tool, save } => {
                crate::build::parse_build(*run, *watch, tool.as_deref(), *save)
            }
            Commands::Search { pattern, case_insensitive, count, files_only, line_numbers, after, before, context, full_context, replace, changed } => {
                crate::core::search(
                    pattern,
                    *case_insensitive,
                    *count,
                    *files_only,
                    *line_numbers,
                    *after,
                    *before,
                    *context,
                    *full_context,
                    replace.as_deref(),
                    *changed,
                )
            }
            Commands::Manifest { verify, out } => {
                crate::core::manifest(*verify, out.as_deref())
            }
            Commands::Report { format, output, open, all } => {
                crate::report::generate_report(
                    format.as_deref(),
                    output.as_deref(),
                    *open,
                    *all,
                )
            }
            Commands::Summary { verbose } => {
                crate::report::summary(*verbose)
            }
            Commands::Stats { top } => {
                crate::report::stats(*top)
            }
            Commands::Blame { author, since, short } => {
                crate::git::blame(author.as_deref(), since.as_deref(), *short)
            }
            Commands::Diff { staged, uncommitted, base } => {
                crate::git::diff(*staged, *uncommitted, base.as_deref())
            }
            Commands::History { clear, verbose } => {
                crate::cache::history(*clear, *verbose)
            }
            Commands::Init { force } => {
                crate::config::init_config(*force)
            }
            Commands::Config { show, edit, set } => {
                crate::config::manage_config(*show, *edit, set.as_deref())
            }
            Commands::Ignore { action } => {
                crate::config::manage_ignore(action)
            }
            Commands::Hooks { action } => {
                crate::git::manage_hooks(action)
            }
            Commands::Ci { fail_on_warning, threshold } => {
                crate::core::ci_mode(*fail_on_warning, *threshold)
            }
            Commands::License { action } => {
                crate::checks::license_check(action)
            }
            Commands::Deps { action } => {
                crate::checks::deps_check(action)
            }
            Commands::Duplicates { delete, move_to_trash } => {
                crate::checks::find_duplicates(*delete, *move_to_trash)
            }
            Commands::Clean { all, dry_run } => {
                crate::cache::clean_cache(*all, *dry_run)
            }
            Commands::Info { detailed } => {
                crate::core::repo_info(*detailed)
            }
            Commands::Watch { interval, fix } => {
                crate::core::watch_files(*interval, *fix)
            }
            Commands::Compare { first, second } => {
                crate::git::compare_commits(first, second)
            }
            Commands::Export { format, output } => {
                crate::report::export_issues(format.as_deref(), output.as_deref())
            }
            Commands::Import { file } => {
                crate::report::import_issues(file)
            }
            Commands::Benchmark { verbose } => {
                crate::utils::benchmark(*verbose)
            }
            Commands::Trim { dry_run } => {
                crate::core::trim_whitespace(*dry_run)
            }
            Commands::Version { check } => {
                crate::core::show_version(*check)
            }
            Commands::Examples => {
                crate::core::show_examples()
            }
        }
    }
}
