<script setup lang="ts">
const capabilities = [
  {
    title: '运行监控',
    text: '仪表盘集中显示状态、PID、运行时间、CPU、内存、网络流量和多服务器运行卡片。',
    icon: '/icon-dashboard.svg',
  },
  {
    title: '存档配置',
    text: '基础配置、高级配置、工作坊、插件、权限都在同一个存档工作台里完成。',
    icon: '/icon-save.svg',
  },
  {
    title: '本地命令 Bridge',
    text: '启动、停止、重启和控制台命令走本地队列，不把日常控制强绑到公网 RCON。',
    icon: '/icon-server.svg',
  },
  {
    title: '自动化托管',
    text: 'SteamCMD 更新、模组更新监控、定时重启和提前提醒面向长期运行场景。',
    icon: '/icon-schedule.svg',
  },
  {
    title: '安全便携',
    text: '打包后不开放 Web 管理端口，运行数据跟随 exe 目录，适合 Windows 服务器便携部署。',
    icon: '/icon-window.svg',
  },
]

const workflow = [
  ['01', '首次引导', '检测 SteamCMD、服务端目录、Rocket 和本地命令 Bridge。'],
  ['02', '配置存档', '设置地图、端口、GSLT、RCON、高级 Config.txt、插件和权限。'],
  ['03', '启动服务器', '选择互联网或局域网模式，查看实时输出、联机码和连接地址。'],
  ['04', '维护运行', '用日志中心、RCON、定时任务和更新页完成日常巡检。'],
]

const shots = [
  ['仪表盘', '/dashboard.png', '/features/dashboard.html'],
  ['服务器控制', '/server.png', '/features/server.html'],
  ['存档配置', '/save.png', '/features/save.html'],
  ['创意工坊', '/workshop.png', '/features/workshop.html'],
  ['RCON 控制台', '/rcon.png', '/features/rcon.html'],
  ['日志中心', '/logs.png', '/features/logs.html'],
]
</script>

<template>
  <main class="product-home">
    <section class="product-hero">
      <div class="product-hero-copy">
        <p class="product-eyebrow">Windows Portable · Tauri v2 · Svelte 5</p>
        <h1>Unturned Server Manager</h1>
        <p class="product-lede">
          面向 Unturned 专用服务器的桌面管理面板，把启动控制、本地命令、存档配置、模组、权限、更新、日志和定时维护放进一个轻量工作台。
        </p>
        <div class="product-actions">
          <a class="product-button product-button-primary" href="/guide/getting-started.html">快速开始</a>
          <a class="product-button" href="/features/dashboard.html">查看功能</a>
          <a class="product-button product-button-ghost" href="https://github.com/ouxiaaocha/UnturnedServerManager-Tauri">GitHub</a>
        </div>
        <div class="product-stats" aria-label="项目特性">
          <span><strong>桌面面板</strong> 本机可视化管理</span>
          <span><strong>Bridge</strong> 本地命令控制</span>
          <span><strong>Portable</strong> 便携目录运行</span>
        </div>
      </div>
      <figure class="product-hero-shot">
        <img src="/hero.png" alt="Unturned Server Manager 主界面" />
      </figure>
    </section>

    <section class="product-section">
      <div class="section-heading">
        <p>Core Capabilities</p>
        <h2>为长期运行的服务器减少手工操作</h2>
      </div>
      <div class="capability-grid">
        <article v-for="item in capabilities" :key="item.title" class="capability-card">
          <img :src="item.icon" alt="" />
          <h3>{{ item.title }}</h3>
          <p>{{ item.text }}</p>
        </article>
      </div>
    </section>

    <section class="product-section product-section-band">
      <div class="section-heading">
        <p>Workflow</p>
        <h2>从首次配置到日常维护，一条线跑完</h2>
      </div>
      <div class="workflow-list">
        <article v-for="[index, title, text] in workflow" :key="title" class="workflow-item">
          <span>{{ index }}</span>
          <div>
            <h3>{{ title }}</h3>
            <p>{{ text }}</p>
          </div>
        </article>
      </div>
    </section>

    <section class="product-section">
      <div class="section-heading">
        <p>Interface</p>
        <h2>界面预览</h2>
      </div>
      <div class="screenshot-gallery">
        <a v-for="[title, image, link] in shots" :key="title" class="screenshot-card" :href="link">
          <img :src="image" :alt="`${title}界面`" />
          <span>{{ title }}</span>
        </a>
      </div>
    </section>
  </main>
</template>
