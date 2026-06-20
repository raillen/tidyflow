use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use globset::{Glob, GlobSetBuilder};
use regex::Regex;

use crate::file_filter::{FileFilter, SymlinkMode};
use crate::job::Job;

pub struct FilterEngine {
    exclude: globset::GlobSet,
    name_regex: Option<Regex>,
    path_regex: Option<Regex>,
}

impl FilterEngine {
    pub fn from_job(job: &Job) -> Result<Self, String> {
        let patterns = job.filters.expanded_exclude_patterns();
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            let glob = Glob::new(&pattern).map_err(|e| e.to_string())?;
            builder.add(glob);
        }
        let exclude = builder.build().map_err(|e| e.to_string())?;

        let name_regex = job
            .filters
            .name_regex
            .as_ref()
            .map(|r| Regex::new(r))
            .transpose()
            .map_err(|e| e.to_string())?;

        let path_regex = job
            .filters
            .path_regex
            .as_ref()
            .map(|r| Regex::new(r))
            .transpose()
            .map_err(|e| e.to_string())?;

        Ok(Self {
            exclude,
            name_regex,
            path_regex,
        })
    }

    pub fn should_process(&self, file: &Path, job: &Job, source_root: &Path) -> Result<bool, String> {
        let filters = &job.filters;
        let meta = fs::metadata(file).map_err(|e| e.to_string())?;

        if filters.skip_empty_files && meta.len() == 0 {
            return Ok(false);
        }

        if !filters.include_hidden {
            if is_hidden(file) {
                return Ok(false);
            }
        }

        if let Some(min) = filters.min_size_bytes {
            if meta.len() < min {
                return Ok(false);
            }
        }
        if let Some(max) = filters.max_size_bytes {
            if meta.len() > max {
                return Ok(false);
            }
        }

        if let Some(max_depth) = filters.max_depth {
            let depth = relative_depth(source_root, file);
            if depth > max_depth {
                return Ok(false);
            }
        }

        if let Some(modified_before) = filters.modified_before {
            if file_modified(file).is_some_and(|m| m > modified_before) {
                return Ok(false);
            }
        }
        if let Some(modified_after) = filters.modified_after {
            if file_modified(file).is_some_and(|m| m < modified_after) {
                return Ok(false);
            }
        }

        if let Some(created_after) = filters.created_after {
            if file_created(file).is_some_and(|c| c < created_after) {
                return Ok(false);
            }
        }
        if let Some(created_before) = filters.created_before {
            if file_created(file).is_some_and(|c| c > created_before) {
                return Ok(false);
            }
        }

        if let Some(days) = filters.older_than_days {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    if elapsed.as_secs() < days as u64 * 86400 {
                        return Ok(false);
                    }
                }
            }
        }

        let path_str = file.to_string_lossy();
        if self.exclude.is_match(path_str.as_ref()) {
            return Ok(false);
        }

        if !filters.include_extensions.is_empty() {
            let ext = file
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e.to_lowercase()))
                .unwrap_or_default();
            if !filters
                .include_extensions
                .iter()
                .any(|allowed| allowed.trim().to_lowercase() == ext)
            {
                return Ok(false);
            }
        }

        if let Some(regex) = &self.name_regex {
            let name = file.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !regex.is_match(name) {
                return Ok(false);
            }
        }

        if let Some(regex) = &self.path_regex {
            if !regex.is_match(&path_str) {
                return Ok(false);
            }
        }

        if let Some(needle) = &filters.content_contains {
            if !content_matches(file, needle, filters)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

fn is_hidden(path: &Path) -> bool {
    file_stem_starts_dot(path) || windows_hidden(path)
}

fn file_stem_starts_dot(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.'))
        .unwrap_or(false)
}

#[cfg(windows)]
fn windows_hidden(path: &Path) -> bool {
    use std::os::windows::fs::MetadataExt;
    fs::metadata(path)
        .map(|m| m.file_attributes() & 0x2 != 0)
        .unwrap_or(false)
}

#[cfg(not(windows))]
fn windows_hidden(_path: &Path) -> bool {
    false
}

fn file_modified(path: &Path) -> Option<chrono::DateTime<chrono::Utc>> {
    fs::metadata(path)
        .ok()?
        .modified()
        .ok()?
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
}

fn file_created(path: &Path) -> Option<chrono::DateTime<chrono::Utc>> {
    let meta = fs::metadata(path).ok()?;
    let created = meta.created().ok().or_else(|| meta.modified().ok())?;
    created
        .duration_since(std::time::UNIX_EPOCH)
        .ok()
        .and_then(|d| chrono::DateTime::from_timestamp(d.as_secs() as i64, 0))
}

fn relative_depth(root: &Path, file: &Path) -> u32 {
    file.strip_prefix(root)
        .map(|rel| rel.components().count().saturating_sub(1) as u32)
        .unwrap_or(0)
}

fn content_matches(file: &Path, needle: &str, filters: &FileFilter) -> Result<bool, String> {
    let ext = file
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| format!(".{}", e.to_lowercase()))
        .unwrap_or_default();
    if !filters.content_extensions.is_empty()
        && !filters
            .content_extensions
            .iter()
            .any(|e| e.trim().to_lowercase() == ext)
    {
        return Ok(false);
    }

    let meta = fs::metadata(file).map_err(|e| e.to_string())?;
    if meta.len() > filters.content_max_bytes {
        return Ok(false);
    }

    let mut buf = Vec::new();
    fs::File::open(file)
        .and_then(|mut f| f.read_to_end(&mut buf))
        .map_err(|e| e.to_string())?;

    let haystack = String::from_utf8_lossy(&buf);
    Ok(haystack.contains(needle))
}

