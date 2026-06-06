<script setup>
import iconWorkshop from '/icon-workshop.svg'
</script>

<div class="feature-header">
  <img :src="iconWorkshop" alt="创意工坊" />
  <h1>创意工坊模组</h1>
</div>

<div class="feature-screenshot">
  <img src="/workshop.png" alt="创意工坊模组配置界面" />
</div>

创意工坊模组页面管理 Unturned 服务器的 Steam Workshop 模组，维护 `WorkshopDownloadConfig.json`。

## 模组管理

- **添加模组**：输入创意工坊模组 ID
- **删除模组**：移除不需要的模组
- **批量操作**：逗号分隔批量添加

## 模组备注

为每个模组添加自定义备注：

- 模组名称
- 用途说明
- 版本信息

## 缓存下载

- 缓存下载模组文件
- 减少服务器启动时的下载时间
- 更新监控检测新版本

## 添加模组流程

1. 在 Steam 创意工坊找到目标模组
2. 复制模组 ID（URL 中的数字部分）
3. 点击"添加模组"，粘贴 ID
4. 确认添加

::: tip 提示
模组 ID 在 URL 中，如 `https://steamcommunity.com/sharedfiles/filedetails/?id=1234567890` 里的 `1234567890`。
:::

## 高级配置

- 缓存时间、重试次数
- 关服倒计时
- 忽略子项 ID
- 缓存下载开关
- 监控更新开关

## 关服提示

服务器关闭时显示关服提示，提醒玩家服务器已关闭。