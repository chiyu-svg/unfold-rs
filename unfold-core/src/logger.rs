use std::{fs, path::{Path, PathBuf}};
use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

pub const LOG_FILE: &str = "unfold_log.json";

/// 操作日志
#[derive(Debug, Serialize, Deserialize)]
pub struct OperationLog {
    /// 操作日志条目
    pub entries: Vec<LogEntry>,
}

/// 操作日志条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// 原始路径
    pub source: PathBuf,
    /// 目标路径
    pub dest: PathBuf,
    /// 操作时间
    pub timestamp: String,
}

/// 保存操作日志
pub fn save_log(entries: &[LogEntry]) -> Result<()> {
    let log = OperationLog {
        entries: entries.to_vec(),
    };
    let json = serde_json::to_string_pretty(&log).with_context(|| "序列化日志失败")?;
    fs::write(LOG_FILE, json).with_context(|| format!("写入日志文件失败: {}", LOG_FILE))?;
    Ok(())
}

/// 查看操作日志
pub fn cmd_log() -> Result<()> {
    // 检查日志文件是否存在
    if !Path::new(LOG_FILE).exists() {
        println!("{} 没有找到操作日志", "ℹ️".yellow());
        return Ok(());
    }
    let log_content = fs::read_to_string(LOG_FILE)
        .with_context(|| format!("无法读取日志文件: {}", LOG_FILE))?;
    let operation_log: OperationLog = serde_json::from_str(&log_content)
        .with_context(|| "解析日志文件失败")?;

    println!("{} 操作日志 (共 {} 条记录):", "📋".cyan(), operation_log.entries.len());
    println!();
    for (i, entry) in operation_log.entries.iter().enumerate() {
        println!("{}. [{}]", i + 1, entry.timestamp.dimmed());
        println!("   {} {}", "→".cyan(), entry.dest.display());
        println!("   {} {}", "←".green(), entry.source.display());
        println!();
    }
    Ok(())
}
