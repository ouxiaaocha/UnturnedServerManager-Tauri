<script setup>
import iconPlugins from '/icon-plugins.svg'
</script>

<div class="feature-header">
  <img :src="iconPlugins" alt="插件管理" />
  <h1>插件管理</h1>
</div>

插件管理功能用于查看和管理 Unturned 服务器的 Rocket 插件，支持插件列表查看、插件备注管理、快速访问插件配置目录，让您轻松管理服务器的扩展功能。

## 功能概览

| 功能 | 说明 |
| --- | --- |
| 插件列表 | 查看已安装的 Rocket 插件及其信息 |
| 插件备注 | 为每个插件添加名称、用途、版本等备注信息 |
| 打开插件目录 | 快速打开插件文件目录 |
| 打开配置目录 | 快速访问插件配置文件目录 |
| 插件状态查看 | 查看插件加载状态和错误信息 |

## 访问插件管理

插件管理功能整合在存档配置页面中：

1. 点击左侧导航栏的「存档」
2. 选择要管理插件的服务器存档
3. 切换到「插件管理」标签页

## 插件列表

插件列表显示当前服务器存档中已安装的所有 Rocket 插件：
- **插件文件名**：插件 DLL 文件的名称
- **插件备注**：您添加的自定义备注
- **文件大小**：插件文件的大小
- **修改时间**：插件文件的最后修改时间

::: tip 提示
插件列表会实时扫描 `Rocket/Plugins/` 目录，无需手动刷新。
:::

## 插件备注

为每个插件添加备注信息，方便管理和记忆：

**推荐备注内容**：
- **插件名称**：如「Essentials - 基础命令插件」
- **插件用途**：如「提供传送、家、礼包等基础命令」
- **版本信息**：如「v2.8.0」
- **依赖关系**：如「需要 Uconomy 插件」
- **配置说明**：如「配置文件：Essentials.configuration.xml」
- **来源链接**：如「GitHub 地址或工作坊链接」

**添加/编辑备注**：
1. 找到目标插件
2. 点击「编辑备注」按钮
3. 输入备注内容
4. 点击「保存」

备注信息保存在程序配置目录中，不会影响服务器的插件文件。

## 快速操作

### 打开插件目录

快速在文件资源管理器中打开插件目录：
1. 点击「打开插件目录」按钮
2. 文件资源管理器自动打开 `Servers/[存档名]/Rocket/Plugins/` 目录
3. 可以手动添加、删除或替换插件 DLL 文件

### 打开配置目录

快速访问插件配置文件目录：
1. 点击「打开配置目录」按钮
2. 文件资源管理器自动打开 `Servers/[存档名]/Rocket/` 目录
3. 可以编辑插件的配置文件（`.configuration.xml` 文件）

::: tip 提示
大部分插件的配置文件位于 `Rocket/` 目录，文件名格式为 `[插件名].configuration.xml`。
:::

## Rocket 插件系统

### 插件目录结构

Rocket 插件相关文件位于服务器存档目录下：

```
Servers/[存档名]/
  ├── Rocket/
  │   ├── Plugins/              # 插件 DLL 文件目录
  │   │   ├── Essentials.dll
  │   │   ├── Uconomy.dll
  │   │   └── ...
  │   ├── Libraries/            # 插件依赖库
  │   ├── Essentials.configuration.xml    # 插件配置文件
  │   ├── Uconomy.configuration.xml
  │   ├── Permissions.config.xml          # 权限配置
  │   └── ...
```

### 常见插件类型

| 插件类型 | 说明 | 示例插件 |
| --- | --- | --- |
| 基础功能 | 提供传送、家、礼包等基础命令 | Essentials, Teleportation |
| 经济系统 | 游戏内货币和商店系统 | Uconomy, ZaupShop |
| 保护插件 | 领地保护、建筑保护 | Regions, Clans |
| 管理工具 | 服务器管理和监控 | AdminTools, ServerInfo |
| PvP 增强 | 击杀奖励、死亡惩罚 | KillRewards, DeathPenalty |
| 社交功能 | 聊天美化、私信系统 | ChatManager, PrivateMessage |

