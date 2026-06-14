# Select 组件使用指南

## 📦 两种 Select 组件

项目现在提供两种下拉框组件，根据需求选择：

### 1. Select.svelte - 原生增强版

**文件：** `src/lib/components/Select.svelte`

**特点：**
- ✅ 基于原生 `<select>` 元素
- ✅ 轻量级，性能最佳
- ✅ 表单集成完美（自动支持 form submit）
- ✅ 键盘导航原生支持（↑↓ 键选择）
- ✅ 样式美化（箭头、阴影、聚焦效果）
- ⚠️ 下拉菜单样式受浏览器限制

**适用场景：**
- 表单中的下拉选择
- 需要与原生表单集成
- 选项数量较多（100+ 项）
- 需要最佳性能

**使用方式：**
```svelte
<script>
import Select from '$lib/components/Select.svelte';
</script>

<!-- 方式1: 使用 children -->
<Select bind:value={perspective} size="md" fullWidth>
  <option value="First">第一人称</option>
  <option value="Third">第三人称</option>
  <option value="Both">两者皆可</option>
</Select>

<!-- 方式2: 使用 options 数组 -->
<Select
  bind:value={selectedId}
  options={[
    { value: 'a', label: '选项 A' },
    { value: 'b', label: '选项 B', disabled: true }
  ]}
  size="md"
/>
```

---

### 2. SelectCustom.svelte - 完全自定义版 ⭐ 推荐用于主要选择器

**文件：** `src/lib/components/SelectCustom.svelte`

**特点：**
- ✅ 完全自定义的下拉菜单
- ✅ 精美的下拉动画（slideDown）
- ✅ 选中项带 ✓ 图标标记
- ✅ 悬停效果完全可控
- ✅ 自定义滚动条样式
- ✅ 点击外部自动关闭
- ⚠️ 不是原生表单元素（需手动处理 form）
- ⚠️ 选项过多时性能略低

**适用场景：**
- 主要的用户交互选择器（如存档选择）
- 需要精美视觉效果的场景
- 选项数量适中（< 50 项）
- 不需要原生表单提交

**使用方式：**
```svelte
<script>
import SelectCustom from '$lib/components/SelectCustom.svelte';

let selectedValue = $state('');
const options = [
  { value: 'save1', label: '存档 1' },
  { value: 'save2', label: '存档 2' },
  { value: 'save3', label: '存档 3', disabled: true }
];
</script>

<SelectCustom
  bind:value={selectedValue}
  options={options}
  placeholder="请选择存档"
  size="md"
  fullWidth
  onchange={(val) => console.log('选择了:', val)}
/>
```

---

## 🎨 视觉效果对比

### Select.svelte（原生增强）
```
✓ 自定义箭头图标
✓ 阴影效果
✓ 聚焦光环
✓ 箭头旋转动画
✗ 下拉菜单样式受浏览器限制
```

### SelectCustom.svelte（完全自定义）
```
✓ 自定义箭头图标
✓ 阴影效果
✓ 聚焦光环
✓ 箭头旋转动画
✓ 下拉菜单完全自定义
✓ 下拉展开动画
✓ 选中项 ✓ 图标
✓ 精美悬停效果
✓ 自定义滚动条
```

---

## 📝 尺寸选项

两个组件都支持 3 种尺寸：

```typescript
size="sm"  // 小尺寸 - text-xs, py-1.5, px-3
size="md"  // 中尺寸 - text-sm, py-2, px-4 (默认)
size="lg"  // 大尺寸 - text-base, py-2.5, px-4
```

---

## 🔧 已更新的文件

### 使用 SelectCustom（完全自定义）
- ✅ **SaveSelector.svelte** - 存档选择器（主要交互）

### 使用 Select（原生增强）
- ✅ **Save.svelte** - 视角选择、默认组、父组
- ✅ **SaveConfigTab.svelte** - 视角选择

---

## 💡 迁移建议

### 哪些应该迁移到 SelectCustom？

**优先迁移：**
1. ✅ **SaveSelector** - 已完成
2. ⚠️ **Save.svelte 中的视角选择** - 可选迁移（视觉效果更好）
3. ⚠️ **权限组选择** - 可选迁移（选项不多时）

**建议保持 Select：**
- 大量选项的下拉框（100+ 项）
- 嵌套在表单中的下拉框
- 对性能要求极高的场景

---

## 🎯 API 对比

### Select.svelte
```typescript
{
  value: any (bindable)
  options?: Array<{ value, label, disabled? }>
  placeholder?: string
  disabled?: boolean
  size?: 'sm' | 'md' | 'lg'
  fullWidth?: boolean
  class?: string
  onchange?: (e: Event) => void
  children?: any  // 支持 slot
}
```

### SelectCustom.svelte
```typescript
{
  value: any (bindable)
  options: Array<{ value, label, disabled? }>  // 必需
  placeholder?: string
  disabled?: boolean
  size?: 'sm' | 'md' | 'lg'
  fullWidth?: boolean
  class?: string
  onchange?: (value: any) => void  // 直接传值，不是 Event
}
```

**注意差异：**
- SelectCustom 的 `options` 是必需的
- SelectCustom 的 `onchange` 直接传递 value，不是 Event 对象
- SelectCustom 不支持 children slot

---

## 🚀 下一步

1. **测试 SaveSelector** - 确保自定义下拉框工作正常
2. **决定是否迁移其他下拉框** - 根据需求选择
3. **统一视觉风格** - 确定项目主要使用哪个组件

---

**建议：** 对于用户主要交互的下拉框（如存档选择），使用 SelectCustom 获得最佳视觉效果。对于次要的、选项多的下拉框，使用 Select 保持性能。
