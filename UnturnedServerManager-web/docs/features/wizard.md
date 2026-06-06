<script setup>
import iconWizard from '/icon-wizard.svg'
</script>

<div class="feature-header">
  <img :src="iconWizard" alt="首次引导" />
  <h1>首次引导</h1>
</div>

<div class="feature-screenshot">
  <img src="/desktop-wizard.png" alt="首次引导界面" />
</div>

首次引导帮助新用户快速完成 Unturned 服务器的初始配置。

## 引导步骤

### 1️⃣ SteamCMD 安装

- 自动检测是否已安装 SteamCMD
- 未安装时自动下载并解压
- 配置 SteamCMD 路径

### 2️⃣ 服务端目录

- 自动检测服务端安装目录
- 支持手动浏览指定

### 3️⃣ Rocket 模块

- 检测 Rocket.Unturned 是否已安装
- 未安装时自动从服务端 Extras 目录安装
- 部署本地命令 Bridge 插件

### 4️⃣ 存档初始化

- 创建新的服务器存档
- 配置基础参数（地图、端口、最大玩家数）
- 生成 Commands.dat 配置文件

### 5️⃣ RCON 配置

- 启用 RCON 功能（可选）
- 设置 RCON 端口
- 生成随机安全密码

## 自动化配置

| 配置项 | 说明 |
| --- | --- |
| SteamCMD 路径 | 自动检测或下载 |
| Rocket 模块 | 从服务端 Extras 自动安装 |
| 本地命令 Bridge | 自动部署到插件目录 |
| 服务器存档 | 自动创建并初始化 |
| RCON 配置 | 自动生成安全密码 |

## 手动配置

自动配置失败时可手动完成：

1. 手动下载 SteamCMD
2. 手动安装 Rocket 模块
3. 在设置页重新部署 Bridge
4. 手动创建存档

::: tip 提示
引导完成后所有配置都可修改。
:::

::: warning 注意
重新引导会覆盖现有配置，请先备份重要数据。
:::

## 窄屏适配

- 桌面端：完整显示所有配置选项
- 移动端：简化界面，突出核心操作