use clap::ValueEnum;

/// 冲突处理策略
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ConflictStrategy {
    /// 覆盖已存在的文件
    Overwrite,
    /// 跳过已存在的文件
    Skip,
    /// 自动重命名（默认）
    Rename,
}
