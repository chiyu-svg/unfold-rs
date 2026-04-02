use std::{ffi::OsStr, fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use chrono::Local;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use walkdir::WalkDir;
use crate::{cli::ConflictStrategy, logger::{LOG_FILE, LogEntry, OperationLog, save_log}, utils::{cleanup_empty_dirs, count_files}};


pub fn cmd_run(
    source: &Path,
    dest: &Path,
    move_files: bool,
    dry_run: bool,
    conflict: ConflictStrategy,
    cleanup: bool,
) -> Result<()> {
    // 演习模式下不创建目录
    if !dry_run {
        fs::create_dir_all(dest)
            .with_context(|| format!("无法创建目标目录： {}", dest.display()))?;
    }
    // 统计文件总数
    let file_count = count_files(source)?;
    if dry_run {
        println!("【演习模式】不会实际写入文件");
    }
    println!("{} {}", "📁 源目录:".cyan(), source.display());
    println!("{} {}", "📂 目标目录:".cyan(), dest.display());
    println!("{} {}", "📊 文件总数:".cyan(), file_count);
    println!(
        "{} {}",
        "⚙️ 操作模式:".cyan(),
        if move_files {
            "移动".yellow()
        } else {
            "复制".green()
        }
    );
    println!("{} {:?}", "🔧 冲突策略:".cyan(), conflict);
    if dry_run {
        println!("{}", "🎭 【演习模式】不会实际写入文件".yellow());
    }
    println!();
    // 创建进度条
    let pb = if dry_run {
        None
    } else {
        let pb = ProgressBar::new(file_count as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    };

    // 收集所有文件路径
    let mut files_to_process = Vec::new();

    for entry in WalkDir::new(source) {
        let entry = entry.with_context(|| "遍历目录时发生错误")?;
        if entry.file_type().is_file() {
            files_to_process.push(entry.path().to_path_buf());
        }
    }
    // 记录移动操作的日志
    let mut moved_files: Vec<LogEntry> = Vec::new();

    // 处理文件
    for source_path in files_to_process {
        let file_name = source_path
            .file_name()
            .with_context(|| format!("无法获取文件名: {}", source_path.display()))?;
        // 根据冲突策略确定最终目标路径
        let dest_path = resolve_conflict(dest, file_name, conflict)?;
        // 如果策略是 Skip 且文件已存在，dest_path 会是 None
        let Some(dest_path) = dest_path else {
            println!("{} {}", "⏭️  跳过:".yellow(), source_path.display());
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
            continue;
        };
        // 执行操作
        match process_file(&source_path, &dest_path, move_files, dry_run) {
            Ok(_) => {
                // 记录移动操作到日志
                if move_files && !dry_run {
                    moved_files.push(LogEntry {
                        source: source_path.clone(),
                        dest: dest_path.clone(),
                        timestamp: Local::now().to_rfc3339(),
                    })
                }
                if let Some(ref pb) = pb {
                    pb.inc(1);
                }
            }
            Err(e) => {
                eprintln!("{} {}", "❌ 错误:".red(), e);
                if let Some(ref pb) = pb {
                    pb.inc(1);
                }
            }
        }
    }

    if let Some(pb) = pb {
        pb.finish_with_message("完成!".green().to_string());
    }

    // 保存操作日志（仅移动模式且非演习模式）
    if move_files && !dry_run && !moved_files.is_empty() {
        save_log(&moved_files)?;
        println!(
            "\n{} 已记录 {} 个文件的移动操作",
            "📝".cyan(),
            moved_files.len()
        );
    }

    // 清理空文件夹
    if cleanup && !dry_run {
        println!("\n{} 清理空文件夹...", "🧹".cyan());
        match cleanup_empty_dirs(source) {
            Ok(deleted_count) => {
                println!("{} 已删除 {} 个空文件夹", "✅".green(), deleted_count);
            }
            Err(e) => {
                eprintln!("{} 清理空文件夹时出错: {}", "⚠️".yellow(), e);
            }
        }
    }
    Ok(())
}

/// 撤销操作
pub fn cmd_undo() -> Result<()> {
    println!("{} 正在撤销上一次操作...", "↩️".cyan());

    // 读取日志文件
    let log_content =
        fs::read_to_string(LOG_FILE).with_context(|| format!("无法读取日志文件: {}", LOG_FILE))?;

    let operation_log: OperationLog =
        serde_json::from_str(&log_content).with_context(|| "解析日志文件失败")?;

    if operation_log.entries.is_empty() {
        println!("{} 没有可撤销的操作", "ℹ️".yellow());
        return Ok(());
    }

    println!(
        "{} 找到 {} 个文件需要恢复",
        "📋".cyan(),
        operation_log.entries.len()
    );
    println!();

    let mut success_count = 0;
    let mut error_count = 0;

    // 反向遍历，按相反顺序恢复
    for entry in operation_log.entries.iter().rev() {
        // 检查目标文件是否存在
        if !entry.dest.exists() {
            println!(
                "{} 目标文件已不存在: {}",
                "⚠️".yellow(),
                entry.dest.display()
            );
            error_count += 1;
            continue;
        }
        // 确保源目录存在
        if let Some(parent) = entry.source.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("无法创建源目录: {}", parent.display()))?;
        }
        // 执行恢复移动
        match fs::rename(&entry.dest, &entry.source) {
            Ok(_) => {
                println!(
                    "{} {} -> {}",
                    "✓".green(),
                    entry.dest.display(),
                    entry.source.display()
                );
                success_count += 1;
            }
            Err(e) => {
                eprintln!(
                    "{} 恢复失败 {} -> {}: {}",
                    "❌".red(),
                    entry.dest.display(),
                    entry.source.display(),
                    e
                );
                error_count += 1;
            }
        }
    }
    println!();
    println!("{} 成功: {}", "✅".green(), success_count);
    if error_count > 0 {
        println!("{} 失败: {}", "❌".red(), error_count);
    }
    // 删除日志文件
    fs::remove_file(LOG_FILE).ok();
    println!("{} 已清除操作日志", "🗑️".cyan());
    Ok(())
}

