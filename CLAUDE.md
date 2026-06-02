# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 语言要求

- 所有对话和文档都使用中文
- 代码注释使用中文
- 错误提示使用中文

## 项目概述

ACE 小助手是一款 Windows 桌面工具，用于优化 ACE（腾讯反作弊）相关进程的优先级、CPU 亲和性、磁盘 I/O 优先级和效率模式。

## 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **桌面框架**: Tauri 2.0（使用 NSIS 打包）
- **后端**: Rust（通过 `winapi` crate 调用 Windows API）
- **目标平台**: Windows 10/11 x86_64

## 常用命令

```bash
# 安装依赖
npm install

# 前端开发（仅 Vite，不含 Tauri）
npm run dev

# Tauri 开发模式（前端 + Rust 后端）
npm run tauri dev

# 构建生产版本
npm run tauri build

# TypeScript 类型检查
vue-tsc -b
```

## 架构

### 前后端通信

前端通过 `@tauri-apps/api` 的 `invoke()` 调用 Rust 后端命令，`src-tauri/src/lib.rs` 中用 `#[tauri::command]` 注册的函数即为可调用的后端命令。数据通过 `serde` 序列化/反序列化在前后端之间传输，Rust 结构体使用 `#[serde(rename_all = "camelCase")]` 与 TypeScript 驼峰命名对齐。

### 前端结构 (`src/`)

- `views/` — 页面组件：`home/`、`optimization/`、`settings/`、`about/`
- `components/` — 通用组件（`ProcessCard`、`Sidebar`、`TopBar` 等）和 `icons/` 图标组件
- `composables/` — Vue 组合式函数：`useProcess`（进程操作）、`useSettings`（设置管理）、`useAutoMonitor`（自动监听）、`useLogs`（日志）
- `types/index.ts` — 前后端共享的 TypeScript 类型定义
- `config/` — 配置常量（`priority.ts`、`ioPriority.ts`）
- `utils/configStorage.ts` — 配置持久化工具

### 后端结构 (`src-tauri/src/`)

- `main.rs` — 入口，处理管理员权限提升（UAC 被拒绝时静默退出）和全局互斥锁防多实例
- `lib.rs` — 核心逻辑，包含所有 `#[tauri::command]` 命令：进程检测、优先级设置、CPU 亲和性、磁盘 I/O 优先级、效率模式、系统托盘等

### 进程状态模型

后端 `ProcessState` 枚举：`Offline(1)` → `Online(2)` → `Optimized(4)` / `Failed(3)`

前端 `FrontendProcessState`：`offline` | `online` | `scanning` | `optimizing` | `optimized` | `failed`

### 关键 Tauri 插件

- `tauri-plugin-updater` — 自动更新（从 GitHub Releases 拉取）
- `tauri-plugin-autostart` — 开机自启
- `tauri-plugin-notification` — 系统通知
- `tauri-plugin-fs` — 文件系统访问

## 注意事项

- Release 模式下 `main.rs` 会自动请求管理员权限（通过 `ShellExecuteW` + `runas`），Debug 模式跳过
- 效率模式功能需要 Windows 11 或 Windows 10 21H2+（Build 19044+），后端会自动检测
- 版本号需同时更新 `package.json`、`src-tauri/Cargo.toml`、`src-tauri/tauri.conf.json` 三处
- NSIS 安装钩子位于 `src-tauri/nsis/hooks.nsi`，用于安装前检测运行中实例
