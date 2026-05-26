# ACE 小助手

> 蜡笔涂鸦 · 可爱但混乱

ACE 小助手是一款专为 ACE 进程优化的 Windows 桌面工具，帮助你轻松管理和优化 ACE 相关进程。

<img width="1251" height="976" alt="image" src="https://github.com/user-attachments/assets/998d0b35-1338-4f4f-84d5-2188beb6af40" />


## ✨ 功能特性

- 🏠 **首页** - 实时查看 ACE 进程状态，一键优化
- 🛠️ **进程优化** - 配置要监控的进程，设置优化优先级
- ⚙️ **通用设置** - 自定义应用设置，持久化保存
- ❓ **关于帮助** - 查看版本信息，检查更新

### 核心功能

- ✅ 自动监听进程状态（每 3 秒检测一次）
- ✅ 一键优化 ACE 进程优先级
- ✅ 智能权限提升（SeDebugPrivilege + PowerShell 回退，可处理内核级保护进程）
- ✅ **磁盘 I/O 优先级优化** - 支持设置进程磁盘 I/O 优先级（非常低/低/正常）
- ✅ **效率模式** - 启用 Windows 效率模式，降低后台进程资源占用（仅 Windows 11 支持）
- ✅ 安装前自动检测运行中实例，防止安装冲突
- ✅ 可爱涂鸦风格 UI 界面
- ✅ 自动更新支持

> **注意**：效率模式功能需要 Windows 11 或 Windows 10 21H2+（Build 19044+），Windows 10 早期版本将自动禁用此功能，其他优化功能仍可正常使用。

## 🔧 ACE_verify.bat 独立验证脚本

项目根目录提供了 `ACE_verify.bat` 独立验证脚本，可在不启动 ACE 小助手的情况下快速测试进程优化：

- 自动检测 SGuardSvc64.exe / SGuard64.exe 是否在运行
- 将优先级设为 Idle（最低）
- 将 CPU 亲和性锁定到最后一个核心
- 设置磁盘 I/O 优先级为 Very Low（非常低）
- **右键以管理员身份运行**即可，无需安装

> **注意**：独立脚本不支持设置效率模式，如需使用效率模式请使用 ACE 小助手主程序（Windows 11+）。

## 📋 系统兼容性

| 项目 | 要求 |
|------|------|
| 操作系统 | Windows 10 / Windows 11 |
| 架构 | x86_64 (64位) |
| 运行时 | 无需额外安装，开箱即用 |

## 🚀 启动命令

### 开发环境

```bash
# 安装依赖
npm install

# 启动开发服务器
npm run dev

# 启动 Tauri 开发模式
npm run tauri dev
```

### 构建发布

```bash
# 构建生产版本
npm run tauri build
```

### 测试更新检测

```bash
# 切换到测试配置（指向本地服务器）
node scripts/switch-config.js dev

# 启动本地更新服务器
node scripts/local-update-server.js

# 运行应用
npm run tauri dev

# 测试完成后切回生产配置
node scripts/switch-config.js prod
```

> 详细测试指南见 [UPDATE_TEST_GUIDE.md](./UPDATE_TEST_GUIDE.md)

## 📥 下载地址

前往 GitHub Releases 下载最新版本：

👉 **[https://github.com/NanCheng-L/ace-helper/releases](https://github.com/NanCheng-L/ace-helper/releases)**

下载 `.exe` 安装包，双击安装即可使用。

## 🛠️ 技术栈

- **前端**: Vue 3 + TypeScript + Vite
- **桌面框架**: Tauri 2.0
- **后端**: Rust
- **UI 风格**: 可爱涂鸦风

## 🙏 致谢

本项目在开发过程中参考了以下开源项目：

- **[FuckACE](https://github.com/shshouse/FuckACE)** - 提供了 ACE 进程优化的核心思路和实现参考

## ⚠️ 免责声明

**ACE 小助手** 是一款第三方开发的辅助工具，与腾讯公司（Tencent）或其 ACE 反作弊系统无任何官方关联。

- 本工具仅供学习和研究使用，请遵守相关软件的使用条款。
- 使用本工具修改进程优先级和 CPU 亲和性可能会影响游戏或应用程序的性能，请谨慎使用。
- 作者不对因使用本工具而导致的任何系统问题、游戏封号或其他损失承担责任。
- 请确保您有权对目标进程进行修改，某些在线游戏可能禁止此类操作。

## 📄 开源协议

MIT License

---

Made with ❤️ by NanCheng
