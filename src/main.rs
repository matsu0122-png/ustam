//! ustam — a simplified ls command implemented in Rust.
//!
//! Reads a directory and prints its entries with optional sorting,
//! long-format metadata, and gitignore-aware filtering.

use std::cmp::Ordering;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Determines how entries are ordered in the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Name,
    Size,
    Modified,
}

/// Runtime configuration parsed from command-line arguments.
#[derive(Debug)]
struct Config {
    path: PathBuf,
    show_hidden: bool,
    long_format: bool,
    sort_key: SortKey,
}

/// Metadata for a single directory entry, including optional extended info.
#[derive(Debug)]
struct FileInfo {
    name: String,
    metadata: Metadata,
    extension_info: Option<String>,
}

/// Patterns loaded from `.gitignore` to exclude entries from output.
#[derive(Debug)]
struct GitignoreRules {
    patterns: Vec<String>,
}

/// Entry point: runs the program and prints any error to stderr with a
/// non-zero exit code.
fn main() {
    if let Err(error) = run() {
        eprintln!("ustam: {error}");
        std::process::exit(1);
    }
}

/// Parses arguments, validates the target path, then collects, sorts, and
/// prints the directory entries.
fn run() -> Result<(), String> {
    let config = parse_args(env::args().skip(1))?;
    validate_target_path(&config.path)?;

    let rules = GitignoreRules::load(&config.path);
    let mut files = collect_file_info(&config, &rules)
        .map_err(|error| format!("{}: {}", config.path.display(), error))?;

    sort_files(&mut files, config.sort_key);
    print_files(&files, config.long_format);

    Ok(())
}

/// Parses CLI arguments into a `Config`.
///
/// Recognizes `--help`/`-h` (prints usage and exits immediately), option
/// flags starting with `-`, and at most one positional path argument
/// (defaults to `.` when omitted).
fn parse_args<I>(args: I) -> Result<Config, String>
where
    I: IntoIterator<Item = String>,
{
    let mut path = None;
    let mut show_hidden = false;
    let mut long_format = false;
    let mut sort_key = SortKey::Name;

    for arg in args {
        if arg == "--help" || arg == "-h" {
            print_help();
            std::process::exit(0);
        }

        if arg.starts_with('-') && arg != "-" {
            apply_option(&arg, &mut show_hidden, &mut long_format, &mut sort_key)?;
        } else if path.is_none() {
            path = Some(PathBuf::from(arg));
        } else {
            return Err("パスは1つだけ指定できます".to_string());
        }
    }

    Ok(Config {
        path: path.unwrap_or_else(|| PathBuf::from(".")),
        show_hidden,
        long_format,
        sort_key,
    })
}

/// Applies a single `-` option argument, which may bundle multiple short
/// flags (e.g. `-al`). Returns an error for any unrecognized flag character.
fn apply_option(
    option: &str,
    show_hidden: &mut bool,
    long_format: &mut bool,
    sort_key: &mut SortKey,
) -> Result<(), String> {
    for flag in option.trim_start_matches('-').chars() {
        match flag {
            'a' => *show_hidden = true,
            'l' => *long_format = true,
            's' => *sort_key = SortKey::Size,
            't' => *sort_key = SortKey::Modified,
            'n' => *sort_key = SortKey::Name,
            _ => return Err(format!("未知のオプションです: -{flag}")),
        }
    }

    Ok(())
}

/// Prints the `--help` usage text to stdout.
fn print_help() {
    println!(
        "Usage: ustam [OPTIONS] [PATH]\n\n\
         Options:\n  \
         -a    隠しファイルを表示\n  \
         -l    サイズ・更新日時・拡張情報を表示\n  \
         -s    ファイルサイズ順にソート\n  \
         -t    更新日時順にソート\n  \
         -n    名前順にソート\n  \
         -h    ヘルプを表示"
    );
}

/// Returns an error if `path` does not exist or is not a directory.
fn validate_target_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Err(format!("パスが存在しません: {}", path.display()));
    }

    if !path.is_dir() {
        return Err(format!(
            "ディレクトリを指定してください: {}",
            path.display()
        ));
    }

    Ok(())
}

