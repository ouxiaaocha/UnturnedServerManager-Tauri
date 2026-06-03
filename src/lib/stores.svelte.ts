import { invoke } from "@tauri-apps/api/core";

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
