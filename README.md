# Unturned Server Manager

<div align="center">

![Unturned Server Manager](docs/images/hero.png)

**轻量、便携、面向长期运行的 Unturned 专用服务器桌面管理工具**

基于 **Tauri v2 + Svelte 5 + Rust** 构建，把服务端启动、本地命令控制、RCON、存档、创意工坊模组、插件、权限、更新、日志和定时任务集中到一个现代化 Windows 面板里。

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?style=for-the-badge&logo=tauri&logoColor=white)
![Svelte](https://img.shields.io/badge/Svelte-5-FF3E00?style=for-the-badge&logo=svelte&logoColor=white)
![Rust](https://img.shields.io/badge/Rust-release-B7410E?style=for-the-badge&logo=rust&logoColor=white)
![Windows](https://img.shields.io/badge/Windows-portable-0078D4?style=for-the-badge&logo=windows&logoColor=white)

</div>

## 界面预览

### 仪表盘

<p align="center">
  <img src="docs/images/dashboard.png" width="90%" alt="仪表盘界面" />
</p>

### 核心工作流

<table>
  <tr>
    <td width="50%"><img src="docs/images/server.png" alt="服务器控制界面" /></td>
    <td width="50%"><img src="docs/images/save.png" alt="存档配置界面" /></td>
  </tr>
  <tr>
    <td align="center">服务器控制</td>
    <td align="center">存档配置</td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/images/workshop.png" alt="创意工坊模组界面" /></td>
    <td width="50%"><img src="docs/images/rcon.png" alt="RCON 控制台界面" /></td>
  </tr>
  <tr>
    <td align="center">创意工坊模组</td>
    <td align="center">RCON 控制台</td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/images/logs.png" alt="日志中心界面" /></td>
    <td width="50%"><img src="docs/images/settings.png" alt="设置界面" /></td>
  </tr>
  <tr>
    <td align="center">日志中心</td>
    <td align="center">设置与窗口管理</td>
  </tr>
</table>

### 首次引导

<table>
  <tr>
    <td width="70%"><img src="docs/images/desktop-wizard.png" alt="桌面端首次引导" /></td>
    <td width="30%"><img src="docs/images/mobile-wizard.png" alt="窄屏首次引导" /></td>
  </tr>
  <tr>
    <td align="center">桌面端引导</td>
    <td align="center">窄屏响应式引导</td>
  </tr>
</table>

## 功能亮点

| 模块 | 能力 |
| --- | --- |
| 仪表盘 | 查看服务器状态、PID、运行时间、CPU、内存、网络流量、多服务器运行卡片，并可快速启停 |
| 服务器控制 | 一键启动、停止、重启、强制停止，支持本地命令输入、实时输出、日志搜索和运行模式切换 |
| 本地命令 Bridge | 启停、重启和定时任务主路径，不依赖公网 RCON |
| RCON 控制台 | 作为额外远程能力连接 Rocket RCON、发送命令并轮询响应 |
| 存档配置 | 标签页工作台：基础配置、高级 Config.txt、工作坊、插件、权限和 RCON |
| 创意工坊模组 | 维护 `WorkshopDownloadConfig.json`，管理模组 ID、备注、缓存下载和更新监控 |
| 插件管理 | 查看 Rocket 插件目录，保存插件备注，快速打开插件配置目录 |
| 权限管理 | 编辑 `Permissions.config.xml`，管理权限组、SteamID64、继承关系和权限节点 |
| 日志中心 | 查看软件日志、操作日志和游戏日志，支持日期切换、分类筛选和搜索 |
| 服务端更新 | 调用 SteamCMD 更新 Unturned 服务端，显示自更新、下载和校验输出 |
| 定时任务 | 创建每日、每周、间隔型自动重启任务，支持提前提醒 |
| 窗口管理 | 支持最小化到托盘、托盘菜单、关闭行为记忆和运行服务器退出保护 |

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面框架 | Tauri v2 |
| 前端 | Svelte 5、TypeScript、Tailwind CSS v4 |
| 后端 | Rust、Tauri commands |
| 构建 | Vite、pnpm、Cargo |
| 系统能力 | WebView2、SteamCMD、Rocket.Unturned、本地命令 Bridge |
| 文档站 | VitePress |

## 系统支持

| 系统版本 | 便携版 | 说明 |
| --- | --- | --- |
| Windows 11 | 支持 | 自带 WebView2，可直接运行 |
| Windows 10 21H2+ | 支持 | 自带 WebView2，可直接运行 |
| Windows 10 1803 - 21H1 | 支持 | 需要安装 WebView2 Runtime |
| Windows Server 2022 | 支持 | 可直接运行 |
| Windows Server 2016/2019 | 支持 | 需要安装 WebView2 Runtime |
| Windows 7/8/8.1 | 不支持 | WebView2 与运行环境不满足 |

macOS 与 Linux 当前未提供预构建包，需要自行编译。

## 快速开始

```bash
pnpm install
pnpm tauri dev
```

生产构建：

```bash
pnpm tauri build
```

构建完成后会生成：

```text
src-tauri/target/release/unturned-server-manager.exe
src-tauri/target/release/bundle/nsis/Unturned Server Manager_3.0.0_x64-setup.exe
```

## 便携版

直接运行下面这个文件即可，无需安装：

```text
src-tauri/target/release/unturned-server-manager.exe
```

程序会在运行目录下自动创建统一运行数据目录：

```text
UnturnedServerManagerData/
  config/      应用配置、服务器配置、定时任务、备注数据
  logs/        软件日志、操作日志、游戏日志
  data/        运行数据
  backups/     备份数据
```

## 公网安全

打包后的桌面管理界面不会启动可被公网访问的 Web 服务。

需要区分的是 Unturned 游戏端口、Rocket RCON 端口和本地命令 Bridge：游戏端口通常需要按开服需求放行；RCON 是额外远程管理端口，不建议对公网开放；本地命令 Bridge 通过服务器存档目录内的本地队列文件工作，不监听公网端口。

## 开发要求

| 工具 | 建议版本 |
| --- | --- |
| Node.js | 22 或更高 |
| pnpm | 11 或更高 |
| Rust | 1.77.2 或更高 |
| Visual Studio Build Tools | C++ 桌面开发工作负载 |
| WebView2 Runtime | 旧版 Windows 需要手动安装 |

## 常见问题

### PowerShell 脚本被阻止

```powershell
pnpm.cmd tauri build
```

### 服务器启动失败

先到设置页运行环境检测，再检查 SteamCMD、服务端目录、Rocket.Unturned、Bridge DLL、端口冲突和日志中心。

### RCON 无法连接

确认目标存档正在运行，并在「存档 > 基础配置」检查 RCON 端口和密码。RCON 不建议对公网开放。

## 文档站

文档站位于 `UnturnedServerManager-web`：

```bash
cd UnturnedServerManager-web
pnpm install
pnpm dev
```

## 相关链接

- [Unturned 官方服务器文档](https://docs.smartlydressedgames.com/en/stable/servers/)
- [Unturned Wiki Commands](https://unturned.wiki.gg/Commands)
- [Tauri v2 文档](https://v2.tauri.app/)
- [Svelte 文档](https://svelte.dev/)
