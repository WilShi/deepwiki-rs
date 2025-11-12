@echo off
REM DeepWiki-RS Windows 启动脚本
REM 使用方法：run_deepwiki.bat "项目路径" "输出路径"

REM 设置默认值
set PROJECT_PATH=%1
set OUTPUT_PATH=%2

REM 如果没有提供参数，显示帮助
if "%PROJECT_PATH%"=="" (
    echo.
    echo DeepWiki-RS - AI 驱动的架构文档生成工具
    echo.
    echo 使用方法：
    echo   run_deepwiki.bat "项目路径" "输出路径"
    echo.
    echo 示例：
    echo   run_deepwiki.bat "C:\my-project" "C:\docs"
    echo.
    echo 或者直接运行 deepwiki-rs.exe --help 查看更多选项
    echo.
    pause
    exit /b 1
)

REM 如果没有提供输出路径，使用默认值
if "%OUTPUT_PATH%"=="" (
    set OUTPUT_PATH=%PROJECT_PATH%\deepwiki-docs
)

REM 运行 deepwiki-rs
echo.
echo 正在分析项目：%PROJECT_PATH%
echo 输出目录：%OUTPUT_PATH%
echo.

deepwiki-rs.exe -p "%PROJECT_PATH%" -o "%OUTPUT_PATH%" %3 %4 %5 %6 %7 %8 %9

echo.
echo 文档生成完成！
echo 输出位置：%OUTPUT_PATH%
echo.
pause