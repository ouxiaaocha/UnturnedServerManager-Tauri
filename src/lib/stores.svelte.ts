import { invoke } from "@tauri-apps/api/core";
import { classifyLogLevel, formatUptime } from "./utils";

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
export const sharedSavesState = $state({ loaded: false });

export async function loadSharedSaves() {
  if (sharedSavesState.loaded) return;
  try {
    const saves = await invoke("list_server_saves");
    sharedSaves.splice(0, sharedSaves.length, ...(saves as any[]));
    sharedSavesState.loaded = true;
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

export async function toggleAutoUpdateHosting(saveId: string | null) {
  const nextEnabled = !sharedSettings.autoUpdateHosting;
  try {
    const settings: any = await invoke("set_auto_update_hosting", {
      enabled: nextEnabled,
      saveId,
    });
    sharedSettings.autoUpdateHosting = !!settings.autoUpdateHosting;
    return {
      success: true,
      message: sharedSettings.autoUpdateHosting ? "托管已开启" : "托管已关闭",
    };
  } catch (e: any) {
    return { success: false, message: `设置失败: ${e}` };
  }
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

/** 刷新服务器状态（供 Dashboard 和 Server 页面共享），返回本次新增输出 */
export async function refreshServerStatus(): Promise<string[]> {
  try {
    let s: any = await invoke("get_server_snapshot", { fromIndex: serverState.outputIndex });

    // 重启后后端输出缓冲会重新计数；检测到计数回退时，从新进程的起点重新同步。
    if (s.output_count < serverState.outputIndex) {
      serverState.outputIndex = 0;
      s = await invoke("get_server_snapshot", { fromIndex: 0 });
    }

    serverState.status = s.state;
    serverState.pid = s.pid ? String(s.pid) : "--";

    // 计算 uptime
    serverState.uptime = formatUptime(s.uptime_secs);

    if (s.output_count > serverState.outputIndex) {
      const newLines = (s.output ?? []) as string[];
      appendServerLogs(newLines);
      serverState.outputIndex = s.output_count;
      return newLines;
    }
    return [];
  } catch (e) {
    console.error("刷新服务器状态失败:", e);
    return [];
  }
}
