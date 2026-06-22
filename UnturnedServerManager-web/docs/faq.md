# 常见问题

## 构建相关

### PowerShell 脚本被阻止

Windows 执行策略可能拦截 `pnpm.ps1`。可以使用：

```powershell
pnpm.cmd tauri build
```

### 首次编译很慢

Rust release 构建会下载并编译依赖，首次耗时较长。后续构建通常会快很多。

### 旧版 Windows 打不开程序

请安装 Microsoft WebView2 Runtime 和 Visual C++ Redistributable。

## 运行相关

### 服务器启动失败

1. 打开「设置」执行运行环境检测。
2. 检查 SteamCMD、服务端目录、Rocket.Unturned 和 Bridge DLL。
3. 检查游戏端口和 RCON 端口是否冲突。
4. 到「日志中心」查看软件日志和游戏日志。

### 本地命令发送失败

1. 确认目标存档服务器正在运行。
2. 确认 Rocket.Unturned 已安装。
3. 确认 `UnturnedServerManagerBridge.dll` 已部署。
4. 修复后重启服务器，让插件重新加载。

### RCON 无法连接

1. 确认目标存档正在运行。
2. 在「存档 > 基础配置」检查 RCON 端口和密码。
3. 确认 RCON 端口没有被其他程序占用。
4. 检查防火墙是否阻止了本机或可信来源连接。

::: warning 安全提示
RCON 是额外远程管理端口，不建议对公网开放。日常启停和重启优先使用本地命令 Bridge。
:::

### 创意工坊模组不生效

1. 检查模组 ID 是否正确。
2. 查看 `WorkshopDownloadConfig.json` 是否已保存。
3. 使用 SteamCMD 更新并校验服务端。
4. 检查客户端 workshop 缓存是否实际更新。
5. 查看游戏日志中是否有 missing asset 或依赖缺失。

### 多服务器端口冲突

每个同时运行的存档都需要独立游戏端口和 RCON 端口。应用启动时会检测冲突，并提供手动调整或自动分配端口的入口。

## 数据与迁移

### 如何备份

复制 exe 同级目录即可，重点保留：

```text
config/
logs/
data/
backups/
```

### 更新程序会影响存档吗

直接替换 `unturned-server-manager.exe` 不会删除配置和存档。更新前仍建议备份整个目录。
