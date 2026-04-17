use std::{collections::VecDeque, fs, path::{Path, PathBuf}};
use colored::Colorize;
use walkdir::WalkDir;
use anyhow::Result;

/// 统计文件数量
pub fn count_files(source: &Path) -> Result<usize> {
    let count = WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();
    Ok(count)
}

/// 清除空目录
pub fn cleanup_empty_dirs(source: &Path) -> Result<usize> {
    let mut deleted_count = 0;
    let mut dirs_to_check: VecDeque<PathBuf> = VecDeque::new();
    // 收集所有目录
    for entry in WalkDir::new(source) {
        let entry = entry?;
        if entry.file_type().is_dir() {
            dirs_to_check.push_back(entry.path().to_path_buf());
        }
    }
    // 按深度排序（深的先处理）
    dirs_to_check.make_contiguous().sort_by(|a, b| {
        let depth_a = a.components().count();
        let depth_b = b.components().count();
        depth_b.cmp(&depth_a)
    });
    // 删除空目录
    for dir in dirs_to_check {
        if dir == source {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&dir) {
            if entries.count() == 0 {
                fs::remove_dir(&dir)?;
                deleted_count += 1;
                println!("  {} 删除空文件夹: {}", "🗑️".cyan(), dir.display());
            }
        }
    }
    Ok(deleted_count)
}
