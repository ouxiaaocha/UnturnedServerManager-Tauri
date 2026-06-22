<FeatureHero
  icon="/icon-rcon.svg"
  eyebrow="Remote Console"
  title="RCON 控制台"
  description="RCON 控制台是额外远程管理能力。它可以连接当前运行存档的 Rocket RCON，发送命令并轮询响应，但日常启停控制优先使用本地命令 Bridge。"
  image="/rcon.png"
  imageAlt="RCON 控制台界面"
  pills="Rocket RCON,连接管理,响应轮询,心跳保活,命令历史"
/>

## 定位

RCON 适合远程命令和补充管理，不建议作为公网开放的主要入口。桌面端本地启停、重启和定时任务优先走本地命令 Bridge。

## 使用流程

1. 在「存档 > 基础配置」中设置 RCON 端口和强密码。
2. 启动目标存档服务器。
3. 在 RCON 页面选择运行中的目标服务器。
4. 连接后输入命令并查看响应。

## 安全建议

| 项目 | 建议 |
| --- | --- |
| 密码 | 使用随机强密码，避免简单口令 |
| 端口 | 不建议对公网开放 RCON 入站 |
| 防火墙 | 游戏端口按需放行，RCON 端口只允许可信来源 |
| 日常控制 | 优先使用本地命令 Bridge |
