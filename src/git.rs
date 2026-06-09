use std::path::{Path, PathBuf};
use std::process::Command;

use crate::detect;

pub struct GitInfo {
    pub tracked: bool,
    pub repo_root: Option<String>,
    pub head_encoding: Option<String>,
    pub head_confidence: Option<f32>,
}

pub fn get_git_info(file_path: &Path) -> Option<GitInfo> {
    let repo_dir = find_repo_root(file_path)?;

    if !is_inside_work_tree(&repo_dir) {
        return None;
    }

    let relative_path = get_relative_path(file_path, &repo_dir)?;

    if !is_tracked(&repo_dir, &relative_path) {
        return Some(GitInfo {
            tracked: false,
            repo_root: Some(repo_dir.to_string_lossy().to_string()),
            head_encoding: None,
            head_confidence: None,
        });
    }

    let head_bytes = get_head_version(&repo_dir, &relative_path)?;

    if head_bytes.is_empty() || looks_emptyish(&head_bytes) {
        return Some(GitInfo {
            tracked: true,
            repo_root: Some(repo_dir.to_string_lossy().to_string()),
            head_encoding: None,
            head_confidence: None,
        });
    }

    let detection = detect::detect(&head_bytes, None).ok()?;

    Some(GitInfo {
        tracked: true,
        repo_root: Some(repo_dir.to_string_lossy().to_string()),
        head_encoding: Some(detection.name.to_string()),
        head_confidence: Some(detection.confidence),
    })
}

fn find_repo_root(file_path: &Path) -> Option<PathBuf> {
    let file_abs = std::fs::canonicalize(file_path).ok()?;
    let mut dir = file_abs.parent()?;
    loop {
        if dir.join(".git").exists() {
            return Some(dir.to_path_buf());
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => return None,
        }
    }
}

fn is_inside_work_tree(repo_dir: &Path) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo_dir)
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout).trim() == "true"
        })
        .unwrap_or(false)
}

fn get_relative_path(file_path: &Path, repo_dir: &Path) -> Option<String> {
    let file_abs = std::fs::canonicalize(file_path).ok()?;
    let repo_parent = std::fs::canonicalize(repo_dir).ok()?;
    let rel = file_abs.strip_prefix(&repo_parent).ok()?;
    Some(rel.to_string_lossy().to_string())
}

fn is_tracked(repo_dir: &Path, relative_path: &str) -> bool {
    Command::new("git")
        .args(["-C"])
        .arg(repo_dir)
        .args(["ls-files", "--error-unmatch", relative_path])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn get_head_version(repo_dir: &Path, relative_path: &str) -> Option<Vec<u8>> {
    let ref_name = format!("HEAD:{}", relative_path);
    Command::new("git")
        .args(["-C"])
        .arg(repo_dir)
        .args(["show", &ref_name])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                Some(o.stdout)
            } else {
                None
            }
        })
}

fn looks_emptyish(bytes: &[u8]) -> bool {
    bytes.iter().all(|&b| b == 0 || b.is_ascii_whitespace())
}