// 执行文件操作
pub fn process_file(source: &Path, dest: &Path, move_files: bool, dry_run: bool) -> Result<()> {
    let action = if move_files { "移动" } else { "复制" };
    if dry_run {
        println!(
            "{} [{}] {} -> {}",
            "🎭".yellow(),
            action.yellow(),
            source.display(),
            dest.display()
        );
        return Ok(());
    }
    if move_files {
        fs::rename(source, dest).with_context(|| {
            format!(
                "{} {} -> {}",
                "移动文件失败:".red(),
                source.display(),
                dest.display()
            )
        })?;
    } else {
        fs::copy(source, dest).with_context(|| {
            format!(
                "{} {} -> {}",
                "复制文件失败:".red(),
                source.display(),
                dest.display()
            )
        })?;
    }
    println!(
        "{} [{}] {} -> {}",
        "✓".green(),
        action.green(),
        source.display(),
        dest.display()
    );
    Ok(())
}

// 根据策略生成文件名
fn generate_unique_name(dest_dir: &Path, file_name: &OsStr) -> Result<PathBuf> {
    let file_name_str = file_name
        .to_str()
        .with_context(|| format!("文件名包含无效字符: {:?}", file_name))?;
    // 分离文件名和扩展名
    let (stem, ext) = match file_name_str.rfind('.') {
        Some(pos) => (&file_name_str[..pos], &file_name_str[pos..]),
        None => (file_name_str, ""),
    };
    // 尝试生成唯一文件名
    for counter in 1..=9999 {
        let new_name = format!("{}-{}{}", stem, counter, ext);
        let new_path = dest_dir.join(&new_name);
        if !new_path.exists() {
            return Ok(new_path);
        }
    }
    anyhow::bail!("无法为 {:?} 生成唯一文件名", file_name)
}

// 解决文件冲突
fn resolve_conflict(
    dest_dir: &Path,
    file_name: &OsStr,
    strategy: ConflictStrategy,
) -> Result<Option<PathBuf>> {
    let dest_path = dest_dir.join(file_name);
    // 文件不存在，直接返回
    if !dest_path.exists() {
        return Ok(Some(dest_path));
    }
    match strategy {
        ConflictStrategy::Overwrite => Ok(Some(dest_path)),
        ConflictStrategy::Skip => Ok(None),
        ConflictStrategy::Rename => {
            let new_path = generate_unique_name(dest_dir, file_name)?;
            return Ok(Some(new_path));
        }
    }
}