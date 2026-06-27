import { invoke } from "@tauri-apps/api/core";
import { classifyLogLevel, formatUptime } from "./utils";
import { CircularBuffer } from "./CircularBuffer";
import { toastStore } from "./stores/toast.svelte";
import type {
  UiPreferences,
  SaveInfo,
  RunningServerInfo,
  ServerRuntimeState,
  ServerInfoState,
  LogEntry,
  RconLogEntry,
  AppSettings,
  ServerSnapshot,
  OperationResult,
  SaveActiveTab,
} from "./types";

export type { SaveActiveTab } from "./types";

const UI_PREFERENCES_STORAGE_KEY = "unturned-server-manager-ui-preferences";
const DEFAULT_UI_PREFERENCES: UiPreferences = {
  selectedSaveId: "",
  saveActiveTab: "save",
};

function isSaveActiveTab(value: unknown): value is import("./types").SaveActiveTab {
  return (
    value === "save" ||
    value === "gameConfig" ||
    value === "workshop" ||
    value === "plugins" ||
    value === "permissions"
  );
}

function loadInitialUiPreferences(): UiPreferences {
  if (typeof localStorage === "undefined") {
    return { ...DEFAULT_UI_PREFERENCES };
  }

  try {
    const raw = localStorage.getItem(UI_PREFERENCES_STORAGE_KEY);
    if (!raw) return { ...DEFAULT_UI_PREFERENCES };

    const parsed = JSON.parse(raw) as Partial<UiPreferences>;
    return {
      selectedSaveId: typeof parsed.selectedSaveId === "string" ? parsed.selectedSaveId : "",
      saveActiveTab: isSaveActiveTab(parsed.saveActiveTab) ? parsed.saveActiveTab : "save",
    };
  } catch {
    return { ...DEFAULT_UI_PREFERENCES };
  }
}

function persistUiPreferences() {
  if (typeof localStorage === "undefined") return;

  try {
    localStorage.setItem(
      UI_PREFERENCES_STORAGE_KEY,
      JSON.stringify({
        selectedSaveId: uiPreferences.selectedSaveId,
        saveActiveTab: uiPreferences.saveActiveTab,
      }),
    );
  } catch {}
}

// Global state that persists across page switches
// 使用循环缓冲区提升性能,避免频繁的数组 splice 操作
const rconLogsBuffer = new CircularBuffer<RconLogEntry>(500);
export const rconLogs: RconLogEntry[] = $state([]);

export function addRconLog(text: string, type = "response") {
  const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
  const entry = { text: `[${time}] ${text}`, type };
  rconLogsBuffer.push(entry);
  rconLogs.splice(0, rconLogs.length, ...rconLogsBuffer.toArray());
}

export function addRconLogs(lines: string[], type = "response") {
  if (lines.length === 0) return;
  const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
  const entries = lines.map((text) => ({ text: `[${time}] ${text}`, type }));
  rconLogsBuffer.pushMany(entries);
  rconLogs.splice(0, rconLogs.length, ...rconLogsBuffer.toArray());
}

// Shared state across Dashboard and Server pages
export const appState = $state({
  launchMode: "internet",
});

export const uiPreferences = $state<UiPreferences>(loadInitialUiPreferences());

export function setSelectedSaveId(id: string) {
  uiPreferences.selectedSaveId = id;
  persistUiPreferences();
  if (!serverView.selectedRunningSaveId) {
    syncSelectedServerRuntime(id);
  }
}

export function ensureSelectedSaveId(saves: SaveInfo[]) {
  const current = uiPreferences.selectedSaveId;
  if (current && saves.some((save) => save.id === current)) {
    return current;
  }

  const fallback = saves[0]?.id ?? "";
  setSelectedSaveId(fallback);
  return fallback;
}

export function setSaveActiveTab(tab: SaveActiveTab) {
  uiPreferences.saveActiveTab = tab;
  persistUiPreferences();
}

// Shared saves list (loaded once, reused by Dashboard & Server)
export const sharedSaves = $state<SaveInfo[]>([]);
export const sharedSavesState = $state({ loaded: false });

export async function loadSharedSaves() {
  if (sharedSavesState.loaded) return;
  try {
    const saves = await invoke<SaveInfo[]>("list_server_saves");
    sharedSaves.splice(0, sharedSaves.length, ...saves);
    sharedSavesState.loaded = true;
  } catch (e) {
    console.error("加载存档列表失败:", e);
    toastStore.error(`加载存档列表失败: ${e instanceof Error ? e.message : String(e)}`);
  }
}

// Shared app settings
export const sharedSettings = $state({
  autoUpdateHosting: false,
  loaded: false,
});

