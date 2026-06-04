# 常见问题

## 构建相关

### 构建提示 PowerShell 脚本被阻止

Windows 执行策略可能拦截 `pnpm.ps1`。可以使用：

```powershell
pnpm.cmd tauri build
```

或者修改 PowerShell 执行策略：

```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

### 旧版 Windows 打不开程序

请安装 Microsoft WebView2 Runtime，并确保已安装 Visual C++ Redistributable。

- [WebView2 Runtime 下载](https://developer.microsoft.com/en-us/microsoft-edge/webview2/)
- [Visual C++ Redistributable 下载](https://learn.microsoft.com/en-us/cpp/windows/latest-supported-vc-redist)

### 首次编译较慢

Rust release 构建会下载并编译依赖，首次耗时较长；后续构建通常会快很多。

::: tip 提示
首次编译可能需要 10-30 分钟，取决于你的网络和硬件性能。请耐心等待。
:::

## 运行相关

### 服务器启动失败

1. 检查 SteamCMD 是否正确安装
2. 检查端口是否被占用
3. 查看日志中心的错误信息
4. 确认防火墙设置

### 无法连接 RCON

1. 确认 Rocket 模块已正确安装
2. 检查 RCON 配置是否正确
3. 确认 RCON 端口未被占用
4. 检查防火墙是否阻止了 RCON 端口

### 本地命令发送失败

1. 确认服务器正在运行
2. 打开设置页运行环境检测
3. 确认 Rocket.Unturned 已安装
4. 确认 `UnturnedServerManagerBridge.dll` 已部署
5. 如果刚刚修复 Bridge，请重启服务器让插件重新加载

### 创意工坊模组下载失败

1. 检查网络连接
2. 确认模组 ID 正确
3. 检查 SteamCMD 是否正常工作
4. 查看更新输出中的错误信息

## 杀毒软件相关

### 杀毒软件误报

部分杀毒软件可能会误报 Unturned Server Manager。如果遇到此问题：

1. 将程序目录添加到杀毒软件的白名单
2. 将 `unturned-server-manager.exe` 添加到排除列表
3. 将 SteamCMD 目录添加到白名单

::: warning 注意
本程序不包含任何恶意代码。误报是由于程序使用了系统级 API（如进程管理、网络监控等）导致的。
:::

## 其他问题

### 如何备份服务器数据？

Unturned Server Manager 的所有数据都存储在 exe 同级目录下：

```text
config/      配置数据
logs/        日志数据
data/        运行数据
backups/     备份数据
```

备份时只需复制整个文件夹即可。

### 如何迁移服务器？

1. 停止服务器
2. 复制整个 Unturned Server Manager 文件夹到新位置
3. 在新位置运行程序
4. 根据需要修改网络配置（端口转发等）

### 如何更新 Unturned Server Manager？

1. 从 GitHub Releases 下载最新版本
2. 替换 `unturned-server-manager.exe` 文件
3. 重新运行程序

::: tip 提示
更新程序不会影响你的服务器配置和数据。
:::
