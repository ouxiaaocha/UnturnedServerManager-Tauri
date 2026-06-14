<script lang="ts">
  /**
   * SaveConfig 组件 - 存档配置表单
   * 从 Save.svelte 拆分出来，负责 Commands.dat 和 Config.json 配置
   */

  import { generatePassword } from "$lib/utils";
  import SelectCustom from "$lib/components/SelectCustom.svelte";

  let {
    // Commands.dat 配置
    cmdName = $bindable(""),
    cmdMap = $bindable(""),
    cmdPort = $bindable(27015),
    cmdMaxPlayers = $bindable(24),
    cmdPassword = $bindable(""),
    cmdOwner = $bindable(""),
    cmdPerspective = $bindable("Both"),
    cmdGslt = $bindable(""),
    cmdPve = $bindable(false),
    cmdCheats = $bindable(false),

    // RCON 配置
    rconPort = $bindable(27115),
    rconPassword = $bindable(""),

    // 状态
    loading = false,
  } = $props();

  let showRconPassword = $state(false);
</script>

<div class="space-y-5">
  <!-- Commands.dat Editor -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
    <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
      Commands.dat 配置
    </h2>

    {#if loading}
      <div class="flex items-center justify-center py-10" role="status" aria-live="polite">
        <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin" aria-hidden="true"></div>
        <span class="sr-only">加载中</span>
      </div>
    {:else}
      <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:gap-5">
        <div>
          <label for="cmd-name" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器名称</label>
          <input
            id="cmd-name"
            type="text"
            bind:value={cmdName}
            placeholder="My Unturned Server"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label for="cmd-map" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">地图</label>
          <input
            id="cmd-map"
            type="text"
            bind:value={cmdMap}
            placeholder="PEI"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label for="cmd-port" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">端口</label>
          <input
            id="cmd-port"
            type="number"
            bind:value={cmdPort}
            min="1024"
            max="65535"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label for="cmd-max-players" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">最大玩家数</label>
          <input
            id="cmd-max-players"
            type="number"
            bind:value={cmdMaxPlayers}
            min="1"
            max="200"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label for="cmd-password" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器密码</label>
          <input
            id="cmd-password"
            type="text"
            bind:value={cmdPassword}
            placeholder="留空表示无密码"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label for="cmd-owner" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">管理员 SteamID64</label>
          <input
            id="cmd-owner"
            type="text"
            bind:value={cmdOwner}
            placeholder="76561198000000000"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>

        <div>
          <label for="cmd-perspective" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">视角</label>
          <SelectCustom
            bind:value={cmdPerspective}
            options={[
              { value: 'First', label: '第一人称' },
              { value: 'Third', label: '第三人称' },
              { value: 'Both', label: '两者皆可' },
              { value: 'Vehicle', label: '载具' }
            ]}
            size="md"
            fullWidth
          />
        </div>

        <div class="md:col-span-2">
          <label for="cmd-gslt" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">GSLT (Game Server Login Token)</label>
          <input
            id="cmd-gslt"
            type="text"
            bind:value={cmdGslt}
            placeholder="可选，用于在服务器浏览器中显示"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
        </div>
      </div>

      <div class="flex flex-wrap gap-4 sm:gap-8 mt-5 pt-5 border-t border-[var(--border)]">
        <label class="flex items-center gap-3 cursor-pointer">
          <button
            type="button"
            role="switch"
            aria-checked={cmdPve}
            class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdPve ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
            onclick={() => cmdPve = !cmdPve}
            aria-label="切换 PvE 模式"
          >
            <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdPve ? 'translate-x-5' : 'translate-x-1'}"></div>
          </button>
          <span class="text-sm text-[var(--text-secondary)]">PvE 模式</span>
        </label>

        <label class="flex items-center gap-3 cursor-pointer">
          <button
            type="button"
            role="switch"
            aria-checked={cmdCheats}
            class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdCheats ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
            onclick={() => cmdCheats = !cmdCheats}
            aria-label="切换作弊模式"
          >
            <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdCheats ? 'translate-x-5' : 'translate-x-1'}"></div>
          </button>
          <span class="text-sm text-[var(--text-secondary)]">启用作弊</span>
        </label>
      </div>
    {/if}
  </div>

  <!-- RCON Config -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
    <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
      <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 11V7a4 4 0 118 0m-4 8v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2z" />
      </svg>
      RCON 配置
    </h2>

    <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:gap-5">
      <div>
        <label for="rcon-port" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 端口</label>
        <input
          id="rcon-port"
          type="number"
          bind:value={rconPort}
          min="1024"
          max="65535"
          class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
        />
      </div>

      <div>
        <label for="rcon-password" class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 密码</label>
        <div class="relative">
          <input
            id="rcon-password"
            type={showRconPassword ? "text" : "password"}
            bind:value={rconPassword}
            placeholder="输入 RCON 密码"
            class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 pr-20 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
          />
          <div class="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
            <button
              type="button"
              class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
              onclick={() => showRconPassword = !showRconPassword}
              aria-label={showRconPassword ? "隐藏密码" : "显示密码"}
            >
              {#if showRconPassword}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                </svg>
              {:else}
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                </svg>
              {/if}
            </button>
            <button
              type="button"
              class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
              onclick={() => rconPassword = generatePassword()}
              aria-label="生成随机密码"
              title="生成随机密码"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" />
              </svg>
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>
