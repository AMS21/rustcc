//! Snapshot-based UI integration test harness for rustcc.
//!
//! Test inputs are C source files under `tests/ui/**/*.c`.
//! Each file may declare directives in leading comment lines:
//!
//! - `// RUN: ${{rustcc}} --print-ast ...`
//! - `// EXPECT-FAILURE`
//!
//! For every input test file `foo.c`, expected outputs are stored adjacent to
//! the test file as:
//!
//! - `foo.stdout`
//! - `foo.stderr`
//!
//! In normal mode this harness compares normalized output against these
//! snapshots. In bless mode (`RUSTCC_BLESS=1`) snapshots are updated in place.
//! When mismatches occur, a colored human-readable diff is produced and the
//! captured actual output is persisted to a temporary file for inspection.

use std::{
    env,
    fmt::{self, Write as _},
    fs,
    io::Write,
    path::{Path, PathBuf},
    process,
};

use anstream::{eprintln, println};
use anstyle::{AnsiColor, Color, Style};
use assert_cmd::cargo::CommandCargoExt;
use similar::{ChangeTag, TextDiff};
use tempfile::NamedTempFile;

const CRASH_STRING: &str = "Oh no rustcc encountered an internal error and has sadly crashed!";

/// Parsed command components extracted from a `RUN` directive.
#[derive(Debug)]
struct ParsedRun {
    /// Binary name found inside `${{...}}`.
    executable: String,
    /// Remaining command-line arguments passed to the binary.
    args: Vec<String>,
}

/// Result of executing one UI test input.
#[derive(Debug)]
struct TestResult {
    /// Test path relative to the UI root directory.
    relative_path: PathBuf,
    /// Accumulated failure diagnostics for this test.
    failures: Vec<String>,
    /// Indicates whether bless mode changed any snapshot file for this test.
    changed: bool,
}

/// Outcome of processing a single output stream snapshot.
enum StreamCheck {
    /// Snapshot already matched the current output.
    Unchanged,
    /// Snapshot was updated in bless mode.
    Changed,
}

/// Pretty-printer helper for optional line numbers in diffs.
struct LineNumber(Option<usize>);

impl fmt::Display for LineNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(idx) => write!(f, "{:>4}", idx + 1),
            None => write!(f, "    "),
        }
    }
}

#[expect(clippy::let_underscore_must_use)]
fn human_readable_diff(
    expected: &str,
    actual: &str,
    baseline_path: &Path,
    temp_path: &Path,
) -> String {
    // We keep color styles local to make the formatting intent explicit.
    const DELETE_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
    const INSERT_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
    const EQUAL_STYLE: Style = Style::new().dimmed();

    let mut output = String::new();
    let _ = writeln!(output, "  baseline: {}", baseline_path.display());
    let _ = writeln!(output, "  actual:   {}\n", temp_path.display());

    // Perform a line-based diff because snapshots are text-oriented.
    let diff = TextDiff::from_lines(expected, actual);

    for change in diff.iter_all_changes() {
        let (sign, style) = match change.tag() {
            ChangeTag::Delete => ('-', DELETE_STYLE),
            ChangeTag::Insert => ('+', INSERT_STYLE),
            ChangeTag::Equal => (' ', EQUAL_STYLE),
        };

        let old_number = LineNumber(change.old_index());
        let new_number = LineNumber(change.new_index());

        let _ = write!(
            output,
            "{old_number} {new_number} |{}{sign}{change}{}",
            style.render(),
            style.render_reset()
        );

        if change.missing_newline() {
            let _ = writeln!(output, "          \\ No newline at end of file");
        }
    }

    output
}

