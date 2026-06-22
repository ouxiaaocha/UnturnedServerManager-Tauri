---
layout: home

hero:
  name: "Unturned Server Manager"
  text: "清新 · 轻量 · 便携"
  tagline: 基于 Tauri v2 + Svelte 5 + Rust 构建的 Unturned 专用服务器管理工具，让服务端启动、本地命令控制、RCON、存档、创意工坊模组、插件、更新、日志和定时任务一站搞定。
  image:
    src: /hero.png
    alt: Unturned Server Manager
  actions:
    - theme: brand
      text: 快速开始
      link: /guide/getting-started
    - theme: alt
      text: 功能介绍
      link: /features/dashboard
    - theme: alt
      text: GitHub
      link: https://github.com/ouxiaaocha/UnturnedServerManager-Tauri

features:
  - icon: <img src='/icon-dashboard.svg' />
    title: 仪表盘监控
    details: 实时查看服务器状态、PID、运行时间、CPU、内存、网络流量，支持通过本地命令快速启动、停止、重启。
    link: /features/dashboard
  - icon: <img src='/icon-server.svg' />
    title: 服务器控制
    details: 一键启动、停止、重启、强制停止，支持本地命令输入、实时日志输出、日志搜索、局域网/互联网模式切换。
    link: /features/server
  - icon: <img src='/icon-rcon.svg' />
    title: RCON 远程管理
    details: 优化的 RCON 连接管理，心跳保活机制，可连接 Rocket RCON 发送命令并接收响应。
    link: /features/rcon
  - icon: <img src='/icon-save.svg' />
    title: 存档配置
    details: 标签页布局整合基础配置、插件管理、工作坊模组、权限管理和 RCON 配置，统一管理入口。
    link: /features/save
  - icon: <img src='/icon-workshop.svg' />
    title: 创意工坊模组
    details: 维护 WorkshopDownloadConfig.json，管理模组 ID、备注、缓存下载，支持快速跳转 Steam 工作坊。
    link: /features/workshop
  - icon: <img src='/icon-plugins.svg' />
    title: 插件管理
    details: 查看 Rocket 插件目录，保存插件备注，快速打开插件配置目录。
    link: /features/plugins
  - icon: <img src='/icon-permissions.svg' />
    title: 权限管理
    details: 可视化编辑 Rocket 权限配置，管理权限组和玩家权限，支持权限继承。
    link: /features/permissions
  - icon: <img src='/icon-config.svg' />
    title: Config.txt 编辑器
    details: 可视化编辑服务器高级配置，配置验证和提示，一键应用并重启服务器。
    link: /features/config-editor
  - icon: <img src='/icon-schedule.svg' />
    title: 定时任务
    details: 创建每日、每周、间隔型自动重启任务，支持提前提醒，智能清理过期任务记录。
    link: /features/schedule
  - icon: <img src='/icon-update.svg' />
    title: 服务端更新
    details: 调用 SteamCMD 更新 Unturned 服务端，实时显示 SteamCMD 自更新、服务端校验和安装输出。
    link: /features/update
  - icon: <img src='/icon-logs.svg' />
    title: 日志中心
    details: 查看软件日志、操作日志和游戏日志，支持日期切换、分类筛选和关键词搜索。
    link: /features/logs
  - icon: <img src='/icon-window.svg' />
    title: 窗口管理
    details: 支持最小化到托盘、托盘菜单快速导航、关闭行为自定义，便捷的后台管理体验。
    link: /features/window
---

<section class="home-section" aria-label="核心能力">
  <div class="home-section-title">
    <img src="/icon-dashboard.svg" alt="" />
    核心能力
  </div>
  <p class="home-section-desc">面向长期运行的 Unturned 服务器，把日常维护动作压缩到少量高频入口。</p>
  <div class="ability-grid">
    <div class="ability-card ability-card-primary">
      <span class="ability-card-kicker">Server Operations</span>
      <strong>启动、停止、重启与日志观察集中在同一工作台</strong>
      <p>日常运维一键搞定，无需手动开终端。</p>
    </div>
    <div class="ability-card">
      <span class="ability-card-kicker">Local Bridge & Saves</span>
      <strong>本地命令、存档配置、插件目录快速衔接</strong>
      <p>常用控制命令走本地 Bridge，安全高效。</p>
    </div>
    <div class="ability-card">
      <span class="ability-card-kicker">Automation</span>
      <strong>更新、模组、定时任务统一托管</strong>
      <p>从手工操作变成可重复的管理流程。</p>
    </div>
  </div>
</section>

<section class="home-section" aria-label="界面预览">
  <div class="home-section-title">
    <img src="/icon-server.svg" alt="" />
    界面预览
  </div>
  <p class="home-section-desc">桌面端界面围绕状态确认、启动控制、日志追踪和配置维护组织，适合本机部署与便携目录。</p>
  <div class="showcase-grid">
    <a class="showcase-card showcase-card-wide" href="/features/dashboard.html">
      <img src="/dashboard.png" alt="仪表盘界面" />
      <span>仪表盘监控</span>
    </a>
    <a class="showcase-card" href="/features/server.html">
      <img src="/server.png" alt="服务器控制界面" />
      <span>服务器控制</span>
    </a>
    <a class="showcase-card" href="/features/rcon.html">
      <img src="/rcon.png" alt="RCON 控制台界面" />
      <span>RCON 控制台</span>
    </a>
    <a class="showcase-card" href="/features/save.html">
      <img src="/save.png" alt="存档配置界面" />
      <span>存档配置</span>
    </a>
    <a class="showcase-card" href="/features/workshop.html">
      <img src="/workshop.png" alt="创意工坊模组" />
      <span>创意工坊模组</span>
    </a>
  </div>
</section>