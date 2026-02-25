# Malou Agent Desktop - 开发环境配置指南

## 系统要求

- Windows 10/11
- Node.js 18+
- Rust 1.70+
- npm 或 yarn 包管理器

## 开发环境搭建

### 1. 安装Node.js依赖
```bash
npm install
```

### 2. 安装Rust工具链
```bash
# 如果未安装Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装tauri-cli
cargo install tauri-cli
```

### 3. 启动开发环境
```bash
# 启动前端开发服务器
npm run dev

# 在新终端中启动桌面应用
cd src-tauri && cargo tauri dev
```

## 项目配置

### 环境变量
创建 `.env` 文件：
```env
VITE_API_BASE_URL=http://localhost:3002
VITE_APP_NAME=Malou Agent
```

### 数据库配置
SQLite数据库文件将自动创建在用户数据目录中。

## 调试技巧

### 前端调试
- 使用浏览器开发者工具
- Vue DevTools插件支持
- Element Plus组件调试

### 后端调试
- 查看控制台日志输出
- 使用 `log::debug!` 宏添加调试信息
- 数据库文件位置：`%APPDATA%/malou-agent/database.sqlite`

## 常见问题解决

### 编译错误
```bash
# 清理缓存
npm run clean
cargo clean

# 重新安装依赖
rm -rf node_modules
npm install
```

### 数据库问题
```bash
# 删除数据库文件重新初始化
del %APPDATA%\malou-agent\database.sqlite
```

### Tauri相关问题
```bash
# 更新tauri-cli
cargo install tauri-cli --force
```