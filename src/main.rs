use std::cmp::Ordering;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, Metadata};
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SortKey {
    Name,
    Size,
    Modified,
}

#[derive(Debug)]
struct Config {
    path: PathBuf,
    show_hidden: bool,
    long_format: bool,
    sort_key: SortKey,
}

#[derive(Debug)]
struct FileInfo {
    name: String,
    metadata: Metadata,
    extension_info: Option<String>,
}

#[derive(Debug)]
struct GitignoreRules {
    patterns: Vec<String>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("ustam: {error}");
        std::process::exit(1);
    }
}

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

fn should_skip_entry(entry: &DirEntry, show_hidden: bool, rules: &GitignoreRules) -> bool {
    let name = entry.file_name().to_string_lossy().to_string();

    (!show_hidden && is_hidden_file(&name)) || rules.is_ignored(&name, &entry.path())
}

fn is_hidden_file(name: &str) -> bool {
    name.starts_with('.')
}

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

fn find_extension_info(path: &Path, metadata: &Metadata) -> Option<String> {
    if metadata.is_dir() {
        return read_readme_tagline(path).map(|tagline| format!("README: {tagline}"));
    }

    if is_pdf(path) {
        return read_pdf_title(path).map(|title| format!("PDF: {title}"));
    }

    None
}

fn sort_files(files: &mut [FileInfo], sort_key: SortKey) {
    match sort_key {
        SortKey::Name => files.sort_by(compare_by_name),
        SortKey::Size => files.sort_by(compare_by_size),
        SortKey::Modified => files.sort_by(compare_by_modified),
    }
}

fn compare_by_name(left: &FileInfo, right: &FileInfo) -> Ordering {
    left.name.to_lowercase().cmp(&right.name.to_lowercase())
}

fn compare_by_size(left: &FileInfo, right: &FileInfo) -> Ordering {
    right
        .metadata
        .len()
        .cmp(&left.metadata.len())
        .then_with(|| compare_by_name(left, right))
}

fn compare_by_modified(left: &FileInfo, right: &FileInfo) -> Ordering {
    let left_time = left.metadata.modified().unwrap_or(UNIX_EPOCH);
    let right_time = right.metadata.modified().unwrap_or(UNIX_EPOCH);

    right_time
        .cmp(&left_time)
        .then_with(|| compare_by_name(left, right))
}

fn print_files(files: &[FileInfo], long_format: bool) {
    for file in files {
        if long_format {
            print_long_file(file);
        } else {
            println!("{}", file.name);
        }
    }
}

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

fn file_kind_label(metadata: &Metadata) -> &'static str {
    if metadata.is_dir() {
        "dir"
    } else if metadata.is_file() {
        "file"
    } else {
        "etc"
    }
}

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

fn format_modified_time(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "-".to_string();
    };

    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => format!("{}s", duration.as_secs()),
        Err(_) => "-".to_string(),
    }
}

fn read_readme_tagline(directory: &Path) -> Option<String> {
    let readme_path = directory.join("README.md");
    let content = fs::read_to_string(readme_path).ok()?;

    extract_readme_tagline(&content)
}

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

fn is_tagline_heading(line: &str) -> bool {
    line.starts_with('#') && line.to_lowercase().contains("tagline")
}

fn is_pdf(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pdf"))
}

fn read_pdf_title(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    let text = String::from_utf8_lossy(&bytes);

    extract_pdf_title(&text)
}

fn extract_pdf_title(text: &str) -> Option<String> {
    extract_between(text, "/Title (", ")")
        .or_else(|| extract_between(text, "<dc:title>", "</dc:title>"))
        .or_else(|| extract_between(text, "<title>", "</title>"))
        .map(clean_pdf_title)
        .filter(|title| !title.is_empty())
}

fn extract_between(text: &str, start: &str, end: &str) -> Option<String> {
    let start_index = text.find(start)? + start.len();
    let rest = &text[start_index..];
    let end_index = rest.find(end)?;

    Some(rest[..end_index].to_string())
}

fn clean_pdf_title(title: String) -> String {
    title
        .replace("\\(", "(")
        .replace("\\)", ")")
        .replace("\\\\", "\\")
        .trim()
        .to_string()
}

impl GitignoreRules {
    fn load(directory: &Path) -> Self {
        let patterns = fs::read_to_string(directory.join(".gitignore"))
            .map(|content| parse_gitignore_patterns(&content))
            .unwrap_or_default();

        Self { patterns }
    }

    fn is_ignored(&self, name: &str, path: &Path) -> bool {
        self.patterns
            .iter()
            .any(|pattern| matches_gitignore_pattern(pattern, name, path))
    }
}

fn parse_gitignore_patterns(content: &str) -> Vec<String> {
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#') && !line.starts_with('!'))
        .map(|line| line.trim_start_matches('/').to_string())
        .collect()
}

fn matches_gitignore_pattern(pattern: &str, name: &str, path: &Path) -> bool {
    if let Some(directory_pattern) = pattern.strip_suffix('/') {
        return path.is_dir() && directory_pattern == name;
    }

    if pattern.contains('*') {
        return matches_wildcard(pattern, name);
    }

    pattern == name
}

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

    #[test]
    fn formats_sizes_for_humans() {
        assert_eq!(human_readable_size(512), "512B");
        assert_eq!(human_readable_size(1536), "1.5KB");
        assert_eq!(human_readable_size(1_048_576), "1.0MB");
    }

    #[test]
    fn extracts_tagline_from_readme() {
        let content = "# demo\n\n## Tagline（1行概要）\n便利な一覧表示ツール\n";

        assert_eq!(
            extract_readme_tagline(content),
            Some("便利な一覧表示ツール".to_string())
        );
    }

    #[test]
    fn extracts_pdf_title_metadata() {
        assert_eq!(
            extract_pdf_title("1 0 obj << /Title (Sample PDF) >> endobj"),
            Some("Sample PDF".to_string())
        );
    }

    #[test]
    fn matches_simple_gitignore_patterns() {
        assert!(matches_wildcard("*.log", "error.log"));
        assert!(matches_wildcard("build-*", "build-debug"));
        assert!(!matches_wildcard("*.log", "error.txt"));
    }
}
