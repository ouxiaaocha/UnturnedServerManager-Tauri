<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open as openShell } from "@tauri-apps/plugin-shell";
  import { generatePassword } from "$lib/utils";
  import {
    uiPreferences,
    setSelectedSaveId,
    ensureSelectedSaveId,
    setSaveActiveTab,
    isSaveRunning,
    refreshRunningServers,
    sharedSaves,
    sharedSavesState,
    type SaveActiveTab
  } from "$lib/stores.svelte";
  import { toastStore } from "../stores/toast.svelte";
  import { listenInstallerProgress } from "../utils/installer";
  import Select from "../components/Select.svelte";
  import SelectCustom from "../components/SelectCustom.svelte";
  import GameConfigTab from "./Save/GameConfigTab.svelte";
  import type {
    CommandsConfig,
    PluginInfo,
    RocketRconInfo,
    SaveInfo,
    WorkshopDownloadConfig,
  } from "../types";

  type PermissionEntry = {
    name: string;
    cooldown: number;
  };

  type PermissionGroup = {
    id: string;
    display_name: string;
    prefix: string;
    suffix: string;
    color: string;
    members: string[];
    parent_group: string | null;
    priority: number;
    permissions: PermissionEntry[];
  };

  type PermissionsConfig = {
    exists: boolean;
    path: string;
    default_group: string;
    groups: PermissionGroup[];
  };

  let activeTab = $derived(uiPreferences.saveActiveTab);
  let saves = $state<SaveInfo[]>([]);
  let selectedSaveId = $derived(uiPreferences.selectedSaveId);
  let selectedSaveRunning = $derived(isSaveRunning(selectedSaveId));
  let loading = $state(false);
  let saving = $state(false);
  let deletingSave = $state(false);
  let pluginsLoading = $state(false);

  let cmdName = $state("");
  let cmdMap = $state("");
  let cmdPort = $state(27015);
  let cmdMaxPlayers = $state(24);
  let cmdPassword = $state("");
  let cmdOwner = $state("");
  let cmdCheats = $state(false);
  let cmdPve = $state(false);
  let cmdPerspective = $state("Both");
  let cmdGslt = $state("");

  let rconPort = $state(27115);
  let rconPassword = $state("");
  let rconPasswordMasked = $state(false);
  let showRconPassword = $state(false);

  let plugins = $state<PluginInfo[]>([]);
  let pluginNotes = $state<Record<string, string>>({});

  let workshopConfig = $state<WorkshopDownloadConfig | null>(null);
  let workshopLoading = $state(false);
  let workshopSaving = $state(false);
  let workshopModNotes = $state<Record<string, string>>({});
  let newModId = $state("");
  let newModNote = $state("");
  let ignoreChildrenInput = $state("");

  let permissionsConfig = $state<PermissionsConfig | null>(null);
  let permissionsLoading = $state(false);
  let permissionsSaving = $state(false);
  let selectedPermissionGroupId = $state("");
  let newMemberInput = $state("");
  let newPermissionName = $state("");
  let newPermissionCooldown = $state(0);
  let selectedPermissionGroup = $derived(
    permissionsConfig?.groups.find((group) => group.id === selectedPermissionGroupId) ?? null
  );

  let showInitPanel = $state(false);
  let newSaveName = $state("Server");
  let initRunning = $state(false);
  let initDone = $state(false);
  let newSaveLogs = $state<string[]>([]);

  let saveHasRocket = $state<boolean | null>(null);
  let rocketInitRunning = $state(false);
  let rocketInitDone = $state(false);
  let rocketInitLogs = $state<string[]>([]);

  // 竞态保护：切换存档时丢弃过期的异步响应
  let loadGeneration = 0;
  let noteSaveTimer: ReturnType<typeof setTimeout> | undefined;
  let lastRawLines: string[] = [];

  function clearLoadedSaveData() {
    cmdName = "";
    cmdMap = "";
    cmdPort = 27015;
    cmdMaxPlayers = 24;
    cmdPassword = "";
    cmdOwner = "";
    cmdCheats = false;
    cmdPve = false;
    cmdPerspective = "Both";
    cmdGslt = "";
    rconPort = 27115;
    rconPassword = "";
    rconPasswordMasked = false;
    showRconPassword = false;
    plugins = [];
    pluginNotes = {};
    workshopConfig = null;
    workshopModNotes = {};
    permissionsConfig = null;
    selectedPermissionGroupId = "";
    lastRawLines = [];
  }

  async function loadSaves() {
    const gen = ++loadGeneration;
    try {
      await refreshRunningServers();
      saves = await invoke<SaveInfo[]>("list_server_saves");
      if (gen !== loadGeneration) return;
      sharedSaves.splice(0, sharedSaves.length, ...saves);
      sharedSavesState.loaded = true;
      const ensuredSaveId = ensureSelectedSaveId(saves);
      if (ensuredSaveId) {
        await loadCommandsDat();
        await checkRocketStatus();
        await loadActiveTabData();
      } else {
        saveHasRocket = null;
        clearLoadedSaveData();
      }
    } catch (e) { console.error("加载存档失败:", e); }
  }

  async function checkRocketStatus() {
    if (!selectedSaveId) {
      saveHasRocket = null;
      return;
    }
    try {
      saveHasRocket = await invoke("check_save_rocket_status", { saveId: selectedSaveId }) as boolean;
    } catch {
      saveHasRocket = false;
    }
  }

  async function initNewSave() {
    if (!newSaveName.trim()) return;
    initRunning = true;
    initDone = false;
    newSaveLogs = [];
    const unlisten = await listenInstallerProgress({
      onDone: () => {
        initDone = true;
        initRunning = false;
        loadSaves();
      },
      onError: (msg) => {
        alert(`初始化失败: ${msg}`);
        initRunning = false;
      },
      onProgress: (msg) => {
        newSaveLogs.push(msg);
        if (newSaveLogs.length > 200) newSaveLogs = newSaveLogs.slice(-100);
      },
    });
    try {
      await invoke("init_server_save", { saveName: newSaveName });
    } catch (e) {
      alert(`启动失败: ${e}`);
      initRunning = false;
      unlisten();
    }
  }

  async function initRocketForSave() {
    if (!selectedSaveId) return;
    if (selectedSaveRunning) {
      toastStore.error("该存档服务器正在运行，请停止后再修改");
      return;
    }
    rocketInitRunning = true;
    rocketInitDone = false;
    rocketInitLogs = [];
    const unlisten = await listenInstallerProgress({
      onDone: () => {
        rocketInitDone = true;
        rocketInitRunning = false;
        checkRocketStatus();
      },
      onError: (msg) => {
        alert(`初始化失败: ${msg}`);
        rocketInitRunning = false;
      },
      onProgress: (msg) => {
        rocketInitLogs.push(msg);
        if (rocketInitLogs.length > 200) rocketInitLogs = rocketInitLogs.slice(-100);
      },
    });
    try {
      await invoke("init_server_save", { saveName: selectedSaveId });
    } catch (e) {
      alert(`启动失败: ${e}`);
      rocketInitRunning = false;
      unlisten();
    }
  }

  async function loadCommandsDat() {
    if (!selectedSaveId) return;
    const gen = ++loadGeneration;
    loading = true;
    try {
      const info = await invoke<CommandsConfig>("read_commands_dat", { saveId: selectedSaveId });
      if (gen !== loadGeneration) return; // 过期响应，丢弃
      cmdName = info.name ?? "";
      cmdMap = info.map ?? "";
      cmdPort = info.port ?? 27015;
      cmdMaxPlayers = info.max_players ?? 24;
      cmdPassword = info.password ?? "";
      cmdOwner = info.owner ?? "";
      cmdCheats = info.cheats ?? false;
      cmdPve = info.pve ?? false;
      cmdPerspective = info.perspective ?? "Both";
      cmdGslt = info.gslt ?? "";
      lastRawLines = info.raw_lines ?? [];
    } catch (e) { console.error("读取 Commands.dat 失败:", e); }
    try {
      const rcon = await invoke<RocketRconInfo>("read_rocket_rcon_config", { saveId: selectedSaveId });
      if (gen !== loadGeneration) return;
      rconPort = rcon.port ?? 27115;
      rconPassword = rcon.password ?? "";
      rconPasswordMasked = rcon.has_password && rconPassword.includes("*");
    } catch (e) { console.error("读取 RCON 配置失败:", e); }
    if (gen === loadGeneration) loading = false;
  }

  async function saveCommandsDat() {
    if (selectedSaveRunning) {
      toastStore.error("该存档服务器正在运行，请停止后再修改");
      return;
    }
    saving = true;
    try {
      await invoke("save_commands_dat", {
        saveId: selectedSaveId,
        info: {
          name: cmdName || null,
          map: cmdMap || null,
          port: cmdPort > 0 ? cmdPort : null,
          max_players: cmdMaxPlayers > 0 ? cmdMaxPlayers : null,
          password: cmdPassword || null,
          owner: cmdOwner || null,
          cheats: cmdCheats,
          pve: cmdPve,
          perspective: cmdPerspective || null,
          gslt: cmdGslt || null,
          raw_lines: lastRawLines,
        },
      });
      await invoke("save_rocket_rcon_config", {
        saveId: selectedSaveId,
        port: rconPort,
        // 如果密码仍为掩码状态（用户未修改），发送空字符串表示保留原密码
        password: rconPasswordMasked ? "" : rconPassword,
      });
      toastStore.success("配置已保存");
    } catch (e) {
      alert(e);
    }
    saving = false;
  }

  async function loadPlugins() {
    if (!selectedSaveId) return;
    const gen = ++loadGeneration;
    pluginsLoading = true;
    try {
      const [p, n] = await Promise.all([
        invoke<PluginInfo[]>("list_plugins", { saveId: selectedSaveId }),
        invoke<Record<string, string>>("load_plugin_notes"),
      ]);
      if (gen !== loadGeneration) return;
      plugins = p;
      pluginNotes = n;
    } catch (e) { console.error("加载插件失败:", e); }
    if (gen === loadGeneration) pluginsLoading = false;
  }

  async function loadWorkshopConfig() {
    if (!selectedSaveId) return;
    const gen = ++loadGeneration;
    workshopLoading = true;
    try {
      const [wc, mn] = await Promise.all([
        invoke<WorkshopDownloadConfig>("read_workshop_config", { saveId: selectedSaveId }),
        invoke<Record<string, string>>("load_workshop_mod_notes"),
      ]);
      if (gen !== loadGeneration) return;
      workshopConfig = wc;
      workshopModNotes = mn;
      ignoreChildrenInput = wc.ignore_children_file_ids.join(", ");
    } catch (e) {
      console.error("加载创意工坊配置失败:", e);
      workshopConfig = {
        file_ids: [],
        ignore_children_file_ids: [],
        query_cache_max_age_seconds: 600,
        max_query_retries: 2,
        use_cached_downloads: true,
        should_monitor_updates: true,
        shutdown_update_detected_timer: 600,
        shutdown_update_detected_message: "Workshop file update detected, shutdown in: {0}",
        shutdown_kick_message: "Shutdown for Workshop file update.",
      };
      ignoreChildrenInput = "";
    }
    if (gen === loadGeneration) workshopLoading = false;
  }

  async function loadPermissionsConfig() {
    if (!selectedSaveId) return;
    const gen = ++loadGeneration;
    permissionsLoading = true;
    try {
      const config = await invoke("read_permissions_config", { saveId: selectedSaveId }) as PermissionsConfig;
      if (gen !== loadGeneration) return;
      permissionsConfig = config;
      if (config.exists && config.groups.length > 0) {
        const defaultGroup = config.groups.find((group) => group.id === config.default_group);
        selectedPermissionGroupId = defaultGroup?.id ?? config.groups[0].id;
      } else {
        selectedPermissionGroupId = "";
      }
      newMemberInput = "";
      newPermissionName = "";
      newPermissionCooldown = 0;
    } catch (e) {
      console.error("加载权限组配置失败:", e);
      alert(e);
    }
    if (gen === loadGeneration) permissionsLoading = false;
  }

  function uniqueGroupId(base: string) {
    if (!permissionsConfig) return base;
    const normalizedBase = (base || "group").trim().replace(/\s+/g, "_") || "group";
    const existing = new Set(permissionsConfig.groups.map((group) => group.id));
    if (!existing.has(normalizedBase)) return normalizedBase;
    let index = 2;
    while (existing.has(`${normalizedBase}_${index}`)) index += 1;
    return `${normalizedBase}_${index}`;
  }

  function getSelectedPermissionGroup() {
    return permissionsConfig?.groups.find((group) => group.id === selectedPermissionGroupId) ?? null;
  }

  function groupColorStyle(color: string) {
    const value = color.trim();
    if (!value) return "background: transparent";
    if (/^[0-9a-fA-F]{6}$/.test(value)) return `background: #${value}`;
    return `background: ${value}`;
  }

  const namedColorHex: Record<string, string> = {
    black: "#000000",
    blue: "#2563eb",
    cyan: "#06b6d4",
    gray: "#6b7280",
    green: "#16a34a",
    grey: "#6b7280",
    magenta: "#d946ef",
    orange: "#f97316",
    pink: "#ec4899",
    purple: "#9333ea",
    red: "#dc2626",
    white: "#ffffff",
    yellow: "#eab308",
  };

  function colorPickerValue(color: string) {
    const value = color.trim();
    if (/^[0-9a-fA-F]{6}$/.test(value)) return `#${value}`;
    if (/^#[0-9a-fA-F]{6}$/.test(value)) return value;
    return namedColorHex[value.toLowerCase()] ?? "#ffffff";
  }

  function setGroupColorFromPicker(groupId: string, value: string) {
    if (selectedSaveRunning) return;
    replacePermissionGroup(groupId, (group) => {
      group.color = value.replace("#", "").toUpperCase();
    });
  }

  function replacePermissionGroup(groupId: string, updater: (group: PermissionGroup) => void) {
    if (selectedSaveRunning) return;
    if (!permissionsConfig) return;
    const index = permissionsConfig.groups.findIndex((group) => group.id === groupId);
    if (index === -1) return;
    const groups = permissionsConfig.groups.map((group) => ({
      ...group,
      members: [...group.members],
      permissions: group.permissions.map((permission) => ({ ...permission })),
    }));
    updater(groups[index]);
    permissionsConfig = { ...permissionsConfig, groups };
  }

  function setDefaultPermissionGroup(groupId: string) {
    if (selectedSaveRunning) return;
    if (!permissionsConfig) return;
    permissionsConfig = { ...permissionsConfig, default_group: groupId };
  }

  function addPermissionGroup() {
    if (selectedSaveRunning) return;
    if (!permissionsConfig?.exists) return;
    const id = uniqueGroupId("new_group");
    const group: PermissionGroup = {
      id,
      display_name: "New Group",
      prefix: "",
      suffix: "",
      color: "white",
      members: [],
      parent_group: permissionsConfig.default_group || null,
      priority: 100,
      permissions: [],
    };
    const defaultGroup = permissionsConfig.default_group || id;
    permissionsConfig = {
      ...permissionsConfig,
      default_group: defaultGroup,
      groups: [...permissionsConfig.groups, group],
    };
    selectedPermissionGroupId = id;
  }

  function duplicatePermissionGroup(group: PermissionGroup) {
    if (selectedSaveRunning) return;
    if (!permissionsConfig?.exists) return;
    const id = uniqueGroupId(`${group.id}_copy`);
    const clone: PermissionGroup = {
      ...group,
      id,
      display_name: `${group.display_name || group.id} Copy`,
      members: [...group.members],
      permissions: group.permissions.map((permission) => ({ ...permission })),
    };
    permissionsConfig = { ...permissionsConfig, groups: [...permissionsConfig.groups, clone] };
    selectedPermissionGroupId = id;
  }

  function removePermissionGroup(group: PermissionGroup) {
    if (selectedSaveRunning) return;
    if (!permissionsConfig) return;
    if (group.id === permissionsConfig.default_group) {
      alert("不能删除默认权限组，请先切换默认组");
      return;
    }
    const child = permissionsConfig.groups.find((item) => item.parent_group === group.id);
    if (child) {
      alert(`权限组 ${child.id} 正在继承该组，请先解除父组引用`);
      return;
    }
    if (!confirm(`删除权限组 ${group.id}？`)) return;

    const groups = permissionsConfig.groups.filter((item) => item.id !== group.id);
    permissionsConfig = { ...permissionsConfig, groups };
    selectedPermissionGroupId = groups.find((item) => item.id === permissionsConfig?.default_group)?.id ?? groups[0]?.id ?? "";
  }

  function renameSelectedPermissionGroup(value: string) {
    if (selectedSaveRunning) return;
    if (!permissionsConfig) return;
    const oldId = selectedPermissionGroupId;
    replacePermissionGroup(oldId, (group) => {
      group.id = value;
    });
    const nextId = value;
    const groups = permissionsConfig.groups.map((group) => ({
      ...group,
      parent_group: group.parent_group === oldId ? nextId : group.parent_group,
    }));
    permissionsConfig = {
      ...permissionsConfig,
      default_group: permissionsConfig.default_group === oldId ? nextId : permissionsConfig.default_group,
      groups,
    };
    selectedPermissionGroupId = nextId;
  }

  function normalizeInputList(values: string[]) {
    const seen = new Set<string>();
    const output: string[] = [];
    for (const value of values) {
      const trimmed = value.trim();
      if (trimmed && !seen.has(trimmed)) {
        seen.add(trimmed);
        output.push(trimmed);
      }
    }
    return output;
  }

  function addMembersToSelectedGroup() {
    if (selectedSaveRunning) return;
    const group = getSelectedPermissionGroup();
    if (!group || !newMemberInput.trim()) return;
    const existing = new Set(group.members.map((member) => member.trim()).filter(Boolean));
    const incoming = normalizeInputList(newMemberInput.split(/[,\s]+/));
    const fresh = incoming.filter((member) => !existing.has(member));
    const duplicateCount = incoming.length - fresh.length;
    if (duplicateCount > 0) {
      toastStore.info(`已跳过 ${duplicateCount} 个重复 SteamID64`);
    }
    if (fresh.length === 0) {
      newMemberInput = "";
      return;
    }
    const members = [...normalizeInputList(group.members), ...fresh];
    replacePermissionGroup(group.id, (draft) => {
      draft.members = members;
    });
    newMemberInput = "";
  }

  function updateMember(groupId: string, index: number, value: string) {
    if (selectedSaveRunning) return;
    replacePermissionGroup(groupId, (group) => {
      group.members[index] = value;
    });
  }

  function removeMember(groupId: string, index: number) {
    if (selectedSaveRunning) return;
    replacePermissionGroup(groupId, (group) => {
      group.members.splice(index, 1);
    });
  }

  function normalizeMembersForGroup(groupId: string) {
    if (selectedSaveRunning) return;
    const group = permissionsConfig?.groups.find((item) => item.id === groupId);
    if (!group) return;
    const normalized = normalizeInputList(group.members);
    if (normalized.length !== group.members.filter((member) => member.trim()).length) {
      toastStore.info("已移除重复或空的 SteamID64");
      replacePermissionGroup(groupId, (draft) => {
        draft.members = normalized;
      });
    }
  }

  function addPermissionToSelectedGroup() {
    if (selectedSaveRunning) return;
    const group = getSelectedPermissionGroup();
    if (!group) return;
    if (!newPermissionName.trim()) {
      alert("请输入权限名");
      return;
    }
    replacePermissionGroup(group.id, (draft) => {
      draft.permissions.push({
        name: newPermissionName.trim(),
        cooldown: Math.max(0, Number(newPermissionCooldown) || 0),
      });
    });
    newPermissionName = "";
    newPermissionCooldown = 0;
  }

  function updatePermission(groupId: string, index: number, patch: Partial<PermissionEntry>) {
    if (selectedSaveRunning) return;
    replacePermissionGroup(groupId, (group) => {
      group.permissions[index] = { ...group.permissions[index], ...patch };
    });
  }

  function removePermission(groupId: string, index: number) {
    if (selectedSaveRunning) return;
    replacePermissionGroup(groupId, (group) => {
      group.permissions.splice(index, 1);
    });
  }

  function normalizedPermissionsPayload() {
    if (!permissionsConfig) throw new Error("权限组配置未加载");
    const groups = permissionsConfig.groups.map((group) => ({
      ...group,
      id: group.id.trim(),
      display_name: group.display_name.trim(),
      color: group.color.trim(),
      members: normalizeInputList(group.members),
      parent_group: group.parent_group?.trim() || null,
      priority: Number(group.priority) || 0,
      permissions: group.permissions.map((permission) => ({
        name: permission.name.trim(),
        cooldown: Math.max(0, Number(permission.cooldown) || 0),
      })),
    }));
    const payload = {
      ...permissionsConfig,
      default_group: permissionsConfig.default_group.trim(),
      groups,
    };
    validatePermissionsPayload(payload);
    return payload;
  }

  function validatePermissionsPayload(config: PermissionsConfig) {
    if (!config.exists) throw new Error("Permissions.config.xml 不存在");
    if (config.groups.length === 0) throw new Error("至少需要一个权限组");
    if (!config.default_group) throw new Error("默认权限组不能为空");

    const ids = new Set<string>();
    for (const group of config.groups) {
      if (!group.id) throw new Error("权限组 ID 不能为空");
      if (ids.has(group.id)) throw new Error(`权限组 ID 重复: ${group.id}`);
      ids.add(group.id);
      for (const permission of group.permissions) {
        if (!permission.name) throw new Error(`权限组 ${group.id} 包含空权限名`);
      }
    }
    if (!ids.has(config.default_group)) throw new Error(`默认权限组不存在: ${config.default_group}`);

    for (const group of config.groups) {
      if (!group.parent_group) continue;
      if (group.parent_group === group.id) throw new Error(`权限组 ${group.id} 不能继承自身`);
      if (!ids.has(group.parent_group)) throw new Error(`权限组 ${group.id} 的父组不存在: ${group.parent_group}`);
    }
  }

  async function savePermissionsConfig() {
    if (selectedSaveRunning) {
      toastStore.error("该存档服务器正在运行，请停止后再修改");
      return;
    }
    if (!selectedSaveId || !permissionsConfig) return;
    permissionsSaving = true;
    try {
      const payload = normalizedPermissionsPayload();
      await invoke("save_permissions_config", {
        saveId: selectedSaveId,
        permissionsConfig: payload,
      });
      permissionsConfig = payload;
      toastStore.success("权限组配置已保存");
    } catch (e) {
      alert(e);
    }
    permissionsSaving = false;
  }

  function parseWorkshopIdList(value: string, label: string): number[] {
    if (!value.trim()) return [];
    const parts = value.split(/[,\s]+/).map((s: string) => s.trim()).filter(Boolean);
    const invalid = parts.find((s: string) => !/^\d+$/.test(s));
    if (invalid) {
      throw new Error(`${label} 包含无效 ID: ${invalid}`);
    }
    return Array.from(new Set(parts.map((s: string) => Number.parseInt(s, 10))));
  }

  async function saveWorkshopConfig() {
    if (selectedSaveRunning) {
      toastStore.error("该存档服务器正在运行，请停止后再修改");
      return;
    }
    if (!selectedSaveId || !workshopConfig) return;
    workshopSaving = true;
    try {
      const normalizedConfig = {
        ...workshopConfig,
        ignore_children_file_ids: parseWorkshopIdList(ignoreChildrenInput, "忽略子项的模组 ID"),
      };
      await invoke("save_workshop_config", {
        saveId: selectedSaveId,
        workshopConfig: normalizedConfig,
      });
      await invoke("save_workshop_mod_notes", { notes: workshopModNotes });
      workshopConfig = normalizedConfig;
      ignoreChildrenInput = normalizedConfig.ignore_children_file_ids.join(", ");
      toastStore.success("创意工坊配置已保存");
    } catch (e) {
      alert(e);
    }
    workshopSaving = false;
  }

  function addWorkshopMod() {
    if (selectedSaveRunning) return;
    if (!newModId.trim() && !newModNote.trim()) {
      alert("请输入创意工坊 ID");
      return;
    }
    if (!newModId.trim()) {
      alert("请输入创意工坊 ID");
      return;
    }
    const ids = newModId.split(/[,\s]+/).map((s: string) => s.trim()).filter((s: string) => s && /^\d+$/.test(s));
    if (ids.length === 0) {
      alert("请输入有效的创意工坊 ID（纯数字）");
      return;
    }

    const currentIds: number[] = workshopConfig?.file_ids || [];
    const existingNotes = { ...workshopModNotes };

    for (const idStr of ids) {
      const id = parseInt(idStr, 10);
      if (!currentIds.includes(id)) {
        currentIds.push(id);
      }
      if (newModNote.trim()) {
        existingNotes[idStr] = newModNote.trim();
      }
    }

    workshopConfig = { ...workshopConfig, file_ids: currentIds };
    workshopModNotes = existingNotes;
    newModId = "";
    newModNote = "";
  }

  function removeWorkshopMod(modId: number) {
    if (selectedSaveRunning) return;
    if (!workshopConfig) return;
    workshopConfig = {
      ...workshopConfig,
      file_ids: workshopConfig.file_ids.filter((id: number) => id !== modId),
    };
  }

  function onWorkshopModNoteChange(modId: string, note: string) {
    if (selectedSaveRunning) return;
    workshopModNotes = { ...workshopModNotes, [modId]: note };
  }

  async function saveWorkshopModNotes() {
    try {
      await invoke("save_workshop_mod_notes", { notes: workshopModNotes });
    } catch (e) {
      alert(e);
    }
  }

  async function openWorkshopUrl() {
    const url = "https://steamcommunity.com/app/304930/workshop/";
    try {
      await openShell(url);
    } catch (e) {
      toastStore.error(`打开失败: ${e}`);
    }
  }

  async function openModPage(modId: number) {
    const url = `https://steamcommunity.com/sharedfiles/filedetails/?id=${modId}`;
    try {
      await openShell(url);
    } catch (e) {
      toastStore.error(`打开失败: ${e}`);
    }
  }

  async function openPluginDir() {
    try {
      await invoke("open_plugin_config_dir", { saveId: selectedSaveId });
    } catch (e) {
      alert(e);
    }
  }

  function onPluginNoteBlur(pluginName: string, note: string) {
    pluginNotes = { ...pluginNotes, [pluginName]: note };
    clearTimeout(noteSaveTimer);
    noteSaveTimer = setTimeout(async () => {
      try {
        await invoke("save_plugin_notes", { notes: pluginNotes });
    } catch (e) {
      alert(e);
      }
    }, 500);
  }

  async function loadActiveTabData() {
    if (activeTab === "plugins") {
      await loadPlugins();
    }
    if (activeTab === "workshop") {
      await loadWorkshopConfig();
    }
    if (activeTab === "permissions") {
      await loadPermissionsConfig();
    }
  }

  async function onSaveChange(value: string) {
    setSelectedSaveId(value);
    await refreshRunningServers();
    rocketInitDone = false;
    rocketInitRunning = false;
    await loadCommandsDat();
    await checkRocketStatus();
    await loadActiveTabData();
  }

  async function deleteSelectedSave() {
    if (!selectedSaveId || deletingSave) return;
    if (selectedSaveRunning) {
      toastStore.error("该存档服务器正在运行，请停止后再删除");
      return;
    }
    const save = saves.find((item) => item.id === selectedSaveId);
    const label = save?.name ? `${save.id} - ${save.name}` : selectedSaveId;
    const confirmed = confirm(
      `确定要删除存档 "${label}" 吗？\n\n这会删除该存档目录及其中的世界数据、配置、插件配置，无法撤销。`
    );
    if (!confirmed) return;

    deletingSave = true;
    try {
      await invoke("delete_server_save", { saveId: selectedSaveId });
      toastStore.success("存档已删除");
      await loadSaves();
    } catch (e) {
      toastStore.error(`删除失败: ${e}`);
    } finally {
      deletingSave = false;
    }
  }

  async function onTabChange(tab: SaveActiveTab) {
    setSaveActiveTab(tab);
    if (tab === "plugins") {
      await loadPlugins();
    }
    if (tab === "workshop") {
      await loadWorkshopConfig();
    }
    if (tab === "permissions") {
      await loadPermissionsConfig();
    }
  }

  $effect(() => {
    loadSaves();
  });

  // 组件卸载时清理定时器，防止内存泄露
  $effect(() => {
    return () => {
      if (noteSaveTimer) {
        clearTimeout(noteSaveTimer);
        noteSaveTimer = undefined;
      }
    };
  });
</script>

<div>
  <div class="flex flex-wrap items-center justify-between gap-3 mb-6">
    <div>
      <h1 class="text-2xl font-bold text-[var(--text-primary)]">存档管理</h1>
      <p class="text-sm text-[var(--text-muted)] mt-1">存档配置、模组、插件与权限组</p>
    </div>
  </div>

  <!-- Save Selector -->
  <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-4 mb-5">
    <div class="flex items-center gap-4 flex-wrap">
      <span class="text-sm text-[var(--text-secondary)]">选择存档:</span>
      {#if saves.length === 0}
        <span class="text-sm text-[var(--text-muted)]">未找到存档</span>
      {:else}
        <SelectCustom
          bind:value={uiPreferences.selectedSaveId}
          options={saves.map(save => ({
            value: save.id,
            label: `${save.id}${save.name ? ` - ${save.name}` : ''}`
          }))}
          onchange={onSaveChange}
          size="sm"
          class="min-w-[200px]"
        />
      {/if}
      <button
        class="px-4 py-2 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2"
        onclick={() => showInitPanel = !showInitPanel}
      >
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        新建存档
      </button>
      <button
        class="px-4 py-2 bg-[var(--bg-elevated)] border border-[var(--border)] hover:border-[var(--danger)] text-[var(--text-secondary)] hover:text-[var(--danger)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
        onclick={deleteSelectedSave}
        disabled={!selectedSaveId || deletingSave || selectedSaveRunning}
      >
        {#if deletingSave}
          <div class="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin"></div>
          删除中...
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6M9 7h6m-7 0V5a2 2 0 012-2h4a2 2 0 012 2v2m-9 0h10" />
          </svg>
          删除存档
        {/if}
      </button>
    </div>

    <!-- Init New Save Panel -->
    {#if showInitPanel}
      <div class="mt-4 pt-4 border-t border-[var(--border)]">
        <p class="text-xs text-[var(--text-muted)] mb-3">创建新存档，自动生成世界数据和配置文件</p>
        <div class="flex flex-col items-stretch gap-3 sm:flex-row sm:items-end">
          <div class="flex-1">
            <span class="block text-xs text-[var(--text-muted)] mb-1.5">存档名称</span>
            <input type="text" bind:value={newSaveName} placeholder="Server"
              disabled={initRunning || initDone}
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors disabled:opacity-50" />
            <p class="text-[10px] text-[var(--danger)] mt-1">仅支持英文、数字和下划线</p>
          </div>
          <button
            class="px-5 py-2 bg-gradient-to-r from-[var(--success)] to-emerald-600 hover:from-emerald-500 hover:to-[var(--success)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2 flex-shrink-0"
            onclick={initNewSave} disabled={initRunning || initDone || !newSaveName.trim()}
          >
            {#if initRunning}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              初始化中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z" />
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              开始初始化
            {/if}
          </button>
        </div>
        {#if initDone}
          <p class="text-xs text-[var(--success)] mt-2 flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            存档初始化成功！
          </p>
        {/if}
        {#if initRunning && newSaveLogs.length > 0}
          <div class="mt-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-32 overflow-y-auto">
            {#each newSaveLogs as log}
              <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if selectedSaveRunning}
    <div class="bg-[var(--warning-glow)] border border-[var(--warning)]/30 rounded-xl p-4 mb-5">
      <div class="flex items-center gap-3">
        <svg class="w-5 h-5 text-[var(--warning)] flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 11V7a4 4 0 00-8 0v4M5 11h14a2 2 0 012 2v7a2 2 0 01-2 2H5a2 2 0 01-2-2v-7a2 2 0 012-2z" />
        </svg>
        <div>
          <p class="text-sm font-medium text-[var(--warning)]">该存档服务器正在运行，配置已锁定</p>
          <p class="text-xs text-[var(--text-secondary)] mt-1">可以查看配置；如需修改、删除或初始化，请先停止该存档对应的服务器。</p>
        </div>
      </div>
    </div>
  {/if}

  <!-- Rocket Status Warning -->
  {#if selectedSaveId && saveHasRocket === false}
    <div class="bg-[var(--warning-glow)] border border-[var(--warning)]/30 rounded-xl p-4 mb-5">
      <div class="flex items-center gap-3 mb-2">
        <svg class="w-5 h-5 text-[var(--warning)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z" />
        </svg>
        <span class="text-sm text-[var(--warning)] font-medium">存档缺少 Rocket 配置</span>
      </div>
      <p class="text-xs text-[var(--text-secondary)] mb-3">运行一次服务端可自动生成 Rocket 配置</p>
      <div class="flex items-center gap-3">
        <button
          class="px-5 py-2 bg-gradient-to-r from-[var(--warning)] to-amber-600 hover:from-amber-500 hover:to-[var(--warning)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
          onclick={initRocketForSave} disabled={rocketInitRunning || rocketInitDone || selectedSaveRunning}
        >
          {#if rocketInitRunning}
            <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
            初始化中...
          {:else}
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
            初始化 Rocket 配置
          {/if}
        </button>
        {#if rocketInitDone}
          <span class="text-xs text-[var(--success)] flex items-center gap-1">
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" /></svg>
            初始化成功
          </span>
        {/if}
      </div>
      {#if rocketInitRunning && rocketInitLogs.length > 0}
        <div class="mt-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 max-h-32 overflow-y-auto">
          {#each rocketInitLogs as log}
            <p class="text-xs text-[var(--text-secondary)] leading-5 font-mono">{log}</p>
          {/each}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Tabs -->
  <div class="flex gap-2 mb-5 flex-wrap">
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'save' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('save')}
    >
      存档配置
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'gameConfig' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('gameConfig')}
    >
      高级配置
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'workshop' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('workshop')}
    >
      创意工坊模组
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'plugins' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('plugins')}
    >
      插件管理
    </button>
    <button
      class="px-4 py-2 rounded-lg text-sm font-medium transition-all cursor-pointer
        {activeTab === 'permissions' ? 'bg-[var(--accent-subtle)] text-[var(--accent-light)] border border-[var(--border-accent)]' : 'text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:bg-[var(--bg-card)] border border-transparent'}"
      onclick={() => onTabChange('permissions')}
    >
      权限组管理
    </button>
  </div>

  {#if activeTab === 'save'}
    <!-- Commands.dat Editor -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
        <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
        Commands.dat 配置
      </h2>

      {#if loading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else}
        <fieldset disabled={selectedSaveRunning} class="contents">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:gap-5">
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器名称</span>
            <input type="text" bind:value={cmdName} placeholder="My Unturned Server"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">地图</span>
            <input type="text" bind:value={cmdMap} placeholder="PEI"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">端口</span>
            <input type="number" bind:value={cmdPort} min="1024" max="65535"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">最大玩家数</span>
            <input type="number" bind:value={cmdMaxPlayers} min="1" max="200"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">服务器密码</span>
            <input type="text" bind:value={cmdPassword} placeholder="留空表示无密码"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>

          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">管理员 SteamID64</span>
            <input type="text" bind:value={cmdOwner} placeholder="76561198000000000"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
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
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">GSLT (Game Server Login Token)</span>
            <input type="text" bind:value={cmdGslt} placeholder="可选，用于在服务器浏览器中显示"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>
        </div>

        <div class="flex flex-wrap gap-4 sm:gap-8 mt-5 pt-5 border-t border-[var(--border)]">
          <span class="flex items-center gap-3 cursor-pointer">
            <button
              class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdPve ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
              onclick={() => cmdPve = !cmdPve}
              aria-label="切换 PvE 模式"
            >
              <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdPve ? 'translate-x-5' : 'translate-x-1'}"></div>
            </button>
            <span class="text-sm text-[var(--text-secondary)]">PvE 模式</span>
          </span>

          <span class="flex items-center gap-3 cursor-pointer">
            <button
              class="w-10 h-6 rounded-full transition-colors cursor-pointer {cmdCheats ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
              onclick={() => cmdCheats = !cmdCheats}
              aria-label="切换作弊模式"
            >
              <div class="w-4 h-4 rounded-full bg-white transform transition-transform {cmdCheats ? 'translate-x-5' : 'translate-x-1'}"></div>
            </button>
            <span class="text-sm text-[var(--text-secondary)]">启用作弊</span>
          </span>
        </div>
        </fieldset>

      {/if}
    </div>

    <!-- RCON Config -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6 mt-5">
      <h2 class="text-base font-semibold text-[var(--text-primary)] mb-5 flex items-center gap-2">
        <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
        </svg>
        RCON 配置
      </h2>
      {#if loading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else}
        <fieldset disabled={selectedSaveRunning} class="contents">
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2 md:gap-5">
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 端口</span>
            <input type="number" bind:value={rconPort} min="1024" max="65535"
              class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
          </div>
          <div>
            <span class="block text-xs text-[var(--text-muted)] mb-2 uppercase tracking-wider">RCON 密码</span>
            <div class="relative">
              <input type={showRconPassword ? "text" : "password"} bind:value={rconPassword} placeholder="输入 RCON 密码"
                class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2.5 pr-20 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              <div class="absolute right-2 top-1/2 -translate-y-1/2 flex gap-1">
                <button type="button"
                  class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
                  onclick={() => showRconPassword = !showRconPassword}
                  aria-label={showRconPassword ? "隐藏密码" : "显示密码"}
                >
                  {#if showRconPassword}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.542 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" /></svg>
                  {:else}
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" /><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" /></svg>
                  {/if}
                </button>
                <button type="button"
                  class="p-1.5 text-[var(--text-muted)] hover:text-[var(--text-primary)] transition-colors cursor-pointer"
                  onclick={() => rconPassword = generatePassword()}
                  aria-label="生成随机密码"
                  title="生成随机密码"
                >
                  <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 7a2 2 0 012 2m4 0a6 6 0 01-7.743 5.743L11 17H9v2H7v2H4a1 1 0 01-1-1v-2.586a1 1 0 01.293-.707l5.964-5.964A6 6 0 1121 9z" /></svg>
                </button>
              </div>
            </div>
          </div>
        </div>
        <div class="mt-4 px-4 py-3 rounded-lg bg-[var(--bg-primary)] border border-[var(--border)] text-xs text-[var(--text-muted)]">
          <svg class="w-4 h-4 inline mr-1 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
          每个存档独立配置，修改后点击"保存配置"
        </div>
        </fieldset>
      {/if}
    </div>

    <div class="mt-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
      <div>
        <p class="text-xs text-[var(--text-muted)]">同时保存服务器和 RCON 配置</p>
      </div>
      <button
        class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-blue-600 hover:from-blue-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center justify-center gap-2 shadow-lg disabled:opacity-40 disabled:cursor-not-allowed"
        onclick={saveCommandsDat}
        disabled={saving || loading || !selectedSaveId || selectedSaveRunning}
      >
        {#if saving}
          <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
          保存中...
        {:else}
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
          保存配置
        {/if}
      </button>
    </div>

  {:else if activeTab === 'gameConfig'}
    <GameConfigTab saveId={selectedSaveId} readonly={selectedSaveRunning} />

  {:else if activeTab === 'workshop'}
    <!-- Workshop Tab -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <div class="flex flex-wrap items-center justify-between gap-3 mb-5">
        <h2 class="text-base font-semibold text-[var(--text-primary)] flex items-center gap-2">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          创意工坊模组配置
        </h2>
        <button
          onclick={openWorkshopUrl}
          class="px-4 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--accent)] transition-all cursor-pointer flex items-center gap-2"
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
          </svg>
          打开创意工坊
        </button>
      </div>

      {#if workshopLoading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if workshopConfig}
        <!-- Add Mod Section -->
        <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 mb-5">
          <h3 class="text-sm font-medium text-[var(--text-primary)] mb-3">添加模组</h3>
          <div class="flex flex-col gap-3 sm:flex-row sm:items-end">
            <div class="flex-1">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">创意工坊 ID</span>
              <input type="text" bind:value={newModId} placeholder="输入 ID，多个用逗号或空格分隔"
                disabled={selectedSaveRunning}
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
            <div class="flex-1">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">备注（可选）</span>
              <input type="text" bind:value={newModNote} placeholder="添加中文备注..."
                disabled={selectedSaveRunning}
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
            <button
              class="px-5 py-2 bg-gradient-to-r from-[var(--accent)] to-cyan-600 hover:from-cyan-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center gap-2 flex-shrink-0"
              onclick={addWorkshopMod}
              disabled={selectedSaveRunning}
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              添加
            </button>
          </div>
        </div>

        <!-- Mod List -->
        <div class="mb-5">
          <h3 class="text-sm font-medium text-[var(--text-primary)] mb-3">已添加模组 ({workshopConfig.file_ids.length})</h3>
          {#if workshopConfig.file_ids.length === 0}
            <div class="text-center py-8 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg">
              <svg class="w-10 h-10 text-[var(--text-muted)] mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
              </svg>
              <p class="text-[var(--text-muted)] text-sm">暂无模组</p>
              <p class="text-[var(--text-muted)] text-xs mt-1">输入创意工坊 ID 添加</p>
            </div>
          {:else}
            <div class="space-y-2 max-h-[400px] overflow-y-auto pr-1">
              {#each workshopConfig.file_ids as modId, index (modId)}
                <div class="flex items-center gap-3 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-3 hover:border-[var(--accent)] transition-all group">
                  <div class="w-8 h-8 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center flex-shrink-0">
                    <span class="text-xs font-medium text-[var(--accent-light)]">{index + 1}</span>
                  </div>
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2">
                      <button
                        onclick={() => openModPage(modId)}
                        class="text-sm font-medium text-[var(--accent-light)] hover:underline cursor-pointer"
                      >
                        {modId}
                      </button>
                    </div>
                    <input
                      type="text"
                      value={workshopModNotes[String(modId)] || ""}
                      onblur={(e) => {
                        onWorkshopModNoteChange(String(modId), (e.target as HTMLInputElement).value);
                        saveWorkshopModNotes();
                      }}
                      placeholder="点击添加备注..."
                      class="w-full bg-transparent border-none text-xs text-[var(--text-muted)] placeholder:text-[var(--text-muted)] focus:outline-none focus:text-[var(--text-primary)] mt-1 p-0"
                    />
                  </div>
                  <button
                    class="p-1.5 text-[var(--text-muted)] hover:text-[var(--danger)] opacity-0 group-hover:opacity-100 transition-all cursor-pointer"
                    onclick={() => removeWorkshopMod(modId)}
                    disabled={selectedSaveRunning}
                    title="移除模组"
                  >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </div>

        <!-- Advanced Settings -->
        <details class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg">
          <summary class="px-4 py-3 cursor-pointer text-sm font-medium text-[var(--text-secondary)] hover:text-[var(--text-primary)] transition-colors">
            高级配置
          </summary>
          <div class="px-4 pb-4 pt-2 border-t border-[var(--border)]">
            <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">查询缓存时间（秒）</span>
                <input type="number" bind:value={workshopConfig.query_cache_max_age_seconds} min="0"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">最大查询重试次数</span>
                <input type="number" bind:value={workshopConfig.max_query_retries} min="0"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">更新检测关服倒计时（秒）</span>
                <input type="number" bind:value={workshopConfig.shutdown_update_detected_timer} min="0"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1.5">忽略子项的模组 ID</span>
                <input type="text" bind:value={ignoreChildrenInput} placeholder="多个用逗号分隔"
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
              </div>
            </div>

            <div class="flex flex-wrap gap-4 sm:gap-8 mt-4 pt-4 border-t border-[var(--border)]">
              <span class="flex items-center gap-3 cursor-pointer">
                <button
                  class="w-10 h-6 rounded-full transition-colors cursor-pointer {workshopConfig.use_cached_downloads ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
                  onclick={() => workshopConfig = { ...workshopConfig, use_cached_downloads: !workshopConfig.use_cached_downloads }}
                  aria-label="切换使用缓存下载"
                >
                  <div class="w-4 h-4 rounded-full bg-white transform transition-transform {workshopConfig.use_cached_downloads ? 'translate-x-5' : 'translate-x-1'}"></div>
                </button>
                <span class="text-sm text-[var(--text-secondary)]">使用缓存下载</span>
              </span>

              <span class="flex items-center gap-3 cursor-pointer">
                <button
                  class="w-10 h-6 rounded-full transition-colors cursor-pointer {workshopConfig.should_monitor_updates ? 'bg-[var(--success)]' : 'bg-[var(--border)]'}"
                  onclick={() => workshopConfig = { ...workshopConfig, should_monitor_updates: !workshopConfig.should_monitor_updates }}
                  aria-label="切换监控更新"
                >
                  <div class="w-4 h-4 rounded-full bg-white transform transition-transform {workshopConfig.should_monitor_updates ? 'translate-x-5' : 'translate-x-1'}"></div>
                </button>
                <span class="text-sm text-[var(--text-secondary)]">监控模组更新</span>
              </span>
            </div>

            <div class="mt-4">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">更新检测关服提示消息</span>
              <input type="text" bind:value={workshopConfig.shutdown_update_detected_message} placeholder="Workshop file update detected, shutdown in: {0}"
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
            <div class="mt-3">
              <span class="block text-xs text-[var(--text-muted)] mb-1.5">关服踢出消息</span>
              <input type="text" bind:value={workshopConfig.shutdown_kick_message} placeholder="Shutdown for Workshop file update."
                class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors" />
            </div>
          </div>
        </details>

        <!-- Save Button -->
        <div class="mt-5 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <p class="text-xs text-[var(--text-muted)]">保存到 WorkshopDownloadConfig.json</p>
          <button
            class="px-6 py-2.5 bg-gradient-to-r from-[var(--accent)] to-blue-600 hover:from-blue-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center justify-center gap-2 shadow-lg disabled:opacity-40 disabled:cursor-not-allowed"
            onclick={saveWorkshopConfig}
            disabled={workshopSaving || workshopLoading || !selectedSaveId || selectedSaveRunning}
          >
            {#if workshopSaving}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              保存中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              保存创意工坊配置
            {/if}
          </button>
        </div>
      {/if}
    </div>

  {:else if activeTab === 'permissions'}
    <!-- Permissions Tab -->
    {#if permissionsLoading}
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      </div>
    {:else if permissionsConfig && !permissionsConfig.exists}
      <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
        <div class="flex flex-col items-center justify-center py-12 text-center">
          <div class="mb-4 flex h-12 w-12 items-center justify-center rounded-lg bg-[var(--warning-glow)] text-[var(--warning)]">
            <svg class="h-6 w-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M12 9v4m0 4h.01M10.29 3.86 1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
            </svg>
          </div>
          <h2 class="text-base font-semibold text-[var(--text-primary)]">Permissions.config.xml 不存在</h2>
          <p class="mt-2 max-w-2xl text-sm text-[var(--text-secondary)]">当前存档没有 Rocket 权限配置文件，软件不会自动创建该文件。</p>
          <p class="mt-3 max-w-full break-all rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-2 font-mono text-xs text-[var(--text-muted)]">{permissionsConfig.path}</p>
        </div>
      </div>
    {:else if permissionsConfig}
      <div class="mb-5 flex flex-col gap-3 rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-4 sm:flex-row sm:items-center sm:justify-between">
        <div class="min-w-0">
          <h2 class="text-base font-semibold text-[var(--text-primary)] flex items-center gap-2">
            <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M12 3 4 6v6c0 5 3.4 8.7 8 9 4.6-.3 8-4 8-9V6l-8-3z" />
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.7" d="M9 12l2 2 4-4" />
            </svg>
            权限组管理
          </h2>
          <p class="mt-1 truncate font-mono text-xs text-[var(--text-muted)]">{permissionsConfig.path}</p>
        </div>
        <div class="flex flex-col gap-2 sm:flex-row sm:items-center">
          <label class="flex items-center gap-2 text-sm text-[var(--text-secondary)]">
            <span class="whitespace-nowrap">默认组</span>
            <SelectCustom
              value={permissionsConfig.default_group}
              options={permissionsConfig.groups.map(g => ({ value: g.id, label: g.id }))}
              onchange={(val) => setDefaultPermissionGroup(val)}
              disabled={selectedSaveRunning}
              size="sm"
              class="min-w-[160px]"
            />
          </label>
          <button
            class="px-5 py-2 bg-gradient-to-r from-[var(--accent)] to-blue-600 hover:from-blue-500 hover:to-[var(--accent)] text-[var(--text-primary)] text-sm font-medium rounded-lg transition-all cursor-pointer flex items-center justify-center gap-2 whitespace-nowrap disabled:opacity-40 disabled:cursor-not-allowed"
            onclick={savePermissionsConfig}
            disabled={permissionsSaving || permissionsLoading || !selectedSaveId || selectedSaveRunning}
          >
            {#if permissionsSaving}
              <div class="w-4 h-4 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
              保存中...
            {:else}
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
              </svg>
              保存权限组
            {/if}
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 gap-5 xl:grid-cols-[minmax(260px,320px)_minmax(0,1fr)]">
        <aside class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-4">
          <div class="mb-4 flex items-center justify-between gap-3">
            <div>
              <h3 class="text-sm font-semibold text-[var(--text-primary)]">权限组</h3>
              <p class="text-xs text-[var(--text-muted)]">{permissionsConfig.groups.length} 个组</p>
            </div>
            <button
              class="flex items-center gap-2 rounded-lg bg-[var(--accent-subtle)] px-3 py-2 text-sm font-medium text-[var(--accent-light)] transition-all hover:border-[var(--accent)]"
              onclick={addPermissionGroup}
            >
              <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
              </svg>
              新增
            </button>
          </div>

          {#if permissionsConfig.groups.length === 0}
            <div class="rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-8 text-center">
              <p class="text-sm text-[var(--text-muted)]">暂无权限组</p>
            </div>
          {:else}
            <div class="max-h-[620px] space-y-2 overflow-y-auto pr-1">
              {#each permissionsConfig.groups as group, index (index)}
                <div class="flex items-center gap-2 rounded-lg border p-2 transition-all {selectedPermissionGroupId === group.id ? 'border-[var(--border-accent)] bg-[var(--accent-subtle)]' : 'border-[var(--border)] bg-[var(--bg-primary)] hover:border-[var(--border-hover)]'}">
                  <button
                    class="flex min-w-0 flex-1 items-center gap-3 text-left"
                    onclick={() => selectedPermissionGroupId = group.id}
                  >
                    <span class="h-8 w-2 shrink-0 rounded-full border border-[var(--border)]" style={groupColorStyle(group.color)}></span>
                    <span class="min-w-0 flex-1">
                      <span class="block truncate text-sm font-medium text-[var(--text-primary)]">{group.id || "未命名组"}</span>
                      <span class="block truncate text-xs text-[var(--text-muted)]">{group.display_name || "无显示名"} · {group.permissions.length} 权限</span>
                    </span>
                    {#if permissionsConfig.default_group === group.id}
                      <span class="rounded-md bg-[var(--success-glow)] px-2 py-0.5 text-[10px] font-medium text-[var(--success)]">默认</span>
                    {/if}
                  </button>
                  <button
                    class="rounded-md p-1.5 text-[var(--text-muted)] transition-colors hover:bg-[var(--bg-card)] hover:text-[var(--text-primary)]"
                    onclick={() => duplicatePermissionGroup(group)}
                    title="复制权限组"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M8 8h10v10H8z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 16H5a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1h10a1 1 0 0 1 1 1v1" />
                    </svg>
                  </button>
                  <button
                    class="rounded-md p-1.5 text-[var(--text-muted)] transition-colors hover:bg-[var(--danger-glow)] hover:text-[var(--danger)] disabled:opacity-35"
                    onclick={() => removePermissionGroup(group)}
                    disabled={permissionsConfig.default_group === group.id}
                    title="删除权限组"
                  >
                    <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M19 7 18.1 19.1A2 2 0 0 1 16.1 21H7.9a2 2 0 0 1-2-1.9L5 7m5 4v6m4-6v6M4 7h16m-5 0V4H9v3" />
                    </svg>
                  </button>
                </div>
              {/each}
            </div>
          {/if}
        </aside>

        <section class="min-w-0 space-y-5">
          {#if selectedPermissionGroup}
            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-5">
              <div class="mb-4 flex flex-wrap items-center justify-between gap-3">
                <h3 class="text-sm font-semibold text-[var(--text-primary)]">组信息</h3>
                <button
                  class="rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-2 text-sm text-[var(--text-secondary)] transition-all hover:border-[var(--accent)] hover:text-[var(--text-primary)]"
                  onclick={() => setDefaultPermissionGroup(selectedPermissionGroup!.id)}
                  disabled={permissionsConfig.default_group === selectedPermissionGroup.id || selectedSaveRunning}
                >
                  设为默认组
                </button>
              </div>
              <div class="grid grid-cols-[repeat(auto-fit,minmax(220px,1fr))] gap-4">
                <div>
                  <span class="block text-xs text-[var(--text-muted)] mb-1.5">组 ID</span>
                  <input
                    type="text"
                    value={selectedPermissionGroup.id}
                    oninput={(e) => renameSelectedPermissionGroup((e.target as HTMLInputElement).value)}
                    class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                  />
                </div>
                <div>
                  <span class="block text-xs text-[var(--text-muted)] mb-1.5">显示名称</span>
                  <input
                    type="text"
                    value={selectedPermissionGroup.display_name}
                    oninput={(e) => replacePermissionGroup(selectedPermissionGroup!.id, (group) => group.display_name = (e.target as HTMLInputElement).value)}
                    class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                  />
                </div>
                <div>
                  <span class="block text-xs text-[var(--text-muted)] mb-1.5">前缀</span>
                  <input
                    type="text"
                    value={selectedPermissionGroup.prefix}
                    oninput={(e) => replacePermissionGroup(selectedPermissionGroup!.id, (group) => group.prefix = (e.target as HTMLInputElement).value)}
                    class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                  />
                </div>
                <div>
                  <span class="block text-xs text-[var(--text-muted)] mb-1.5">后缀</span>
                  <input
                    type="text"
                    value={selectedPermissionGroup.suffix}
                    oninput={(e) => replacePermissionGroup(selectedPermissionGroup!.id, (group) => group.suffix = (e.target as HTMLInputElement).value)}
                    class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                  />
                </div>
                <div>
                  <span class="block text-xs text-[var(--text-muted)] mb-1.5">颜色</span>
                  <div class="flex gap-2">
                    <label class="relative h-10 w-10 shrink-0 cursor-pointer overflow-hidden rounded-lg border border-[var(--border)] shadow-[var(--shadow-sm)]" title="打开调色板">
                      <span class="absolute inset-0" style={groupColorStyle(selectedPermissionGroup.color)}></span>
                      <input
                        type="color"
                        value={colorPickerValue(selectedPermissionGroup.color)}
                        oninput={(e) => setGroupColorFromPicker(selectedPermissionGroup!.id, (e.target as HTMLInputElement).value)}
                        class="absolute inset-0 h-full w-full cursor-pointer opacity-0"
                        aria-label="选择权限组颜色"
                      />
                    </label>
                    <input
                      type="text"
                      value={selectedPermissionGroup.color}
                      oninput={(e) => replacePermissionGroup(selectedPermissionGroup!.id, (group) => group.color = (e.target as HTMLInputElement).value)}
                      placeholder="点击色块选择颜色"
                      class="min-w-0 flex-1 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                    />
                  </div>
                </div>
                <div>
                  <span class="block text-xs text-[var(--text-muted)] mb-1.5">优先级</span>
                  <input
                    type="number"
                    min="0"
                    value={selectedPermissionGroup.priority}
                    oninput={(e) => replacePermissionGroup(selectedPermissionGroup!.id, (group) => group.priority = Math.max(0, Number((e.target as HTMLInputElement).value) || 0))}
                    class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                  />
                </div>
                <div class="min-w-0">
                  <label for="parent-group" class="block text-xs text-[var(--text-muted)] mb-1.5">父组</label>
                  <SelectCustom
                    value={selectedPermissionGroup.parent_group ?? ""}
                    options={[
                      { value: '', label: '无父组' },
                      ...permissionsConfig.groups
                        .filter((group) => group.id !== selectedPermissionGroup?.id)
                        .map(g => ({
                          value: g.id,
                          label: g.display_name ? `${g.id} - ${g.display_name}` : g.id
                        }))
                    ]}
                    onchange={(val) => replacePermissionGroup(selectedPermissionGroup!.id, (group) => group.parent_group = val || null)}
                    size="md"
                    fullWidth
                  />
                </div>
              </div>
            </div>

            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-5">
              <div class="mb-4 flex flex-wrap items-end justify-between gap-3">
                <div>
                  <h3 class="text-sm font-semibold text-[var(--text-primary)]">成员</h3>
                  <p class="text-xs text-[var(--text-muted)]">{selectedPermissionGroup.members.length} 个 SteamID64</p>
                </div>
                <div class="flex flex-1 flex-col gap-2 sm:max-w-2xl sm:flex-row">
                  <input
                    type="text"
                    bind:value={newMemberInput}
                    placeholder="SteamID64，多个用空格或逗号分隔，重复会跳过"
                    class="min-w-0 flex-1 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                    onkeydown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        addMembersToSelectedGroup();
                      }
                    }}
                  />
                  <button
                    class="px-4 py-2 bg-[var(--accent-subtle)] text-[var(--accent-light)] text-sm font-medium rounded-lg transition-all cursor-pointer"
                    onclick={addMembersToSelectedGroup}
                  >
                    添加成员
                  </button>
                </div>
              </div>

              {#if selectedPermissionGroup.members.length === 0}
                <div class="rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-8 text-center text-sm text-[var(--text-muted)]">暂无成员</div>
              {:else}
                <div class="grid grid-cols-[repeat(auto-fit,minmax(220px,1fr))] gap-2">
                  {#each selectedPermissionGroup.members as member, index (index)}
                    <div class="flex items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] p-2">
                      <input
                        type="text"
                        value={member}
                        oninput={(e) => updateMember(selectedPermissionGroup!.id, index, (e.target as HTMLInputElement).value)}
                        onblur={() => normalizeMembersForGroup(selectedPermissionGroup!.id)}
                        class="min-w-0 flex-1 bg-transparent px-2 py-1 font-mono text-xs text-[var(--text-primary)] focus:outline-none"
                      />
                      <button
                        class="rounded-md p-1.5 text-[var(--text-muted)] transition-colors hover:bg-[var(--danger-glow)] hover:text-[var(--danger)]"
                        onclick={() => removeMember(selectedPermissionGroup!.id, index)}
                        title="移除成员"
                      >
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M6 6l12 12M18 6 6 18" />
                        </svg>
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>

            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-5">
              <div class="mb-4 flex flex-wrap items-end justify-between gap-3">
                <div>
                  <h3 class="text-sm font-semibold text-[var(--text-primary)]">权限</h3>
                  <p class="text-xs text-[var(--text-muted)]">{selectedPermissionGroup.permissions.length} 条权限</p>
                </div>
                <div class="grid flex-1 grid-cols-1 gap-2 sm:max-w-2xl sm:grid-cols-[minmax(180px,1fr)_150px_auto]">
                  <input
                    type="text"
                    bind:value={newPermissionName}
                    placeholder="权限名，例如 rocket 或 heal"
                    class="min-w-0 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                    onkeydown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        addPermissionToSelectedGroup();
                      }
                    }}
                  />
                  <div class="relative">
                    <input
                      type="number"
                      min="0"
                      bind:value={newPermissionCooldown}
                      aria-label="冷却时间，单位秒"
                      placeholder="冷却时间"
                      class="w-full bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg px-4 py-2 pr-9 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                    />
                    <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-[var(--text-muted)]">秒</span>
                  </div>
                  <button
                    class="px-4 py-2 bg-[var(--accent-subtle)] text-[var(--accent-light)] text-sm font-medium rounded-lg transition-all cursor-pointer whitespace-nowrap"
                    onclick={addPermissionToSelectedGroup}
                  >
                    添加权限
                  </button>
                </div>
              </div>

              {#if selectedPermissionGroup.permissions.length === 0}
                <div class="rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-8 text-center text-sm text-[var(--text-muted)]">暂无权限</div>
              {:else}
                <div class="space-y-2">
                  {#each selectedPermissionGroup.permissions as permission, index (index)}
                    <div class="grid grid-cols-1 gap-2 rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] p-2 sm:grid-cols-[minmax(180px,1fr)_150px_auto]">
                      <input
                        type="text"
                        value={permission.name}
                        oninput={(e) => updatePermission(selectedPermissionGroup!.id, index, { name: (e.target as HTMLInputElement).value })}
                        class="min-w-0 bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                      />
                      <div class="relative">
                        <input
                          type="number"
                          min="0"
                          value={permission.cooldown}
                          aria-label="冷却时间，单位秒"
                          oninput={(e) => updatePermission(selectedPermissionGroup!.id, index, { cooldown: Math.max(0, Number((e.target as HTMLInputElement).value) || 0) })}
                          class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-3 py-2 pr-9 text-sm text-[var(--text-primary)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                        />
                        <span class="pointer-events-none absolute right-3 top-1/2 -translate-y-1/2 text-xs text-[var(--text-muted)]">秒</span>
                      </div>
                      <button
                        class="rounded-lg px-3 py-2 text-sm text-[var(--text-muted)] transition-colors hover:bg-[var(--danger-glow)] hover:text-[var(--danger)]"
                        onclick={() => removePermission(selectedPermissionGroup!.id, index)}
                      >
                        删除
                      </button>
                    </div>
                  {/each}
                </div>
              {/if}
            </div>
          {:else}
            <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] px-4 py-12 text-center text-sm text-[var(--text-muted)]">请选择或新增一个权限组</div>
          {/if}
        </section>
      </div>
    {/if}

  {:else if activeTab === 'plugins'}
    <!-- Plugins Tab -->
    <div class="bg-[var(--bg-card)] border border-[var(--border)] rounded-xl p-6">
      <div class="flex flex-wrap items-center justify-between gap-3 mb-5">
        <h2 class="text-base font-semibold text-[var(--text-primary)] flex items-center gap-2">
          <svg class="w-5 h-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
          </svg>
          已安装插件
        </h2>
        <button
          class="px-4 py-2 bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg text-sm text-[var(--text-secondary)] hover:text-[var(--text-primary)] hover:border-[var(--accent)] transition-all cursor-pointer flex items-center gap-2"
          onclick={openPluginDir}
        >
          <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 19a2 2 0 01-2-2V7a2 2 0 012-2h4l2 2h4a2 2 0 012 2v1M5 19h14a2 2 0 002-2v-5a2 2 0 00-2-2H9a2 2 0 00-2 2v5a2 2 0 01-2 2z" />
          </svg>
          打开插件目录
        </button>
      </div>

      {#if pluginsLoading}
        <div class="flex items-center justify-center py-10">
          <div class="w-8 h-8 border-2 border-[var(--accent)] border-t-transparent rounded-full animate-spin"></div>
        </div>
      {:else if plugins.length === 0}
        <div class="text-center py-10">
          <svg class="w-12 h-12 text-[var(--text-muted)] mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
          </svg>
          <p class="text-[var(--text-muted)] text-sm">未找到插件</p>
          <p class="text-[var(--text-muted)] text-xs mt-1">请将插件放入 Rocket/Plugins 目录</p>
        </div>
      {:else}
        <div class="space-y-3">
          {#each plugins as plugin (plugin.file_name)}
            <div class="bg-[var(--bg-primary)] border border-[var(--border)] rounded-lg p-4 hover:border-[var(--accent)] transition-all">
              <div class="flex items-center gap-3 mb-3">
                <div class="w-8 h-8 rounded-lg bg-[var(--accent-subtle)] flex items-center justify-center flex-shrink-0">
                  <svg class="w-4 h-4 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20 7l-8-4-8 4m16 0l-8 4m8-4v10l-8 4m0-10L4 7m8 4v10M4 7v10l8 4" />
                  </svg>
                </div>
                <div class="flex-1 min-w-0">
                  <p class="text-sm font-medium text-[var(--text-primary)] truncate">{plugin.name}</p>
                  <p class="text-xs text-[var(--text-muted)] truncate">{plugin.file_name}</p>
                </div>
              </div>
              <div>
                <span class="block text-xs text-[var(--text-muted)] mb-1">备注</span>
                <input
                  type="text"
                  value={pluginNotes[plugin.name] || ""}
                  onblur={(e) => onPluginNoteBlur(plugin.name, (e.target as HTMLInputElement).value)}
                  placeholder="添加中文备注..."
                  class="w-full bg-[var(--bg-card)] border border-[var(--border)] rounded-lg px-3 py-2 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] focus:outline-none focus:border-[var(--accent)] transition-colors"
                />
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>


