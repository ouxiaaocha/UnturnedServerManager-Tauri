<script setup>
import iconPermissions from '/icon-permissions.svg'
</script>

<div class="feature-header">
  <img :src="iconPermissions" alt="权限管理" />
  <h1>权限管理</h1>
</div>

权限管理功能提供可视化界面编辑 Rocket 权限系统配置，支持权限组管理、玩家权限分配、权限继承设置，让您轻松管理服务器的权限体系。

## 功能概览

| 功能 | 说明 |
| --- | --- |
| 权限组管理 | 创建和编辑权限组，配置组名、父组和权限节点 |
| 玩家权限分配 | 通过 SteamID 将玩家分配到指定权限组 |
| 权限继承 | 配置权限组的父子关系，实现权限继承 |
| 可视化编辑 | 图形界面编辑 Permissions.config.xml，无需手动修改 XML |
| 配置验证 | 自动验证权限配置的正确性 |

## 访问权限管理

权限管理功能整合在存档配置页面中：

1. 点击左侧导航栏的「存档」
2. 选择要配置权限的服务器存档
3. 切换到「权限管理」标签页

## 权限组管理

### 查看权限组

权限组列表显示所有已配置的权限组：
- **组名**：权限组的唯一标识符
- **父组**：继承权限的父权限组
- **权限节点数量**：该组包含的权限数量

### 创建权限组

1. 点击「添加权限组」按钮
2. 填写权限组信息：
   - **组名**：权限组的唯一标识符（如 `admin`、`vip`、`default`）
   - **父组**（可选）：选择要继承权限的父组
   - **权限节点**：添加该组拥有的权限

3. 点击「保存」

**常见权限组示例**：
- `admin`：管理员组，拥有所有权限
- `moderator`：版主组，继承自 admin，部分权限受限
- `vip`：VIP 玩家组，拥有特殊权限
- `default`：默认玩家组，基础权限

::: tip 提示
建议使用英文小写命名权限组，避免使用空格和特殊字符。
:::

### 编辑权限节点

为权限组添加或删除权限节点：

**添加权限**：
1. 选择要编辑的权限组
2. 点击「添加权限」
3. 输入权限节点（如 `rocket.teleport`、`essentials.kit`）
4. 点击「确认」

**删除权限**：
1. 找到要删除的权限节点
2. 点击该权限旁的「删除」按钮

**常用权限节点示例**：
```
rocket.*                    # Rocket 所有权限
rocket.teleport             # 传送权限
rocket.god                  # 无敌模式
rocket.vehicle             # 刷车权限
essentials.*                # Essentials 插件所有权限
essentials.kit              # 礼包权限
essentials.heal             # 治疗权限
essentials.home             # 家传送权限
```

::: warning 注意
权限节点的具体格式取决于您安装的插件。请查阅插件文档获取完整的权限列表。
:::

### 权限继承

通过设置父组实现权限继承：

**继承规则**：
- 子组自动继承父组的所有权限
- 子组可以添加额外的权限
- 可以构建多级继承关系

**继承示例**：
```
default (基础权限)
  └── vip (继承 default + VIP 特权)
      └── vip_plus (继承 vip + 更多特权)
  └── moderator (继承 default + 管理权限)
      └── admin (继承 moderator + 完全权限)
```

## 玩家权限配置

### 添加玩家到权限组

1. 在权限组列表中选择目标组
2. 点击「添加玩家」
3. 输入玩家的 **SteamID64**
4. 点击「确认」