/// Remove ANSI escape sequences so snapshots are deterministic.
///
/// This supports common CSI and OSC sequences as well as simple two-byte
/// escape forms.
fn strip_ansi(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        chars.next();
                        if c.is_ascii() && (0x40..=0x7e).contains(&(c as u8)) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(&c) = chars.peek() {
                        if c == '\x07' {
                            chars.next();
                            break;
                        }
                        if c == '\x1b' {
                            chars.next();
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                        chars.next();
                    }
                }
                Some(_) => {
                    chars.next();
                }
                None => {}
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Normalize output for stable cross-platform comparisons.
///
/// Normalization includes:
/// - CRLF to LF conversion
/// - path separator normalization (`\\` to `/`)
/// - stripping `crates/` segment occurrences
/// - collapse of accidental duplicate separators
/// - usage line `.exe` normalization
fn normalize_output_for_compare(s: &str) -> String {
    let mut normalized = s.replace("\r\n", "\n");
    normalized = normalized.replace('\\', "/");
    normalized = normalized.replace("crates/", "");

    while normalized.contains("//") {
        normalized = normalized.replace("//", "/");
    }

    let lines: Vec<String> = normalized
        .lines()
        .map(|line| {
            const PREFIX: &str = "Usage: ";
            line.strip_prefix(PREFIX).map_or_else(
                || line.to_string(),
                |rest| {
                    let mut iter = rest.splitn(2, ' ');
                    let prog = iter.next().unwrap_or("");
                    let tail = iter.next().unwrap_or("");
                    let prog = prog.strip_suffix(".exe").unwrap_or(prog);
                    if tail.is_empty() {
                        format!("{PREFIX}{prog}")
                    } else {
                        format!("{PREFIX}{prog} {tail}")
                    }
                },
            )
        })
        .collect();

    let mut output = lines.join("\n");
    if normalized.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Stream-specific normalization hook.
///
/// In addition to generic normalization this rewrites test-path occurrences to
/// the final filename (for example `foo.c`). This keeps snapshots readable and
/// avoids path-separator churn across platforms.
fn normalize_stream(s: &str, test_path: &Path, command_input_path: &Path) -> String {
    let mut out = strip_ansi(s);

    let file_name = test_path.file_name().map_or_else(
        || test_path.display().to_string(),
        |n| n.to_string_lossy().to_string(),
    );

    let mut path_variants = vec![
        test_path.to_string_lossy().to_string(),
        command_input_path.to_string_lossy().to_string(),
    ];

    if let Ok(canonical_test_path) = test_path.canonicalize() {
        path_variants.push(canonical_test_path.to_string_lossy().to_string());
    }

    for path in path_variants {
        if path.is_empty() {
            continue;
        }

        out = out.replace(&path, &file_name);

        // Replace slash-normalized forms as well, since diagnostics can emit
        // either style depending on toolchain and host platform.
        out = out.replace(&path.replace('\\', "/"), &file_name);
        out = out.replace(&path.replace('/', "\\"), &file_name);

        // LLVM IR double-quoted strings (source_filename = "...") use C-style
        // escaping, so backslashes in the path appear doubled.
        out = out.replace(&path.replace('\\', "\\\\"), &file_name);
    }

    normalize_output_for_compare(&out)
}

/// Parse the first `RUN` directive from the test source.
///
/// Expected format:
/// `// RUN: ${{binary_name}} [args ...]`
fn parse_run_directive(input: &str) -> Result<ParsedRun, String> {
    let run_command = input
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("//")
                .map(str::trim)
                .and_then(|comment| comment.strip_prefix("RUN:"))
                .map(str::trim)
                .filter(|command| !command.is_empty())
        })
        .ok_or_else(|| String::from("Missing run directive"))?;

    let start = run_command
        .find("${{")
        .ok_or_else(|| String::from("Missing executable name in run directive"))?;
    let rest = &run_command[start + 3..];
    let end_rel = rest
        .find("}}")
        .ok_or_else(|| String::from("Missing executable name in run directive"))?;

    let executable = rest[..end_rel].trim();
    if executable.is_empty() {
        return Err(String::from("Missing executable name in run directive"));
    }

    let placeholder = format!("${{{{{executable}}}}}");
    let args = run_command
        .replace(&placeholder, "")
        .split_whitespace()
        .map(ToString::to_string)
        .collect();

    Ok(ParsedRun {
        executable: executable.to_string(),
        args,
    })
}

/// Returns true when the test source opts into expected-failure semantics.
fn is_expect_failure(input: &str) -> bool {
    input.lines().any(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("//")
            .map(str::trim)
            .is_some_and(|comment| comment == "EXPECT-FAILURE")
    })
}

/// Recursively discover all `.c` test files under `dir`.
fn discover_c_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];

    while let Some(current) = dirs.pop() {
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().and_then(|ext| ext.to_str()) == Some("c") {
                    files.push(path);
                }
            }
        }
    }

    files.sort();
    files
}

/// Determine whether the harness runs in bless mode.
///
/// Bless mode updates snapshots instead of failing on mismatches.
fn bless_mode_enabled() -> bool {
    env::var("RUSTCC_BLESS").is_ok_and(|v| !v.is_empty() && v != "0")
}

