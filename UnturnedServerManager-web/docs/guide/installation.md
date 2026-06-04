# 安装说明

## 下载安装包

从 [GitHub Releases](https://github.com/ouxiaaocha/UnturnedServerManager-Tauri/releases) 页面下载最新版本的安装包。

### 安装包类型

| 文件 | 说明 |
| --- | --- |
| `Unturned Server Manager_2.1.0_x64-setup.exe` | NSIS 安装程序，推荐使用 |
| `unturned-server-manager.exe` | 便携版可执行文件，无需安装 |

## 系统要求

### Windows 11 / Windows 10 21H2+

- 无需额外安装，直接运行即可
- 系统自带 WebView2 Runtime

### Windows 10 1803 - 21H1

需要手动安装 WebView2 Runtime：

1. 访问 [Microsoft WebView2 下载页面](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
2. 下载并安装 "Evergreen Standalone Installer"
3. 安装完成后即可运行本工具

### Windows Server 2022

- 无需额外安装，直接运行即可

### Windows Server 2016/2019

需要安装 WebView2 Runtime，步骤同 Windows 10 旧版。

### 不支持的系统

以下系统不支持运行本工具：

- Windows 7
- Windows 8
- Windows 8.1
- Windows 10 1803 以下版本

::: warning 注意
macOS 和 Linux 当前未提供预构建包。如需在这些系统上使用，需要自行编译源码。
:::

## 开发环境

如果你想从源码构建或参与开发，需要安装以下工具：

| 工具 | 版本要求 | 安装说明 |
| --- | --- | --- |
| Node.js | 18+ | [下载页面](https://nodejs.org/) |
| pnpm | 11+ | `npm install -g pnpm` |
| Rust | 1.77.2+ | [rustup.rs](https://rustup.rs/) |
| Visual Studio Build Tools | C++ 桌面开发工作负载 | [下载页面](https://visualstudio.microsoft.com/visual-cpp-build-tools/) |

::: tip 提示
Visual Studio Build Tools 是编译 Rust 原生代码所必需的。安装时请确保勾选 "C++ 桌面开发" 工作负载。
:::
