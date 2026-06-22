# 快速开始

## 开发环境要求

| 工具 | 建议版本 |
| --- | --- |
| Node.js | 22 或更高 |
| pnpm | 11 或更高 |
| Rust | 1.77.2 或更高 |
| Visual Studio Build Tools | C++ 桌面开发工作负载 |
| WebView2 Runtime | 旧版 Windows 需要手动安装 |

## 安装依赖

```bash
pnpm install
```

## 开发模式

```bash
pnpm tauri dev
```

开发模式会启动本地调试服务，并打开桌面窗口。

## 生产构建

```bash
pnpm tauri build
```

构建产物：

```text
src-tauri/target/release/unturned-server-manager.exe
src-tauri/target/release/bundle/nsis/Unturned Server Manager_2.1.0_x64-setup.exe
```

## 首次运行流程

1. 打开程序后进入首次引导。
2. 检测或安装 SteamCMD。
3. 安装 Unturned Dedicated Server。
4. 初始化 Rocket.Unturned 和 Bridge 插件。
5. 创建或选择存档，配置端口、地图和 RCON。
6. 在仪表盘或服务器页启动服务器。

## 项目结构

```text
src/
  App.svelte              主壳层、窗口栏、导航与托盘事件
  app.css                 全局主题与响应式样式
  lib/components/         通用组件
  lib/pages/              仪表盘、服务器、RCON、存档、日志等页面
  lib/stores.svelte.ts    全局状态与偏好
src-tauri/
  src/commands/           Tauri 命令
  src/services/           配置、日志、进程、RCON、调度服务
UnturnedServerManager-web/
  docs/                   VitePress 文档站
```