pub fn collect_files(job: &Job) -> Result<Vec<PathBuf>, std::io::Error> {
    let source = Path::new(&job.source_path);
    let mut files = Vec::new();
    if !source.exists() {
        return Ok(files);
    }

    let engine = FilterEngine::from_job(job).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if source.is_file() {
        if engine.should_process(source, job, source).unwrap_or(false) {
            files.push(source.to_path_buf());
        }
        return Ok(files);
    }

    if job.filters.recursive {
        for entry in walkdir::WalkDir::new(source)
            .follow_links(job.filters.symlink_mode == SymlinkMode::Follow)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path().to_path_buf();
            if entry.file_type().is_symlink() && job.filters.symlink_mode == SymlinkMode::Skip {
                continue;
            }
            if !entry.file_type().is_file() {
                continue;
            }
            if engine.should_process(&path, job, source).unwrap_or(false) {
                files.push(path);
            }
        }
    } else if source.is_dir() {
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                && engine.should_process(&path, job, source).unwrap_or(false)
            {
                files.push(path);
            }
        }
    }

    Ok(files)
}

pub fn should_process_file(file: &Path, job: &Job) -> bool {
    let source = Path::new(&job.source_path);
    FilterEngine::from_job(job)
        .ok()
        .and_then(|e| e.should_process(file, job, source).ok())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::Job;

    #[test]
    fn filters_by_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let pdf = dir.path().join("doc.PDF");
        let txt = dir.path().join("doc.txt");
        std::fs::write(&pdf, b"x").expect("write pdf");
        std::fs::write(&txt, b"x").expect("write txt");

        let mut job = Job::default();
        job.source_path = dir.path().display().to_string();
        job.filters.include_extensions = vec![".pdf".into()];
        assert!(should_process_file(&pdf, &job));
        assert!(!should_process_file(&txt, &job));
    }

    #[test]
    fn exclude_preset_node_modules() {
        let mut job = Job::default();
        job.filters.exclude_preset_ids = vec!["node_modules".into()];
        assert!(!should_process_file(
            Path::new("project/node_modules/pkg/index.js"),
            &job
        ));
    }
}