/// Reads `config.path` and builds a `FileInfo` for each entry that is not
/// filtered out by `should_skip_entry`.
fn collect_file_info(config: &Config, rules: &GitignoreRules) -> io::Result<Vec<FileInfo>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(&config.path)? {
        let entry = entry?;

        if should_skip_entry(&entry, config.show_hidden, rules) {
            continue;
        }

        files.push(build_file_info(entry)?);
    }

    Ok(files)
}

/// Returns `true` if `entry` should be excluded from output: a hidden file
/// when `show_hidden` is false, or a path matched by `.gitignore` rules.
fn should_skip_entry(entry: &DirEntry, show_hidden: bool, rules: &GitignoreRules) -> bool {
    let name = entry.file_name().to_string_lossy().to_string();

    (!show_hidden && is_hidden_file(&name)) || rules.is_ignored(&name, &entry.path())
}

/// Returns `true` if `name` starts with a dot, per Unix hidden-file convention.
fn is_hidden_file(name: &str) -> bool {
    name.starts_with('.')
}

/// Builds a `FileInfo` from a directory entry, including any extension
/// info discovered by `find_extension_info`.
fn build_file_info(entry: DirEntry) -> io::Result<FileInfo> {
    let path = entry.path();
    let metadata = entry.metadata()?;
    let name = entry.file_name().to_string_lossy().to_string();
    let extension_info = find_extension_info(&path, &metadata);

    Ok(FileInfo {
        name,
        metadata,
        extension_info,
    })
}

/// Looks up extra descriptive info for an entry: a README tagline for
/// directories, or a title for PDF files. Returns `None` otherwise.
fn find_extension_info(path: &Path, metadata: &Metadata) -> Option<String> {
    if metadata.is_dir() {
        return read_readme_tagline(path).map(|tagline| format!("README: {tagline}"));
    }

    if is_pdf(path) {
        return read_pdf_title(path).map(|title| format!("PDF: {title}"));
    }

    None
}

/// Sorts `files` in place according to `sort_key`.
fn sort_files(files: &mut [FileInfo], sort_key: SortKey) {
    match sort_key {
        SortKey::Name => files.sort_by(compare_by_name),
        SortKey::Size => files.sort_by(compare_by_size),
        SortKey::Modified => files.sort_by(compare_by_modified),
    }
}

/// Compares entries case-insensitively by name, ascending.
fn compare_by_name(left: &FileInfo, right: &FileInfo) -> Ordering {
    left.name.to_lowercase().cmp(&right.name.to_lowercase())
}

/// Compares entries by file size, descending, breaking ties by name.
fn compare_by_size(left: &FileInfo, right: &FileInfo) -> Ordering {
    right
        .metadata
        .len()
        .cmp(&left.metadata.len())
        .then_with(|| compare_by_name(left, right))
}

/// Compares entries by modification time, most recent first, treating
/// unreadable timestamps as the Unix epoch and breaking ties by name.
fn compare_by_modified(left: &FileInfo, right: &FileInfo) -> Ordering {
    let left_time = left.metadata.modified().unwrap_or(UNIX_EPOCH);
    let right_time = right.metadata.modified().unwrap_or(UNIX_EPOCH);

    right_time
        .cmp(&left_time)
        .then_with(|| compare_by_name(left, right))
}

/// Prints each entry, one per line, using long format when requested.
fn print_files(files: &[FileInfo], long_format: bool) {
    for file in files {
        if long_format {
            print_long_file(file);
        } else {
            println!("{}", file.name);
        }
    }
}

/// Prints a single entry in long format: kind, size, modified time, name,
/// and any extension info.
fn print_long_file(file: &FileInfo) {
    let kind = file_kind_label(&file.metadata);
    let size = human_readable_size(file.metadata.len());
    let modified = format_modified_time(file.metadata.modified().ok());
    let info = file
        .extension_info
        .as_ref()
        .map(|text| format!("  {text}"))
        .unwrap_or_default();

    println!("{kind:<3} {size:>8} {modified:<14} {}{info}", file.name);
}

