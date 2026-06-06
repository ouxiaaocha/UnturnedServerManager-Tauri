<script setup>
import iconSave from '/icon-save.svg'
</script>

<div class="feature-header">
  <img :src="iconSave" alt="存档配置" />
  <h1>存档配置</h1>
</div>

<div class="feature-screenshot">
  <img src="/save.png" alt="存档配置界面" />
</div>

存档配置页面管理 Unturned 服务器的各项参数，包括 Commands.dat、RCON 设置、游戏规则等。

## 基础配置

| 配置项 | 说明 |
| --- | --- |
| 地图 | 服务器运行的地图名称 |
| 端口 | 服务器监听端口 |
| 最大玩家数 | 最大玩家数量 |
| 服务器名称 | 服务器列表显示名称 |
| 密码 | 服务器连接密码 |
| 视角 | 第一人称/第三人称 |

## 游戏规则

| 配置项 | 说明 |
| --- | --- |
| PvE 模式 | 启用/禁用 PvE |
| 作弊模式 | 启用/禁用作弊命令 |

## Rocket RCON 配置

| 配置项 | 说明 |
| --- | --- |
| RCON 端口 | RCON 监听端口 |
| RCON 密码 | 连接密码（支持随机生成） |

## GSLT 配置

GSLT（Game Server Login Token）是 Steam 游戏服务器登录令牌，让服务器出现在公共列表中。

::: tip 提示
可在 [Steam 游戏服务器账户管理](https://steamcommunity.com/dev/managegameservers) 创建 GSLT 令牌。
:::

## Commands.dat

Commands.dat 是主配置文件。存档配置页面支持：

- 查看/编辑 Commands.dat 内容
- 添加或删除启动参数
- 保存修改并重启服务器

## 配置备份

配置文件会自动备份到 `backups/` 目录。