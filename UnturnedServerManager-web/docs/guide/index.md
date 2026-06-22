# 项目介绍

Unturned Server Manager 是面向 Windows 的 Unturned 专用服务器桌面管理工具。它基于 Tauri v2、Svelte 5 和 Rust 构建，把开服、存档配置、本地命令、RCON、创意工坊、插件、权限、日志、更新和定时维护集中到一个便携面板里。

## 适合谁

<div class="doc-card-grid">
  <div class="doc-card"><h3>个人服主</h3><p>希望快速开服、少碰命令行，同时保留必要配置能力。</p></div>
  <div class="doc-card"><h3>小型社区</h3><p>需要长期运行、定时重启、工作坊维护和日志排障。</p></div>
  <div class="doc-card"><h3>Windows 服务器</h3><p>需要便携部署，不想额外开放 Web 管理端口。</p></div>
</div>

## 核心能力

| 模块 | 能力 |
| --- | --- |
| 仪表盘 | 状态、PID、运行时间、系统资源、多服务器卡片和快捷操作 |
| 服务器控制 | 启动、停止、重启、本地命令 Bridge、实时输出和日志搜索 |
| 存档配置 | 基础配置、高级 Config.txt、工作坊、插件、权限和 RCON |
| RCON 控制台 | 作为额外远程功能连接 Rocket RCON 并发送命令 |
| 日志中心 | 软件日志、操作日志、游戏日志的日期切换、筛选和搜索 |
| 自动维护 | SteamCMD 更新、工作坊更新监控、定时重启和提前提醒 |
| 窗口管理 | 最小化到托盘、关闭确认、托盘菜单和退出保护 |

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面框架 | Tauri v2 |
| 前端 | Svelte 5、TypeScript、Tailwind CSS v4 |
| 后端 | Rust、Tauri commands |
| 构建 | Vite、pnpm、Cargo |
| 系统能力 | WebView2、SteamCMD、Rocket.Unturned、本地命令 Bridge |

## 系统支持

| 系统版本 | 支持情况 | 说明 |
| --- | --- | --- |
| Windows 11 | 支持 | 自带 WebView2，可直接运行 |
| Windows 10 21H2+ | 支持 | 自带 WebView2 |
| Windows 10 1803-21H1 | 支持 | 需安装 WebView2 Runtime |
| Windows Server 2022 | 支持 | 可直接运行 |
| Windows Server 2016/2019 | 支持 | 需安装 WebView2 Runtime |
| Windows 7/8/8.1 | 不支持 | WebView2 与运行环境不满足 |

::: tip 提示
macOS 与 Linux 当前未提供预构建包，需要自行编译。
:::
