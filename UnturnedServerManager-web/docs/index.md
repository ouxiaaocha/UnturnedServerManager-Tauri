---
layout: home

hero:
  name: "Unturned Server Manager"
  text: "清新 · 轻量 · 便携"
  tagline: 基于 Tauri v2 + Svelte 5 + Rust 构建的 Unturned 专用服务器管理工具，把服务端启动、RCON、存档、创意工坊模组、插件、更新、日志和定时任务集中到一个现代化桌面面板里。
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
  - icon: 🖥️
    title: 仪表盘监控
    details: 实时查看服务器状态、PID、运行时间、CPU、内存、网络流量，支持快速启动、停止、重启操作。
    link: /features/dashboard
  - icon: 🎮
    title: 服务器控制
    details: 一键启动、停止、重启、强制停止，支持实时日志输出、日志搜索、局域网/互联网模式切换。
    link: /features/server
  - icon: 🔧
    title: RCON 远程管理
    details: 连接 Rocket RCON 远程管理服务器，发送命令并接收响应，服务器启动后可自动连接。
    link: /features/rcon
  - icon: 📦
    title: 创意工坊模组
    details: 维护 WorkshopDownloadConfig.json，管理模组 ID、备注、缓存下载、更新监控和关服提示。
    link: /features/workshop
  - icon: ⏰
    title: 定时任务
    details: 创建每日、每周、间隔型自动重启任务，支持提前提醒，保障服务器稳定运行。
    link: /features/schedule
  - icon: 📋
    title: 日志中心
    details: 查看软件日志、操作日志和游戏日志，支持日期切换、分类筛选和关键词搜索。
    link: /features/logs
  - icon: 💾
    title: 存档配置
    details: 管理 Commands.dat、Rocket RCON、PvE、作弊、GSLT、地图、端口和最大玩家数等配置。
    link: /features/save
  - icon: 🚀
    title: 首次引导
    details: 自动检测/下载 SteamCMD，安装 Rocket 模块，初始化存档和 RCON，开箱即用。
    link: /features/wizard
---