/// Returns a short label describing the entry kind: `dir`, `file`, or `etc`.
fn file_kind_label(metadata: &Metadata) -> &'static str {
    if metadata.is_dir() {
        "dir"
    } else if metadata.is_file() {
        "file"
    } else {
        "etc"
    }
}

/// Converts a raw byte count to a human-readable string.
/// Uses binary prefixes (1 KB = 1024 B), consistent with common Unix tools.
fn human_readable_size(size: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];

    let mut value = size as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{size}{}", UNITS[unit_index])
    } else {
        format!("{value:.1}{}", UNITS[unit_index])
    }
}

/// Formats a modification time as seconds since the Unix epoch, or `"-"`
/// when the time is unavailable.
fn format_modified_time(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "-".to_string();
    };

    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}s", duration.as_secs()),
        Err(_) => "-".to_string(),
    }
}

/// Reads `README.md` in `directory` and extracts its tagline, if any.
fn read_readme_tagline(directory: &Path) -> Option<String> {
    let readme_path = directory.join("README.md");
    let content = fs::read_to_string(readme_path).ok()?;

    extract_readme_tagline(&content)
}

/// Returns the first non-empty, non-heading line that follows a tagline
/// heading (see `is_tagline_heading`) in `content`.
fn extract_readme_tagline(content: &str) -> Option<String> {
    let mut found_tagline_heading = false;

    for line in content.lines() {
        let trimmed = line.trim();

        if found_tagline_heading && !trimmed.is_empty() && !trimmed.starts_with('#') {
            return Some(trimmed.to_string());
        }

        if is_tagline_heading(trimmed) {
            found_tagline_heading = true;
        }
    }

    None
}

/// Returns `true` if `line` is a Markdown heading containing "tagline"
/// (case-insensitive).
fn is_tagline_heading(line: &str) -> bool {
    line.starts_with('#') && line.to_lowercase().contains("tagline")
}

/// Returns `true` if `path` has a `.pdf` extension (case-insensitive).
fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

/// Reads `path` as (lossy) text and extracts its title, if any.
fn read_pdf_title(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);

    extract_pdf_title(&text)
}

/// Extracts a PDF title from raw PDF bytes decoded as text, trying the
/// `/Title (...)` info dictionary entry, then Dublin Core `<dc:title>`,
/// then HTML-style `<title>`, in that order.
fn extract_pdf_title(text: &str) -> Option<String> {
    extract_between(text, "/Title (", ")")
        .or_else(|| extract_between(text, "<dc:title>", "</dc:title>"))
        .or_else(|| extract_between(text, "<title>", "</title>"))
        .map(clean_pdf_title)
        .filter(|title| !title.is_empty())
}

/// Returns the substring of `text` strictly between the first occurrence
/// of `start` and the following occurrence of `end`.
fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let start_index = text.find(start)? + start.len();
    let rest = &text[start_index..];
    let end_index = rest.find(end)?;

    Some(rest[..end_index].to_string())
}

/// Unescapes PDF string escapes (`\(`, `\)`, `\\`) and trims whitespace.
fn clean_pdf_title(title: String) -> String {
    title
        .replace("\\(", "(")
        .replace("\\)", ")")
        .replace("\\\\", "\\")
        .trim()
        .to_string()
}

impl GitignoreRules {
    /// Loads `.gitignore` patterns from `directory`, or an empty rule set if
    /// the file is missing or unreadable.
    fn load(directory: &Path) -> Self {
        let patterns = fs::read_to_string(directory.join(".gitignore"))
            .map(|content| parse_gitignore_patterns(&content))
            .unwrap_or_default();

        Self { patterns }
    }

    /// Returns `true` if any loaded pattern matches `name`/`path`.
    fn is_ignored(&self, name: &str, path: &Path) -> bool {
        self.patterns
            .iter()
            .any(|pattern| matches_gitignore_pattern(pattern, name, path))
    }
}

/// Parses `.gitignore` content into patterns, skipping blank lines,
/// comments (`#`), and negations (`!`), and stripping a leading `/`.
fn parse_gitignore_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('!'))
        .map(|line| line.trim_start_matches('/').to_string())
        .collect()
}

