# Malou Agent Desktop - Agent记忆系统

## 📋 系统概述

本目录包含Malou Agent Desktop项目的agent记忆和配置系统，用于确保项目信息能够快速准确地加载和处理。

## 📁 目录结构

```
.codebuddy/
├── README.md                    # 本文件
├── memory-config.json           # 记忆系统配置
├── QUICK_LOAD_GUIDE.md          # 快速加载指南
├── plans/                       # 项目计划和路线图
│   ├── malou-agent-fix-plan_92c9bbd6.md         # 已完成的问题修复计划
│   └── malou-agent-development-plan_20250226.md # 当前开发计划
├── contexts/                    # 项目上下文快照
│   └── project-snapshot-20250226.md             # 最新项目状态快照
└── rules/                       # 开发规则和规范
    └── tcb/                     # TCB相关规则
```

## 🔄 系统工作流程

### 1. 初始化加载
当agent开始处理项目时，自动加载：
- `memory-config.json` - 记忆系统配置
- `PROJECT_KNOWLEDGE_BASE.md` - 项目知识库索引
- `QUICK_LOAD_GUIDE.md` - 加载策略指南

### 2. 上下文加载
根据任务类型加载相应上下文：
- **开发任务**: 加载最新项目快照和开发计划
- **问题排查**: 加载相关代码和错误日志
- **代码审查**: 加载代码质量评估和优化建议

### 3. 记忆更新
在任务执行过程中：
- 自动更新项目快照状态
- 记录重要决策和变更
- 维护知识库的时效性

## 📊 记忆类型说明

### 计划记忆 (Plans)
- **位置**: `plans/` 目录
- **内容**: 项目开发计划、路线图、任务清单
- **保留策略**: 长期保留，支持版本管理
- **使用场景**: 项目规划、进度跟踪、任务分配

### 上下文记忆 (Contexts)
- **位置**: `contexts/` 目录
- **内容**: 当前项目状态、开发重点、技术挑战
- **保留策略**: 会话期间有效，定期快照
- **使用场景**: 快速理解当前开发状态

### 规则记忆 (Rules)
- **位置**: `rules/` 目录
- **内容**: 开发规范、最佳实践、技术标准
- **保留策略**: 长期保留，支持更新
- **使用场景**: 确保代码质量和技术一致性

## 🚀 快速开始

### 对于新agent
1. 首先阅读 `QUICK_LOAD_GUIDE.md` 了解加载策略
2. 查看 `memory-config.json` 理解记忆系统配置
3. 加载 `PROJECT_KNOWLEDGE_BASE.md` 获取项目概览
4. 根据任务类型选择加载相应的上下文

### 对于项目维护
1. 定期更新项目快照反映最新状态
2. 维护计划文件的时效性和准确性
3. 及时清理过时的记忆内容
4. 确保不同记忆类型间的一致性

## 🔧 配置说明

### 记忆系统配置 (`memory-config.json`)
```json
{
  "version": "1.0.0",
  "project": {
    "name": "malou-agent",
    "type": "desktop-application",
    "framework": "tauri-vue-rust"
  },
  "memory": {
    "structure": {
      "plans": "project-plans-and-roadmaps",
      "rules": "development-rules-and-guidelines",
      "contexts": "session-specific-contexts"
    },
    "autoLoad": ["plans", "rules"],
    "priority": {
      "high": ["current-plan", "active-rules"],
      "medium": ["project-structure", "dependencies"]
    }
  }
}
```

### 文件索引模式
系统会自动索引以下文件模式：
- `README.md`, `PROJECT_*.md`, `docs/*.md`
- `src/**/*.vue`, `src/**/*.ts`
- `src-tauri/src/**/*.rs`
- `package.json`, `Cargo.toml`

## 🎯 最佳实践

### 文档维护
- 保持文档的时效性和准确性
- 定期检查并更新过时信息
- 确保不同文档间的一致性

### 记忆优化
- 按功能模块组织记忆内容
- 避免记忆冗余和重复
- 定期清理无效的记忆

### 协作开发
- 建立统一的记忆更新流程
- 记录重要的开发决策
- 共享知识和经验

## 📞 技术支持

### 常见问题
- **记忆加载失败**: 检查配置文件和索引完整性
- **信息不一致**: 更新过时文档，统一信息源
- **性能问题**: 优化文件大小，配置合理加载策略

### 维护建议
- 每周检查记忆系统健康状态
- 每月进行记忆内容清理和优化
- 重要变更时更新相关文档

---

*本记忆系统将持续优化，为Malou Agent Desktop项目提供高效准确的信息支持。*