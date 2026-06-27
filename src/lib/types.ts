/**
 * 类型定义文件
 * 消除 any 类型，提供完整的 TypeScript 类型安全
 */

// ========== 后端返回类型定义 ==========

/** 应用设置 */
export interface AppSettings {
  language: string;
  theme: string;
  logRetentionDays: number;
  autoStartLastServer: boolean;
  autoUpdateHosting: boolean;
  closeToTray: boolean;
  closeActionRemembered: boolean;
}

/** 通用配置保存结果 */
export interface ConfigSaveResult {
  message: string;
  rocket_sync_warning?: string | null;
}

/** RCON 配置 */
export interface RconConfig {
  enabled: boolean;
  host: string;
  port: number;
  password: string;
}

/** 单个服务端配置 */
export interface ServerProfile {
  id: string;
  name: string;
  steamCmdPath: string;
  serverRoot: string;
  serverEntry: string;
  rcon: RconConfig;
}

/** 服务端配置集合 */
export interface ServersConfig {
  servers: ServerProfile[];
}

/** 服务器快照 */
export interface ServerSnapshot {
  save_id: string;
  state: string;
  pid: number | null;
  uptime_secs: number;
  output_count: number;
  output?: string[];
  hosting?: {
    mode: string;
    session: string;
  } | null;
  game_config?: GameConfig | null;
  commands_config?: CommandsConfig | null;
}

/** 服务器状态 */
export interface ServerStatus {
  state: string;
  pid: number | null;
  uptime_secs: number;
  output_count: number;
}

/** 游戏配置 */
export interface GameConfig {
  exists: boolean;
  path: string;
  version: string | null;
  source_hash: string;
  line_ending: string;
  sections: GameConfigSection[];
}

/** 游戏配置分组 */
export interface GameConfigSection {
  name: string;
  entries: GameConfigEntry[];
}

/** 游戏配置项 */
export interface GameConfigEntry {
  id: string;
  section: string;
  key: string;
  value: string;
  has_value: boolean;
  value_kind: string;
  description: string[];
  default_hint: string | null;
  options: string[];
}

/** Commands.dat 配置 */
export interface CommandsConfig {
  name: string | null;
  map: string | null;
  port: number | null;
  max_players: number | null;
  password: string | null;
  owner: string | null;
  cheats: boolean;
  pve: boolean;
  perspective: string | null;
  gslt: string | null;
  raw_lines: string[];
}

/** 自动检测路径结果 */
export interface DetectResult {
  steam_cmd_path: string | null;
  server_root: string | null;
  server_id: string | null;
}

/** 存档信息 */
export interface SaveInfo {
  id: string;
  name?: string;
  has_commands_dat: boolean;
}

/** 存档端口信息 */
export interface SavePortInfo {
  save_id: string;
  name?: string | null;
  game_port: number;
  rcon_port: number;
}

/** 存档端口问题 */
export interface SavePortIssue {
  kind: string;
  port: number;
  save_ids: string[];
  message: string;
}

/** 存档端口检测报告 */
export interface SavePortCheckReport {
  ok: boolean;
  saves: SavePortInfo[];
  issues: SavePortIssue[];
}

/** 运行环境检测项 */
export interface EnvironmentCheckItem {
  key: string;
  label: string;
  ok: boolean;
  required: boolean;
  message: string;
  path?: string | null;
}

/** 运行环境检测报告 */
export interface EnvironmentCheckReport {
  ok: boolean;
  saveId?: string | null;
  steamCmdPath?: string | null;
  serverRoot?: string | null;
  items: EnvironmentCheckItem[];
}

/** 系统资源快照 */
export interface SystemStats {
  cpu_name: string;
  physical_core_count?: number | null;
  logical_core_count: number;
  cpu_frequency_mhz: number;
  cpu_usage: number;
  total_memory: number;
  used_memory: number;
  memory_percentage: number;
  bytes_received: number;
  bytes_transmitted: number;
}

/** 运行中服务器信息 */
export interface RunningServerInfo {
  save_id: string;
  state: string;
  pid?: number | null;
  uptime_secs: number;
  output_count: number;
  launch_mode: string;
}

/** 插件信息 */
export interface PluginInfo {
  name: string;
  file_name: string;
  path: string;
}

/** WorkshopDownloadConfig.json 配置 */
export interface WorkshopDownloadConfig {
  file_ids: number[];
  ignore_children_file_ids: number[];
  query_cache_max_age_seconds: number;
  max_query_retries: number;
  use_cached_downloads: boolean;
  should_monitor_updates: boolean;
  shutdown_update_detected_timer: number;
  shutdown_update_detected_message: string;
  shutdown_kick_message: string;
}

/** Rocket RCON 配置 */
export interface RocketRconInfo {
  port: number;
  password: string;
  has_password: boolean;
}

/** GitHub Release 更新信息 */
export interface UpdateInfo {
  latest_version: string;
  current_version: string;
  has_update: boolean;
  body: string;
  html_url: string;
  published_at: string;
}

/** 定时重启任务 */
export interface ScheduleTask {
  id: string;
  enabled: boolean;
  type: string;
  time: string | null;
  interval_hours: number | null;
  weekday: number | null;
  announce_minutes: number[];
  server_id?: string | null;
}

/** 定时任务配置 */
export interface ScheduleConfig {
  tasks: ScheduleTask[];
}

// ========== 前端状态类型定义 ==========

/** 服务器运行时状态 */
export interface ServerRuntimeState {
  status: string;
  pid: string;
  uptime: string;
  loading: "" | "starting" | "stopping" | "restarting";
  outputIndex: number;
}

/** 服务器信息状态 */
export interface ServerInfoState {
  serverCode: string;
  port: number;
  portLoading: boolean;
  codeParsed: boolean;
}

/** 日志条目 */
export interface LogEntry {
  text: string;
  level: string;
}

/** RCON 日志条目 */
export interface RconLogEntry {
  text: string;
  type: string;
}

/** UI 偏好设置 */
export interface UiPreferences {
  selectedSaveId: string;
  saveActiveTab: SaveActiveTab;
}

/** 存档活动标签页 */
export type SaveActiveTab = "save" | "gameConfig" | "workshop" | "plugins" | "permissions";

/** 操作结果 */
export interface OperationResult {
  success: boolean;
  message: string;
}