## 安装插件

### 从 GitHub 安装

大部分 Rocket 插件托管在 GitHub：

1. **查找插件**：
   - 在 GitHub 搜索 `Unturned Rocket Plugin [插件名]`
   - 访问插件的 GitHub 仓库

2. **下载插件**：
   - 在 Releases 页面下载最新版本的 `.dll` 文件
   - 或下载源代码自行编译

3. **安装插件**：
   - 将 `.dll` 文件复制到 `Servers/[存档名]/Rocket/Plugins/` 目录
   - 如果有依赖库，复制到 `Rocket/Libraries/` 目录

4. **重启服务器**：
   - 插件需要服务器重启才能加载
   - 在「服务器控制」页面重启服务器

5. **验证安装**：
   - 查看服务器日志，确认插件已加载
   - 在插件管理列表中查看新插件
   - 在游戏内测试插件功能

### 从工作坊安装

某些插件也可以通过 Steam 创意工坊安装：

1. 在创意工坊搜索插件
2. 订阅插件（获取模组 ID）
3. 在「工作坊」标签页添加模组 ID
4. 重启服务器

::: warning 注意
工作坊中的插件可能不是最新版本，建议优先从 GitHub 获取。
:::

## 配置插件

### 查找配置文件

插件的配置文件通常位于：
```
Servers/[存档名]/Rocket/[插件名].configuration.xml
```

例如，Essentials 插件的配置文件是：
```
Servers/[存档名]/Rocket/Essentials.configuration.xml
```

### 编辑配置文件

1. 点击「打开配置目录」按钮
2. 找到目标插件的配置文件
3. 使用文本编辑器打开（推荐 VS Code、Notepad++）
4. 根据插件文档修改配置
5. 保存文件
6. 重启服务器或使用 `/reload` 命令重载配置

::: tip 提示
编辑 XML 配置文件时，注意保持 XML 格式正确，标签必须闭合。
:::

### 常见配置示例

**Essentials 插件配置**：
```xml
<?xml version="1.0" encoding="utf-8"?>
<EssentialsConfiguration>
  <EnableJoinMessage>true</EnableJoinMessage>
  <EnableLeaveMessage>true</EnableLeaveMessage>
  <JoinMessage>欢迎 {player} 来到服务器！</JoinMessage>
  <LeaveMessage>{player} 离开了服务器</LeaveMessage>
  <HomeLimit>3</HomeLimit>
  <KitCooldown>3600</KitCooldown>
</EssentialsConfiguration>
```

**Uconomy 插件配置**：
```xml
<?xml version="1.0" encoding="utf-8"?>
<UconomyConfiguration>
  <InitialBalance>100</InitialBalance>
  <DatabaseTableName>uconomy</DatabaseTableName>
</UconomyConfiguration>
```

## 卸载插件

### 安全卸载流程

1. **关闭服务器**
2. **备份插件文件**：
   - 复制插件 DLL 文件到安全位置
   - 备份插件配置文件
3. **删除插件**：
   - 从 `Rocket/Plugins/` 删除插件 DLL 文件
   - （可选）删除插件配置文件
4. **清理依赖**：
   - 检查是否有其他插件依赖该插件
   - 如果有，需要同时卸载或更换依赖
5. **重启服务器**
6. **验证**：
   - 查看服务器日志，确认没有加载错误
   - 在游戏内测试服务器功能

::: warning 注意
卸载插件前，请确认没有其他插件依赖它，否则可能导致服务器启动失败。
:::

## 插件权限

大部分 Rocket 插件使用权限系统控制功能访问。插件权限配置请参见 [权限管理](/features/permissions) 页面。

### 查看插件权限节点

