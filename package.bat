@echo off
echo Malou Agent 打包脚本
echo ===================

set APP_NAME=Malou Agent
set VERSION=0.1.0
set BUILD_DIR=src-tauri\target\release
set OUTPUT_DIR=dist\release

echo 创建发布目录...
mkdir "%OUTPUT_DIR%" 2>nul

echo 复制可执行文件...
copy "%BUILD_DIR%\malou-agent.exe" "%OUTPUT_DIR%\"

echo 复制依赖文件...
xcopy "%BUILD_DIR%\*.dll" "%OUTPUT_DIR%\" /Y 2>nul

echo 创建快捷方式...
echo Set oWS = WScript.CreateObject("WScript.Shell") > CreateShortcut.vbs
echo sLinkFile = "%OUTPUT_DIR%\%APP_NAME%.lnk" >> CreateShortcut.vbs
echo Set oLink = oWS.CreateShortcut(sLinkFile) >> CreateShortcut.vbs
echo oLink.TargetPath = "%cd%\%OUTPUT_DIR%\malou-agent.exe" >> CreateShortcut.vbs
echo oLink.Save >> CreateShortcut.vbs
cscript CreateShortcut.vbs
del CreateShortcut.vbs

echo 创建自述文件...
echo %APP_NAME% v%VERSION% > "%OUTPUT_DIR%\README.txt"
echo ==================== >> "%OUTPUT_DIR%\README.txt"
echo 这是一个Windows桌面AI助手应用程序。 >> "%OUTPUT_DIR%\README.txt"
echo. >> "%OUTPUT_DIR%\README.txt"
echo 使用说明： >> "%OUTPUT_DIR%\README.txt"
echo 1. 双击 malou-agent.exe 启动应用 >> "%OUTPUT_DIR%\README.txt"
echo 2. 在左侧设置面板中配置OpenAI API密钥 >> "%OUTPUT_DIR%\README.txt"
echo 3. 在右侧聊天区域与AI助手对话 >> "%OUTPUT_DIR%\README.txt"

echo 打包完成！
echo 发布文件位于: %OUTPUT_DIR%
echo 包含文件：
dir "%OUTPUT_DIR%"
pause