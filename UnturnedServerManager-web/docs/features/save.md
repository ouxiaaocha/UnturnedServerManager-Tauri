<FeatureHero
  icon="/icon-save.svg"
  eyebrow="Save Workspace"
  title="存档配置"
  description="存档配置页是服务器配置的主工作台。当前标签包括基础配置、高级配置、工作坊、插件和权限，RCON 端口与密码归入基础配置。"
  image="/save.png"
  imageAlt="存档配置界面"
  pills="基础配置,高级配置,工作坊,插件,权限,RCON"
/>

## 标签结构

<div class="doc-card-grid">
  <div class="doc-card"><h3>基础配置</h3><p>编辑服务器名称、地图、端口、最大玩家数、GSLT、游戏规则和 RCON 配置。</p></div>
  <div class="doc-card"><h3>高级配置</h3><p>可视化编辑 Config.txt，适合调整难度、性能和游戏规则细项。</p></div>
  <div class="doc-card"><h3>工作坊</h3><p>维护 WorkshopDownloadConfig.json、模组 ID、缓存下载和更新监控。</p></div>
  <div class="doc-card"><h3>插件</h3><p>查看 Rocket 插件目录，保存插件备注，快速打开插件配置目录。</p></div>
  <div class="doc-card"><h3>权限</h3><p>编辑 Permissions.config.xml，管理权限组、成员 SteamID64 和权限节点。</p></div>
  <div class="doc-card"><h3>运行保护</h3><p>存档正在运行时，关键配置会限制修改，降低误操作风险。</p></div>
</div>

## 基础配置包含什么

| 配置 | 说明 |
| --- | --- |
| 服务器信息 | 名称、地图、端口、最大玩家数、密码、Owner、GSLT |
| 游戏规则 | PvE、作弊、视角限制等 Commands.dat 常用项 |
| RCON 配置 | Rocket RCON 端口和密码，支持随机密码生成 |
| 存档初始化 | 新建存档、初始化 Rocket、检测存档运行状态 |

## 推荐流程

1. 新建或选择存档。
2. 在基础配置里设置地图、端口、人数、GSLT 和 RCON。
3. 在高级配置里确认 Config.txt 细项。
4. 按需添加工作坊模组、插件备注和权限组。
5. 保存后到「服务器」或「仪表盘」启动。

::: tip 提示
端口冲突会影响多服务器同时运行。启动时如检测到冲突，应用会引导你手动调整或自动分配端口。
:::
