<FeatureHero
  icon="/icon-permissions.svg"
  eyebrow="Permissions"
  title="权限管理"
  description="权限标签可视化编辑 Permissions.config.xml，适合维护 Rocket 权限组、玩家 SteamID64、父组继承、优先级和权限节点。"
  image="/save.png"
  imageAlt="权限管理入口界面"
  pills="权限组,SteamID64,父组继承,权限节点,颜色标记"
/>

## 管理对象

| 对象 | 说明 |
| --- | --- |
| 权限组 | 组 ID、显示名、前缀、后缀、颜色、优先级 |
| 成员 | SteamID64，可批量粘贴并跳过重复项 |
| 权限节点 | 权限名和冷却时间 |
| 继承关系 | 选择父组，构建管理员、VIP、默认玩家层级 |

## 典型场景

<div class="doc-card-grid">
  <div class="doc-card"><h3>管理员组</h3><p>创建 admin 组，添加管理员 SteamID64 和管理命令权限。</p></div>
  <div class="doc-card"><h3>VIP 组</h3><p>设置展示前缀、颜色和插件提供的额外权限节点。</p></div>
  <div class="doc-card"><h3>默认玩家</h3><p>维护 default 组，避免误删默认权限组导致权限异常。</p></div>
</div>

::: warning 注意
权限文件不存在时，需要先初始化 Rocket 或运行一次服务端生成对应目录。
:::
