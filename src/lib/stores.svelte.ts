// Global state that persists across page switches
export const rconLogs: Array<{text: string, type: string}> = $state([]);

export function addRconLog(text: string, type = "response") {
  const time = new Date().toLocaleTimeString("zh-CN", { hour12: false });
  rconLogs.push({ text: `[${time}] ${text}`, type });
  if (rconLogs.length > 500) rconLogs.splice(0, rconLogs.length - 500);
}
