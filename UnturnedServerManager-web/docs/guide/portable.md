# 便携版说明

便携版直接运行 `unturned-server-manager.exe`，不需要安装到系统目录。

## 目录结构

程序会在 exe 同级目录创建统一运行数据目录：

```text
UnturnedServerManagerData/
  config/      应用配置、服务器配置、定时任务、备注数据
  logs/        软件日志、操作日志、游戏日志
  data/        运行数据
  backups/     备份数据
```

## 迁移方式

1. 停止所有服务器。
2. 关闭 Unturned Server Manager。
3. 复制整个程序目录到新位置。
4. 在新位置运行 exe。
5. 按实际网络环境调整端口、防火墙或公网配置。

## 备份建议

| 数据 | 建议 |
| --- | --- |
| `UnturnedServerManagerData/config/` | 修改存档、任务、RCON 前备份 |
| `UnturnedServerManagerData/logs/` | 排查问题时保留近期日志 |
| `UnturnedServerManagerData/backups/` | 定期复制到其他磁盘或云端 |
| 服务端目录 | 更新服务端或插件前建议整体备份 |

::: tip 提示
便携版的优点是干净、可迁移。不要把 exe 单独复制走，迁移时应复制整个目录。
:::