/// Returns `true` if `pattern` matches `name`/`path`. A trailing `/` matches
/// only directories by name; a `*` triggers wildcard matching; otherwise an
/// exact name match is required.
fn matches_gitignore_pattern(pattern: &str, name: &str, path: &Path) -> bool {
    if let Some(directory_pattern) = pattern.strip_suffix('/') {
        return path.is_dir() && directory_pattern == name;
    }

    if pattern.contains('*') {
        return matches_wildcard(pattern, name);
    }

    pattern == name
}

/// Matches `name` against a glob-style `pattern` containing `*` wildcards.
fn matches_wildcard(pattern: &str, name: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        return pattern == name;
    }

    let mut remaining = name;

    if let Some(first) = parts.first() {
        if !first.is_empty() && !remaining.starts_with(first) {
            return false;
        }
        remaining = remaining.strip_prefix(first).unwrap_or(remaining);
    }

    for part in parts.iter().skip(1).take(parts.len().saturating_sub(2)) {
        let Some(index) = remaining.find(part) else {
            return false;
        };
        remaining = &remaining[index + part.len()..];
    }

    if let Some(last) = parts.last() {
        last.is_empty() || remaining.ends_with(last)
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- human_readable_size ---

    #[test]
    fn formats_sizes_for_humans() {
        assert_eq!(human_readable_size(512), "512B");
        assert_eq!(human_readable_size(1536), "1.5KB");
        assert_eq!(human_readable_size(1_048_576), "1.0MB");
    }

    #[test]
    fn formats_zero_bytes() {
        assert_eq!(human_readable_size(0), "0B");
    }

    #[test]
    fn formats_boundary_values() {
        assert_eq!(human_readable_size(1023), "1023B");
        assert_eq!(human_readable_size(1024), "1.0KB");
        assert_eq!(human_readable_size(1024 * 1024 * 1024), "1.0GB");
    }

    // --- is_hidden_file ---

    #[test]
    fn detects_hidden_files() {
        assert!(is_hidden_file(".gitignore"));
        assert!(is_hidden_file(".env"));
        assert!(!is_hidden_file("README.md"));
        assert!(!is_hidden_file(""));
    }

    // --- is_tagline_heading ---

    #[test]
    fn detects_tagline_heading() {
        assert!(is_tagline_heading("## Tagline"));
        assert!(is_tagline_heading("## tagline"));
        assert!(is_tagline_heading("## Tagline（1行概要）"));
        assert!(is_tagline_heading("# Tagline"));
        assert!(!is_tagline_heading("## Overview"));
        assert!(!is_tagline_heading("Tagline"));
    }

    // --- is_pdf ---

    #[test]
    fn detects_pdf_extension() {
        assert!(is_pdf(Path::new("report.pdf")));
        assert!(is_pdf(Path::new("report.PDF")));
        assert!(!is_pdf(Path::new("report.txt")));
        assert!(!is_pdf(Path::new("report")));
    }

    // --- extract_readme_tagline ---

    #[test]
    fn extracts_tagline_from_readme() {
        let content = "# demo\n\n## Tagline（1行概要）\n便利な一覧表示ツール\n";

        assert_eq!(
            extract_readme_tagline(content),
            Some("便利な一覧表示ツール".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_tagline_heading() {
        assert_eq!(extract_readme_tagline("# Overview\nsome text\n"), None);
    }

    #[test]
    fn returns_none_when_tagline_heading_has_no_content() {
        assert_eq!(extract_readme_tagline("## Tagline\n"), None);
    }

    #[test]
    fn returns_none_for_empty_readme() {
        assert_eq!(extract_readme_tagline(""), None);
    }

    // --- extract_pdf_title ---

    #[test]
    fn extracts_pdf_title_metadata() {
        assert_eq!(
            extract_pdf_title("1 0 obj << /Title (Sample PDF) >> endobj"),
            Some("Sample PDF".to_string())
        );
    }

    #[test]
    fn extracts_pdf_title_from_dc_title() {
        assert_eq!(
            extract_pdf_title("<dc:title>My Document</dc:title>"),
            Some("My Document".to_string())
        );
    }

    #[test]
    fn extracts_pdf_title_from_html_title() {
        assert_eq!(
            extract_pdf_title("<title>Page Title</title>"),
            Some("Page Title".to_string())
        );
    }

    #[test]
    fn returns_none_when_no_pdf_title() {
        assert_eq!(extract_pdf_title("no title here"), None);
    }

    #[test]
    fn returns_none_for_empty_pdf_title() {
        assert_eq!(extract_pdf_title("/Title ()"), None);
    }

    #[test]
    fn cleans_escaped_parens_in_pdf_title() {
        // extract_between stops at the first ')' so escaped parens must be
        // cleaned after extraction; test clean_pdf_title directly here.
        assert_eq!(clean_pdf_title("foo \\(bar\\)".to_string()), "foo (bar)");
        assert_eq!(clean_pdf_title("a \\\\b".to_string()), "a \\b");
    }

    // --- parse_args ---

    fn args(v: &[&str]) -> impl Iterator<Item = String> {
        v.iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
            .into_iter()
    }

    #[test]
    fn parse_args_defaults() {
        let config = parse_args(args(&[])).unwrap();
        assert_eq!(config.path, PathBuf::from("."));
        assert!(!config.show_hidden);
        assert!(!config.long_format);
        assert_eq!(config.sort_key, SortKey::Name);
    }

    #[test]
    fn parse_args_show_hidden() {
        let config = parse_args(args(&["-a"])).unwrap();
        assert!(config.show_hidden);
    }

    #[test]
    fn parse_args_long_format() {
        let config = parse_args(args(&["-l"])).unwrap();
        assert!(config.long_format);
    }

    #[test]
    fn parse_args_sort_keys() {
        assert_eq!(parse_args(args(&["-s"])).unwrap().sort_key, SortKey::Size);
        assert_eq!(
            parse_args(args(&["-t"])).unwrap().sort_key,
            SortKey::Modified
        );
        assert_eq!(parse_args(args(&["-n"])).unwrap().sort_key, SortKey::Name);
    }

    #[test]
    fn parse_args_combined_flags() {
        let config = parse_args(args(&["-al"])).unwrap();
        assert!(config.show_hidden);
        assert!(config.long_format);
    }

    #[test]
    fn parse_args_explicit_path() {
        let config = parse_args(args(&["src"])).unwrap();
        assert_eq!(config.path, PathBuf::from("src"));
    }

    #[test]
    fn parse_args_unknown_flag_returns_error() {
        assert!(parse_args(args(&["-z"])).is_err());
    }

    #[test]
    fn parse_args_two_paths_returns_error() {
        assert!(parse_args(args(&["src", "docs"])).is_err());
    }

    // --- parse_gitignore_patterns ---

    #[test]
    fn parses_gitignore_patterns() {
        let content = "# comment\n\ntarget/\n*.log\n!keep.log\n/dist\n";
        let patterns = parse_gitignore_patterns(content);
        assert!(!patterns.iter().any(|p| p.starts_with('#')));
        assert!(!patterns.contains(&String::new()));
        assert!(!patterns.iter().any(|p| p.starts_with('!')));
        assert!(patterns.contains(&"target/".to_string()));
        assert!(patterns.contains(&"*.log".to_string()));
        assert!(patterns.contains(&"dist".to_string()));
    }

    // --- matches_wildcard ---

    #[test]
    fn matches_simple_gitignore_patterns() {
        assert!(matches_wildcard("*.log", "error.log"));
        assert!(matches_wildcard("build-*", "build-debug"));
        assert!(!matches_wildcard("*.log", "error.txt"));
    }

    #[test]
    fn wildcard_matches_any_suffix() {
        assert!(matches_wildcard("*.rs", "main.rs"));
        assert!(!matches_wildcard("*.rs", "main.go"));
    }

    #[test]
    fn wildcard_matches_any_prefix() {
        assert!(matches_wildcard("test_*", "test_foo"));
        assert!(!matches_wildcard("test_*", "foo_test"));
    }

    // --- format_modified_time ---

    #[test]
    fn formats_none_time_as_dash() {
        assert_eq!(format_modified_time(None), "-");
    }

    #[test]
    fn formats_unix_epoch_as_zero_seconds() {
        assert_eq!(format_modified_time(Some(UNIX_EPOCH)), "0s");
    }
}
