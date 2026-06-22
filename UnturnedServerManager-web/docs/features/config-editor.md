<script setup>
import iconConfig from '/icon-config.svg'
</script>

<div class="feature-header">
  <img :src="iconConfig" alt="Config.txt 编辑器" />
  <h1>Config.txt 高级编辑器</h1>
</div>

Config.txt 高级编辑器提供可视化界面编辑 Unturned 服务器的高级配置文件，支持常用配置项快速编辑、配置验证、一键应用，让您无需手动修改配置文件即可完成服务器调优。

## 功能概览

| 功能 | 说明 |
| --- | --- |
| 可视化编辑 | 图形界面编辑 Config.txt，无需手动修改文本文件 |
| 配置分类 | 按服务器基础、性能、游戏规则、高级选项分类展示 |
| 配置验证 | 自动验证配置值的合法性，防止错误配置 |
| 配置提示 | 每个配置项提供说明和推荐值 |
| 一键应用 | 保存后自动应用到服务器配置 |
| 配置备份 | 修改前自动备份原配置 |

## 访问 Config.txt 编辑器

Config.txt 编辑器整合在存档配置页面中：

1. 点击左侧导航栏的「存档」
2. 选择要配置的服务器存档
3. 切换到「高级配置」或「Config.txt」标签页

::: tip 提示
首次使用时，编辑器会自动加载服务器的现有配置。如果配置文件不存在，将使用默认配置。
:::

## 配置项说明

### 服务器基础配置

| 配置项 | 说明 | 默认值 | 示例 |
| --- | --- | --- | --- |
| Name | 服务器名称，显示在服务器列表中 | `Unturned Server` | `我的 Unturned 服务器` |
| Map | 地图名称 | `PEI` | `Washington`, `Russia`, `Germany` |
| Port | 服务器端口 | `27015` | `27015`, `27016` |
| Password | 服务器密码（留空表示公开） | `（空）` | `mypassword` |
| Mode | 游戏模式 | `Normal` | `Easy`, `Normal`, `Hard` |

::: warning 注意
修改端口后，需要在防火墙和路由器中放行新端口，并更新服务器列表注册信息。
:::

### 性能配置

| 配置项 | 说明 | 默认值 | 推荐值 |
| --- | --- | --- | --- |
| Max_Players | 最大玩家数 | `24` | 根据服务器性能调整（8-48） |
| Max_Vehicles | 最大车辆数 | `16` | 玩家数的 50-75% |
| Tick_Rate | 服务器刷新率（Hz） | `50` | `30`（省资源）,`50`（平衡）,`100`（高性能） |
| Timeout | 玩家超时时间（秒） | `30` | `30-60` |
| Queue_Size | 队列大小 | `8` | 最大玩家数的 30-50% |

**性能优化建议**：
- **低配服务器**：Max_Players ≤ 12, Tick_Rate = 30
- **中配服务器**：Max_Players = 12-24, Tick_Rate = 50
- **高配服务器**：Max_Players = 24-48, Tick_Rate = 50-100

### 游戏规则配置

| 配置项 | 说明 | 默认值 | 选项 |
| --- | --- | --- | --- |
| PvP | 启用玩家对战 | `true` | `true`（启用PvP）, `false`（PvE） |
| Cheats | 允许作弊命令 | `false` | `true`, `false` |
| Difficulty | 游戏难度 | `Normal` | `Easy`, `Normal`, `Hard`, `Gold` |
| Perspective | 视角限制 | `Both` | `First`（第一人称）, `Third`（第三人称）, `Both`（两者） |
| Gold | 金服模式（高难度） | `false` | `true`, `false` |

**常见配置场景**：
- **PvE 休闲服**：`PvP=false`, `Difficulty=Easy`, `Cheats=false`
- **PvP 竞技服**：`PvP=true`, `Difficulty=Hard`, `Perspective=First`
- **创造服**：`Cheats=true`, `Difficulty=Easy`, `PvP=false`
- **金服**：`Gold=true`, `Difficulty=Gold`, `PvP=true`

