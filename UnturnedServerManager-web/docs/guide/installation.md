# 安装说明

## 推荐方式

优先使用便携版 `unturned-server-manager.exe`。把它放到独立目录运行，程序会在同级目录创建 `UnturnedServerManagerData/`，配置、日志、数据和备份都会统一放在这个目录内。

## 运行前准备

| 项目 | 说明 |
| --- | --- |
| WebView2 Runtime | Windows 11 与新版 Windows 10 通常已内置 |
| Visual C++ Redistributable | 旧系统缺失运行库时需要安装 |
| 网络访问 | SteamCMD 下载服务端和工作坊内容时需要联网 |
| 防火墙 | 游戏端口按开服需求放行，RCON 端口不建议对公网开放 |

## 首次引导会处理什么

- 检测 SteamCMD。
- 安装或校验 Unturned Dedicated Server。
- 初始化 Rocket.Unturned。
- 部署本地命令 Bridge。
- 初始化默认存档和 RCON 配置。

## 开发者安装

如果需要从源码运行：

```bash
pnpm install
pnpm tauri dev
```

如果 PowerShell 执行策略阻止 `pnpm.ps1`，可以使用：

```powershell
pnpm.cmd tauri dev
```

## 公网安全

打包后的桌面程序不会启动可被公网访问的 Web 管理服务。需要按开服需求放行的是 Unturned 游戏端口；Rocket RCON 是额外管理端口，不建议直接对公网开放。
