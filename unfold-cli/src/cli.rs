use std::path::PathBuf;
use clap::{Parser, Subcommand};
use unfold_core::ConflictStrategy;

#[derive(Parser, Debug)]
#[command(name = "unfold-rs")]
#[command(about = "将嵌套目录中的文件平铺到目标目录")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 执行文件平铺操作
    Run {
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
        /// 清理空文件夹
        #[arg(long)]
        cleanup: bool,
    },
    /// 撤销上一次移动操作
    Undo,
    /// 查看操作日志
    Log,
}
