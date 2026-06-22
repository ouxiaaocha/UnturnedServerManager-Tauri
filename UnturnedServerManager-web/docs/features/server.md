<FeatureHero
  icon="/icon-server.svg"
  eyebrow="Server Control"
  title="服务器控制"
  description="服务器控制页负责启动、停止、重启、强制停止和本地命令输入。它把实时输出、日志搜索、运行模式和命令发送放在同一个工作区。"
  image="/server.png"
  imageAlt="服务器控制界面"
  pills="本地命令 Bridge,实时输出,日志搜索,互联网/局域网模式,强制停止"
/>

## 主要能力

<div class="doc-card-grid">
  <div class="doc-card"><h3>启动控制</h3><p>按存档启动服务器，支持互联网和局域网两种启动模式。</p></div>
  <div class="doc-card"><h3>本地命令</h3><p>命令写入本地队列，由 Rocket Bridge 插件在服务器内执行。</p></div>
  <div class="doc-card"><h3>输出追踪</h3><p>实时显示服务端输出，支持搜索、清空和快速定位异常。</p></div>
</div>

## 本地命令 Bridge

本地命令 Bridge 是当前主控制路径。它不要求把 RCON 暴露到公网，适合本机桌面管理场景。

| 动作 | 说明 |
| --- | --- |
| 发送命令 | 输入 Unturned 控制台命令后写入本地队列 |
| 停止服务器 | 优先使用本地命令优雅关闭 |
| 重启服务器 | 停止后按当前存档和模式重新启动 |
| 环境检测 | 如果发送失败，可在设置页检查 Rocket 和 Bridge DLL |

::: warning 注意
如果服务器正在运行，不建议直接手动改存档配置。先停止服务器，再修改端口、地图、RCON 或高级配置。
:::
