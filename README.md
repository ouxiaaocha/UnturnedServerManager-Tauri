# Unturned Server Manager

<div align="center">

![Unturned Server Manager](docs/images/hero.png)

**清新、轻量、便携的 Unturned 专用服务器管理工具**

基于 **Tauri v2 + Svelte 5 + Rust** 构建，把服务端启动、本地命令控制、RCON、存档、创意工坊模组、插件、更新、日志和定时任务集中到一个现代化桌面面板里。

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
    <td width="50%"><img src="docs/images/rcon.png" alt="RCON 控制台界面" /></td>
  </tr>
  <tr>
    <td align="center">服务器控制</td>
    <td align="center">RCON 控制台</td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/images/save.png" alt="存档配置界面" /></td>
    <td width="50%"><img src="docs/images/workshop.png" alt="创意工坊模组配置界面" /></td>
  </tr>
  <tr>
    <td align="center">存档配置</td>
    <td align="center">创意工坊模组</td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/images/schedule.png" alt="定时任务界面" /></td>
    <td width="50%"><img src="docs/images/update.png" alt="服务端更新界面" /></td>
  </tr>
  <tr>
    <td align="center">定时任务</td>
    <td align="center">服务端更新</td>
  </tr>
  <tr>
    <td width="50%"><img src="docs/images/logs.png" alt="日志中心界面" /></td>
    <td width="50%"><img src="docs/images/settings.png" alt="设置界面" /></td>
  </tr>
  <tr>
    <td align="center">日志中心</td>
    <td align="center">设置</td>
  </tr>
</table>

### 首次引导

<table>
  <tr>
    <td width="70%"><img src="docs/images/desktop-wizard.png" alt="桌面端首次引导" /></td>
    <td width="30%"><img src="docs/images/mobile-wizard.png" alt="移动端首次引导" /></td>
  </tr>
  <tr>
    <td align="center">桌面端引导</td>
    <td align="center">窄屏响应式引导</td>
  </tr>
</table>

## 功能亮点

| 模块 | 能力 |
| --- | --- |
| 仪表盘 | 查看服务器状态、PID、运行时间、CPU、内存、网络流量，并可通过本地命令快速启动、停止、重启服务器 |
| 服务器控制 | 一键启动、停止、重启、强制停止，支持本地命令输入、实时输出、日志搜索、局域网/互联网模式切换 |
| RCON 控制台 | 作为额外远程功能连接 Rocket RCON、发送命令、轮询响应，优化的连接管理和心跳保活机制 |
| 存档配置 | 标签页布局，管理 `Commands.dat`、Rocket RCON、PvE、作弊、GSLT、地图、端口和最大玩家数 |
| 创意工坊模组 | 维护 `WorkshopDownloadConfig.json`，管理模组 ID、备注、缓存下载、更新监控，快速跳转 Steam 工作坊 |
| 插件管理 | 查看 Rocket 插件目录，保存插件备注，快速打开插件配置目录 |
| 权限管理 | 编辑 `Permissions.config.xml`，管理权限组和 SteamID 权限配置 |
| 日志中心 | 查看软件日志、操作日志和游戏日志，支持日期切换、分类筛选和搜索 |
| 服务端更新 | 调用 SteamCMD 更新 Unturned 服务端，显示 SteamCMD 自更新和服务端校验输出 |
| 定时任务 | 创建每日、每周、间隔型自动重启任务，支持提前提醒，智能清理过期任务 |
| 窗口管理 | 支持最小化到托盘、托盘菜单快速导航、关闭行为自定义 |
| 首次引导 | 自动检测/下载 SteamCMD，安装 Rocket 模块，初始化存档、RCON 和本地命令 Bridge |

## 最近更新

### v2.2.0 (2026-06-15)

**🐛 稳定性改进**
- 修复长时间运行内存泄露问题，可 7×24 小时稳定运行
  - 修复前端 Tauri 事件监听器泄露（App.svelte）
  - 修复页面切换时定时器未清理（Save.svelte、Update.svelte）
  - 优化 RCON 响应缓冲区，降低内存峰值 50%
  - 调度器自动清理孤儿任务记录
- 日志保留天数优化：默认从 30 天降至 15 天，减少磁盘占用

**🎨 UI/UX 优化**
- 新增通用 UI 组件库：Button、Card、Select、SelectCustom
- 重构存档管理页面，采用标签页布局
  - 基础配置 | 插件管理 | 工作坊 | 权限管理 | RCON 配置
- 新增工作坊模组管理：模组列表、Steam 工作坊跳转、模组备注
- 新增权限管理界面：权限列表编辑、SteamID 权限配置
- 优化所有页面的布局、样式和交互体验

**⚙️ 新增功能**
- 窗口管理：最小化到托盘、托盘菜单、关闭行为自定义
- 托盘菜单：快速导航、自动托管切换、窗口显示/隐藏
- 关闭确认对话框：支持记住用户选择

**🚀 性能优化**
- 优化 RCON 缓冲区：上限从 1000 降至 500 条，批次从 500 降至 250
- 单条响应长度限制 1000 字符，防止超长日志占用内存
- 优化轮询机制的生命周期管理
- 改进异步任务调度和错误处理

**🔒 安全增强**
- 增强日志文件路径验证，防止路径穿越攻击
- 优化子进程环境变量清理，防止敏感信息泄露

### v2.1.0

