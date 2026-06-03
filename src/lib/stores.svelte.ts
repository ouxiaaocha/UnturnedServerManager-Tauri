import { invoke } from "@tauri-apps/api/core";
import { classifyLogLevel } from "./utils";

// Global state that persists across page switches
export const rconLogs: Array<{text: string, type: string}> = $state([]);

export function addRconLog(text: string, type = "response") {
  const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
  rconLogs.push({ text: `[${time}] ${text}`, type });
  if (rconLogs.length > 500) rconLogs.splice(0, rconLogs.length - 500);
}

export function addRconLogs(lines: string[], type = "response") {
  if (lines.length === 0) return;
  const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
  rconLogs.push(...lines.map((text) => ({ text: `[${time}] ${text}`, type })));
  if (rconLogs.length > 500) rconLogs.splice(0, rconLogs.length - 500);
}

// Shared state across Dashboard and Server pages
export const appState = $state({
  launchMode: "internet",
});

// Shared saves list (loaded once, reused by Dashboard & Server)
export const sharedSaves = $state<any[]>([]);
export let sharedSavesLoaded = false;

export async function loadSharedSaves() {
  if (sharedSavesLoaded) return;
  try {
    const saves = await invoke("list_server_saves");
    sharedSaves.splice(0, sharedSaves.length, ...(saves as any[]));
    sharedSavesLoaded = true;
  } catch (e) { console.error("加载存档列表失败:", e); }
}

// Shared app settings
export const sharedSettings = $state({
  autoUpdateHosting: false,
  loaded: false,
});

export async function loadSharedSettings() {
  if (sharedSettings.loaded) return;
  try {
    const settings: any = await invoke("get_app_settings");
    sharedSettings.autoUpdateHosting = !!settings.autoUpdateHosting;
    sharedSettings.loaded = true;
  } catch (e) { console.error("加载应用设置失败:", e); }
}

// 服务器运行时信息（Dashboard 使用）
export const serverInfo = $state({
  serverCode: "",
  publicIp: "",
  port: 0,
  ipLoading: false,
  portLoading: false,
  runningSaveId: "",
  codeParsed: false,
});

// ========== 共享服务器状态 ==========

/** 服务器运行状态 */
export const serverState = $state({
  status: "已停止",
  pid: "--",
  uptime: "--",
  loading: "" as "" | "starting" | "stopping" | "restarting",
  outputIndex: 0,
  isStarting: false,
});

/** 服务器输出日志 */
export const serverLogs: Array<{text: string, level: string}> = $state([]);

/** 清空服务器日志 */
export function clearServerLogs() {
  serverLogs.splice(0, serverLogs.length);
  serverState.outputIndex = 0;
}

/** 添加服务器日志行 */
export function appendServerLogs(lines: string[]) {
  const appended = lines.map((line) => ({ text: line, level: classifyLogLevel(line) }));
  serverLogs.push(...appended);
  // 限制日志数量
  if (serverLogs.length > 500) {
    serverLogs.splice(0, serverLogs.length - 500);
  }
}

/** 刷新服务器状态（供 Dashboard 和 Server 页面共享） */
export async function refreshServerStatus(): Promise<boolean> {
  try {
    const s: any = await invoke("get_server_snapshot", { fromIndex: serverState.outputIndex });
    serverState.status = s.state;
    serverState.pid = s.pid ? String(s.pid) : "--";

    // 计算 uptime
    if (s.uptime_secs > 0) {
      const h = Math.floor(s.uptime_secs / 3600);
      const m = Math.floor((s.uptime_secs % 3600) / 60);
      const sec = Math.floor(s.uptime_secs % 60);
      serverState.uptime = h > 0 ? `${h}时${m}分${sec}秒` : m > 0 ? `${m}分${sec}秒` : `${sec}秒`;
    } else {
      serverState.uptime = "--";
    }

    // 处理新输出
    if (!serverState.isStarting && s.output_count > serverState.outputIndex) {
      const newLines = (s.output ?? []) as string[];
      appendServerLogs(newLines);
      serverState.outputIndex = s.output_count;
      return true; // 有新输出
    }
    return false;
  } catch {
    return false;
  }
}