export async function loadSharedSettings() {
  if (sharedSettings.loaded) return;
  try {
    const settings = await invoke<AppSettings>("get_app_settings");
    sharedSettings.autoUpdateHosting = !!settings.autoUpdateHosting;
    sharedSettings.loaded = true;
  } catch (e) {
    console.error("加载应用设置失败:", e);
    toastStore.error(`加载应用设置失败: ${e instanceof Error ? e.message : String(e)}`);
  }
}

export async function toggleAutoUpdateHosting(saveId: string | null): Promise<OperationResult> {
  const nextEnabled = !sharedSettings.autoUpdateHosting;
  try {
    const settings = await invoke<AppSettings>("set_auto_update_hosting", {
      enabled: nextEnabled,
      saveId,
    });
    sharedSettings.autoUpdateHosting = !!settings.autoUpdateHosting;
    return {
      success: true,
      message: sharedSettings.autoUpdateHosting ? "托管已开启" : "托管已关闭",
    };
  } catch (e) {
    const errorMessage = e instanceof Error ? e.message : String(e);
    return { success: false, message: `设置失败: ${errorMessage}` };
  }
}

// 服务器公网信息（Dashboard 使用）
export const serverInfo = $state({
  publicIp: "",
  ipLoading: false,
});

// ========== 共享服务器状态 ==========

export const serverView = $state({
  selectedRunningSaveId: "",
});

export const serverInfoBySave = $state<Record<string, ServerInfoState>>({});

function createServerInfoState(): ServerInfoState {
  return {
    serverCode: "",
    port: 0,
    portLoading: false,
    codeParsed: false,
  };
}

export function getServerInfoForSave(saveId: string) {
  const key = saveId || "__default__";
  if (!serverInfoBySave[key]) {
    serverInfoBySave[key] = createServerInfoState();
  }
  return serverInfoBySave[key];
}

export function resetServerInfoForSave(saveId: string) {
  const key = saveId || "__default__";
  serverInfoBySave[key] = createServerInfoState();
}

function createRuntimeState(): ServerRuntimeState {
  return {
    status: "已停止",
    pid: "--",
    uptime: "--",
    loading: "",
    outputIndex: 0,
  };
}

function ensureRuntime(saveId: string) {
  const key = saveId || "__default__";
  if (!serverStatesBySave[key]) {
    serverStatesBySave[key] = createRuntimeState();
  }
  if (!serverLogsBySave[key]) {
    serverLogsBySave[key] = [];
  }
  // 确保该存档有对应的循环缓冲区
  if (!serverLogsBuffers.has(key)) {
    serverLogsBuffers.set(key, new CircularBuffer<LogEntry>(500));
  }
  return { key, runtime: serverStatesBySave[key], logs: serverLogsBySave[key] };
}

/** 服务器运行状态 */
export const serverState = $state({
  status: "已停止",
  pid: "--",
  uptime: "--",
  loading: "" as "" | "starting" | "stopping" | "restarting",
  outputIndex: 0,
});

/** 服务器输出日志 */
export const serverLogs: LogEntry[] = $state([]);

export const serverStatesBySave = $state<Record<string, ServerRuntimeState>>({});
export const serverLogsBySave = $state<Record<string, LogEntry[]>>({});
// 为每个存档维护独立的循环缓冲区
const serverLogsBuffers = new Map<string, CircularBuffer<LogEntry>>();
export const runningServers: RunningServerInfo[] = $state([]);

function activeRuntimeSaveId() {
  return serverView.selectedRunningSaveId || uiPreferences.selectedSaveId;
}

function activeRuntimeKey() {
  return activeRuntimeSaveId() || "__default__";
}

export function syncSelectedServerRuntime(saveId = activeRuntimeSaveId()) {
  const { runtime, logs } = ensureRuntime(saveId);
  serverState.status = runtime.status;
  serverState.pid = runtime.pid;
  serverState.uptime = runtime.uptime;
  serverState.loading = runtime.loading;
  serverState.outputIndex = runtime.outputIndex;
  serverLogs.splice(0, serverLogs.length, ...logs);
}

export function setServerLoading(saveId: string, loading: ServerRuntimeState["loading"]) {
  const { runtime } = ensureRuntime(saveId);
  runtime.loading = loading;
  if ((saveId || "__default__") === activeRuntimeKey()) {
    serverState.loading = loading;
  }
}

export function isSaveRunning(saveId: string) {
  if (!saveId) return false;
  return runningServers.some((server) => server.save_id === saveId && server.state === "运行中");
}

export function selectRunningServer(saveId: string) {
  if (!saveId || !runningServers.some((server) => server.save_id === saveId)) {
    return;
  }
  serverView.selectedRunningSaveId = saveId;
  syncSelectedServerRuntime(saveId);
}

export function ensureSelectedRunningServer() {
  if (runningServers.length === 0) {
    serverView.selectedRunningSaveId = "";
    syncSelectedServerRuntime(uiPreferences.selectedSaveId);
    return "";
  }

  const selected = serverView.selectedRunningSaveId;
  if (selected && runningServers.some((server) => server.save_id === selected)) {
    syncSelectedServerRuntime(selected);
    return selected;
  }

  const next = runningServers[0].save_id;
  serverView.selectedRunningSaveId = next;
  syncSelectedServerRuntime(next);
  return next;
}

