# UnturnedServerManager-Tauri 代码审查报告

**审查日期：** 2026-06-03  
**项目版本：** 2.0.0  
**技术栈：** Tauri v2 + Svelte 5 + Rust + Tailwind CSS v4

---

## 目录

1. [项目概述](#项目概述)
2. [严重问题 Critical](#严重问题critical)
3. [高危问题 High](#高危问题high)
4. [中等问题 Medium](#中等问题medium)
5. [低级问题 Low](#低级问题low)
6. [积极发现](#积极发现)
7. [改进建议优先级](#改进建议优先级)

---

## 项目概述

UnturnedServerManager-Tauri 是一个用于管理 Unturned 游戏专用服务器的桌面应用程序。项目采用 Tauri v2 框架，后端使用 Rust 处理服务器进程管理、RCON 通信、配置读写等核心功能，前端使用 Svelte 5 构建管理界面。

项目结构：

- `src/` — Svelte 5 前端（App.svelte + 9 个页面组件 + store + utils）
- `src-tauri/src/` — Rust 后端（lib.rs + 9 个 command 模块 + 7 个 service 模块 + 4 个 model）
- 前端组件：Dashboard, Server, Save, Wizard, Rcon, Logs, Schedule, Settings, About
- 后端服务：ProcessManager, RconClient, ConfigService, Scheduler, LogService, SystemMonitor

---

## 严重问题 Critical

### C1. PowerShell 命令注入漏洞

**文件：** `src-tauri/src/commands/installer.rs` (lines 129-138)

SteamCMD 下载解压过程中，路径通过 `format!()` 直接拼接进 PowerShell 命令字符串，使用单引号包裹：

```rust
&format!(
    "Expand-Archive -Path '{}' -DestinationPath '{}' -Force",
    zip_path.display(),
    steamcmd_dir.display()
),
```

如果路径中包含单引号（如 `C:\Users\O'Brien\SteamCMD`），则可以突破字符串边界执行任意 PowerShell 命令。

**修复建议：** 改用 `zip` crate 解压（项目已引入该依赖），或使用 PowerShell 的 `-LiteralPath` 参数并对路径做转义处理。

---

### C2. 密码加密密钥推导方式极其薄弱

**文件：** `src-tauri/src/services/config_service.rs` (lines 43-56)

AES-256-GCM 加密密钥由硬编码盐值 `"UnturnedSM-"` 加上 `COMPUTERNAME` 和 `USERNAME` 环境变量推导而来。这些值在任意 Windows 机器上都极易获取——计算机名在网络中可见，用户名在进程列表和目录名中可见，盐值写在公开的源码中。任何了解这些信息的人都可以解密存储的 RCON 密码，AES-256-GCM 加密在此等于零安全效果。

```rust
fn machine_key() -> &'static [u8] {
    static KEY: OnceLock<Vec<u8>> = OnceLock::new();
    KEY.get_or_init(|| {
        let mut hasher = Sha256::new();
        hasher.update(b"UnturnedSM-");
        if let Ok(hostname) = std::env::var("COMPUTERNAME") {
            hasher.update(hostname.as_bytes());
        }
        if let Ok(username) = std::env::var("USERNAME") {
            hasher.update(username.as_bytes());
        }
        hasher.finalize().to_vec()
    })
}
```

**修复建议：** 使用 Windows DPAPI（数据保护 API）或 Windows Credential Manager 进行密钥管理，或至少使用随机生成的密钥存储于安全位置。

---

### C3. AES-GCM 固定零Nonce向后兼容（密码学灾难）

**文件：** `src-tauri/src/services/config_service.rs` (lines 89-101)

向后兼容代码中使用固定零Nonce解密旧格式密码。AES-GCM 重用Nonce会彻底破坏所有保密性保证——攻击者可恢复任意两个密文的 XOR，伪造认证密文，恢复认证密钥。

```rust
let nonce_bytes = [0u8; 12];
let nonce = Nonce::from_slice(&nonce_bytes);
if let Ok(plaintext) = cipher.decrypt(nonce, decoded.as_slice()) {
```

**修复建议：** 在加载时将旧格式密码迁移至新格式（带随机Nonce），并移除零Nonce解密路径。

---

### C4. 硬编码 XOR "加密"密钥

**文件：** `src-tauri/src/services/config_service.rs` (lines 103-113)

向后兼容的 XOR 编码使用公开可见的硬编码密钥 `b"UnturnedSM2024!@"`。XOR 编码可瞬间逆向——任何人阅读源码或配置文件中的 `b64:` 前缀即可解码所有旧密码。

```rust
const XOR_KEY: &[u8] = b"UnturnedSM2024!@";
```

**修复建议：** 与 C3 同步处理——加载时迁移旧密码，不再使用 XOR 存储。

---

### C5. 多锁获取导致潜在死锁

**文件：** `src-tauri/src/commands/server.rs` (lines 287-306, 561-573), `services/scheduler.rs` (line 216)

`stop_server` 和 `restart_server` 在持有 `rcon` Mutex 的同时获取 `active_rcon` Mutex。scheduler 的 `send_announce` 在持有 `config` Mutex 后获取 `rcon` Mutex。如果任何代码路径以不同顺序获取这些锁，就会发生死锁。当前顺序未被文档化或强制执行。

```rust
let mut rcon_client = rcon.lock().unwrap_or_else(|e| e.into_inner());
if !rcon_client.is_connected() {
    let ar = active_rcon.lock().unwrap_or_else(|e| e.into_inner()); // 第二个锁
```

**修复建议：** 建立并文档化严格的锁获取顺序，或改用无锁设计（如将 RCON 连接信息复制到 RCON client 内部）。

---

### C6. XSS 风险 — `@html` 渲染用户可控数据

**文件：** `src/lib/pages/Server.svelte` (line 375), `Rcon.svelte` (line 231), `Logs.svelte` (line 167)

三处使用 `{@html highlightText(log.text, logSearch)}` 渲染日志内容。`@html` 绕过了 Svelte 的 XSS 防护。虽然 `highlightText()` 手动转义了 HTML 实体后再注入 `<mark>` 标签，但这个模式本质脆弱——一旦转义逻辑有 bug，XSS 就会发生，且无框架层面的兜底保护。RCON 日志包含远程服务器数据，信任等级低于本地日志。

```typescript
// utils.ts line 13
return escaped.replace(new RegExp(`(${q})`, 'gi'), '<mark class="bg-yellow-500/30 ...">$1</mark>');
```

**修复建议：** 考虑使用 CSS 高亮替代而非 `@html`，或使用专门的 sanitize 库做二次防护。

---

### C7. RCON 密码以明文发送到前端

**文件：** `src-tauri/src/commands/save.rs` (lines 515-519, 569)

`read_rocket_rcon_config` 命令将 RCON 密码作为明文字段序列化传输到 WebView 前端。这意味着密码通过 IPC 桥接在 JavaScript 可访问的范围内以明文存在。

```rust
#[derive(Serialize)]
pub struct RocketRconInfo {
    pub port: u16,
    pub password: String,
}
```

**修复建议：** 评估前端是否真正需要原始密码。如仅需显示"密码已设置"，则应做掩码处理。

---

## 高危问题 High

### H1. 异步命令中的阻塞操作冻结 UI

**文件：** `src-tauri/src/commands/update.rs`, `server.rs`, `updater.rs`

Tauri v2 使用异步运行时。多个 `#[tauri::command(async)]` 命令内部调用了阻塞操作（SteamCMD 子进程执行、`reqwest::blocking::Client` HTTP 请求），阻塞整个运行时线程。SteamCMD 更新可耗时 10+ 分钟，期间所有其他 Tauri 命令无法执行，UI 完全冻结。

**修复建议：** 使用 `tokio::task::spawn_blocking` 将阻塞操作转移到专用线程池，或改用异步 `reqwest` 客户端。

---

### H2. 路径遍历保护因错误处理而失效

**文件：** `src-tauri/src/commands/logs.rs` (lines 47-48)

当 `log_dir.canonicalize()` 失败时，`unwrap_or_default()` 返回空 `PathBuf`。`Path::starts_with("")` 对任意绝对路径返回 `true`，导致路径遍历检查完全失效。此外，`file_path.canonicalize()` 失败时整个检查被跳过。

```rust
if !resolved.starts_with(log_dir.canonicalize().unwrap_or_default()) {
```

**修复建议：** 当 `canonicalize` 失败时应返回错误而非继续执行；确保目录存在后再做检查。

---

### H3. 脆弱的手动 XML 字符串操作

**文件：** `src-tauri/src/services/config_service.rs` (lines 260-323), `commands/save.rs` (lines 550-566), `commands/server.rs` (lines 68-97)

三处使用字符串 `find`/切片方式修改 XML 属性。`find("Port=\"")` 仅定位第一次出现，可能匹配注释或无关元素；字节切片在多字节 UTF-8 字符边界上会 panic。同一脆弱模式在三个不同文件中重复出现。

```rust
if let Some(start) = new_content.find("Port=\"") {
    let after = start + 6;
    new_content = format!("{}{}{}", &new_content[..after], port, &new_content[after + end..]);
}
```

**修复建议：** 使用 `quick-xml` crate 解析和修改 XML，消除字符串操作的脆弱性和重复代码。

---

### H4. Mutex Poisoning 恢复模式掩盖底层错误

**遍布整个代码库：** 30+ 处使用 `.lock().unwrap_or_else(|e| e.into_inner())`

当线程持有 Mutex 时 panic，`into_inner()` 恢复模式提取数据但静默吞掉导致 panic 的错误。这意味着数据可能处于不一致状态，根本原因从不被记录或报告。

**修复建议：** 至少在恢复时记录警告日志，或考虑在 poison 时重新初始化数据而非使用可能损坏的状态。

---

### H5. 无优雅关闭机制的后台线程

**文件：** `services/scheduler.rs`, `commands/server.rs`, `services/process.rs`, `services/rcon_client.rs`

所有后台线程使用 `loop { sleep; ... }` 模式，无 shutdown 信号。应用关闭时线程继续运行直到进程被杀死，无 `join` 操作，数据可能丢失（如 `GameLogWriter` flush 未完成）。

**修复建议：** 使用 `CancellationToken` 或 channel 发送关闭信号，在应用退出时等待线程完成。

---

### H6. 大型单体组件

**文件：** `src/lib/pages/Save.svelte` (~1010 行, 50.7KB), `Wizard.svelte` (~822 行, 42.3KB)

Save.svelte 包含三个完全独立的子功能（Commands.dat 编辑器、Workshop Mod 管理、插件管理）却共用一个文件和 30+ 状态变量。Wizard.svelte 包含 6 个步骤视图，同样是单一组件。两者都应按功能/步骤拆分为独立子组件。

---

### H7. 普遍的静默错误吞没

**遍布前端：** 12+ 处使用空 `catch {}` 块

| 文件 | 位置 | 影响 |
|------|------|------|
| `stores.svelte.ts` | line 34, 49 | 存档/设置加载失败，用户无反馈 |
| `Dashboard.svelte` | line 179, 212 | 状态/系统信息刷新失败 |
| `Save.svelte` | line 68, 161, 167, 216 | 存档/配置/插件加载失败 |
| `Schedule.svelte` | line 29 | 定时任务加载失败 |
| `Settings.svelte` | line 29 | 配置加载失败 |

用户看到过时或错误的数据，而没有任何错误提示。

**修复建议：** 将所有空 catch 替换为至少 `console.error()` + 用户可见的 toast/inline 错误消息。

---

### H8. Dashboard 与 Server 页面大量代码重复

**文件：** `Dashboard.svelte`, `Server.svelte`

两文件共享约 80 行相同的轮询基础设施（`nextPollDelay`, `pollLoop`, `restartPolling`, visibility change handler），以及近乎相同的 `startServer/stopServer/restartServer` 逻辑和 `toggleAutoUpdateHosting` 函数。

**修复建议：** 将共享逻辑提取为 composable utility 或共享 Svelte 组件。

---

### H9. 普遍使用 `any` 类型丧失 TypeScript 安全

**遍布前端：** 所有 Tauri `invoke()` 结果均 cast 为 `any`

无任何 TypeScript interface 定义 Tauri 命令响应结构。`tsconfig.json` 开启了 `strict: true`，但代码广泛绕过了类型检查。

```typescript
const s: any = await invoke("get_server_status");
let saves = $state<any[]>([]);
```

**修复建议：** 为每个 Tauri 命令响应定义 TypeScript interface，逐步替换 `any`。

---

### H10. `download_server` 命令接受任意可执行路径

**文件：** `src-tauri/src/commands/installer.rs` (lines 235-239)

`download_server` 接受来自前端的 `steamcmd_path` 参数并直接在 `Command::new()` 中使用。仅检查路径是否存在，不验证是否为 SteamCMD 可执行文件。恶意前端可传入任意可执行文件路径。

**修复建议：** 验证可执行文件名为 `steamcmd.exe`，或限制路径到已知位置。

---

### H11. 破坏性操作无确认对话框

**文件：** `Server.svelte` (forceStop), `Schedule.svelte` (removeTask), `Save.svelte` (removeWorkshopMod)

强制停止服务器、删除定时任务、移除 Workshop Mod 均无确认步骤，用户误操作无法撤回。

**修复建议：** 为所有破坏性操作添加确认对话框。

---

## 中等问题 Medium

### M1. 默认 RCON 密码为 "changeme"

**文件：** `src-tauri/src/models/config.rs` (line 44), `src/lib/pages/Wizard.svelte` (line 12)

后端默认值和 Wizard 前端默认值均为 `"changeme"`，这是众所周知的弱密码。任何本地进程可用此密码连接 RCON 执行任意服务器命令。

**修复建议：** Wizard 启动时自动生成随机密码，或强制用户设置密码。

---

### M2. `atomic_write` 在 Windows 上非真正原子

**文件：** `src-tauri/src/services/config_service.rs` (lines 17-22)

Windows 上 `fs::rename` 在目标文件被其他进程打开时会失败。`.tmp` 文件在 rename 失败时不会被清理。

**修复建议：** 在 rename 失败时清理临时文件，或使用 Windows 特定的原子写入方案。

---

### M3. 配置缓存数据一致性风险

**文件：** `src-tauri/src/services/config_service.rs`

内存缓存不会因外部修改而失效。如果 JSON 文件被外部编辑，缓存与文件内容不一致。缓存存储解码后的密码（明文），文件存储编码密码，存在微妙的不一致性。

**修复建议：** 添加文件修改时间检查或 hash 比较来验证缓存有效性。

---

### M4. 无 Linting/Formatting 配置

项目中无 ESLint、Prettier、rustfmt.toml、clippy.toml 配置文件。代码风格完全依赖开发者自觉。

**修复建议：** 添加 ESLint + Prettier + rustfmt + clippy 配置，并在 package.json 和 CI 中加入 lint/format 脚本。

---

### M5. 无前端测试基础设施

前端零测试覆盖。无 vitest.config、无 .test/.spec 文件。后端有部分单元测试但覆盖有限。

**修复建议：** 添加 Vitest 配置，优先为 `utils.ts` 和 store 编写测试。

---

### M6. 无 Tauri 应用的 CI/CD

仅有 VitePress 文档站部署到 GitHub Pages 的 workflow，主 Tauri 应用无任何 CI——无构建验证、无 Rust 测试运行、无前端 linting、无自动发布构建。

**修复建议：** 添加 GitHub Actions workflow 执行 `cargo test`, `cargo clippy`, `pnpm build` 验证。

---

### M7. Tauri CSP 包含 `unsafe-inline` 样式

**文件：** `src-tauri/tauri.conf.json` (line 26)

CSP 的 `style-src` 包含 `'unsafe-inline'`。这对 Tailwind CSS v4 是必要的，但降低了 CSP 对样式注入攻击的防护效果。

**修复建议：** 文档化为何需要 `unsafe-inline`，长期考虑 nonce-based CSP。

---

### M8. 密码生成中的模偏差

**文件：** `src/lib/utils.ts` (line 24)

使用 `v % chars.length` 对 `Uint32Array` 值取模到 72 字符集。当字符集长度不整除 2^32 时，某些字符统计上更可能出现。偏差极小但违反密码学最佳实践。

```typescript
return Array.from(arr, v => chars[v % chars.length]).join("");
```

**修复建议：** 拒绝 >= 2^32 - (2^32 % 72) 的值并重新采样。

---

### M9. 前端事件监听器泄漏

**文件：** `Save.svelte` (lines 88-110, 118-140), `Wizard.svelte` (lines 87-113)

`initNewSave` 和 `initRocketForSave` 注册 `listen("installer-progress", ...)` 事件。`unlisten()` 仅在成功/错误回调中调用。如果组件在安装完成前被销毁（用户导航离开），监听器持续存在并引用可能不存在的组件状态。

**修复建议：** 在 `$effect` 返回的清理函数中调用 `unlisten()`。

---

### M10. 缺少 ARIA 无障碍属性

**文件：** `Save.svelte` (lines 637-657, 884-904)

开关按钮缺少 `role="switch"` 和 `aria-checked` 属性，屏幕阅读器将其解读为普通按钮而非开关控件。所有表单输入使用 `<span>` 作标签而非 `<label>` 元素。

**修复建议：** 为所有开关添加 `role="switch"` + `aria-checked`，将 `<span>` 标签替换为 `<label>`。

---

### M11. 页面导航销毁组件状态

**文件：** `App.svelte` (lines 234-252)

路由使用 `{#if}/{:else if}` 块，切换页面时 Svelte 销毁并重建组件。搜索查询、RCON 连接状态、滚动位置等在导航时丢失。

**修复建议：** 使用 `{#key}` 包裹或考虑将关键状态提升到 store 中持久化。

---

### M12. 前端无输入验证

**文件：** `Save.svelte` (lines 171-201)

HTML 输入有 `min/max` 属性，但 `saveCommandsDat()` 在发送到后端前不做任何验证。端口值 1（低于 1024 有效范围）会被接受并保存。

**修复建议：** 在保存前对关键字段（端口、玩家数等）做范围验证。

---

### M13. `Date.now().toString()` 作为任务 ID

**文件：** `Schedule.svelte` (line 43)

快速连续点击可在同一毫秒创建两个相同 ID 的任务，导致数据重复。

**修复建议：** 使用 `crypto.randomUUID()` 或加计数器后缀确保唯一性。

---

## 低级问题 Low

### L1. 魔法数字未命名

**文件：** `installer.rs`, `update.rs`, `save.rs` 中 `0x08000000` (CREATE_NO_WINDOW) 出现 5+ 处未定义为命名常量。

### L2. 缺少公共函数文档注释

大部分公共函数和结构体缺少 `///` 文档注释。

### L3. 使用 String 作为错误类型而非枚举

所有函数返回 `Result<T, String>`，阻止结构化错误处理和错误匹配。应使用 `thiserror` 定义专用错误类型。

### L4. 中文字符检测不完整

**文件：** `commands/config.rs` (lines 9-11) — `contains_chinese()` 遗漏 CJK Extension B-F 及兼容汉字补充区段。

### L5. Google Fonts CDN 依赖

**文件：** `index.html` (lines 7-9) — 桌面应用从外部 CDN 加载字体。首次离线启动将显示系统字体，可能破坏视觉设计。应将字体本地打包。

### L6. 硬编码中文 UI 文本无 i18n

所有 14 个源文件包含硬编码中文文本，无国际化框架，无法本地化。

### L7. CSS 类字符串大量重复

Tailwind 类组合在组件间重复 10-20 次。应提取为可复用 CSS 类或 Svelte 组件。

### L8. 动态列表缺少 `{#each}` key

**文件：** `Schedule.svelte`, `Save.svelte`, `Server.svelte`, `Rcon.svelte` — 支持增删操作的列表缺少显式 key 标识符。

### L9. 搜索输入无 debounce

日志搜索在每次按键时过滤最多 500 条日志。150-300ms debounce 可改善性能。

### L10. package.json 元数据缺失

`description` 和 `author` 为空字符串，`main` 指向不存在的 `index.js`。

### L11. ISC 与空 License 不一致

package.json 声明 ISC license，Cargo.toml license 为空字符串，无 LICENSE 文件。

### L12. Cargo.toml authors 为占位符 "you"

### L13. Rust 依赖版本过于宽松

`serde_json = "1.0"`, `tokio = "1"` 等仅锁定主版本。关键安全依赖应锁定到具体版本。

### L14. README Node.js 版本要求与 pnpm 11 不匹配

README 称需 Node.js 18+，但 pnpm@11.1.2 要求 node >= 22.13。

### L15. ProcessManager.is_running() 要求 &mut self

`try_wait()` 需要 `&mut Child`，导致整个方法必须 `&mut self`，阻止并发状态检查。应使用 `RefCell` 实现内部可变性。

### L16. 无 .npmrc/.pnpmrc 配置

缺少确定性依赖管理配置。

### L17. pnpm-lock.yaml 不完整

锁文件仅包含 pnpm 自身，未包含项目的实际依赖。应重新生成完整锁文件。

### L18. Tauri 缺少 capability/权限定义

缺少 `capabilities` 部分，默认权限可能过宽。

### L19. 无前端 test/check 脚本

package.json 无 `"test"` 和 `"check"` (svelte-check) 脚本。

---

## 积极发现

以下方面做得较好，值得保持和推广：

1. **路径验证：** `validate_id()` 正确拒绝路径遍历字符和空字符串。
2. **Workshop URL 验证：** `validate_workshop_url()` 仅允许特定 Steam Workshop URL，有测试覆盖。
3. **原子写入：** `atomic_write()` 使用 `.tmp` + rename 模式防止部分写入。
4. **Rust 单元测试：** 多个模块包含 `#[cfg(test)]` 测试。
5. **.gitignore 完善：** 正确排除 node_modules, target, dist, 运行时目录, OS 文件。
6. **Cargo.lock 存在：** 确保 Rust 构建确定性。

---

## 改进建议优先级

按紧急程度排序的建议改进路径：

1. **紧急安全修复（立即）：** 替换 PowerShell 命令注入（C1），改用 `zip` crate；将密码存储迁移至 DPAPI/Credential Manager，移除零Nonce和 XOR 兼容路径（C2-C4）；建立锁获取顺序文档（C5）；修复 `@html` XSS（C6）；掩码前端密码（C7）。
2. **高优先级（1-2 周）：** 将阻塞操作移至 `spawn_blocking`（H1）；修复路径遍历检查（H2）；引入 `quick-xml` 替换手动 XML 操作（H3）；为破坏性操作添加确认对话框（H11）；验证 SteamCMD 路径（H10）。
3. **中优先级（2-4 周）：** 添加 ESLint/Prettier/rustfmt/clippy 配置（M4）；添加 Vitest 前端测试（M5）；添加 CI workflow（M6）；拆分 Save.svelte 和 Wizard.svelte 为子组件（H6）；定义 TypeScript interface 替换 `any`（H9）；消除空 catch 块（H7）；提取共享逻辑（H8）。
4. **长期改进：** 替换空 catch 块为错误反馈（H7）；添加 i18n 框架（L6）；本地打包字体（L5）；改善无障碍性（M10）；使用 `thiserror` 定义错误类型（L3）；完善 package.json 元数据（L10-L12）。

---

## 总结统计

| 严重级别 | 数量 | 关键领域 |
|----------|------|----------|
| Critical | 7 | 安全（命令注入、加密、XSS、死锁、明文密码） |
| High | 11 | 代码质量（阻塞async、路径遍历、XML、mutex、线程、组件、类型） + 安全（弱密码、任意可执行文件） |
| Medium | 13 | 配置（缓存、原子写入、reqwest） + 前端（组件、状态管理、事件监听、无障碍） + 工程化（lint、测试、CI） |
| Low | 19 | 代码风格（文档、常量、错误类型） + 配置（i18n、CSP、标签、依赖管理） |

**总计问题数：50**

---

*本报告由 QoderWork 代码审查工具生成，基于对项目全部源代码文件（Rust 后端 26 个文件 + Svelte 前端 14 个文件 + 配置文件）的静态分析。*