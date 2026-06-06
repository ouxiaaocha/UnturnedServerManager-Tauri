<script setup>
import iconDashboard from '/icon-dashboard.svg'
</script>

<div class="feature-header">
  <img :src="iconDashboard" alt="仪表盘" />
  <h1>仪表盘</h1>
</div>

<div class="feature-screenshot">
  <img src="/dashboard.png" alt="仪表盘界面" />
</div>

仪表盘是 Unturned Server Manager 的核心监控页面，提供服务器运行状态的实时概览。

## 实时监控

仪表盘实时显示以下服务器信息：

| 指标 | 说明 |
| --- | --- |
| 运行状态 | 服务器是否正在运行 |
| PID | 服务器进程 ID |
| 运行时间 | 服务器已运行的时长 |
| CPU 使用率 | 服务器进程的 CPU 占用 |
| 内存使用 | 服务器占用的内存大小 |
| 网络流量 | 上传/下载流量 |

## 快速操作

- **启动**：启动 Unturned 服务器
- **停止**：通过本地命令保存并关闭服务器
- **重启**：等待旧进程退出后重新启动
- **强制停止**：强制终止服务器进程

::: tip 提示
建议优先使用"停止"或"重启"而非"强制停止"，以确保数据正确保存。
:::

## 界面布局

- 顶部：服务器运行状态 + 快速操作按钮
- 中部：CPU、内存、网络等性能指标卡片
- 底部：PID、运行时间、运行存档等基本信息