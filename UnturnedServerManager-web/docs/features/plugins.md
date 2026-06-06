<script setup>
import iconPlugins from '/icon-plugins.svg'
</script>

<div class="feature-header">
  <img :src="iconPlugins" alt="插件管理" />
  <h1>插件管理</h1>
</div>

插件管理页面查看和管理 Unturned 服务器的 Rocket 插件。

## 插件列表

- 显示 Rocket 插件目录中的所有插件
- 查看插件文件信息
- 快速定位插件配置目录

## 插件备注

为每个插件添加自定义备注：

- 插件名称和用途
- 版本信息
- 配置说明

## 快速操作

- **打开插件目录**：在文件资源管理器中打开
- **打开配置目录**：快速访问插件配置文件

## Rocket 插件

插件通常存储在：

```text
Servers/[存档名]/Rocket/Plugins/
```

### 常见插件类型

| 类型 | 说明 |
| --- | --- |
| 管理插件 | 管理员命令、权限管理 |
| 经济插件 | 游戏内经济系统 |
| 传送插件 | 玩家传送功能 |
| 保护插件 | 领地保护、建筑保护 |

::: tip 提示
插件备注保存在 `config/` 目录中，不影响插件本身的配置文件。
:::

## 安装插件

1. 从 Rocket 模组官网或社区下载
2. 将插件文件放入 `Rocket/Plugins/` 目录
3. 重启服务器以加载
4. 根据需要编辑插件配置