### 高级选项

| 配置项 | 说明 | 默认值 |
| --- | --- | --- |
| Loadout | 玩家初始装备配置ID | `0` |
| Day_Night_Length | 昼夜循环时长（秒） | `3600` |
| Sync_Max_Zombies | 最大僵尸数量 | `64` |
| Sync_Max_Animals | 最大动物数量 | `32` |
| Allow_Underwater_Features | 允许水下特性 | `true` |
| Can_Suicide | 允许自杀命令 | `true` |
| Gameplay_Config_Overrides | 游戏配置覆盖 | `（空）` |

::: tip 提示
高级选项通常不需要修改。如果不确定某个选项的作用，建议保持默认值。
:::

## 使用方法

### 编辑配置

1. **加载配置**：
   - 编辑器自动加载选定存档的 Config.txt
   - 配置项按分类展示（基础、性能、游戏规则、高级）

2. **修改配置**：
   - 直接在对应输入框中修改配置值
   - 使用下拉菜单选择预设选项
   - 参考配置说明和推荐值

3. **配置验证**：
   - 输入框会实时验证配置值的合法性
   - 非法值会显示红色边框和错误提示
   - 必须修正所有错误才能保存

4. **保存配置**：
   - 点击「保存」按钮
   - 编辑器自动备份原配置到 `Config.txt.backup`
   - 将新配置写入 `Config.txt`
   - 显示保存成功提示

5. **应用配置**：
   - **新配置需要重启服务器才能生效**
   - 可以在「服务器控制」页面重启服务器
   - 或使用「保存并重启」按钮（如果可用）

### 恢复备份

如果修改配置后服务器出现问题：

1. 进入服务器存档目录：
   ```
   Servers/[存档名]/Config.txt.backup
   ```

2. 将 `Config.txt.backup` 重命名为 `Config.txt`

3. 重启服务器

::: tip 提示
每次保存配置时，编辑器都会自动创建 `.backup` 备份，保留最近一次的旧配置。
:::

## 注意事项

### 修改前备份

- ✅ 编辑器自动创建 `.backup` 备份
- ✅ 建议手动复制一份到安全位置
- ✅ 重要配置修改前截图记录原值

### 配置生效规则

| 配置类型 | 是否需要重启 | 说明 |
| --- | --- | --- |
| 服务器名称、地图 | ✅ 需要 | 必须重启服务器 |
| 端口、密码 | ✅ 需要 | 必须重启服务器 |
| 游戏规则（PvP、难度） | ✅ 需要 | 必须重启服务器 |
| 最大玩家数 | ✅ 需要 | 必须重启服务器 |
| 大部分高级选项 | ✅ 需要 | 必须重启服务器 |

::: warning 重要
Config.txt 的几乎所有配置都需要重启服务器才能生效。修改后请及时重启。
:::

### 配置验证规则

编辑器会自动验证以下规则：
- **端口范围**：1024-65535
- **玩家数**：1-200
- **刷新率**：10-200
- **超时时间**：10-300
- **地图名称**：必须是有效的地图名
- **游戏模式**：必须是 `Easy`、`Normal`、`Hard`、`Gold` 之一

## 常见配置场景

### PvE 休闲服务器

适合新手玩家和休闲建造：

```
Name = "PvE 休闲服 - 欢迎新手"
Map = "PEI"
Port = 27015
Password = （留空，公开服务器）
Mode = "Easy"
Max_Players = 24
PvP = false
Difficulty = "Easy"
Cheats = false
Perspective = "Both"
Tick_Rate = 50
```

### PvP 竞技服务器

适合竞技玩家和团队对战：

```
Name = "PvP 竞技服 - 高手对决"
Map = "Russia"
Port = 27015
Password = （留空或设置密码）
Mode = "Hard"
Max_Players = 48
PvP = true
Difficulty = "Hard"
Cheats = false
Perspective = "First"
Tick_Rate = 100
```

### 高性能优化配置

适合高配服务器，追求流畅体验：

