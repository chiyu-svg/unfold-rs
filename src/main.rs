use std::{fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use clap::Parser;
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    fs::create_dir_all(&args.dest)
        .with_context(|| format!("无法创建目标目录： {}", args.dest.display()))?; // 包装错误信息

    println!("源文件: {}", args.source.display());
    println!("目标目录: {}", args.dest.display());
    println!("操作模式: {}", if args.move_files { "移动" } else {"复制"});
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
            let dest_path = args.dest.join(file_name);
            // 执行复制或移动
            if let Err(e) = process_file(source_path, &dest_path, args.move_files) {
                eprintln!("处理文件 {} 时出错: {}", source_path.display(), e);
            } else {
                println!("成功处理文件 {}", source_path.display());
            }
        }
    };
    Ok(())
}

fn process_file(source: &Path, dest: &Path, move_files: bool) -> Result<()> {
    if move_files {
        fs::rename(source,dest)
        .with_context(|| format!("移动文件失败: {} -> {}", source.display(), dest.display()))?;
    } else {
        fs::copy(source, dest)
        .with_context(|| format!("复制文件失败: {} -> {}", source.display(), dest.display()))?;
    }
    Ok(())
}