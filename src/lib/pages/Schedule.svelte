<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  interface ScheduleTask {
    id: string;
    enabled: boolean;
    type: string;
    time: string | null;
    interval_hours: number | null;
    weekday: number | null;
    announce_minutes: number[];
  }

  let tasks: ScheduleTask[] = $state([]);
  let showAdd = $state(false);
  let newType = $state("daily");
  let newTime = $state("04:00");
  let newInterval = $state(6);
  let newWeekday = $state(0);
  let message = $state("");

  const weekdays = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];

  async function loadSchedules() {
    try {
      const config: any = await invoke("get_schedules");
      tasks = config.tasks || [];
    } catch {}
  }

  async function saveSchedules() {
    try {
      await invoke("save_schedules", { tasks: { tasks } });
      message = "已保存";
      setTimeout(() => message = "", 2000);
    } catch (e: any) {
      message = `保存失败: ${e}`;
    }
  }

  function addTask() {
    const task: ScheduleTask = {
      id: Date.now().toString(),
      enabled: true,
      type: newType,
      time: newType === "daily" || newType === "weekly" ? newTime : null,
      interval_hours: newType === "interval" ? newInterval : null,
      weekday: newType === "weekly" ? newWeekday : null,
      announce_minutes: [30, 10, 5, 1],
    };
    tasks = [...tasks, task];
    showAdd = false;
    saveSchedules();
  }

  function removeTask(id: string) {
    tasks = tasks.filter(t => t.id !== id);
    saveSchedules();
  }

  function toggleTask(id: string) {
    tasks = tasks.map(t => t.id === id ? { ...t, enabled: !t.enabled } : t);
    saveSchedules();
  }

  function describeTask(t: ScheduleTask): string {
    switch (t.type) {
      case "daily": return `每天 ${t.time} 重启`;
      case "interval": return `每 ${t.interval_hours} 小时重启`;
      case "weekly": return `每${weekdays[t.weekday || 0]} ${t.time} 重启`;
      default: return "未知";
    }
  }

  $effect(() => { loadSchedules(); });
</script>

<div class="flex flex-col gap-5 h-full overflow-y-auto">
  <div class="flex flex-wrap items-center justify-between gap-3 flex-shrink-0">
    <div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">定时任务</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">配置自动重启计划</p>
    </div>
    <button
      class="px-5 py-2.5 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] cursor-pointer flex items-center gap-2"
      onclick={() => showAdd = !showAdd}
    >
      {#if showAdd}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
        取消
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        添加任务
      {/if}
    </button>
  </div>

  <!-- Add Task Form -->
  {#if showAdd}
    <div class="bg-[var(--bg-card)] border border-[var(--accent)] rounded-xl p-6 flex-shrink-0">
      <div class="flex gap-5 items-end flex-wrap">
        <div>
          <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">类型</span>
          <select bind:value={newType}
            class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] cursor-pointer">
            <option value="daily">每日定时</option>
            <option value="interval">固定间隔</option>
            <option value="weekly">每周定时</option>
          </select>
        </div>

        {#if newType === "daily" || newType === "weekly"}
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">时间</span>
            <input type="time" bind:value={newTime}
              class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          </div>
        {/if}

        {#if newType === "interval"}
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">间隔（小时）</span>
            <input type="number" bind:value={newInterval} min="1" max="24"
              class="w-full sm:w-[120px] bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)]" />
          </div>
        {/if}

        {#if newType === "weekly"}
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">星期</span>
            <select bind:value={newWeekday}
              class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:border-[var(--accent)] transition-colors duration-[var(--transition-normal)] cursor-pointer">
              {#each weekdays as day, i}
                <option value={i}>{day}</option>
              {/each}
            </select>
          </div>
        {/if}

        <button
          class="px-5 py-2.5 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all duration-[var(--transition-normal)] cursor-pointer flex items-center gap-2"
          onclick={addTask}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          确认添加
        </button>
      </div>
      <p class="text-xs text-[var(--text-muted)] mt-4 flex items-center gap-1">
        <svg class="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        重启前会在 30、10、5、1 分钟时发送公告
      </p>
    </div>
  {/if}

  <!-- Task List -->
  <div class="flex-1 overflow-y-auto">
    {#if tasks.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-[var(--text-muted)]">
        <svg class="w-16 h-16 mb-4 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        <p class="text-sm">暂无定时任务</p>
        <p class="text-xs mt-1">点击"添加任务"创建</p>
      </div>
    {:else}
      <div class="space-y-3">
        {#each tasks as task}
          <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4 flex flex-wrap items-center justify-between gap-3 hover:border-[var(--border-hover)] transition-all duration-[var(--transition-normal)]">
            <div class="flex items-center gap-4">
              <button
                class="w-5 h-5 rounded-md border-2 {task.enabled ? 'bg-[var(--accent)] border-[var(--accent)]' : 'border-[var(--border)]'} transition-all duration-[var(--transition-fast)] cursor-pointer flex items-center justify-center"
                onclick={() => toggleTask(task.id)}
              >
                {#if task.enabled}
                  <svg class="w-3 h-3 text-[var(--text-primary)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                  </svg>
                {/if}
              </button>
              <div>
                <span class="text-sm text-[var(--text-primary)] {!task.enabled ? 'opacity-50' : ''}">{describeTask(task)}</span>
                <p class="text-xs text-[var(--text-muted)] mt-0.5">提前 {task.announce_minutes.join('、')} 分钟公告</p>
              </div>
            </div>
            <button
              class="text-xs text-[var(--text-muted)] hover:text-[var(--danger)] transition-colors duration-[var(--transition-fast)] px-3 py-1.5 rounded-lg hover:bg-[var(--danger-glow)] cursor-pointer"
              onclick={() => removeTask(task.id)}
            >
              删除
            </button>
          </div>
        {/each}
      </div>
    {/if}
  </div>

  {#if message}
    <div class="flex-shrink-0 px-4 py-2 rounded-lg {message.includes('失败') ? 'bg-[var(--danger-glow)] text-[var(--danger)]' : 'bg-[var(--success-glow)] text-[var(--success)]'} text-sm">
      {message}
    </div>
  {/if}
</div>



