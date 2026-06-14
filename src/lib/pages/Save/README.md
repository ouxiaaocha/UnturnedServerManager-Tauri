# Save.svelte 重构指南

## 📋 概述

Save.svelte 原始文件有 **1703 行**，包含 4 个主要功能模块。本文档说明如何将其拆分为可维护的子组件。

## 🎯 拆分策略

### 原始结构分析
```
Save.svelte (1703 行)
├─ 脚本部分 (1-906 行)
│  ├─ 类型定义
│  ├─ 状态管理
│  └─ 业务逻辑函数
└─ 模板部分 (907-1703 行)
   ├─ 存档配置标签 (907-1080, 173行)
   ├─ Workshop 标签 (1080-1273, 193行)
   ├─ 权限管理标签 (1273-1636, 363行)
   └─ 插件管理标签 (1636-1703, 67行)
```

### 拆分后的文件结构
```
src/lib/pages/Save/
├─ types.ts                 ✅ 已创建 - 共享类型定义
├─ SaveConfigTab.svelte     ✅ 已创建 - 存档配置表单
├─ WorkshopTab.svelte       ⚠️ 待创建 - Workshop 管理
├─ PermissionsTab.svelte    ⚠️ 待创建 - 权限管理
├─ PluginsTab.svelte        ⚠️ 待创建 - 插件管理
└─ hooks.svelte.ts          ⚠️ 待创建 - 共享业务逻辑
```

## ✅ 已完成的工作

### 1. types.ts - 类型定义
已提取所有共享类型：
- `PermissionEntry`
- `PermissionGroup`
- `PermissionsConfig`
- `SaveInfo`
- `WorkshopItem`
- `PluginInfo`

### 2. SaveConfigTab.svelte - 存档配置组件
**功能：**
- Commands.dat 配置表单
- RCON 配置表单
- 密码显示/隐藏切换
- 密码自动生成

**改进：**
- ✅ 所有表单字段添加 `<label>` 和 `id` 关联
- ✅ 加载状态添加 ARIA 支持
- ✅ 开关按钮使用 `role="switch"`
- ✅ 所有图标添加 `aria-hidden="true"`
- ✅ 使用 `$bindable` 实现双向绑定

**使用示例：**
```svelte
<script>
import SaveConfigTab from './Save/SaveConfigTab.svelte';

let cmdName = $state("");
let cmdMap = $state("");
// ... 其他状态
</script>

<SaveConfigTab
  bind:cmdName
  bind:cmdMap
  bind:cmdPort
  bind:rconPassword
  loading={isLoading}
/>
```

## 🔨 继续拆分指南

### 剩余组件拆分建议

#### WorkshopTab.svelte (193 行)
**功能：**
- Workshop 项目列表
- 添加/删除 Workshop 项目
- Workshop ID 输入

**需要的 props：**
```typescript
{
  workshopItems: WorkshopItem[] (bindable)
  loading: boolean
  onAdd: (id: string) => Promise<void>
  onRemove: (id: string) => Promise<void>
}
```

#### PermissionsTab.svelte (363 行)
**功能：**
- 权限组列表
- 权限组编辑
- 成员管理

**需要的 props：**
```typescript
{
  config: PermissionsConfig (bindable)
  loading: boolean
  onSave: () => Promise<void>
  onRefresh: () => Promise<void>
}
```

**进一步拆分建议：**
由于权限标签最大（363行），可以再拆分为：
- `PermissionGroupList.svelte` - 权限组列表
- `PermissionGroupEditor.svelte` - 权限组编辑器
- `PermissionMemberList.svelte` - 成员列表

#### PluginsTab.svelte (67 行)
**功能：**
- 插件列表
- 安装/卸载插件

**需要的 props：**
```typescript
{
  plugins: PluginInfo[] (bindable)
  loading: boolean
  onInstall: (name: string) => Promise<void>
  onUninstall: (name: string) => Promise<void>
}
```

### hooks.svelte.ts - 共享逻辑
将重复的业务逻辑提取为 composable 函数：

```typescript
// 存档选择器
export function useSaveSelector() {
  let saves = $state<SaveInfo[]>([]);
  let selectedId = $state("");
  
  async function loadSaves() { /* ... */ }
  async function selectSave(id: string) { /* ... */ }
  
  return {
    get saves() { return saves; },
    get selectedId() { return selectedId; },
    loadSaves,
    selectSave
  };
}

// Workshop 管理
export function useWorkshop(saveId: string) {
  let items = $state<WorkshopItem[]>([]);
  let loading = $state(false);
  
  async function addItem(id: string) { /* ... */ }
  async function removeItem(id: string) { /* ... */ }
  
  return { items, loading, addItem, removeItem };
}

// 权限管理
export function usePermissions(saveId: string) {
  let config = $state<PermissionsConfig | null>(null);
  let loading = $state(false);
  
  async function load() { /* ... */ }
  async function save() { /* ... */ }
  
  return { config, loading, load, save };
}
```

