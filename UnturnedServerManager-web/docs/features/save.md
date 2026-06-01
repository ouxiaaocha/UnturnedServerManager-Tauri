# 存档配置

存档配置页面用于管理 Unturned 服务器的各种配置参数，包括 Commands.dat、RCON 设置、游戏规则等。

<p align="center">
  <img src="/save.png" width="90%" alt="存档配置界面" />
</p>

## 配置项说明

### 基础配置

| 配置项 | 说明 |
| --- | --- |
| 地图 | 服务器运行的地图名称 |
| 端口 | 服务器监听的端口号 |
| 最大玩家数 | 服务器允许的最大玩家数量 |
| 服务器名称 | 在服务器列表中显示的名称 |

### 游戏规则

| 配置项 | 说明 |
| --- | --- |
| PvE 模式 | 启用/禁用玩家对环境模式 |
| 作弊模式 | 启用/禁用作弊命令 |
| 难度 | 设置游戏难度 |

### Rocket RCON 配置

| 配置项 | 说明 |
| --- | --- |
| RCON 端口 | RCON 监听的端口号 |
| RCON 密码 | 连接 RCON 所需的密码 |

### GSLT 配置

GSLT（Game Server Login Token）是 Steam 游戏服务器登录令牌，用于让服务器出现在公共服务器列表中。

::: tip 提示
可以在 [Steam 游戏服务器账户管理页面](https://steamcommunity.com/dev/managegameservers) 创建 GSLT 令牌。
:::

## Commands.dat

Commands.dat 是 Unturned 服务器的主配置文件。通过存档配置页面，你可以：

- 查看和编辑 Commands.dat 的内容
- 添加或删除启动参数
- 保存修改并重启服务器

## 配置备份

Unturned Server Manager 会自动备份你的配置文件。你可以在 `backups/` 目录中找到历史备份。