```
Max_Players = 48
Max_Vehicles = 32
Tick_Rate = 100
Timeout = 60
Queue_Size = 20
Sync_Max_Zombies = 128
Sync_Max_Animals = 64
```

### 低配优化配置

适合低配服务器，减少资源占用：

```
Max_Players = 12
Max_Vehicles = 8
Tick_Rate = 30
Timeout = 30
Queue_Size = 4
Sync_Max_Zombies = 32
Sync_Max_Animals = 16
```

## 与基础配置的关系

### 基础配置 vs Config.txt

存档配置页面有两个配置入口：

1. **基础配置标签页**：
   - 常用配置的简化界面
   - 包括服务器名称、地图、端口、GSLT 等
   - 适合日常使用

2. **Config.txt 编辑器**：
   - 完整的高级配置选项
   - 包括性能调优、游戏规则、高级特性
   - 适合深度定制

### 优先级说明

- 两个界面编辑的是**同一个** `Config.txt` 文件
- 基础配置中修改的选项会同步到 Config.txt 编辑器
- Config.txt 编辑器中的修改也会反映在基础配置中
- **没有优先级冲突**：它们是同一配置的不同视图

### 配置冲突处理

如果同时修改基础配置和 Config.txt：
1. 最后保存的修改会覆盖之前的修改
2. 建议只在一个界面中完成所有修改后再保存
3. 避免同时打开两个标签页分别修改

## 常见问题

### Config.txt 在哪里？

配置文件位于：
```
Servers/[存档名]/Config.txt
```

例如，存档名为 `MyServer` 的配置文件在：
```
Servers/MyServer/Config.txt
```

### 修改配置后服务器无法启动怎么办？

1. **查看错误日志**：
   - 打开「日志中心」查看游戏日志
   - 查找配置相关的错误提示

2. **恢复备份**：
   - 将 `Config.txt.backup` 重命名为 `Config.txt`
   - 重启服务器

3. **重置为默认配置**：
   - 删除 `Config.txt`
   - 服务器启动时会自动生成默认配置

### 有些配置项找不到怎么办？

Config.txt 编辑器只显示常用配置项。如果需要修改未列出的高级选项：

1. **手动编辑 Config.txt**：
   - 使用文本编辑器（如 VS Code、Notepad++）打开
   - 参考 Unturned 官方文档查找配置项

2. **常见未列出的配置**：
   - VAC 安全：`VAC_Secure=true`
   - BattlEye：`BattlEye_Secure=true`
   - 工作线程：`Gameplay_Config_Overrides`

### 如何手动编辑 Config.txt？

如果您更喜欢手动编辑：

1. 使用文本编辑器打开 `Config.txt`
2. 每行一个配置项，格式为 `键=值`
3. 使用 UTF-8 编码保存
4. 重启服务器

**示例**：
```
Name=My Server
Map=PEI
Port=27015
Max_Players=24
PvP=true
Difficulty=Normal
```

::: warning 注意
手动编辑时务必注意：
- 不要修改配置项名称（等号左边）
- 布尔值使用 `true` 或 `false`（小写）
- 字符串值不需要引号
- 数字值直接写数字
:::

### Config.txt 和 Commands.dat 有什么区别？

| 文件 | 用途 | 配置内容 |
| --- | --- | --- |
| `Config.txt` | 服务器核心配置 | 服务器名称、地图、端口、游戏规则、性能参数 |
| `Commands.dat` | 自动执行命令 | 服务器启动时自动执行的控制台命令 |

两者互补：
- **Config.txt**：服务器启动参数和游戏规则
- **Commands.dat**：启动后执行的管理命令（如欢迎消息、定时通知）

## 相关功能

- [存档配置](/features/save)：Config.txt 编辑器整合在存档配置页面中
- [服务器控制](/features/server)：修改配置后需要在此页面重启服务器
- [日志中心](/features/logs)：查看配置错误和服务器启动日志
- [首次引导](/features/wizard)：首次设置时的配置向导