## 📊 重构后的 Save.svelte 主文件

重构后的主文件应该只有 **~200 行**，负责：
1. 标签页切换逻辑
2. 协调各子组件
3. 保存和加载数据

```svelte
<script lang="ts">
import { invoke } from "@tauri-apps/api/core";
import SaveConfigTab from "./Save/SaveConfigTab.svelte";
import WorkshopTab from "./Save/WorkshopTab.svelte";
import PermissionsTab from "./Save/PermissionsTab.svelte";
import PluginsTab from "./Save/PluginsTab.svelte";
import { useSaveSelector, useWorkshop, usePermissions } from "./Save/hooks.svelte";

let activeTab = $state("save");

// 存档选择
const saveSelector = useSaveSelector();

// 各标签的状态
let cmdName = $state("");
let cmdMap = $state("");
// ... 其他配置状态

const workshop = useWorkshop(saveSelector.selectedId);
const permissions = usePermissions(saveSelector.selectedId);

// 保存函数
async function save() {
  await invoke("save_config", { /* ... */ });
}

$effect(() => {
  if (saveSelector.selectedId) {
    loadSaveData(saveSelector.selectedId);
  }
});
</script>

<div class="container">
  <!-- 存档选择器 -->
  <SaveSelector bind:selectedId={saveSelector.selectedId} saves={saveSelector.saves} />

  <!-- 标签导航 -->
  <div class="tabs">
    <button onclick={() => activeTab = 'save'}>存档配置</button>
    <button onclick={() => activeTab = 'workshop'}>Workshop</button>
    <button onclick={() => activeTab = 'permissions'}>权限</button>
    <button onclick={() => activeTab = 'plugins'}>插件</button>
  </div>

  <!-- 标签内容 -->
  {#if activeTab === 'save'}
    <SaveConfigTab
      bind:cmdName
      bind:cmdMap
      bind:cmdPort
      bind:rconPassword
      loading={false}
    />
  {:else if activeTab === 'workshop'}
    <WorkshopTab
      bind:items={workshop.items}
      loading={workshop.loading}
      onAdd={workshop.addItem}
      onRemove={workshop.removeItem}
    />
  {:else if activeTab === 'permissions'}
    <PermissionsTab
      bind:config={permissions.config}
      loading={permissions.loading}
      onSave={permissions.save}
    />
  {:else if activeTab === 'plugins'}
    <PluginsTab plugins={[]} loading={false} />
  {/if}

  <!-- 保存按钮 -->
  <button onclick={save}>保存配置</button>
</div>
```

## 🎯 重构收益

### 代码可维护性
- ✅ 从 1703 行拆分为 5-7 个 < 400 行的文件
- ✅ 每个组件职责单一，易于理解
- ✅ 类型定义集中管理

### 代码复用
- ✅ 表单字段可以跨标签复用
- ✅ 业务逻辑通过 hooks 复用
- ✅ ARIA 和可访问性模式一致

### 开发体验
- ✅ 更快的文件加载和编辑
- ✅ 更精确的错误定位
- ✅ 更容易的代码审查

### 性能
- ✅ 按需加载子组件
- ✅ 更细粒度的响应式更新
- ✅ 减少不必要的重渲染

## 📝 后续步骤

1. **立即可做：** 在主 Save.svelte 中导入并使用 SaveConfigTab
2. **短期：** 创建 WorkshopTab 和 PluginsTab（较简单）
3. **中期：** 拆分 PermissionsTab（最复杂，可能需要再拆分）
4. **长期：** 提取共享逻辑到 hooks.svelte.ts

## 💡 注意事项

1. **渐进式重构：** 一次拆分一个标签，每次都保证功能正常
2. **保持向后兼容：** 使用 `$bindable` 确保状态同步
3. **测试驱动：** 拆分后立即测试该标签的所有功能
4. **文档同步：** 更新每个组件的 props 和事件文档

## ✅ 已优化的最佳实践

所有新组件都应遵循以下模式：
- ✅ 表单字段使用 `<label>` + `id` 关联
- ✅ 图标添加 `aria-hidden="true"`
- ✅ 按钮添加 `aria-label`
- ✅ 加载状态使用 `role="status"` + `aria-live="polite"`
- ✅ 开关使用 `role="switch"` + `aria-checked`
- ✅ 使用 `.sr-only` 类提供屏幕阅读器文本

---

**文件创建时间：** 2026-06-14  
**当前进度：** SaveConfigTab ✅ | 其他组件待创建
