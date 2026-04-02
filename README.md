# unfold-rs

一个用于将嵌套目录中的文件平铺到目标目录的 Rust CLI 工具。

## 功能特性

- 📁 **目录平铺**：递归遍历源目录，将所有文件移动到目标目录的根层级
- 🔄 **移动/复制模式**：支持移动或复制文件
- ⚡ **冲突处理**：支持覆盖、跳过、自动重命名三种冲突处理策略
- 🎭 **演习模式**：预览操作结果而不实际执行
- 🧹 **空目录清理**：操作后自动清理空文件夹
- ↩️ **撤销操作**：支持撤销上一次的移动操作
- 📝 **操作日志**：记录所有操作历史
- 📊 **进度显示**：带进度条的操作反馈

## 安装

### 从源码构建

```bash
git clone <repository-url>
cd unfold-rs
cargo build --release
```

构建完成后，可执行文件位于 `target/release/unfold-rs`。

## 使用方法

### 基本用法

```bash
# 复制模式（默认）：将源目录中的文件复制到目标目录
unfold-rs run <源目录> <目标目录>

# 移动模式：将源目录中的文件移动到目标目录
unfold-rs run <源目录> <目标目录> --move-files
```

### 命令选项

```
unfold-rs run [OPTIONS] <SOURCE> <DEST>

参数:
  <SOURCE>  源目录
  <DEST>    目标目录

选项:
  -m, --move-files      使用移动而非复制
      --dry-run         演习模式（不实际执行操作）
      --conflict <STRATEGY>  冲突处理策略 [默认: rename] [可选: overwrite, skip, rename]
      --cleanup         清理空文件夹
  -h, --help            显示帮助信息
```

### 冲突处理策略

- `overwrite` - 覆盖已存在的文件
- `skip` - 跳过已存在的文件
- `rename`（默认）- 自动重命名文件（如 `file.txt` → `file_1.txt`）

### 示例

```bash
# 基本复制操作
unfold-rs run ./downloads ./flattened

# 移动文件并清理空目录
unfold-rs run ./downloads ./flattened --move-files --cleanup

# 演习模式（预览操作结果）
unfold-rs run ./downloads ./flattened --dry-run

# 使用覆盖策略
unfold-rs run ./downloads ./flattened --conflict overwrite

# 撤销上一次移动操作
unfold-rs undo

# 查看操作日志
unfold-rs log
```

## 项目结构

```
unfold-rs/
├── src/
│   ├── main.rs    # 程序入口
│   ├── cli.rs     # 命令行参数定义
│   ├── core.rs    # 核心逻辑实现
│   ├── logger.rs  # 日志记录功能
│   └── utils.rs   # 工具函数
├── Cargo.toml
└── README.md
```

## 依赖项

- [anyhow](https://crates.io/crates/anyhow) - 错误处理
- [chrono](https://crates.io/crates/chrono) - 日期时间处理
- [clap](https://crates.io/crates/clap) - 命令行参数解析
- [colored](https://crates.io/crates/colored) - 终端颜色输出
- [indicatif](https://crates.io/crates/indicatif) - 进度条显示
- [serde](https://crates.io/crates/serde) - 序列化
- [serde_json](https://crates.io/crates/serde_json) - JSON 处理
- [walkdir](https://crates.io/crates/walkdir) - 目录遍历

## 许可证

[MIT](LICENSE)
