# Malou Agent Desktop 项目知识库索引

## 📋 项目概述

**项目名称**: Malou Agent Desktop  
**技术栈**: Tauri v2 + Vue 3 + TypeScript + Rust + SQLite  
**项目状态**: 核心架构已搭建完成，具备基础功能  
**最后更新**: 2025-02-26

## 🔧 技术架构

### 前端架构 (Vue 3 + TypeScript)
- **框架**: Vue 3 + TypeScript + Element Plus
- **构建工具**: Vite
- **状态管理**: Pinia
- **通信**: Tauri API

### 后端架构 (Rust + Tauri)
- **框架**: Tauri v2 + Rust
- **异步运行时**: Tokio
- **数据库**: SQLite
- **AI推理**: ONNX (tract库)

### 核心模块

| 模块 | 状态 | 完成度 | 描述 |
|------|------|--------|------|
| 聊天功能 | ✅ 完成 | 90% | 支持消息发送、实时显示、模型切换 |
| 文档管理 | ⚠️ 部分 | 70% | 文档CRUD操作和搜索功能 |
| 配置管理 | ✅ 完成 | 100% | 统一的配置管理系统 |
| 数据库层 | ✅ 完成 | 95% | SQLite数据库访问层 |
| ONNX推理 | ❌ 未激活 | 30% | 本地AI模型推理引擎 |
| 向量搜索 | ❌ 未实现 | 10% | ChromaDB向量数据库集成 |

## 📁 项目目录结构

### 项目本地结构
```
malou-agent/
├── .codebuddy/                 # 项目级Agent记忆系统
│   ├── plans/                  # 项目计划和任务管理
│   ├── rules/                  # 开发规则和规范
│   ├── contexts/               # 项目上下文快照
│   └── memory-config.json      # 项目记忆配置
├── src/                       # Vue前端代码
│   ├── components/            # 组件目录
│   │   └── business/          # 业务组件
│   ├── api/                   # API接口定义
│   └── App.vue                # 主应用组件
├── src-tauri/                 # Rust后端代码
│   ├── src/                   # Rust源码
│   │   ├── database/          # 数据库模块
│   │   ├── chat/              # 聊天服务模块
│   │   └── config/            # 配置管理模块
│   └── Cargo.toml             # Rust依赖配置
├── docs/                      # 技术文档
├── 项目文档.md                # 主要项目文档
└── 配置文件                   # 各种配置文件
```

### 项目记忆系统结构
```
malou-agent/
├── .ai-memory/                 # 项目专用AI记忆系统
│   ├── config.json            # 项目记忆配置
│   ├── shared/                # 共享知识库
│   │   ├── best-practices/    # 开发最佳实践
│   │   └── templates/         # 项目模板库
│   └── monitoring/            # 系统监控
├── .codebuddy/               # CodeBuddy专用配置
│   ├── plans/                 # 项目计划和任务管理
│   ├── rules/                 # 开发规则和规范
│   ├── contexts/              # 项目上下文快照
│   └── memory-config.json    # 记忆配置
└── 项目主体文件...
```

## 📊 项目状态快照

### 当前版本
- **前端**: Vue 3.4+ 正常运行
- **后端**: Rust编译存在依赖问题
- **数据库**: SQLite集成完成
- **UI**: 传统桌面风格，支持Mac风格优化

### 已知问题
1. **ONNX推理未激活** - 依赖问题待解决
2. **错误处理机制需要完善** - 统一错误处理机制
3. **向量搜索未实现** - 需要集成ChromaDB

### 近期目标
- 解决编译依赖问题
- 完善基础功能测试
- 优化UI体验

## 🔍 快速导航

### 核心文件
- `src/App.vue` - 主应用布局和导航
- `src/components/business/ChatView.vue` - 聊天界面
- `src-tauri/src/main.rs` - 后端入口
- `src-tauri/src/config/mod.rs` - 配置管理

### 配置管理
- `package.json` - 前端依赖
- `Cargo.toml` - 后端依赖
- `tauri.conf.json` - Tauri应用配置

### 文档索引
- `README.md` - 项目总览
- `PROJECT_SUMMARY.md` - 详细项目总结
- `PROGRESS_REPORT.md` - 开发进度
- `docs/` - 技术文档集合

### 记忆系统导航
- `.ai-memory/config.json` - 项目专用AI记忆配置
- `.ai-memory/shared/` - 共享知识库和最佳实践
- `.codebuddy/memory-config.json` - CodeBuddy专用配置
- `.codebuddy/plans/` - 项目计划和任务管理

## 🚀 开发指南

### 环境要求
- Node.js 18+
- Rust 1.70+
- Tauri CLI

### 快速开始
```bash
# 安装依赖
npm install

# 启动开发服务器
npm run tauri dev
```

### 构建发布
```bash
# 构建应用
npm run tauri build
```

### AI记忆系统支持
本项目内置专用AI记忆系统，支持多种AI工具的协同使用：

**项目特性**:
- **项目专用**: 专注于malou-agent项目的记忆管理
- **标准化**: 统一的记忆格式和访问接口
- **自动同步**: 项目状态和配置的自动记忆
- **智能管理**: 基于时间戳的冲突解决

**记忆管理**:
- 项目开发状态和进度跟踪
- 技术栈和依赖关系记忆
- 开发最佳实践和代码模板
- 性能优化和系统监控

## 📞 技术支持

### 常见问题
- **编译失败**: 检查依赖版本兼容性
- **前端问题**: 确保Vue开发服务器正常运行
- **错误处理**: 统一使用AppResult错误处理机制
- **数据库问题**: 验证SQLite连接配置

### 调试技巧
- 使用浏览器开发者工具调试前端
- 查看Rust编译错误信息
- 检查Tauri日志输出

---

*本知识库索引定期更新，确保项目信息的准确性和时效性。*