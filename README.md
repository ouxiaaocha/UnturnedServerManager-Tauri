# Unturned 服务器管理工具

基于 Tauri v2 + Svelte 5 开发的 Unturned 专用服务器管理工具，提供可视化的服务器管理、RCON 控制、存档管理、插件管理等功能。

## 系统支持

本软件基于 Tauri 2 构建，使用 WebView2 作为渲染引擎，支持以下 Windows 系统：

| 系统版本 | 便携版支持 | 说明 |
|---------|-----------|------|
| Windows 11 | ✅ | 全版本自带 WebView2，直接运行 |
| Windows 10 21H2 及以上 | ✅ | 自带 WebView2，直接运行 |
| Windows 10 1803 ~ 21H1 | ✅（需安装运行时） | 需先安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| Windows Server 2022 | ✅ | 自带 WebView2，直接运行 |
| Windows Server 2019 | ✅（需安装运行时） | 需先安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| Windows Server 2016 | ✅（需安装运行时） | 需先安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |
| Windows 10 1803 以下 | ❌ | 不支持 WebView2 |
| Windows Server 2012 R2 及以下 | ❌ | 不支持 WebView2 |
| Windows 7 / 8 / 8.1 | ❌ | 不支持 |

> macOS 和 Linux 暂未构建，如需支持请自行编译。

## 功能概览

- **仪表盘** — 服务器状态监控、CPU/内存/网络实时数据、一键启动/停止/重启
- **服务器控制** — 实时控制台输出、日志搜索过滤
- **RCON** — 远程控制台终端，支持发送任意命令
- **定时任务** — 定时重启服务器，支持每日/每周/间隔模式
- **存档管理** — 读写 Commands.dat 配置、管理插件、打开插件目录
- **更新** — 通过 SteamCMD 一键更新服务端，实时显示进度
- **引导设置** — 首次运行自动下载 SteamCMD 和服务端、自动安装 Rocket 模块、自动初始化存档

## 技术栈

| 层级 | 技术 |
|------|------|
| 前端 | Svelte 5 (runes) + Tailwind CSS v4 + TypeScript |
| 后端 | Rust + Tauri v2 |
| 构建 | Vite + pnpm |
| 系统监控 | sysinfo (CPU/内存/网络) |
| HTTP | reqwest (blocking) |
| 压缩 | zip crate |

## 环境要求

- **Node.js** >= 18
- **pnpm** >= 11
- **Rust** >= 1.77.2
- **Visual Studio Build Tools** (C++ 桌面开发工作负载)
- **WebView2** (Windows 10 21H2+ / Windows 11 自带，旧版需手动安装)

## 安装依赖

```bash
# 安装前端依赖
pnpm install

# Rust 依赖会在首次构建时自动下载
```

## 构建指南

> **重要：不同的构建命令会产生不同的输出，请根据需求选择。**

### 开发模式

```bash
pnpm tauri dev
```

- 启动 Vite 开发服务器 + Rust debug 编译
- 支持热重载（前端修改实时生效）
- Rust 修改需要重新编译
- 窗口标题带 `[DEBUG]` 标记

### 生产构建（推荐）

```bash
pnpm tauri build
```

执行过程：
1. `pnpm build` — Vite 生产构建，输出到 `dist/`
2. Rust release 编译 — 优化后的二进制文件
3. NSIS 打包 — 生成安装包

输出文件：
```
src-tauri/target/release/unturned-server-manager.exe           # 便携版可执行文件
src-tauri/target/release/bundle/nsis/xxx_x64-setup.exe       # NSIS 安装包
```

### 构建产物说明

| 文件 | 说明 | 用途 |
|------|------|------|
| `unturned-server-manager.exe` | 便携版主程序 | 直接运行，无需安装 |
| `xxx_x64-setup.exe` | NSIS 安装包 | 分发给其他用户安装 |

### 便携版使用

将 `src-tauri/target/release/unturned-server-manager.exe` 复制到任意目录即可运行。软件会在同目录下自动创建：
- `config/` — 配置文件
- `logs/` — 日志文件
- `data/` — 数据文件
- `backups/` — 备份文件

### 常见构建问题

**Q: 构建时提示 "Cannot find module rolldown/parseAst"**

A: 删除 `node_modules` 重新安装：
```bash
rm -rf node_modules
pnpm install
```

**Q: Rust 编译很慢**

A: 首次编译需要下载和编译所有依赖（约 5-10 分钟），后续编译只编译修改的部分（约 1 分钟）。

**Q: 构建出的 exe 无法运行**

A: 确保已安装 [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)。Windows 10 21H2 以下版本还需安装 [WebView2 Runtime](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)。

## 项目结构

```
├── src/                          # 前端源码
│   ├── lib/pages/                # 页面组件
│   │   ├── Dashboard.svelte      # 仪表盘
│   │   ├── Server.svelte         # 服务器控制
│   │   ├── Rcon.svelte           # RCON 终端
│   │   ├── Save.svelte           # 存档管理
│   │   ├── Schedule.svelte       # 定时任务
│   │   ├── Update.svelte         # 更新
│   │   ├── Logs.svelte           # 日志
│   │   ├── Settings.svelte       # 设置
│   │   ├── Wizard.svelte         # 首次引导
│   │   └── About.svelte          # 关于
│   ├── App.svelte                # 主应用（侧边栏路由）
│   ├── app.css                   # 全局样式
│   └── main.ts                   # 入口
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── commands/             # Tauri 命令
│   │   │   ├── server.rs         # 服务器控制
│   │   │   ├── rcon.rs           # RCON 操作
│   │   │   ├── config.rs         # 配置管理
│   │   │   ├── save.rs           # 存档/插件管理
│   │   │   ├── installer.rs      # SteamCMD/服务端下载
│   │   │   ├── setup.rs          # Rocket 模块/存档初始化
│   │   │   ├── system.rs         # 系统监控
│   │   │   ├── update.rs         # 服务端更新
│   │   │   ├── schedule.rs       # 定时任务
│   │   │   └── logs.rs           # 日志读取
│   │   ├── services/             # 业务服务
│   │   ├── models/               # 数据模型
│   │   ├── lib.rs                # Tauri 入口
│   │   └── main.rs               # 程序入口
│   ├── Cargo.toml                # Rust 依赖
│   └── tauri.conf.json           # Tauri 配置
├── package.json                  # 前端依赖
└── vite.config.ts                # Vite 配置
```

## 配置文件说明

软件运行时在同目录下生成配置：

```
config/
├── servers.json          # 服务器配置（路径、RCON 等）
├── appsettings.json      # 应用设置
├── plugin_notes.json     # 插件备注
└── schedules.json        # 定时任务
```

## 相关文档

- [Unturned 官方服务器文档](https://docs.smartlydressedgames.com/en/stable/servers/)
- [Unturned Wiki Commands](https://unturned.wiki.gg/Commands)
- [Tauri v2 文档](https://v2.tauri.app/)
- [Svelte 5 文档](https://svelte.dev/)
