use std::{ffi::OsStr, fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
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
    #[arg(long,short)]
    move_files: bool,
    /// 演习模式
    #[arg(long)]
    dry_run: bool,
    /// 冲突处理策略
    #[arg(long, value_enum, default_value = "rename")]
    conflict: ConflictStrategy,
    /// 清理控文件夹
    #[arg(long)]
    cleanup: bool
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
    println!("源目录: {}", args.source.display());
    println!("目标目录: {}", args.dest.display());
    println!("操作模式: {}", if args.move_files { "移动" } else {"复制"});
    println!("冲突策略: {:?}", args.conflict);
    if args.dry_run {
        println!("【演习模式】不会实际写入文件");
    }
    println!();


    // 递归遍历源目录
    for entry in WalkDir::new(&args.source) {
        let entry = entry.with_context(|| "遍历目录时发生错误")?;
        // 只处理文件，忽略文件夹
        if entry.file_type().is_file() {
            let source_path = entry.path();
            let file_name = source_path
                .file_name()
                .with_context(|| format!("无法获取文件名: {}", source_path.display()))?;
            // 根据冲突策略确定最终目标路径
            let dest_path = resolve_conflict(&args.dest, file_name, args.conflict)?;
            // 如果策略是 Skip 且文件已存在, dest_path 会是 None
            let Some(dest_path) = dest_path else {
                println!("跳过: {}", source_path.display());
                continue;
            };   
            // 执行操作
            if let Err(e) = process_file(source_path, &dest_path, args.move_files, args.dry_run) {
                eprintln!("错误: {}", e);
            }
            
        }
    };
    Ok(())
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
    let action = if move_files {"Moving"} else { "Copying"};
    if dry_run {
        println!("[演习] {} {} -> {}", action, source.display(), dest.display());
        return Ok(());
    }
    if move_files {
        fs::rename(source,dest)
        .with_context(|| format!("移动文件失败: {} -> {}", source.display(), dest.display()))?;
    } else {
        fs::copy(source, dest)
        .with_context(|| format!("复制文件失败: {} -> {}", source.display(), dest.display()))?;
    }
    println!("{} {} -> {}", action, source.display(), dest.display());
    Ok(())
}