export async function refreshRunningServers() {
  try {
    const servers = await invoke("list_running_servers") as RunningServerInfo[];
    runningServers.splice(0, runningServers.length, ...servers);

    const runningIds = new Set(servers.map((server) => server.save_id));
    for (const server of servers) {
      const { runtime } = ensureRuntime(server.save_id);
      runtime.status = server.state;
      runtime.pid = server.pid ? String(server.pid) : "--";
      runtime.uptime = formatUptime(server.uptime_secs);
    }
    for (const [key, runtime] of Object.entries(serverStatesBySave)) {
      if (key !== "__default__" && !runningIds.has(key) && runtime.loading === "") {
        runtime.status = "已停止";
        runtime.pid = "--";
        runtime.uptime = "--";
      }
    }

    ensureSelectedRunningServer();
    return servers;
  } catch (e) {
    console.error("刷新运行服务器列表失败:", e);
    // 静默失败,避免频繁轮询时打扰用户
    return runningServers;
  }
}

/** 清空服务器日志 */
export function clearServerLogs(saveId = activeRuntimeSaveId()) {
  const { runtime, logs, key } = ensureRuntime(saveId);
  logs.splice(0, logs.length);
  runtime.outputIndex = 0;
  // 清空对应的循环缓冲区
  const buffer = serverLogsBuffers.get(key);
  if (buffer) {
    buffer.clear();
  }
  if (key === activeRuntimeKey()) {
    serverLogs.splice(0, serverLogs.length);
    serverState.outputIndex = 0;
  }
}

/** 添加服务器日志行 */
export function appendServerLogs(lines: string[], saveId = activeRuntimeSaveId()) {
  const { logs, key } = ensureRuntime(saveId);
  const appended = lines.map((line) => ({ text: line, level: classifyLogLevel(line) }));

  // 使用循环缓冲区高效管理日志
  const buffer = serverLogsBuffers.get(key);
  if (buffer) {
    buffer.pushMany(appended);
    logs.splice(0, logs.length, ...buffer.toArray());
  } else {
    // 降级方案：直接使用数组
    logs.push(...appended);
    if (logs.length > 500) {
      logs.splice(0, logs.length - 500);
    }
  }

  if (key === activeRuntimeKey()) {
    serverLogs.splice(0, serverLogs.length, ...logs);
  }
}

// 防止 refreshServerStatus 竞态条件的锁
const refreshLocks = new Map<string, Promise<string[]>>();

/** 刷新服务器状态（供 Dashboard 和 Server 页面共享），返回本次新增输出 */
export async function refreshServerStatus(saveId = activeRuntimeSaveId()): Promise<string[]> {
  const lockKey = saveId || "__default__";

  // 如果已经有正在进行的刷新，返回现有的 Promise
  if (refreshLocks.has(lockKey)) {
    return refreshLocks.get(lockKey)!;
  }

  // 创建新的刷新 Promise
  const refreshPromise = (async () => {
    const { runtime, key } = ensureRuntime(saveId);
    try {
      let snapshot = await invoke<ServerSnapshot>("get_server_snapshot", {
        saveId: saveId || null,
        fromIndex: runtime.outputIndex,
      });

      // 重启后后端输出缓冲会重新计数；检测到计数回退时，从新进程的起点重新同步。
      if (snapshot.output_count < runtime.outputIndex) {
        runtime.outputIndex = 0;
        snapshot = await invoke<ServerSnapshot>("get_server_snapshot", {
          saveId: saveId || null,
          fromIndex: 0,
        });
      }

      runtime.status = snapshot.state;
      runtime.pid = snapshot.pid ? String(snapshot.pid) : "--";

      // 计算 uptime
      runtime.uptime = formatUptime(snapshot.uptime_secs);

      let newLines: string[] = [];
      if (snapshot.output_count > runtime.outputIndex) {
        newLines = snapshot.output ?? [];
        appendServerLogs(newLines, saveId);
        runtime.outputIndex = snapshot.output_count;
        if (key === activeRuntimeKey()) {
          syncSelectedServerRuntime(saveId);
        }
        await refreshRunningServers();
        return newLines;
      }

      if (key === activeRuntimeKey()) {
        syncSelectedServerRuntime(saveId);
      }
      await refreshRunningServers();
      return newLines;
    } catch (e) {
      console.error("刷新服务器状态失败:", e);
      // 静默失败,避免频繁轮询时打扰用户
      return [];
    } finally {
      // 清理锁
      refreshLocks.delete(lockKey);
    }
  })();

  // 存储 Promise
  refreshLocks.set(lockKey, refreshPromise);
  return refreshPromise;
}