**获取 SteamID64**：
- 方法 1：访问 [SteamID Finder](https://steamid.io/)，输入玩家 Steam 个人资料链接
- 方法 2：在游戏内使用命令 `/steamid 玩家名称`
- 方法 3：查看服务器日志，玩家加入时会显示 SteamID

::: tip 提示
SteamID64 是一串 17 位数字，格式类似：`76561198XXXXXXXXX`
:::

### 移除玩家权限

1. 找到要移除的玩家
2. 点击玩家旁的「移除」按钮
3. 确认操作

### 查看玩家所属组

在玩家列表中可以查看每个玩家所属的权限组：
- 一个玩家可以同时属于多个权限组
- 玩家将拥有所有所属组的权限（合并）
- 未分配权限组的玩家默认属于 `default` 组

## 权限验证

### 测试权限配置

保存权限配置后，建议在游戏内测试：

1. **重启服务器**或使用命令重载权限：
   ```
   /reload
   ```

2. **测试权限**：
   - 使用对应权限组的账号登录游戏
   - 尝试使用该组应有的命令
   - 验证权限是否生效

3. **排查问题**：
   - 如果权限未生效，检查权限节点拼写是否正确
   - 确认玩家已正确分配到权限组
   - 查看服务器日志是否有权限相关错误

### 常见权限节点说明

| 权限节点 | 说明 | 适用插件 |
| --- | --- | --- |
| `*` | 所有权限（超级管理员） | 全部 |
| `rocket.*` | Rocket 核心所有权限 | Rocket |
| `rocket.teleport` | 传送命令权限 | Rocket |
| `rocket.vehicle` | 刷车命令权限 | Rocket |
| `essentials.*` | Essentials 所有权限 | Essentials |
| `essentials.kit` | 礼包命令权限 | Essentials |
| `essentials.home` | 家传送权限 | Essentials |
| `uconomy.*` | 经济系统所有权限 | Uconomy |
| `factions.*` | 派系系统所有权限 | Factions |

## Permissions.config.xml 文件结构

权限管理界面实际上是在编辑 `Permissions.config.xml` 文件，该文件位于：

```
Servers/[存档名]/Rocket/Permissions.config.xml
```

### XML 文件格式

```xml
<?xml version="1.0" encoding="utf-8"?>
<RocketPermissions>
  <Group>
    <Id>default</Id>
    <ParentGroup></ParentGroup>
    <Priority>100</Priority>
    <Members>
      <SteamId>76561198XXXXXXXXX</SteamId>
    </Members>
    <Permissions>
      <Permission>essentials.home</Permission>
      <Permission>essentials.kit</Permission>
    </Permissions>
  </Group>
  <Group>
    <Id>admin</Id>
    <ParentGroup>default</ParentGroup>
    <Priority>0</Priority>
    <Members>
      <SteamId>76561198YYYYYYYYY</SteamId>
    </Members>
    <Permissions>
      <Permission>*</Permission>
    </Permissions>
  </Group>
</RocketPermissions>
```

### 手动编辑注意事项

如果您选择手动编辑 XML 文件：
- ✅ 保持 XML 格式正确，注意标签闭合
- ✅ 编辑前备份原文件
- ✅ 使用 UTF-8 编码保存
- ✅ 编辑后重启服务器或重载权限
- ❌ 避免使用 Windows 记事本编辑（可能破坏编码）

::: warning 注意
推荐使用权限管理界面进行配置，避免因 XML 格式错误导致权限系统失效。
:::

### 备份与恢复

**备份权限配置**：
1. 复制 `Permissions.config.xml` 文件
2. 保存到安全位置
3. 建议定期备份

**恢复权限配置**：
1. 将备份的文件复制回原位置
2. 覆盖现有文件
3. 重启服务器

## 使用场景

### 场景 1：新服务器权限初始化

**需求**：新建服务器，需要设置管理员和默认玩家权限。

**操作步骤**：
1. 创建 `default` 权限组，添加基础权限（如 `essentials.home`、`essentials.kit`）
2. 创建 `admin` 权限组，父组设为 `default`，添加 `*` 权限
3. 将自己的 SteamID 添加到 `admin` 组
4. 保存并重启服务器

**效果**：管理员拥有所有权限，普通玩家只有基础权限。

### 场景 2：设置 VIP 权限

**需求**：为付费 VIP 玩家提供额外权限，如更多礼包、传送次数等。

**操作步骤**：
1. 创建 `vip` 权限组，父组设为 `default`
2. 添加 VIP 专属权限（如 `essentials.kit.vip`、`essentials.teleport.unlimited`）
3. 将 VIP 玩家的 SteamID 添加到 `vip` 组
4. 保存配置

**效果**：VIP 玩家拥有普通玩家的所有权限，外加 VIP 特权。

### 场景 3：多级管理员体系

**需求**：建立"超级管理员 → 管理员 → 版主"的三级管理体系。

**操作步骤**：
1. 创建 `moderator` 组（版主），添加基础管理权限
2. 创建 `admin` 组（管理员），父组设为 `moderator`，添加更多管理权限
3. 创建 `superadmin` 组（超管），父组设为 `admin`，添加 `*` 权限
4. 将对应人员的 SteamID 分配到各自组

**效果**：实现分级管理，每一级自动继承下级权限。

## 最佳实践

::: tip 最佳实践
1. **权限最小化原则**：只给予必要的权限，避免滥用
2. **使用继承**：通过父组继承减少重复配置
3. **定期审查**：定期检查权限配置，移除离职管理员
4. **备份配置**：修改权限前先备份，便于回滚
5. **测试验证**：权限配置后务必在游戏内测试
:::

## 常见问题

### 权限配置后不生效怎么办？

排查步骤：
1. **检查权限节点拼写**：确保权限节点与插件文档一致
2. **确认玩家分组**：检查玩家是否正确分配到权限组
3. **重载权限**：在服务器控制台执行 `/reload` 命令
4. **重启服务器**：部分权限需要重启服务器才能生效
5. **查看日志**：检查 `Rocket.log` 是否有权限相关错误

### 权限继承规则是什么？

- **子组继承父组**：子组自动拥有父组的所有权限
- **权限合并**：子组可以添加额外权限，不会覆盖父组权限
- **多级继承**：支持多层继承关系，子组会继承所有祖先组的权限
- **优先级**：权限 Priority 值越小优先级越高（0 = 最高）

### 权限节点的 `*` 通配符如何使用？

通配符规则：
- `*`：所有权限（超级管理员）
- `插件名.*`：该插件的所有权限（如 `rocket.*`、`essentials.*`）
- `插件名.类别.*`：该类别的所有权限（如 `essentials.teleport.*`）

示例：
```
*                    # 所有权限
rocket.*             # Rocket 所有权限
essentials.kit.*     # Essentials 所有礼包权限
essentials.teleport.*  # Essentials 所有传送权限
```

### 如何找到插件的权限节点？

1. **查看插件文档**：每个插件通常在 GitHub 或官网提供权限列表
2. **查看插件代码**：在插件的源代码中查找 `[RocketCommand]` 标记
3. **服务器日志**：启动服务器时会输出已加载的权限
4. **测试命令**：在游戏内尝试使用命令，权限不足时会提示所需权限

### 一个玩家可以属于多个权限组吗？

可以。玩家可以同时属于多个权限组：
- 玩家将拥有**所有所属组**的权限（权限合并）
- 如果多个组有相同权限，不会冲突
- 适用于需要临时赋予特殊权限的场景

### 与插件权限的关系是什么？

- **Rocket 权限系统**：统一管理所有插件的权限
- **插件注册权限**：插件在加载时向 Rocket 注册权限节点
- **权限检查**：插件执行命令时，Rocket 检查玩家是否有对应权限
- **独立配置**：部分插件有独立的权限配置，需同时配置

## 相关功能

- [存档配置](/features/save)：权限管理整合在存档配置的标签页中
- [插件管理](/features/plugins)：了解如何安装和管理 Rocket 插件
- [RCON 控制台](/features/rcon)：使用 RCON 远程执行权限相关命令
