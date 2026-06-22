<FeatureHero
  icon="/icon-update.svg"
  eyebrow="SteamCMD"
  title="服务端更新"
  description="更新页调用 SteamCMD 安装或校验 Unturned Dedicated Server，实时显示 SteamCMD 自更新、服务端下载和校验输出。"
  image="/update.png"
  imageAlt="服务端更新界面"
  pills="SteamCMD,服务端校验,自动托管,实时输出"
/>

## 更新流程

1. 检查 SteamCMD 是否可用。
2. 调用 SteamCMD 更新 AppID `1110390` 对应的服务端。
3. 显示下载、校验和安装输出。
4. 更新完成后按需重启存档服务器。

## 和自动托管的关系

仪表盘可以切换自动更新托管。开启后，应用会把更新维护纳入长期运行流程，但仍建议在低峰时段执行重启。

::: warning 注意
更新服务端前，请确认目标服务器没有关键玩家活动。公开服建议提前公告。
:::
