# UnturnedServerManager-web 设计系统改造方案

基于 Impeccable 项目的设计理念，对现有文档网站进行视觉升级。

## 核心设计理念（来自 Impeccable）

### 1. 品牌寄存器（Brand Register）
- **设计即产品**：文档网站的视觉体验本身就是产品的一部分
- **反 AI Slop 测试**：避免让人觉得"AI 生成"的平庸设计
- **独特性高于一切**：让访问者问"这是如何制作的"，而不是"哪个 AI 做的"

### 2. 字体选择原则

#### 禁用列表（Training-data defaults）
以下字体已被过度使用，应避免：
- Fraunces, Newsreader, Lora, Crimson, Playfair Display, Cormorant
- Inter, DM Sans, Outfit, Plus Jakarta Sans, Instrument Sans
- IBM Plex Mono, Space Mono, Space Grotesk

#### 推荐字体策略
当前项目使用：
- 标题：Playfair Display（**需要更换**，属于禁用列表）
- 正文：Inter（**需要更换**，属于禁用列表）

**建议替换方案**：
1. **方案 A - 现代清新风格**
   - 标题：Work Sans / Lexend
   - 正文：Public Sans / System UI
   - 特点：清晰、现代、开源

2. **方案 B - 优雅衬线风格**
   - 标题：Lora 的替代品 - Eczar / Bitter
   - 正文：Source Sans 3
   - 特点：优雅但不过度使用

3. **方案 C - 极简系统字体**
   - 使用系统字体栈：-apple-system, BlinkMacSystemFont, "Segoe UI"
   - 优势：零加载时间，原生观感

### 3. 配色系统

#### 当前问题
- 主色：`#4cada8`（小清新绿色）- 可保留作为品牌色
- 背景系统过于简单：纯白 + 浅灰
- 缺少语义化色彩系统

#### Impeccable 的配色原则
1. **使用 OKLCH 色彩空间**（而非 HSL）
   - 感知均匀：相同的明度步进看起来是均匀的
   - 更精确的色彩控制

2. **色调中性灰（Tinted Neutrals）**
   ```css
   /* 错误：纯灰色 */
   --gray-100: hsl(0, 0%, 95%);
   
   /* 正确：品牌色调的灰色 */
   --gray-100: oklch(95% 0.008 180); /* 向品牌色倾斜 */
   ```

3. **60-30-10 法则**
   - 60%：中性背景、留白
   - 30%：文本、边框、非活跃状态
   - 10%：强调色、CTA、焦点状态

#### 建议配色方案
基于现有品牌色 `#4cada8` 构建完整色彩系统：

```css
:root {
  /* 主品牌色（保留并扩展） */
  --color-primary-50: oklch(96% 0.02 180);
  --color-primary-100: oklch(92% 0.04 180);
  --color-primary-500: oklch(65% 0.09 180); /* #4cada8 */
  --color-primary-600: oklch(55% 0.09 180);
  --color-primary-900: oklch(25% 0.05 180);
  
  /* 色调中性灰（向主品牌色倾斜） */
  --color-paper: oklch(99% 0.002 180);
  --color-cream: oklch(97% 0.005 180);
  --color-mist: oklch(90% 0.008 180);
  --color-ash: oklch(60% 0.008 180);
  --color-ink: oklch(15% 0.005 180);
  
  /* 语义色（使用 OKLCH） */
  --color-success: oklch(65% 0.15 145); /* 绿色 */
  --color-warning: oklch(70% 0.15 85);  /* 橙色 */
  --color-error: oklch(60% 0.20 25);    /* 红色 */
  --color-info: oklch(65% 0.15 250);    /* 蓝色 */
}
```

### 4. 排版系统

#### Impeccable 的排版原则
1. **模块化比例尺**：使用 ≥1.25 的比例
2. **流式排版**：使用 `clamp()` 实现响应式字号
3. **垂直节奏**：所有垂直间距基于行高的倍数

#### 建议排版系统
```css
:root {
  /* 字体家族 */
  --font-display: 'Work Sans', sans-serif;
  --font-body: 'Public Sans', -apple-system, system-ui, sans-serif;
  --font-mono: 'JetBrains Mono', 'Cascadia Mono', monospace;
  
  /* 流式字号（1.25 比例） */
  --text-xs: clamp(0.75rem, 0.7rem + 0.2vw, 0.875rem);
  --text-sm: clamp(0.875rem, 0.8rem + 0.3vw, 1rem);
  --text-base: clamp(1rem, 0.95rem + 0.25vw, 1.125rem);
  --text-lg: clamp(1.25rem, 1.1rem + 0.6vw, 1.5rem);
  --text-xl: clamp(1.5rem, 1.3rem + 1vw, 2rem);
  --text-2xl: clamp(2rem, 1.5rem + 2vw, 3rem);
  --text-3xl: clamp(2.5rem, 2rem + 2.5vw, 4rem);
  
  /* 行高 */
  --leading-tight: 1.1;
  --leading-normal: 1.5;
  --leading-relaxed: 1.7;
  
  /* 字重 */
  --font-light: 300;
  --font-normal: 400;
  --font-medium: 500;
  --font-semibold: 600;
  --font-bold: 700;
}
```