- v2.1.0 将服务器控制主路径切换为本地命令 Bridge，启动、停止、重启和定时提醒不再依赖 RCON。
- 服务器页面新增本地命令输入栏，命令会写入本地队列并由 Rocket Bridge 插件在服务器内执行。
- 设置页新增运行环境检测与修复入口，可检查 SteamCMD、服务端目录、Rocket.Unturned 和 Bridge DLL。
- 更新检测会同时显示本地版本与云端版本，GitHub Release 日志安全渲染。
- 优化 SteamCMD 更新流程、自动重启日志续接、启动期间页面卡顿和重启延时问题。

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 桌面框架 | Tauri v2 |
| 前端 | Svelte 5、TypeScript、Tailwind CSS v4 |
| 后端 | Rust、Tauri commands |
| 构建 | Vite、pnpm、Cargo |
| 系统能力 | WebView2、sysinfo、reqwest、zip |

## 系统支持

| 系统版本 | 便携版 | 说明 |
| --- | --- | --- |
| Windows 11 | 支持 | 自带 WebView2，可直接运行 |
| Windows 10 21H2+ | 支持 | 自带 WebView2，可直接运行 |
| Windows 10 1803 - 21H1 | 支持 | 需要安装 WebView2 Runtime |
| Windows Server 2022 | 支持 | 可直接运行 |
| Windows Server 2016/2019 | 支持 | 需要安装 WebView2 Runtime |
| Windows 7/8/8.1 | 不支持 | WebView2 与运行环境不满足要求 |

> macOS 与 Linux 当前未提供预构建包，需要自行编译。

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
src-tauri/target/release/bundle/nsis/Unturned Server Manager_2.1.0_x64-setup.exe
```

## 便携版

直接运行下面这个文件即可，无需安装：

```text
src-tauri/target/release/unturned-server-manager.exe
```

程序会在运行目录下自动创建运行数据：

```text
config/      应用配置、服务器配置、定时任务、备注数据
logs/        应用日志、操作日志、游戏日志
data/        运行数据
backups/     备份数据
```

## 公网安全

打包后的桌面管理界面不会启动可被公网访问的 Web 服务，别人不能通过 `服务器IP:1420` 打开这个软件界面。开发模式和预览模式固定监听 `127.0.0.1`，只允许本机访问。

需要区分的是 Unturned 游戏端口、Rocket RCON 端口和本地命令 Bridge：游戏端口通常需要按开服需求放行；RCON 是额外远程管理端口，不建议对公网开放；本地命令 Bridge 通过服务器存档目录内的本地队列文件工作，不监听公网端口。建议在 Windows 防火墙或云服务器安全组中只放行游戏端口，阻止 RCON 端口的公网入站访问，并使用强随机 RCON 密码。

## 项目结构

```text
src/
  App.svelte              主壳层、窗口栏、导航与托盘事件
  app.css                 全局主题、CSS 变量与响应式样式
  lib/components/
    Button.svelte         通用按钮组件
    Card.svelte           卡片容器组件
    Select.svelte         原生下拉选择组件
    SelectCustom.svelte   自定义下拉选择组件
    CloseConfirmDialog    关闭确认对话框
    LogPanel.svelte       日志面板组件
    SaveSelector.svelte   存档选择器组件
    Toast.svelte          通知提示组件
  lib/pages/
    Dashboard.svelte      仪表盘
    Server.svelte         服务器控制
    Rcon.svelte           RCON 控制台
    Save.svelte           存档、创意工坊模组、插件与权限
    Schedule.svelte       定时任务
    Update.svelte         服务端更新
    Logs.svelte           日志中心
    Settings.svelte       设置
    Wizard.svelte         首次引导
    About.svelte          关于
  lib/stores.svelte.ts    全局状态管理
  lib/utils.ts            工具函数
  lib/utils/
    polling.svelte.ts     轮询工具类
    composables.svelte.ts 组合式函数（useClickOutside）

src-tauri/
  src/commands/
    server.rs             服务器控制命令
    rcon.rs               RCON 命令
    save.rs               存档配置命令
    logs.rs               日志命令
    schedule.rs           定时任务命令
    update.rs             更新命令
    system.rs             系统监控命令
    config.rs             配置命令
    window.rs             窗口管理命令
  src/services/
    config_service.rs     配置服务
    log_service.rs        日志服务
    process.rs            进程管理服务
    rcon_client.rs        RCON 客户端服务
    scheduler.rs          调度服务
    system_monitor.rs     系统监控服务
    local_command_bridge.rs 本地命令桥接
  src/models/             数据模型
  tauri.conf.json         Tauri 应用配置
```

## 开发要求

| 工具 | 建议版本 |
| --- | --- |
| Node.js | 22 或更高 |
| pnpm | 11 或更高 |
| Rust | 1.77.2 或更高 |
| Visual Studio Build Tools | C++ 桌面开发工作负载 |
| WebView2 Runtime | 旧版 Windows 需要手动安装 |

## 常见问题

### 构建提示 PowerShell 脚本被阻止

Windows 执行策略可能拦截 `pnpm.ps1`。可以使用：

```powershell
pnpm.cmd tauri build
```

### 旧版 Windows 打不开程序

请安装 Microsoft WebView2 Runtime，并确保已安装 Visual C++ Redistributable。

### 首次编译较慢

Rust release 构建会下载并编译依赖，首次耗时较长；后续构建通常会快很多。

## 相关链接

- [Unturned 官方服务器文档](https://docs.smartlydressedgames.com/en/stable/servers/)
- [Unturned Wiki Commands](https://unturned.wiki.gg/Commands)
- [Tauri v2 文档](https://v2.tauri.app/)
- [Svelte 文档](https://svelte.dev/)
