# prep

`prep` is a command-line tool that audits your Git repository for common issues, automatically fixes formatting problems, and parses build output (Gradle, Maven, Cargo) to show structured errors and warnings.

## Features

- Detect secrets (AWS keys, private keys, JWT tokens)
- Find unresolved merge conflict markers
- Check trailing whitespace, missing newline at EOF
- Auto-fix formatting with `prep fix`
- Generate SHA-256 manifest with `prep manifest`
- Parse build output (Gradle, Maven, Cargo) with `prep build`
- Search with regex (like `grep`) with `prep search`
- Git blame integration to see who introduced issues
- Git hooks installation for pre-commit checks
- HTML, JSON, Markdown, CSV reports
- `.prepignore` support
- Parallel scanning for speed
- CI-friendly exit codes and output

## Installation

### From source (requires Rust)

```bash
cargo install --git https://github.com/frostre1997/prep
```

Or clone and build manually:

```bash
git clone https://github.com/frostre1997/prep
cd prep
cargo build --release
sudo cp target/release/prep /usr/local/bin/
```

## Pre-built binaries

Download the latest binary for your platform from the Releases page.

# Usage

## Basic commands

- `prep audit` - Scan all files for issues (default).

- `prep audit -r` - Scan only files changed in the last commit.

- `prep fix` - Auto-fix formatting issues.

- `prep fix --dry-run` - Show what would be fixed without changing files.

- `prep build` - Parse build output from stdin (pipe).

- `prep build --run` - Run the build and parse output (supports gradle, maven, cargo).

- `prep search "pattern"` - Search for a regex pattern.

- `prep search -i "pattern"` - Case-insensitive search.

- `prep manifest` - Generate SHA-256 manifest of all text files.

- `prep report --format html` - Generate an HTML report.

- `prep hooks install` - Install a pre-commit hook.

- `prep blame` - Show git blame for each issue.

- `prep info` - Show repository information.

# Example

```bash
prep audit
```

To scan only files changed in the last commit:

```bash
prep audit -r
```

---

## `prep fix` – Fix formatting problems automatically

Removes trailing whitespace from all lines and adds a missing newline at the end of every file. Use `--dry-run` to preview changes.

```bash
prep fix
```

Preview only:

```bash
prep fix --dry-run
```

---

## `prep build` – Parse build output (Gradle, Maven, Cargo)

Pipes the output of your build tool into prep. It extracts compiler errors and warnings and prints them with file names and line numbers.

```bash
./gradlew assembleDebug 2>&1 | prep build
```

To have prep run the build for you:

```bash
prep build --run --tool gradle
```

---

## `prep search "pattern"` – Find text with regex

Works like grep -r but respects your .gitignore and .prepignore files. Shows file paths and line numbers.

```bash
prep search "TODO"
```

Case‑insensitive search:

```bash
prep search -i "fixme"
```

Show only filenames:

```bash
prep search -l "error"
```

---

## `prep report` – Generate an HTML report

Runs a full audit and creates a visual HTML summary of all issues found. The report lists every error and warning with file paths.

```bash
prep report --format html --output report.html
```

Other formats: json, markdown, csv.

```bash
prep report --format json --output issues.json
```

---

## `prep hooks install` – Automate checks before every commit

Installs a Git pre‑commit hook that runs prep audit (and optionally prep fix) before each commit. Prevents secrets, conflicts, and formatting issues from entering the repository.

```bash
prep hooks install
```

To remove the hook:

```bash
prep hooks uninstall
```

## Configuration

prep respects a .prepignore file in the root of your repository to skip files and directories (similar to .gitignore). You can also define custom regex rules in a prep.toml file (future feature).

## Contributing

Contributions are welcome! Please open an issue or pull request on GitHub. For major changes, please discuss them first.

## License

This project is licensed under the MIT License – see the [LICENSE](https://github.com/frostre1997/prep/blob/master/LICENSE) file for details.