插件的权限节点通常在以下位置找到：
1. **插件文档**：GitHub README 或 Wiki
2. **配置文件注释**：某些插件在配置文件中注明权限
3. **服务器日志**：启动时可能输出权限列表
4. **源代码**：查看插件源代码中的 `[RocketCommand]` 属性

## 常见问题

### 插件无法加载怎么办？

排查步骤：
1. **检查日志**：
   - 打开「日志中心」查看游戏日志
   - 查找插件加载相关的错误信息

2. **验证文件**：
   - 确认插件 DLL 文件完整且未损坏
   - 确认文件扩展名为 `.dll`（不是 `.dll.txt`）

3. **检查依赖**：
   - 某些插件需要依赖库（如 `Newtonsoft.Json.dll`）
   - 将依赖库放入 `Rocket/Libraries/` 目录

4. **版本兼容**：
   - 确认插件支持当前 Rocket 和 Unturned 版本
   - 查看插件文档了解版本要求

5. **权限问题**：
   - 确保程序有读取插件文件的权限
   - 尝试以管理员身份运行

### 插件冲突如何排查？

逐个排除法：
1. 备份所有插件
2. 删除所有插件
3. 逐个添加插件并测试
4. 找到冲突的插件组合
5. 查看插件文档是否有兼容性说明

常见冲突原因：
- 多个插件注册相同命令
- 插件之间的事件处理冲突
- 依赖库版本不一致

### 如何更新插件？

1. **查看当前版本**：
   - 在插件备注中记录当前版本
   - 或查看插件文件的修改时间

2. **下载新版本**：
   - 访问插件的 GitHub 仓库
   - 在 Releases 页面下载最新版本

3. **备份旧版本**：
   - 将旧插件 DLL 文件复制到备份目录

4. **替换文件**：
   - 关闭服务器
   - 用新版本 DLL 覆盖旧文件

5. **检查配置**：
   - 查看更新日志是否有配置文件变更
   - 必要时更新配置文件

6. **测试验证**：
   - 启动服务器
   - 查看日志确认加载成功
   - 在游戏内测试插件功能

### 在哪里找到 Rocket 插件？

**推荐资源**：
1. **GitHub**：
   - 搜索 `Unturned Rocket Plugin`
   - 常见仓库组织：RocketMod,RestoreMonarchy, SammyJoeOsborne

2. **ImperialPlugins**：
   - https://imperialplugins.com/
   - 提供免费和付费插件

3. **RestoreMonarchy**：
   - https://restoremonarchy.com/
   - 社区插件资源站

4. **RocketMod 论坛**（已关闭）：
   - 历史资源可通过存档访问

5. **Steam 创意工坊**：
   - 搜索 Rocket 插件模组
   - 数量较少，不如 GitHub 全面

### 插件配置文件丢失怎么办？

如果插件配置文件意外删除：
1. **重新生成**：
   - 删除配置文件后，重启服务器
   - Rocket 会自动生成默认配置文件

2. **从备份恢复**：
   - 从 `backups/` 目录恢复配置文件

3. **手动创建**：
   - 参考插件文档创建配置文件
   - 复制其他服务器的配置文件

## 与权限管理的关系

插件管理和权限管理是两个独立但相关的功能：
- **插件管理**：负责插件文件的安装、查看、备注
- **权限管理**：负责配置哪些玩家可以使用插件的哪些功能

两者的关联：
1. 安装插件后，需要在权限管理中配置权限
2. 插件注册的权限节点在权限管理中可见
3. 权限配置决定插件功能的可用性

详细说明请参见 [权限管理](/features/permissions) 页面。

## 相关功能

- [存档配置](/features/save)：插件管理整合在存档配置的标签页中
- [权限管理](/features/permissions)：配置插件的权限和访问控制
- [创意工坊模组](/features/workshop)：某些插件可通过工作坊安装
- [服务器控制](/features/server)：安装或卸载插件后需要重启服务器
- [日志中心](/features/logs)：查看插件加载日志和错误信息