### 5. 间距系统

#### Impeccable 的间距原则
- 基于 8px 基准
- 使用 CSS 变量统一管理
- 垂直间距应该是行高的倍数

#### 建议间距系统
```css
:root {
  --spacing-xs: 0.5rem;   /* 8px */
  --spacing-sm: 1rem;     /* 16px */
  --spacing-md: 1.5rem;   /* 24px */
  --spacing-lg: 2rem;     /* 32px */
  --spacing-xl: 3rem;     /* 48px */
  --spacing-2xl: 5rem;    /* 80px */
  --spacing-3xl: 7.5rem;  /* 120px */
}
```

## 具体改造计划

### 阶段 1：基础系统升级（核心优先）

#### 1.1 色彩系统重构
**文件**：`docs/.vitepress/theme/styles/variables.css`（新建）

**内容**：
- 定义完整的 OKLCH 色彩系统
- 色调中性灰（向 #4cada8 倾斜）
- 语义色彩（成功、警告、错误、信息）
- 明暗模式支持

#### 1.2 字体系统升级
**文件**：`docs/.vitepress/config.ts`

**操作**：
- 移除 Playfair Display 和 Inter
- 引入新字体（通过 Google Fonts 或系统字体）
- 更新字体加载策略（font-display: swap）

**文件**：`docs/.vitepress/theme/styles/typography.css`（新建）

**内容**：
- 流式字号系统
- 字重定义
- 行高系统
- 字间距规则

#### 1.3 间距系统标准化
**文件**：`docs/.vitepress/theme/styles/spacing.css`（新建）

**内容**：
- 统一间距变量
- 替换现有硬编码的间距值

### 阶段 2：组件级优化

#### 2.1 Hero 区域重构
**当前问题**：
- 渐变背景过于简单
- 缺少视觉层次

**改进方向**：
- 添加微妙的网格/点阵背景
- 标题文字渐变动画
- Hero 图片悬浮效果

#### 2.2 Feature 卡片升级
**当前问题**：
- 图标尺寸偏小
- 悬停效果单一
- 视觉层次不够

**改进方向**：
- 图标尺寸增大（44px → 52px）
- 添加渐变背景
- 优化悬停动画（scale + rotate 微动效）
- 左侧 4px 色块装饰条

#### 2.3 导航栏增强
**改进方向**：
- 滚动时的收缩动画
- Logo 悬停旋转效果
- 导航项下划线动画指示器
- 搜索框优化（快捷键提示）

### 阶段 3：动画与交互

#### 3.1 页面过渡动画
```css
.VPContent {
  animation: fadeInUp 0.5s ease-out;
}

@keyframes fadeInUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}
```

#### 3.2 滚动触发动画
- Feature 卡片依次淡入
- 文档页面标题滑入
- 自定义区域（核心能力、界面预览）

#### 3.3 微交互
- 按钮点击水波纹效果
- 卡片悬停光泽移动
- 链接下划线展开动画

## 实施优先级

### P0（必须完成）
1. 字体系统替换
2. OKLCH 色彩系统
3. 间距系统标准化

### P1（强烈建议）
1. Hero 区域重构
2. Feature 卡片升级
3. 导航栏增强

### P2（时间允许）
1. 页面过渡动画
2. 滚动触发动画
3. 微交互优化

## 风险与注意事项

### 1. 浏览器兼容性
- OKLCH 需要较新的浏览器支持
- 提供 HSL 降级方案

### 2. 字体加载性能
- 使用 font-display: swap 避免 FOIT
- 预加载关键字重
- 考虑使用系统字体（零加载时间）

### 3. 视觉一致性
- 避免过度动画（干扰阅读）
- 保持品牌识别度（主色不变）
- 渐进式改造（不破坏现有功能）

## 验证清单

### 视觉验证
- [ ] 明暗模式切换无样式错乱
- [ ] 字体层次清晰可辨
- [ ] 色彩对比度符合 WCAG AA 标准
- [ ] 响应式布局在三端显示正常

### 性能验证
- [ ] 字体加载无明显闪烁（FOUT）
- [ ] 动画流畅无卡顿（60fps）
- [ ] Lighthouse 性能分数 ≥ 90

### 可访问性验证
- [ ] 键盘导航完整可用
- [ ] 屏幕阅读器正确读取
- [ ] 色盲模式友好

---

**总结**：此方案借鉴 Impeccable 的设计理念，在保留现有品牌色的基础上，通过字体升级、OKLCH 色彩系统、流式排版和精致动画，将文档网站提升到专业级水准。核心原则是"克制但有力"——避免过度设计，让每个设计决策都有明确目的。
