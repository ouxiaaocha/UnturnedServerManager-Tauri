<script setup>
import iconRcon from '/icon-rcon.svg'
</script>

<div class="feature-header">
  <img :src="iconRcon" alt="RCON" />
  <h1>RCON 控制台</h1>
</div>

<div class="feature-screenshot">
  <img src="/rcon.png" alt="RCON 控制台界面" />
</div>

RCON 控制台允许你通过 Rocket RCON 协议远程管理服务器。当前版本中，本地命令是默认控制路径；RCON 保留为额外远程功能。

## 什么是 RCON？

RCON（Remote Console）是远程控制协议，允许通过网络连接服务器执行管理命令。如果只在本机管理，建议优先使用服务器页面的本地命令输入栏——它不需要开放额外端口。

## 连接管理

- 手动连接到 RCON 服务器
- 服务器启动后自动连接
- 断开连接和重新连接

## 命令执行

- 命令输入框中输入 RCON 命令
- 支持命令历史记录（上下箭头切换）
- 实时显示命令执行结果

## 常用命令

| 命令 | 说明 |
| --- | --- |
| `say [消息]` | 向所有玩家广播 |
| `kick [玩家名]` | 踢出指定玩家 |
| `ban [玩家名] [原因] [时长]` | 封禁玩家 |
| `unban [玩家名]` | 解封玩家 |
| `give [玩家名] [物品ID] [数量]` | 给予物品 |
| `teleport [玩家名] [坐标]` | 传送玩家 |

::: tip 提示
使用 RCON 前请确保 Rocket 模块已正确安装，且 RCON 已在配置中启用。
:::

## 配置说明

1. 安装 Rocket 模块（首次引导可自动安装）
2. 在存档配置中启用 RCON
3. 设置 RCON 端口和密码
4. 确保防火墙允许 RCON 端口连接

::: warning 安全建议
RCON 端口不建议对公网开放，建议仅在局域网内使用或通过 VPN 连接。使用强随机密码并定期更换。
:::