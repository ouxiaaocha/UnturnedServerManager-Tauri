# 📦 安装说明

## 下载安装包

从 [GitHub Releases](https://github.com/ouxiaaocha/UnturnedServerManager-Tauri/releases) 下载最新版本。

### 安装包类型

| 文件 | 说明 |
| --- | --- |
| `Unturned Server Manager_x64-setup.exe` | NSIS 安装程序，推荐使用 |
| `unturned-server-manager.exe` | 便携版，无需安装 |

## 系统要求

### Windows 11 / Windows 10 21H2+

无需额外安装，系统自带 WebView2 Runtime。

### Windows 10 1803 - 21H1

需手动安装 WebView2 Runtime：

1. 访问 [Microsoft WebView2 下载页面](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
2. 下载 "Evergreen Standalone Installer"
3. 安装完成后即可运行

### Windows Server

| 版本 | 说明 |
| --- | --- |
| 2022 | 直接运行 |
| 2016/2019 | 需安装 WebView2 Runtime |

### 不支持的系统

- Windows 7、8、8.1
- Windows 10 1803 以下

::: warning 注意
macOS 和 Linux 未提供预构建包，需自行编译源码。
:::

## 🔨 开发环境

从源码构建需安装以下工具：

| 工具 | 版本 | 安装方式 |
| --- | --- | --- |
| Node.js | 18+ | [nodejs.org](https://nodejs.org/) |
| pnpm | 11+ | `npm install -g pnpm` |
| Rust | 1.77.2+ | [rustup.rs](https://rustup.rs/) |
| VS Build Tools | C++ 桌面开发 | [visualstudio.microsoft.com](https://visualstudio.microsoft.com/visual-cpp-build-tools/) |

::: tip 提示
VS Build Tools 是编译 Rust 原生代码必需的，请勾选 "C++ 桌面开发" 工作负载。
:::