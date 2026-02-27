---
name: malou-agent-fix-plan
version: 2.0.0
overview: 修复 Malou Agent Desktop 核心问题：布局优化、API调用修复、日志系统、Mac风格UI、记忆系统优化
status: completed
priority: high
dateCreated: 2025-02-25
dateUpdated: 2025-02-26
design:
  architecture:
    framework: vue
    backend: rust
    database: sqlite
  styleKeywords:
    - Glassmorphism
    - MacOS Style
    - Modern Minimal
    - Rounded Corners
  fontSystem:
    fontFamily: PingFang-SC, -apple-system, BlinkMacSystemFont, Segoe UI
    heading:
      size: 20px
      weight: 600
    subheading:
      size: 16px
      weight: 500
    body:
      size: 14px
      weight: 400
  colorSystem:
    primary:
      - "#007AFF"
      - "#5856D6"
      - "#34C759"
    background:
      - "#F5F5F7"
      - "#FFFFFF"
      - "#FEFEFF"
    text:
      - "#1D1D1F"
      - "#86868B"
      - "#FFFFFF"
    functional:
      - "#FF3B30"
      - "#34C759"
      - "#FF9500"
memory:
  autoLoad: true
  priority: high
  tags: ["fix", "optimization", "ui", "backend"]
todos:
  - id: fix-config-sync
    content: 修复后端配置同步问题：从ConfigManager加载配置初始化AppState.openai_config
    status: completed
    impact: critical
    effort: medium
  - id: enhance-backend-logging
    content: 增强后端日志：在main.rs初始化logger，关键函数添加日志
    status: completed
    dependencies:
      - fix-config-sync
    impact: high
    effort: low
  - id: fix-frontend-layout
    content: 修复前端布局：移除el-container，使用flex布局
    status: completed
    impact: high
    effort: medium
  - id: redesign-mac-ui
    content: 重构UI为Mac风格：重写App.vue和组件样式
    status: completed
    dependencies:
      - fix-frontend-layout
    impact: medium
    effort: high
  - id: optimize-memory-system
    content: 优化agent记忆系统：创建知识库索引、标准化记忆结构
    status: completed
    impact: high
    effort: medium
---

## 需求概述

修复4个问题：1) 布局问题 2) 发送消息失败 3) 后端增加日志 4) Mac风格UI

## 核心功能

1. 布局修复：修复App.vue中使用el-container导致的布局问题，确保聊天界面正常显示
2. 发送消息失败修复：根因是AppState.openai_config启动时为空(api_key:"")，用户配置保存在ConfigManager中，但send_message读取AppState.openai_config，导致API调用失败
3. 后端日志增强：在关键位置添加日志记录，便于问题排查
4. Mac风格UI：重设计UI为macOS风格（毛玻璃效果、圆角、现代配色）

## 技术栈

- 后端：Rust + Tauri 2.0
- 前端：Vue 3 + TypeScript + Element Plus
- 日志：log + env_logger (已有)

## 问题根因分析

### 问题2根因

- main.rs中AppState.openai_config初始化为空(api_key: "")
- save_openai_config命令只更新内存中的AppState，未与ConfigManager同步
- send_message从AppState.openai_config读取配置，导致API调用失败

## 技术方案

### 修复2：配置同步

1. 在main.rs启动时，从ConfigManager加载已保存的远程模型配置
2. 将配置同步到AppState.openai_config
3. save_openai_config需要持久化配置到ConfigManager

### 修复3：日志增强

1. 初始化env_logger日志系统
2. 在关键路径添加日志：数据库操作、API调用、配置加载

### 修复4：Mac风格UI

- 使用CSS实现毛玻璃效果(backdrop-filter: blur)
- 统一圆角风格(border-radius: 12px+)
- 采用macOS配色(浅灰背景#f5f5f7，深色文字#1d1d1f)
- 添加阴影和层次感

## 设计风格

采用macOS Sonoma风格设计，打造现代化桌面应用体验。使用毛玻璃效果、柔和阴影和精致圆角，呈现清新优雅的视觉效果。

## 设计内容

1. 顶部导航栏：使用毛玻璃效果，固定高度60px，包含应用标题和标签页
2. 左侧会话列表：宽度280px，浅灰色背景，圆角卡片式会话项
3. 右侧聊天区域：白色背景，消息气泡使用圆角和阴影
4. 输入区域：悬浮式设计，底部固定，带圆角

## 页面规划

- 聊天主页面（包含侧边栏+聊天窗口）
- 知识库页面
- 设置页面