use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::{
    collections::VecDeque,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "unfold-rs")]
#[command(about = "将嵌套目录中的文件平铺到目标目录")]
struct Args {
    /// 源目录
    source: PathBuf,
    /// 目标目录
    dest: PathBuf,
    /// 使用移动而非复制
    #[arg(long, short)]
    move_files: bool,
    /// 演习模式
    #[arg(long)]
    dry_run: bool,
    /// 冲突处理策略
    #[arg(long, value_enum, default_value = "rename")]
    conflict: ConflictStrategy,
    /// 清理控文件夹
    #[arg(long)]
    cleanup: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ConflictStrategy {
    /// 覆盖已存在的文件
    Overwrite,
    /// 跳过已存在的文件
    Skip,
    /// 自动重命名（默认）
    Rename,
}

fn main() -> Result<()> {
    let args = Args::parse();
    // 演习模式下不创建目录
    if !args.dry_run {
        fs::create_dir_all(&args.dest)
            .with_context(|| format!("无法创建目标目录： {}", args.dest.display()))?;
    }
    // 统计文件总数
    let file_count = count_files(&args.source)?;
    if args.dry_run {
        println!("【演习模式】不会实际写入文件");
    }
    println!("{} {}", "📁 源目录:".cyan(), args.source.display());
    println!("{} {}", "📂 目标目录:".cyan(), args.dest.display());
    println!("{} {}", "📊 文件总数:".cyan(), file_count);
    println!(
        "{} {}",
        "⚙️ 操作模式:".cyan(),
        if args.move_files {
            "移动".yellow()
        } else {
            "复制".green()
        }
    );
    println!("{} {:?}", "🔧 冲突策略:".cyan(), args.conflict);
    if args.dry_run {
        println!("{}", "🎭 【演习模式】不会实际写入文件".yellow());
    }
    println!();
    // 创建进度条
    let pb = if args.dry_run {
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

    for entry in WalkDir::new(&args.source) {
        let entry = entry.with_context(|| "遍历目录时发生错误")?;
        if entry.file_type().is_file() {
            files_to_process.push(entry.path().to_path_buf());
        }
    }
    // 处理文件
    for source_path in files_to_process {
        let file_name = source_path
            .file_name()
            .with_context(|| format!("无法获取文件名: {}", source_path.display()))?;
        // 根据冲突策略确定最终目标路径
        let dest_path = resolve_conflict(&args.dest, file_name, args.conflict)?;
        // 如果策略是 Skip 且文件已存在，dest_path 会是 None
        let Some(dest_path) = dest_path else {
            println!("{} {}", "⏭️  跳过:".yellow(), source_path.display());
            if let Some(ref pb) = pb {
                pb.inc(1);
            }
            continue;
        };
        // 执行操作
        match process_file(&source_path, &dest_path, args.move_files, args.dry_run) {
            Ok(_) => {
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
    // 清理空文件夹
    if args.cleanup && !args.dry_run {
        println!("\n{} 清理空文件夹...", "🧹".cyan());
        match cleanup_empty_dirs(&args.source) {
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

// 统计文件数量
fn count_files(source: &Path) -> Result<usize> {
    let count = WalkDir::new(source)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .count();
    Ok(count)
}
// 清除空目录
fn cleanup_empty_dirs(source: &Path) -> Result<usize> {
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

fn process_file(source: &Path, dest: &Path, move_files: bool, dry_run: bool) -> Result<()> {
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
