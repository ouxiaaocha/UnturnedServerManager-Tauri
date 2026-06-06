<script setup>
import iconServer from '/icon-server.svg'
</script>

<div class="feature-header">
  <img :src="iconServer" alt="服务器控制" />
  <h1>服务器控制</h1>
</div>

<div class="feature-screenshot">
  <img src="/server.png" alt="服务器控制界面" />
</div>

服务器控制页面提供对 Unturned 服务器的完整控制能力，包括启停操作、本地命令发送和实时日志查看。

## 服务器操作

| 操作 | 说明 |
| --- | --- |
| 启动 | 启动 Unturned 服务器进程 |
| 停止 | 通过本地命令 Bridge 发送保存和关闭命令 |
| 重启 | 优雅关闭后重新启动 |
| 强制停止 | 立即终止进程，不等待数据保存 |

## 本地命令

运行时可输入 Unturned 控制台命令，例如：

```text
Save
Say hello
Shutdown
```

命令写入本地队列文件，由 `UnturnedServerManagerBridge.dll` 在服务器内执行。不走 RCON，不监听公网端口。

## 实时输出

- 自动滚动到最新日志
- 日志搜索功能
- 按级别着色显示（error/warning/info/system）
- 命令发送后追加 `[命令] > ...` 记录

## 运行模式

- **局域网模式**：仅局域网玩家可连接
- **互联网模式**：公网玩家可连接（需配置端口转发）

::: warning 注意
切换运行模式需要重启服务器才能生效。
:::

## 故障排查

- 日志搜索定位错误信息
- 查看启动过程中的警告和错误
- 强制停止卡死的进程