/// Compare or bless a single stream snapshot (`stdout` or `stderr`).
///
/// Behavior:
/// - unchanged snapshot => `Ok(StreamCheck::Unchanged)`
/// - snapshot updated in bless mode => `Ok(StreamCheck::Changed)`
/// - mismatch or I/O problem in compare mode => `Err(details)`
fn check_or_bless_split(
    stream_name: &str,
    baseline_path: &Path,
    actual: &str,
    bless: bool,
) -> Result<StreamCheck, String> {
    // Missing snapshot is treated as empty expected output.
    let expected = fs::read_to_string(baseline_path).unwrap_or_default();

    if expected == actual {
        return Ok(StreamCheck::Unchanged);
    }

    if bless {
        // In bless mode, empty output means the snapshot should not exist.
        if actual.trim().is_empty() {
            drop(fs::remove_file(baseline_path));
        } else {
            if let Some(parent) = baseline_path.parent() {
                drop(fs::create_dir_all(parent));
            }
            if let Err(err) = fs::write(baseline_path, actual) {
                return Err(format!(
                    "Failed to write {} baseline '{}': {err}",
                    stream_name,
                    baseline_path.display()
                ));
            }
        }
        return Ok(StreamCheck::Changed);
    }

    let mut temp_file = match NamedTempFile::new() {
        Ok(file) => file,
        Err(err) => {
            return Err(format!(
                "Failed to create tempfile for {stream_name}: {err}"
            ));
        }
    };

    let temp_path = temp_file.path().to_path_buf();
    if let Err(err) = temp_file.write_all(actual.as_bytes()) {
        return Err(format!(
            "Failed to write tempfile for {} mismatch '{}': {err}",
            stream_name,
            baseline_path.display()
        ));
    }

    // Persist the tempfile so users can inspect exact actual output after
    // the test process exits.
    if let Err(err) = temp_file.persist(&temp_path) {
        return Err(format!(
            "Failed to persist tempfile for {} mismatch '{}': {err}",
            stream_name,
            baseline_path.display()
        ));
    }

    Err(format!(
        "{} mismatch:\n{}",
        stream_name,
        human_readable_diff(&expected, actual, baseline_path, &temp_path)
    ))
}

/// Execute one UI test input and evaluate it against snapshots.
///
/// This function handles directive parsing, process execution, status checks,
/// crash detection, normalization, and snapshot compare/bless logic.
#[expect(clippy::too_many_lines)]
fn run_single_test(
    test_path: &Path,
    ui_root: &Path,
    command_working_dir: &Path,
    bless: bool,
) -> TestResult {
    let relative_path = test_path
        .strip_prefix(ui_root)
        .unwrap_or(test_path)
        .to_path_buf();

    let mut failures = Vec::new();
    let mut changed = false;

    let input = match fs::read_to_string(test_path) {
        Ok(content) => content,
        Err(err) => {
            failures.push(format!(
                "Failed to read input file '{}': {err}",
                test_path.display()
            ));
            return TestResult {
                relative_path,
                failures,
                changed,
            };
        }
    };

    let parsed_run = match parse_run_directive(&input) {
        Ok(parsed) => parsed,
        Err(err) => {
            failures.push(err);
            return TestResult {
                relative_path,
                failures,
                changed,
            };
        }
    };

    let expect_failure = is_expect_failure(&input);

    let Ok(mut command) = process::Command::cargo_bin(&parsed_run.executable) else {
        failures.push(format!("Executable '{}' not found", parsed_run.executable));
        return TestResult {
            relative_path,
            failures,
            changed,
        };
    };

    // Execute from the workspace's `crates/` directory using a relative path.
    // This preserves stable path rendering in diagnostics and snapshots.
    let command_input_path = test_path
        .strip_prefix(command_working_dir)
        .unwrap_or(test_path)
        .to_path_buf();

    command.current_dir(command_working_dir);

    let output = match command
        .arg(&command_input_path)
        .args(&parsed_run.args)
        .output()
    {
        Ok(output) => output,
        Err(err) => {
            failures.push(format!(
                "Failed to execute binary '{}': {err}",
                parsed_run.executable
            ));
            return TestResult {
                relative_path,
                failures,
                changed,
            };
        }
    };

    let Some(status_code) = output.status.code() else {
        failures.push(String::from(
            "Failed to extract status code from test process",
        ));
        return TestResult {
            relative_path,
            failures,
            changed,
        };
    };

    // Validate exit behavior according to EXPECT-FAILURE directive.
    if !expect_failure && status_code != 0 {
        failures.push(format!(
            "Test unexpectedly failed with status code: {status_code}"
        ));
    } else if expect_failure && status_code == 0 {
        failures.push(String::from("Test unexpectedly passed"));
    }

    let stdout_raw = String::from_utf8_lossy(&output.stdout);
    let stderr_raw = String::from_utf8_lossy(&output.stderr);

    // Guard against explicit internal crash sentinel output.
    if stdout_raw.contains(CRASH_STRING) || stderr_raw.contains(CRASH_STRING) {
        failures.push(String::from("Test crashed"));
    }

    let stdout_norm = normalize_stream(&stdout_raw, test_path, &command_input_path);
    let stderr_norm = normalize_stream(&stderr_raw, test_path, &command_input_path);
    let stdout_baseline = test_path.with_extension("stdout");
    let stderr_baseline = test_path.with_extension("stderr");

    // Evaluate both stream snapshots independently for clearer diagnostics.
    for (stream_name, baseline_path, actual) in [
        ("stdout", &stdout_baseline, stdout_norm.as_str()),
        ("stderr", &stderr_baseline, stderr_norm.as_str()),
    ] {
        match check_or_bless_split(stream_name, baseline_path, actual, bless) {
            Ok(StreamCheck::Unchanged) => {}
            Ok(StreamCheck::Changed) => {
                changed = true;
            }
            Err(msg) => failures.push(msg),
        }
    }

    TestResult {
        relative_path,
        failures,
        changed,
    }
}

