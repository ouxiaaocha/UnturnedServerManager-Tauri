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
