# 快速开始

## 环境要求

| 工具 | 建议版本 |
| --- | --- |
| Node.js | 18 或更高 |
| pnpm | 11 或更高 |
| Rust | 1.77.2 或更高 |
| Visual Studio Build Tools | C++ 桌面开发工作负载 |
| WebView2 Runtime | 旧版 Windows 需要手动安装 |

## 安装依赖

```bash
pnpm install
```

## 开发模式

启动开发服务器，支持热重载：

```bash
pnpm tauri dev
```

::: tip 提示
开发模式下，前端页面会在 `http://127.0.0.1:1420` 启动，仅允许本机访问。
:::

## 生产构建

构建可执行文件和安装包：

```bash
pnpm tauri build
```

构建完成后会生成：

```text
src-tauri/target/release/unturned-server-manager.exe
src-tauri/target/release/bundle/nsis/Unturned Server Manager_2.1.0_x64-setup.exe
```

## 项目结构

```text
UnturnedServerManager-Tauri/
├── src/                          # 前端源码
│   ├── App.svelte                # 主壳层、窗口栏与导航
│   ├── app.css                   # 全局主题与响应式样式
│   └── lib/
│       ├── pages/                # 页面组件
│       │   ├── Dashboard.svelte  # 仪表盘
│       │   ├── Server.svelte     # 服务器控制
│       │   ├── Rcon.svelte       # RCON 控制台
│       │   ├── Save.svelte       # 存档、创意工坊模组与插件
│       │   ├── Schedule.svelte   # 定时任务
│       │   ├── Update.svelte     # 服务端更新
│       │   ├── Logs.svelte       # 日志中心
│       │   ├── Settings.svelte   # 设置
│       │   ├── Wizard.svelte     # 首次引导
│       │   └── About.svelte      # 关于
│       ├── stores.svelte.ts      # 状态管理
│       └── utils.ts              # 工具函数
├── src-tauri/                    # Rust 后端源码
│   └── src/
│       ├── commands/             # Tauri 命令
│       ├── services/             # 服务层
│       └── models/               # 数据模型
├── UnturnedServerManager-web/    # 文档网站（本项目）
├── package.json
├── Cargo.toml
└── tauri.conf.json
```