/// Main UI integration test entrypoint.
///
/// This function orchestrates discovery, execution, per-test reporting and the
/// final summary.
#[test]
fn ui() {
    const BLESSED_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow)));
    const PASSED_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green)));
    const FAILED_STYLE: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let test_root = manifest_dir.join("tests");
    let command_working_dir = manifest_dir.parent().unwrap_or(&manifest_dir);
    let ui_dir = test_root.join("ui");

    assert!(
        ui_dir.exists(),
        "UI test directory not found: {}",
        ui_dir.display()
    );

    let input_files = discover_c_files(&ui_dir);

    assert!(
        !input_files.is_empty(),
        "No .c tests found in {}",
        ui_dir.display()
    );

    let bless = bless_mode_enabled();
    let mut failures = Vec::new();
    let mut unchanged = 0usize;
    let mut changed = 0usize;

    for input_path in &input_files {
        let result = run_single_test(input_path, &ui_dir, command_working_dir, bless);
        if result.failures.is_empty() {
            if bless && result.changed {
                changed += 1;
                println!(
                    "{}blessed{}: {}",
                    BLESSED_STYLE.render(),
                    BLESSED_STYLE.render_reset(),
                    result.relative_path.display()
                );
            } else {
                unchanged += 1;
                println!(
                    "{}ok{}:      {}",
                    PASSED_STYLE.render(),
                    PASSED_STYLE.render_reset(),
                    result.relative_path.display()
                );
            }
        } else {
            println!(
                "{}FAILED{}:  {}",
                FAILED_STYLE.render(),
                FAILED_STYLE.render_reset(),
                result.relative_path.display()
            );
            failures.push(result);
        }
    }

    println!();
    if bless {
        println!(
            "{} tests: {}{}{} already good, {}{}{} changed, {}{}{} failed",
            input_files.len(),
            PASSED_STYLE.render(),
            unchanged,
            PASSED_STYLE.render_reset(),
            BLESSED_STYLE.render(),
            changed,
            BLESSED_STYLE.render_reset(),
            FAILED_STYLE.render(),
            failures.len(),
            FAILED_STYLE.render_reset()
        );
    } else {
        println!(
            "{} tests: {}{}{} passed, {}{}{} failed",
            input_files.len(),
            PASSED_STYLE.render(),
            unchanged,
            PASSED_STYLE.render_reset(),
            FAILED_STYLE.render(),
            failures.len(),
            FAILED_STYLE.render_reset()
        );
    }

    if !failures.is_empty() {
        eprintln!();
        for failed in &failures {
            eprintln!("---- {} ----", failed.relative_path.display());
            for message in &failed.failures {
                eprintln!("{message}");
            }
        }
    }

    assert!(
        failures.is_empty(),
        "{} UI test(s) failed.\nTip: run 'RUSTCC_BLESS=1 cargo test -p rustcc-compiler --test ui \
         -- --nocapture' to update baselines.",
        failures.len()
    );
}
