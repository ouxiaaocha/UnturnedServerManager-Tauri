<script setup>
import iconUpdate from '/icon-update.svg'
</script>

<div class="feature-header">
  <img :src="iconUpdate" alt="服务端更新" />
  <h1>服务端更新</h1>
</div>

<div class="feature-screenshot">
  <img src="/update.png" alt="服务端更新界面" />
</div>

服务端更新页面通过 SteamCMD 更新 Unturned 服务端到最新版本。

## 更新流程

1. 点击"开始更新"启动 SteamCMD
2. SteamCMD 先检查并更新自身
3. 执行 `app_update 1110390 validate`
4. 实时显示更新输出日志
5. 更新完成后校验并提示结果

## 更新输出

- 实时显示 SteamCMD 执行输出
- SteamCMD 自更新进度
- 下载进度和速度
- 服务端校验进度
- 按关键词着色：错误红、系统蓝、完成绿

## SteamCMD

SteamCMD 是 Valve 的命令行工具，用于下载和更新 Steam 游戏服务器。

### 自动安装

首次引导会自动下载和安装 SteamCMD。

### 手动安装

自动安装失败时可手动操作：

1. 从 [SteamCMD 官方页面](https://developer.valvesoftware.com/wiki/SteamCMD) 下载
2. 解压到任意目录
3. 在设置中配置 SteamCMD 路径

::: warning 服务器状态
更新前请确保服务器已停止。SteamCMD 无法更新正在运行的服务器文件。
:::

::: tip 备份配置
更新不会覆盖配置和存档数据，但建议在重要更新前备份。
:::

## 常见问题

- **更新失败**：检查网络连接、SteamCMD 路径、服务端目录
- **更新后无法启动**：检查配置格式变更、日志错误、重新安装 Rocket