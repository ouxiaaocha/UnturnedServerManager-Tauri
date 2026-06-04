import { defineConfig } from 'vitepress'

export default defineConfig({
  lang: 'zh-CN',
  title: 'Unturned Server Manager',
  description: '清新、轻量、便携的 Unturned 专用服务器管理工具',

  // GitHub Pages 部署路径（自定义域名使用根路径）
  base: '/',

  // 主题配置
  themeConfig: {
    // 导航栏
    nav: [
      { text: '首页', link: '/' },
      { text: '使用指南', link: '/guide/' },
      { text: '功能介绍', link: '/features/dashboard' },
      { text: '常见问题', link: '/faq' },
      { text: '更新日志', link: '/changelog' },
    ],

    // 侧边栏
    sidebar: {
      '/guide/': [
        {
          text: '使用指南',
          items: [
            { text: '项目介绍', link: '/guide/' },
            { text: '快速开始', link: '/guide/getting-started' },
            { text: '安装说明', link: '/guide/installation' },
            { text: '便携版说明', link: '/guide/portable' },
          ],
        },
      ],
      '/features/': [
        {
          text: '功能介绍',
          items: [
            { text: '仪表盘', link: '/features/dashboard' },
            { text: '服务器控制', link: '/features/server' },
            { text: 'RCON 控制台', link: '/features/rcon' },
            { text: '存档配置', link: '/features/save' },
            { text: '创意工坊模组', link: '/features/workshop' },
            { text: '插件管理', link: '/features/plugins' },
            { text: '定时任务', link: '/features/schedule' },
            { text: '服务端更新', link: '/features/update' },
            { text: '日志中心', link: '/features/logs' },
            { text: '首次引导', link: '/features/wizard' },
          ],
        },
      ],
    },

    // 社交链接
    socialLinks: [
      { icon: 'github', link: 'https://github.com/ouxiaaocha/UnturnedServerManager-Tauri' },
    ],

    // 页脚
    footer: {
      message: '基于 Tauri v2 + Svelte 5 + Rust 构建',
      copyright: '© 2024 Unturned Server Manager',
    },

    // 搜索
    search: {
      provider: 'local',
      options: {
        translations: {
          button: { buttonText: '搜索文档', buttonAriaLabel: '搜索文档' },
          modal: {
            noResultsText: '无法找到相关结果',
            resetButtonTitle: '清除查询条件',
            footer: { selectText: '选择', navigateText: '切换', closeText: '关闭' },
          },
        },
      },
    },

    // 编辑链接
    editLink: {
      pattern: 'https://github.com/ouxiaaocha/UnturnedServerManager-Tauri/edit/main/UnturnedServerManager-web/docs/:path',
      text: '在 GitHub 上编辑此页面',
    },

    // 最后更新时间
    lastUpdated: {
      text: '最后更新于',
    },

    // 文档页脚导航
    docFooter: {
      prev: '上一页',
      next: '下一页',
    },

    // 大纲标题
    outlineTitle: '页面导航',
    outline: [2, 3],

    // 返回顶部
    returnToTopLabel: '回到顶部',

    // 侧边栏菜单标签
    sidebarMenuLabel: '菜单',

    // 深色模式切换
    darkModeSwitchLabel: '主题',
    lightModeSwitchTitle: '切换到浅色模式',
    darkModeSwitchTitle: '切换到深色模式',
  },

  // Markdown 配置
  markdown: {
    lineNumbers: true,
  },

  // 头部信息
  head: [
    ['meta', { name: 'theme-color', content: '#43aa8b' }],
    ['meta', { property: 'og:type', content: 'website' }],
    ['meta', { property: 'og:title', content: 'Unturned Server Manager' }],
    ['meta', { property: 'og:description', content: '清新、轻量、便携的 Unturned 专用服务器管理工具' }],
  ],
})
