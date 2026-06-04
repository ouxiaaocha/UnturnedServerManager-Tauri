# 项目介绍

**Unturned Server Manager** 是一款清新、轻量、便携的 Unturned 专用服务器管理工具。

基于 **Tauri v2 + Svelte 5 + Rust** 构建，把服务端启动、本地命令控制、RCON、存档、创意工坊模组、插件、更新、日志和定时任务集中到一个现代化桌面面板里。

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面框架 | Tauri v2 |
| 前端 | Svelte 5、TypeScript、Tailwind CSS v4 |
| 后端 | Rust、Tauri commands |
| 构建 | Vite、pnpm、Cargo |
| 系统能力 | WebView2、sysinfo、reqwest、zip |

## 功能亮点

| 模块 | 能力 |
| --- | --- |
| 仪表盘 | 查看服务器状态、PID、运行时间、CPU、内存、网络流量，并可通过本地命令快速启动、停止、重启服务器 |
| 服务器控制 | 一键启动、停止、重启、强制停止，支持本地命令输入、实时输出、日志搜索、局域网/互联网模式切换 |
| RCON 控制台 | 作为额外远程功能连接 Rocket RCON、发送命令、轮询响应，服务器启动后可自动连接 |
| 存档配置 | 管理 Commands.dat、Rocket RCON、PvE、作弊、GSLT、地图、端口和最大玩家数 |
| 创意工坊模组 | 维护 WorkshopDownloadConfig.json，管理模组 ID、备注、缓存下载、更新监控和关服提示 |
| 插件管理 | 查看 Rocket 插件目录，保存插件备注，快速打开插件配置目录 |
| 日志中心 | 查看软件日志、操作日志和游戏日志，支持日期切换、分类筛选和搜索 |
| 服务端更新 | 调用 SteamCMD 更新 Unturned 服务端，显示 SteamCMD 自更新和服务端校验输出 |
| 定时任务 | 创建每日、每周、间隔型自动重启任务，支持提前提醒 |
| 首次引导 | 自动检测/下载 SteamCMD，安装 Rocket 模块，初始化存档、RCON 和本地命令 Bridge |

## 系统支持

| 系统版本 | 支持情况 | 说明 |
| --- | --- | --- |
| Windows 11 | ✅ 支持 | 自带 WebView2，可直接运行 |
| Windows 10 21H2+ | ✅ 支持 | 自带 WebView2，可直接运行 |
| Windows 10 1803 - 21H1 | ✅ 支持 | 需要安装 WebView2 Runtime |
| Windows Server 2022 | ✅ 支持 | 可直接运行 |
| Windows Server 2016/2019 | ✅ 支持 | 需要安装 WebView2 Runtime |
| Windows 7/8/8.1 | ❌ 不支持 | WebView2 与运行环境不满足要求 |

::: tip 提示
macOS 与 Linux 当前未提供预构建包，需要自行编译。
:::

## 界面预览

### 仪表盘

<p align="center">
  <img src="/dashboard.png" width="90%" alt="仪表盘界面" />
</p>

### 核心工作流

<table>
  <tr>
    <td width="50%"><img src="/server.png" alt="服务器控制界面" /></td>
    <td width="50%"><img src="/rcon.png" alt="RCON 控制台界面" /></td>
  </tr>
  <tr>
    <td align="center">服务器控制</td>
    <td align="center">RCON 控制台</td>
  </tr>
  <tr>
    <td width="50%"><img src="/save.png" alt="存档配置界面" /></td>
    <td width="50%"><img src="/workshop.png" alt="创意工坊模组配置界面" /></td>
  </tr>
  <tr>
    <td align="center">存档配置</td>
    <td align="center">创意工坊模组</td>
  </tr>
</table>
