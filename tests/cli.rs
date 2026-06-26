use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn binary() -> PathBuf {
    env!("CARGO_BIN_EXE_ustam").into()
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(binary()).args(args).output().unwrap()
}

fn run_in(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(binary())
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap()
}

// --- helpers ---

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).to_string()
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

// --- basic listing ---

#[test]
fn lists_specified_directory() {
    let out = run(&["src"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("main.rs"));
}

#[test]
fn default_lists_current_directory() {
    let dir = tempdir();
    fs::write(dir.path().join("hello.txt"), "").unwrap();
    let out = run_in(dir.path(), &[]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("hello.txt"));
}

// --- hidden files ---

#[test]
fn hidden_files_hidden_by_default() {
    let dir = tempdir();
    fs::write(dir.path().join("visible.txt"), "").unwrap();
    fs::write(dir.path().join(".hidden"), "").unwrap();
    let out = run_in(dir.path(), &[]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("visible.txt"));
    assert!(!text.contains(".hidden"));
}

#[test]
fn hidden_files_shown_with_a_flag() {
    let dir = tempdir();
    fs::write(dir.path().join(".hidden"), "").unwrap();
    let out = run_in(dir.path(), &["-a"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains(".hidden"));
}

// --- long format ---

#[test]
fn long_format_shows_type_and_size() {
    let dir = tempdir();
    fs::write(dir.path().join("file.txt"), "hello").unwrap();
    let out = run_in(dir.path(), &["-l"]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("file"));
    assert!(text.contains("file.txt"));
}

// --- error cases ---

#[test]
fn error_on_nonexistent_path() {
    let out = run(&["/nonexistent_path_ustam_test"]);
    assert!(!out.status.success());
    assert!(!stderr(&out).is_empty());
}

#[test]
fn error_on_file_not_directory() {
    let dir = tempdir();
    let file = dir.path().join("file.txt");
    fs::write(&file, "").unwrap();
    let out = run(&[file.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(!stderr(&out).is_empty());
}

#[test]
fn error_on_unknown_option() {
    let out = run(&["-z"]);
    assert!(!out.status.success());
    assert!(!stderr(&out).is_empty());
}

#[test]
fn error_on_two_paths() {
    let out = run(&["src", "docs"]);
    assert!(!out.status.success());
    assert!(!stderr(&out).is_empty());
}

// --- help ---

#[test]
fn help_flag_prints_usage() {
    for flag in &["-h", "--help"] {
        let out = run(&[flag]);
        assert!(stdout(&out).contains("Usage:"));
    }
}

// --- gitignore filtering ---

#[test]
fn gitignore_patterns_are_respected() {
    let dir = tempdir();
    fs::write(dir.path().join("keep.txt"), "").unwrap();
    fs::write(dir.path().join("ignore.log"), "").unwrap();
    fs::write(dir.path().join(".gitignore"), "*.log\n").unwrap();
    let out = run_in(dir.path(), &[]);
    assert!(out.status.success());
    let text = stdout(&out);
    assert!(text.contains("keep.txt"));
    assert!(!text.contains("ignore.log"));
}

// --- tempdir helper ---
// Uses a dedicated struct so the directory is cleaned up when the test ends.

struct TempDir(PathBuf);

impl TempDir {
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn tempdir() -> TempDir {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let path = std::env::temp_dir().join(format!("ustam_test_{nanos}"));
    fs::create_dir_all(&path).unwrap();
    TempDir(path)
}
