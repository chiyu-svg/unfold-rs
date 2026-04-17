// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::PathBuf;
use unfold_core::{cmd_run, cmd_undo, cmd_log, ConflictStrategy};
use serde::{Deserialize, Serialize};

/// 执行文件平铺操作的参数
#[derive(Debug, Serialize, Deserialize)]
pub struct RunParams {
    pub source: String,
    pub dest: String,
    #[serde(rename = "moveFiles")]
    pub move_files: bool,
    #[serde(rename = "dryRun")]
    pub dry_run: bool,
    pub conflict: String,
    pub cleanup: bool,
}

/// 执行文件平铺操作
#[tauri::command]
fn run(params: RunParams) -> Result<String, String> {
    let source = PathBuf::from(&params.source);
    let dest = PathBuf::from(&params.dest);
    let conflict = match params.conflict.as_str() {
        "overwrite" => ConflictStrategy::Overwrite,
        "skip" => ConflictStrategy::Skip,
        "rename" => ConflictStrategy::Rename,
        _ => return Err("无效的冲突策略".to_string()),
    };

    cmd_run(&source, &dest, params.move_files, params.dry_run, conflict, params.cleanup)
        .map(|_| "操作完成".to_string())
        .map_err(|e| e.to_string())
}

/// 撤销上一次操作
#[tauri::command]
fn undo() -> Result<String, String> {
    cmd_undo()
        .map(|_| "撤销完成".to_string())
        .map_err(|e| e.to_string())
}

/// 查看操作日志
#[tauri::command]
fn log() -> Result<String, String> {
    cmd_log()
        .map(|_| "日志查看完成".to_string())
        .map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![run, undo, log])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
