# Malou Agent Desktop

一个基于Tauri + Vue 3的Windows桌面AI代理应用。

## 功能特性

- 🤖 智能聊天对话
- 📚 文档管理与检索
- 🧠 本地ONNX模型推理
- 🔍 向量数据库搜索
- ⚡ 插件化技能系统
- 🎨 传统桌面UI风格

## 技术栈

### 前端
- Vue 3 + TypeScript
- Element Plus UI组件库
- Vite 构建工具
- Pinia 状态管理

### 后端
- Tauri v2 桌面框架
- Rust 编程语言
- tract ONNX推理引擎
- SQLite 本地数据库
- ChromaDB 向量数据库

## 快速开始

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 构建生产版本
npm run build

# 启动桌面应用（需先安装tauri-cli）
cd src-tauri && cargo tauri dev
```

## 项目结构

```
malou-agent/
├── src/                    # 前端源码
│   ├── components/        # Vue组件
│   │   └── business/     # 业务组件
│   ├── api/              # API接口
│   └── App.vue           # 主应用组件
├── src-tauri/            # Rust后端
│   ├── src/             # Rust源码
│   │   ├── main.rs      # 应用入口
│   │   ├── database.rs  # 数据库模块
│   │   └── onnx.rs      # ONNX推理模块
│   └── Cargo.toml       # Rust依赖配置
└── docs/                # 文档目录
```

## 开发进度

- [x] 项目基础架构搭建
- [x] 聊天界面实现
- [x] 文档管理功能
- [x] 数据库访问层
- [ ] 本地计算切换
- [ ] 向量数据库集成
- [ ] 技能插件系统
- [ ] 配置管理界面

## 许可证

MIT License