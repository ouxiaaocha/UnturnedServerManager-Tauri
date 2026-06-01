# 服务端更新

服务端更新页面用于通过 SteamCMD 更新 Unturned 服务器到最新版本。

<p align="center">
  <img src="/update.png" width="90%" alt="服务端更新界面" />
</p>

## 功能说明

### 更新流程

1. 点击"更新"按钮启动 SteamCMD
2. SteamCMD 会自动下载最新的服务端文件
3. 更新过程中显示实时输出日志
4. 更新完成后提示用户

### 更新输出

- 实时显示 SteamCMD 的执行输出
- 显示下载进度和速度
- 显示更新完成状态

## SteamCMD

SteamCMD 是 Valve 提供的命令行工具，用于下载和更新 Steam 游戏服务器。

### 自动安装

Unturned Server Manager 首次引导会自动下载和安装 SteamCMD。如果未安装，更新页面会提示你进行安装。

### 手动安装

如果自动安装失败，可以手动安装：

1. 从 [SteamCMD 官方页面](https://developer.valvesoftware.com/wiki/SteamCMD) 下载
2. 解压到任意目录
3. 在设置中配置 SteamCMD 路径

## 更新策略

### 何时更新

- Unturned 发布新版本后
- 服务器出现版本不兼容问题时
- 定期检查更新

### 更新注意事项

::: warning 服务器状态
更新前请确保服务器已停止。SteamCMD 无法更新正在运行的服务器文件。
:::

::: tip 备份配置
更新不会覆盖你的服务器配置和存档数据，但建议在重要更新前备份配置。
:::

## 常见问题

### 更新失败

- 检查网络连接
- 确认 SteamCMD 路径正确
- 查看更新输出中的错误信息

### 更新后服务器无法启动

- 检查是否有新的配置格式变更
- 查看服务器日志中的错误信息
- 尝试重新安装 Rocket 模块
