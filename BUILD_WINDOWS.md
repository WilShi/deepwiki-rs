# Windows 版本编译指南

## 方法一：在 Windows 系统上直接编译（推荐）

1. 安装 Rust：访问 https://rustup.rs/ 并按照指示安装
2. 安装 Visual Studio Build Tools 2019 或更高版本
   - 下载地址：https://visualstudio.microsoft.com/zh-hans/downloads/
   - 选择 "Build Tools for Visual Studio"
   - 在安装选项中勾选 "C++ 生成工具"
3. 克隆项目并编译：
   ```cmd
   git clone https://github.com/sopaco/deepwiki-rs.git
   cd deepwiki-rs
   cargo build --release
   ```
4. 编译完成后，可执行文件位于：`target/release/deepwiki-rs.exe`

## 方法二：使用 WSL（Windows Subsystem for Linux）

1. 在 Windows 上安装 WSL：`wsl --install`
2. 在 WSL 中按照 Linux 方式编译，然后复制到 Windows

## 方法三：使用 GitHub Actions（无需本地环境）

1. Fork 本项目到你的 GitHub 账户
2. 进入项目的 Actions 页面
3. 运行 "Build Windows Binary" 工作流
4. 下载生成的 `deepwiki-rs-windows.zip`

## 方法四：使用 Docker（需要 Docker Desktop）

1. 安装 Docker Desktop for Windows
2. 在项目根目录运行：
   ```cmd
   docker build -f Dockerfile.windows -t deepwiki-rs-windows .
   docker run --rm -v %CD%\output:/output deepwiki-rs-windows
   ```

## 使用说明

编译完成后，在 Windows 命令行中使用：

```cmd
# 基本用法
deepwiki-rs.exe -p C:\path\to\your\project -o C:\path\to\output

# 使用 MiniMax 模型
deepwiki-rs.exe -p C:\path\to\your\project -o C:\path\to\output ^
  --model-efficient "MiniMax/MiniMax-M2" ^
  --llm-api-base-url "https://api-inference.modelscope.cn/v1" ^
  --llm-api-key "YOUR_API_KEY"

# 查看帮助
deepwiki-rs.exe --help
```

## 注意事项

1. Windows 路径使用反斜杠 `\` 或双引号包裹
2. 长命令行需要使用 `^` 续行
3. 确保有足够的磁盘空间（建议至少 1GB）
4. 首次编译会下载依赖，需要稳定的网络连接