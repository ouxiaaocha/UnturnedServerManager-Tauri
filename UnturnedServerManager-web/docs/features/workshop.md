<FeatureHero
  icon="/icon-workshop.svg"
  eyebrow="Workshop"
  title="创意工坊模组"
  description="工作坊标签维护 WorkshopDownloadConfig.json，用于管理模组 ID、备注、缓存下载、更新监控和 Steam 工作坊跳转。"
  image="/workshop.png"
  imageAlt="创意工坊模组界面"
  pills="模组 ID,备注,缓存下载,更新监控,Steam 跳转"
/>

## 管理内容

<div class="doc-card-grid">
  <div class="doc-card"><h3>模组列表</h3><p>添加、删除、查看 file_id，并为常用模组写中文备注。</p></div>
  <div class="doc-card"><h3>缓存下载</h3><p>控制 use_cached_downloads、查询缓存时间和重试次数。</p></div>
  <div class="doc-card"><h3>更新监控</h3><p>配置模组更新后关服倒计时消息和踢出提示。</p></div>
</div>

## 常见操作

| 操作 | 说明 |
| --- | --- |
| 添加模组 | 输入 Steam 工作坊 ID，并可填写备注 |
| 打开工作坊 | 跳转到对应 Steam 页面查看说明和依赖 |
| 忽略子项 | 配置 ignore_children_file_ids |
| 保存配置 | 写回 WorkshopDownloadConfig.json |

::: tip 提示
如果客户端进服报 missing asset，除了检查模组 ID，也要检查服务端目录结构和客户端 workshop 缓存是否真正更新。
:::
