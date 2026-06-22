<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import SelectCustom from "$lib/components/SelectCustom.svelte";
  import { toastStore } from "$lib/stores/toast.svelte";

  type GameConfigEntry = {
    id: string;
    section: string;
    key: string;
    value: string;
    has_value: boolean;
    value_kind: string;
    description: string[];
    default_hint: string | null;
    options: string[];
  };

  type GameConfigSection = {
    name: string;
    entries: GameConfigEntry[];
  };

  type GameConfigInfo = {
    exists: boolean;
    path: string;
    version: string | null;
    source_hash: string;
    line_ending: string;
    sections: GameConfigSection[];
  };

  const SECTION_LABELS: Record<string, string> = {
    Browser: "服务器展示",
    Server: "服务器基础",
    UnityEvents: "Unity 事件",
    Items: "物品刷新",
    Vehicles: "载具",
    Zombies: "僵尸",
    Animals: "动物",
    Barricades: "路障",
    Structures: "建筑",
    Players: "玩家",
    Objects: "世界物件",
    Events: "事件",
    Gameplay: "玩法",
  };

  const KEY_LABELS: Record<string, string> = {
    Icon: "大厅图标",
    Thumbnail: "服务器列表缩略图",
    Desc_Hint: "大厅短描述",
    Desc_Full: "大厅详细描述",
    Desc_Server_List: "服务器列表短描述",
    Login_Token: "登录令牌",
    BookmarkHost: "收藏主机地址",
    Is_Using_Anycast_Proxy: "使用 Anycast 代理",
    Monetization: "商业化类型",
    Links: "大厅按钮链接",
    VAC_Secure: "启用 VAC 反作弊",
    BattlEye_Secure: "启用 BattlEye 反作弊",
    Max_Ping_Milliseconds: "最大延迟",
    Timeout_Queue_Seconds: "排队超时",
    Timeout_Game_Seconds: "游戏内超时",
    Max_Packets_Per_Second: "每秒最大数据包",
    Enable_Scheduled_Shutdown: "启用定时关服",
    Scheduled_Shutdown_Time: "定时关服时间",
    Scheduled_Shutdown_Warnings: "定时关服警告",
    Enable_Update_Shutdown: "启用更新关服",
    Update_Steam_Beta_Name: "更新检测分支",
    Update_Shutdown_Warnings: "更新关服警告",
    Chat_Always_Use_Rich_Text: "聊天始终使用富文本",
    Use_FakeIP: "使用 Fake IP",
    Spawn_Chance: "刷新比例",
    Loot_Chance: "掉落概率",
    Respawn_Time: "重生时间",
    Despawn_Dropped_Time: "玩家掉落物消失时间",
    Despawn_Natural_Time: "自然刷新物消失时间",
    Quality_Full_Chance: "满品质概率",
    Quality_Multiplier: "品质倍率",
    Has_Durability: "启用耐久",
    Weapons_Spawn_At_Full_Quality: "武器满品质刷新",
    Weapons_Have_Durability: "武器有耐久消耗",
    Gun_Lowcal_Damage_Multiplier: "低口径枪械对载具伤害倍率",
    Gun_Highcal_Damage_Multiplier: "高口径枪械对载具伤害倍率",
    Damage_Multiplier: "伤害倍率",
    Armor_Multiplier: "护甲/承伤倍率",
    Health_Default: "初始生命值",
    Food_Default: "初始饥饿值",
    Water_Default: "初始口渴值",
    Virus_Default: "初始免疫值",
    Experience_Multiplier: "经验倍率",
    Lose_Experience_PvP: "PvP 死亡保留经验比例",
    Lose_Experience_PvE: "PvE 死亡保留经验比例",
    Lose_Items_PvP: "PvP 死亡掉落物品概率",
    Lose_Items_PvE: "PvE 死亡掉落物品概率",
    Lose_Clothes_PvP: "PvP 死亡掉落衣物",
    Lose_Clothes_PvE: "PvE 死亡掉落衣物",
    Lose_Weapons_PvP: "PvP 死亡掉落武器",
    Lose_Weapons_PvE: "PvE 死亡掉落武器",
    Can_Hurt_Legs: "允许摔伤腿部",
    Can_Break_Legs: "允许摔断腿",
    Can_Fix_Legs: "允许断腿自动恢复",
    Can_Start_Bleeding: "允许开始流血",
    Can_Stop_Bleeding: "允许流血自动停止",
    Spawn_With_Max_Skills: "出生满技能",
    Spawn_With_Stamina_Skills: "出生满体力类技能",
    Allow_Per_Character_Saves: "允许每角色独立存档",
    Hitmarkers: "命中提示",
    Crosshair: "准星",
    Ballistics: "弹道下坠",
    Chart: "默认拥有纸质地图",
    Satellite: "默认拥有 GPS",
    Compass: "默认拥有指南针",
    Group_Map: "地图显示队友",
    Group_HUD: "HUD 显示队友",
    Group_Player_List: "玩家列表显示队伍关系",
    Allow_Static_Groups: "允许 Steam 群组队伍",
    Allow_Dynamic_Groups: "允许游戏内动态组队",
    Friendly_Fire: "组队友伤",
    Can_Suicide: "允许自杀",
    Max_Group_Members: "最大队伍人数",
    Timer_Exit: "退出等待时间",
    Timer_Respawn: "复活等待时间",
    Timer_Home: "床复活等待时间",
    Timer_Leave_Group: "离队等待时间",
    Max_Clients_With_Same_IP_Address: "同 IP 最大连接数",
    Max_Clients_With_Same_IP_Address_Log_Warnings: "记录同 IP 限制警告",
    Fake_Lag_Threshold_Seconds: "疑似卡延迟阈值",
    Fake_Lag_Log_Warnings: "记录卡延迟警告",
    Fake_Lag_Damage_Penalty_Multiplier: "卡延迟伤害惩罚倍率",
    Enable_Kick_Input_Spam: "踢出输入刷屏玩家",
    Enable_Kick_Input_Timeout: "踢出长时间无输入玩家",
    Validate_EconInfo_Hash: "校验 EconInfo 哈希",
    Reset_Vehicles_Outside_Horizontal_Distance: "越界载具重置距离",
    Allow_Server_Messages: "允许服务端广播消息",
    Allow_Server_Commands: "允许服务端执行命令",
    Allow_Client_Messages: "允许客户端广播消息",
    Allow_Client_Commands: "允许客户端执行命令",
    Food_Spawns_At_Full_Quality: "食物满品质刷新",
    Water_Spawns_At_Full_Quality: "饮品满品质刷新",
    Clothing_Spawns_At_Full_Quality: "衣物满品质刷新",
    Clothing_Has_Durability: "衣物有耐久消耗",
    Crawler_Chance: "爬行僵尸概率",
    Sprinter_Chance: "奔跑僵尸概率",
    Flanker_Chance: "绕后僵尸概率",
    Burner_Chance: "燃烧僵尸概率",
    Acid_Chance: "酸液僵尸概率",
    Boss_Electric_Chance: "电系 Boss 概率",
    Boss_Wind_Chance: "震地 Boss 概率",
    Boss_Fire_Chance: "喷火 Boss 概率",
    Respawn_Day_Time: "白天重生时间",
    Respawn_Night_Time: "夜晚重生时间",
    Beacon_Experience_Multiplier: "尸潮信标经验倍率",
    Full_Moon_Experience_Multiplier: "满月经验倍率",
    Repair_Level_Max: "最高维修等级限制",
    Allow_Lobby_Groups: "允许大厅自动组队",
    Allow_Shoulder_Camera: "允许肩后视角",
    Bypass_Buildable_Mobility: "允许载具上特殊建造",
    Allow_Holidays: "允许节日内容",
    Allow_Freeform_Buildables: "允许自由摆放建筑",
    Allow_Freeform_Buildables_On_Vehicles: "允许载具自由摆放",
    Enable_Damage_Flinch: "启用受伤晃动",
    Enable_Explosion_Camera_Shake: "启用爆炸镜头震动",
    Enable_Workstation_Requirements: "启用工作站需求",
    Disable_Motion_Sickness_Options: "忽略防晕动选项",
    Use_2D_Scope_Overlay: "使用 2D 瞄准镜遮罩",
    Explosion_Launch_Speed_Multiplier: "爆炸击飞速度倍率",
    Bypass_No_Building_Zones: "允许无建筑区建造",
    Bypass_Building_In_Safezones: "允许安全区建造",
    Disable_Foliage_Off: "禁止关闭最低植被",
    Enable_Fishing_Catch_Challenge: "启用钓鱼挑战",
    Gun_Bullets_Full_Chance: "枪械满弹药概率",
    Magazine_Bullets_Full_Chance: "弹匣满弹药概率",
    Crate_Bullets_Full_Chance: "弹药箱满弹药概率",
    Beacon_Max_Rewards: "尸潮信标最大掉落数",
    Beacon_Rewards_Multiplier: "尸潮信标掉落倍率",
    Armor_Hightier_Multiplier: "高级护甲承伤倍率",
    Health_Regen_Min_Food: "回血所需最低饥饿值",
    Health_Regen_Min_Water: "回血所需最低口渴值",
    Lose_Skills_PvP: "PvP 死亡丢失技能概率",
    Lose_Skills_PvE: "PvE 死亡丢失技能概率",
    Lose_Skill_Levels_PvP: "PvP 死亡损失技能等级",
    Lose_Skill_Levels_PvE: "PvE 死亡损失技能等级",
    Skillset_Reduces_Skill_Cost: "专长降低技能消耗",
    Skillset_Prevents_Skill_Loss: "专长防止技能丢失",
  };

  const SECTION_KEY_LABELS: Record<string, string> = {
    "Vehicles.Gun_Lowcal_Damage_Multiplier": "低口径枪械对载具伤害倍率",
    "Vehicles.Gun_Highcal_Damage_Multiplier": "高口径枪械对载具伤害倍率",
    "Barricades.Gun_Lowcal_Damage_Multiplier": "低口径枪械对路障伤害倍率",
    "Barricades.Gun_Highcal_Damage_Multiplier": "高口径枪械对路障伤害倍率",
    "Structures.Gun_Lowcal_Damage_Multiplier": "低口径枪械对建筑伤害倍率",
    "Structures.Gun_Highcal_Damage_Multiplier": "高口径枪械对建筑伤害倍率",
    "Barricades.Melee_Damage_Multiplier": "近战对路障伤害倍率",
    "Structures.Melee_Damage_Multiplier": "近战对建筑伤害倍率",
    "Vehicles.Armor_Lowtier_Multiplier": "低级载具护甲承伤倍率",
    "Vehicles.Armor_Hightier_Multiplier": "高级载具护甲承伤倍率",
    "Barricades.Armor_Lowtier_Multiplier": "低级路障承伤倍率",
    "Barricades.Armor_Hightier_Multiplier": "高级路障承伤倍率",
    "Structures.Armor_Lowtier_Multiplier": "低级建筑承伤倍率",
    "Structures.Armor_Hightier_Multiplier": "高级建筑承伤倍率",
  };

  const TOKEN_LABELS: Record<string, string> = {
    Min: "最小",
    Max: "最大",
    Default: "默认",
    Enable: "启用",
    Allow: "允许",
    Prevent: "阻止",
    Validate: "校验",
    Disable: "禁用",
    Use: "使用",
    Can: "可以",
    Should: "是否",
    Spawn: "刷新",
    Spawns: "刷新",
    Respawn: "重生",
    Despawn: "消失",
    Chance: "概率",
    Multiplier: "倍率",
    Time: "时间",
    Timer: "计时",
    Seconds: "秒",
    Milliseconds: "毫秒",
    Damage: "伤害",
    Armor: "护甲",
    Health: "生命",
    Food: "饥饿",
    Water: "口渴",
    Virus: "免疫",
    Experience: "经验",
    Skills: "技能",
    PvP: "PvP",
    PvE: "PvE",
    Items: "物品",
    Clothes: "衣物",
    Weapons: "武器",
    Vehicles: "载具",
    Vehicle: "载具",
    Zombies: "僵尸",
    Zombie: "僵尸",
    Animals: "动物",
    Group: "队伍",
    Groups: "队伍",
    Player: "玩家",
    Players: "玩家",
    Map: "地图",
    Radius: "半径",
    Distance: "距离",
    Quality: "品质",
    Durability: "耐久",
    Full: "满",
    Bullets: "弹药",
    Gun: "枪械",
    Guns: "枪械",
    Lowcal: "低口径",
    Highcal: "高口径",
    Lowtier: "低级",
    Hightier: "高级",
    Magazine: "弹匣",
    Crate: "弹药箱",
    Beacon: "尸潮信标",
    Rewards: "奖励",
    Regen: "恢复",
    Skillset: "专长",
    Reduces: "降低",
    Cost: "消耗",
    Prevents: "防止",
    Loss: "丢失",
    Battery: "电池",
    Tire: "轮胎",
    Fuel: "燃料",
    Bleed: "流血",
    Legs: "腿部",
    Headshot: "爆头",
    Recoil: "后坐力",
    Airdrop: "空投",
    Weather: "天气",
    Rain: "雨",
    Snow: "雪",
    Arena: "竞技场",
    Log: "记录",
    Warnings: "警告",
    Secure: "安全",
    Timeout: "超时",
    Packets: "数据包",
    Clients: "客户端",
    Same: "相同",
    Address: "地址",
    Input: "输入",
    Spam: "刷屏",
    Scheduled: "定时",
    Shutdown: "关服",
    Update: "更新",
    Steam: "Steam",
    Beta: "测试分支",
    Name: "名称",
  };

  const OPTION_LABELS: Record<string, Record<string, string>> = {
    Monetization: {
      Unspecified: "未指定",
      Any: "任意",
      None: "无商业化",
      NonGameplay: "非玩法商业化",
      Monetized: "商业化",
    },
  };

  const DESCRIPTION_LABELS: Record<string, string> = {
    Icon: "服务器大厅左上角显示的 64x64 图片 URL。",
    Thumbnail: "服务器列表中显示的 32x32 图片 URL。",
    Desc_Hint: "服务器大厅名称下方的短描述，支持 Unturned 富文本标签和中文。",
    Desc_Full: "服务器大厅右下角的详细介绍，支持富文本标签、中文和 \\n 换行。",
    Desc_Server_List: "服务器列表名称下方显示的短描述。",
    Login_Token: "Steam 游戏服务器登录令牌，用于让服务器在浏览器中正常展示。",
    BookmarkHost: "对外公布的 IP、域名或返回连接地址的网页接口。",
    Links: "服务器大厅按钮链接列表。复杂结构会以原始块文本编辑，保存时不改字段名。",
    Max_Ping_Milliseconds: "玩家延迟高于该值时会被踢出服务器。",
    Enable_Scheduled_Shutdown: "开启后服务器会在指定时间自动关闭。",
    Scheduled_Shutdown_Warnings: "定时关服前广播警告的时间列表。",
    Enable_Update_Shutdown: "检测到服务端更新后是否自动关服。",
    Update_Shutdown_Warnings: "更新关服前广播警告的时间列表。",
    Spawn_Chance: "对应对象的刷新比例，通常为 0 到 1；部分服主会设置更高值。",
    Respawn_Time: "对象重生或重新刷新的等待时间，单位通常为秒。",
    Damage_Multiplier: "伤害倍率。高于 1 增强伤害，低于 1 降低伤害。",
    Armor_Multiplier: "承伤倍率。高于 1 更耐打，低于 1 更脆。",
    Experience_Multiplier: "玩家获取经验的倍率。",
    Hitmarkers: "玩家造成伤害时是否显示命中反馈。",
    Crosshair: "持枪时是否显示准星。",
    Ballistics: "子弹是否受重力和飞行时间影响。",
    Chart: "没有纸质地图物品时，玩家是否仍可查看地图。",
    Satellite: "没有 GPS 物品时，玩家是否仍可查看 GPS 地图。",
    Compass: "没有指南针物品时，玩家是否仍显示方向 HUD。",
  };

  const DESCRIPTION_PATTERNS: Array<[RegExp, string]> = [
    [/^Default:\s*(.+)$/i, "默认值：$1"],
    [/^Easy:\s*(.+?)\s+Normal:\s*(.+?)\s+Hard:\s*(.+)$/i, "简单：$1    普通：$2    困难：$3"],
    [/^Options:\s*(.+)$/i, "可选值：$1"],
    [/^Documentation:\s*(.+)$/i, "文档：$1"],
    [/^Percentage \[0 to 1\] of (.+) to use\.$/i, "$1 的使用比例，范围 0 到 1。"],
    [/^Percentage \[0 to 1\] chance of (.+)\.$/i, "$1 的概率，范围 0 到 1。"],
    [/^Percentage \[0 to 1\] probability of (.+)\.$/i, "$1 的概率，范围 0 到 1。"],
    [/^How long \(in seconds\) before (.+)\.$/i, "$1 前等待的时间，单位为秒。"],
    [/^How quickly (.+)\.$/i, "$1 的速度。"],
    [/^Scales the amount of (.+)\.$/i, "调整 $1 的倍率。"],
    [/^Scales (.+)\.$/i, "调整 $1 的倍率。"],
    [/^Minimum (.+)\.$/i, "最小 $1。"],
    [/^Maximum (.+)\.$/i, "最大 $1。"],
    [/^If true, (.+)\.$/i, "开启时，$1。"],
    [/^If false, (.+)\.$/i, "关闭时，$1。"],
    [/^Whether (.+)\.$/i, "是否 $1。"],
    [/^Should (.+)\?$/i, "是否 $1。"],
    [/^Should (.+)\.$/i, "是否 $1。"],
  ];

  const DESCRIPTION_TERMS: Array<[RegExp, string]> = [
    [/\bserver lobby menu\b/gi, "服务器大厅菜单"],
    [/\bserver list\b/gi, "服务器列表"],
    [/\bserver lobby\b/gi, "服务器大厅"],
    [/\bserver\b/gi, "服务器"],
    [/\bplayers\b/gi, "玩家"],
    [/\bplayer\b/gi, "玩家"],
    [/\bzombies\b/gi, "僵尸"],
    [/\bzombie\b/gi, "僵尸"],
    [/\banimals\b/gi, "动物"],
    [/\banimal\b/gi, "动物"],
    [/\bvehicles\b/gi, "载具"],
    [/\bvehicle\b/gi, "载具"],
    [/\bitems\b/gi, "物品"],
    [/\bitem\b/gi, "物品"],
    [/\bguns\b/gi, "枪械"],
    [/\bgun\b/gi, "枪械"],
    [/\bmagazines\b/gi, "弹匣"],
    [/\bmagazine\b/gi, "弹匣"],
    [/\bammo\b/gi, "弹药"],
    [/\bquality\b/gi, "品质"],
    [/\bdurability\b/gi, "耐久"],
    [/\bhealth\b/gi, "生命值"],
    [/\bdamage\b/gi, "伤害"],
    [/\barmor\b/gi, "护甲"],
    [/\bXP\b/g, "经验"],
    [/\bexperience\b/gi, "经验"],
    [/\bskills\b/gi, "技能"],
    [/\bskill\b/gi, "技能"],
    [/\bfood\b/gi, "饥饿值"],
    [/\bwater\b/gi, "口渴值"],
    [/\bimmunity\b/gi, "免疫值"],
    [/\bbleeding\b/gi, "流血"],
    [/\bbleed\b/gi, "流血"],
    [/\bbroken legs\b/gi, "断腿"],
    [/\blegs\b/gi, "腿部"],
    [/\bmap\b/gi, "地图"],
    [/\bGPS\b/g, "GPS"],
    [/\bcompass\b/gi, "指南针"],
    [/\bgroup members\b/gi, "队友"],
    [/\bgroups\b/gi, "队伍"],
    [/\bgroup\b/gi, "队伍"],
    [/\bPvP\b/g, "PvP"],
    [/\bPvE\b/g, "PvE"],
    [/\bseconds\b/gi, "秒"],
    [/\bminutes\b/gi, "分钟"],
    [/\bhours\b/gi, "小时"],
    [/\bURL\b/g, "URL"],
    [/\bIP address\b/gi, "IP 地址"],
    [/\bDNS name\b/gi, "DNS 名称"],
    [/\bweb address\b/gi, "网页地址"],
    [/\brich text\b/gi, "富文本"],
    [/\bfull moon\b/gi, "满月"],
    [/\bairdrop\b/gi, "空投"],
    [/\bweather\b/gi, "天气"],
    [/\brain\b/gi, "雨天"],
    [/\bsnow\b/gi, "雪天"],
    [/\barena\b/gi, "竞技场"],
    [/\btrue\b/gi, "True"],
    [/\bfalse\b/gi, "False"],
  ];

  let { saveId = "", readonly = false } = $props<{ saveId: string; readonly?: boolean }>();

  let config = $state<GameConfigInfo | null>(null);
  let loading = $state(false);
  let saving = $state(false);
  let query = $state("");
  let filter = $state<"all" | "overridden" | "changed">("all");
  let selectedSection = $state("");
  let values = $state<Record<string, string>>({});
  let originalValues = $state<Record<string, string>>({});
  let loadGeneration = 0;

  const allEntries = $derived(config?.sections.flatMap((section) => section.entries) ?? []);
  const changedCount = $derived(allEntries.filter((entry) => isChanged(entry)).length);
  const selectedSectionData = $derived(
    config?.sections.find((section) => section.name === selectedSection) ?? config?.sections[0] ?? null
  );
  const visibleEntries = $derived(
    (selectedSectionData?.entries ?? []).filter((entry) => matchesFilters(entry))
  );

  function entryValue(entry: GameConfigEntry) {
    return values[entry.id] ?? "";
  }

  function normalizedBoolValue(entry: GameConfigEntry) {
    const value = entryValue(entry).trim().toLowerCase();
    if (value === "true") return "True";
    if (value === "false") return "False";
    return "";
  }

  function numberValue(entry: GameConfigEntry) {
    const value = entryValue(entry).trim();
    if (!value) return null;
    const parsed = Number(value);
    return Number.isFinite(parsed) ? parsed : null;
  }

  function isPercentInRecommendedRange(entry: GameConfigEntry) {
    const value = numberValue(entry);
    return value !== null && value >= 0 && value <= 1;
  }

  function isPercentOutsideRecommendedRange(entry: GameConfigEntry) {
    const value = numberValue(entry);
    return value !== null && (value < 0 || value > 1);
  }

  function isChanged(entry: GameConfigEntry) {
    return entryValue(entry) !== (originalValues[entry.id] ?? "");
  }

  function isOverridden(entry: GameConfigEntry) {
    return entryValue(entry).trim() !== "";
  }

  function matchesFilters(entry: GameConfigEntry) {
    const needle = query.trim().toLowerCase();
    const text = [
      entry.section,
      sectionLabel(entry.section),
      entry.key,
      entryLabel(entry),
      entryValue(entry),
      valueStateLabel(entry),
      controlLabel(entry),
      descriptionText(entry),
      entry.description.join(" "),
    ].join(" ").toLowerCase();
    if (needle && !text.includes(needle)) return false;
    if (filter === "overridden") return isOverridden(entry);
    if (filter === "changed") return isChanged(entry);
    return true;
  }

  function sectionStats(section: GameConfigSection) {
    let overridden = 0;
    let changed = 0;
    for (const entry of section.entries) {
      if (isOverridden(entry)) overridden += 1;
      if (isChanged(entry)) changed += 1;
    }
    return { total: section.entries.length, overridden, changed };
  }

  function setEntryValue(entry: GameConfigEntry, value: string) {
    if (readonly) return;
    values = { ...values, [entry.id]: value };
  }

  function resetEntry(entry: GameConfigEntry) {
    if (readonly) return;
    setEntryValue(entry, "");
  }

  async function loadGameConfig() {
    if (!saveId) {
      config = null;
      values = {};
      originalValues = {};
      selectedSection = "";
      return;
    }

    const gen = ++loadGeneration;
    loading = true;
    try {
      const result = await invoke("read_game_config", { saveId }) as GameConfigInfo;
      if (gen !== loadGeneration) return;
      config = result;
      const nextValues: Record<string, string> = {};
      for (const entry of result.sections.flatMap((section) => section.entries)) {
        nextValues[entry.id] = entry.value ?? "";
      }
      values = nextValues;
      originalValues = { ...nextValues };
      selectedSection = result.sections[0]?.name ?? "";
    } catch (e: any) {
      alert(e);
    } finally {
      if (gen === loadGeneration) loading = false;
    }
  }

  async function saveGameConfig() {
    if (readonly || !config || !saveId || changedCount === 0) return;
    saving = true;
    try {
      const changes = allEntries
        .filter((entry) => isChanged(entry))
        .map((entry) => ({
          section: entry.section,
          key: entry.key,
          original_value: originalValues[entry.id] ?? "",
          value: entryValue(entry),
        }));

      await invoke("save_game_config", {
        saveId,
        sourceHash: config.source_hash,
        changes,
      });
      toastStore.success("高级配置已保存");
      await loadGameConfig();
    } catch (e: any) {
      alert(e);
    } finally {
      saving = false;
    }
  }

  function controlLabel(entry: GameConfigEntry) {
    if (entry.default_hint) return translateHint(entry.default_hint);
    if (!entry.has_value) return "当前使用游戏默认值，保存时不会写入显式值";
    return "当前已覆盖游戏默认值";
  }

  function sectionLabel(section: string) {
    return SECTION_LABELS[section] ?? section;
  }

  function entryLabel(entry: GameConfigEntry) {
    const sectionKeyLabel = SECTION_KEY_LABELS[`${entry.section}.${entry.key}`];
    if (sectionKeyLabel) return sectionKeyLabel;
    if (KEY_LABELS[entry.key]) return KEY_LABELS[entry.key];
    const tokens = entry.key.split("_").filter(Boolean);
    const translated = tokens.map((token) => TOKEN_LABELS[token] ?? token);
    if (translated.some((token, index) => token !== tokens[index])) {
      return translated.join("").replace(/\s+/g, "");
    }
    return entry.key.replaceAll("_", " ");
  }

  function optionLabel(entry: GameConfigEntry, option: string) {
    const label = OPTION_LABELS[entry.key]?.[option];
    return label ? `${label} (${option})` : option;
  }

  function descriptionText(entry: GameConfigEntry) {
    const translated = DESCRIPTION_LABELS[entry.key];
    if (translated) return translated;
    const lines = entry.description.map(translateDescriptionLine).filter(Boolean);
    return lines.length > 0 ? lines.join("\n") : "暂无中文说明";
  }

  function translateHint(hint: string) {
    return hint
      .replace(/Default:/g, "默认值：")
      .replace(/Easy:/g, "简单：")
      .replace(/Normal:/g, "普通：")
      .replace(/Hard:/g, "困难：")
      .replace(/\[0 to 1\]/g, "范围 0 到 1")
      .replace(/\bTrue\b/g, "开启")
      .replace(/\bFalse\b/g, "关闭");
  }

  function valueStateLabel(entry: GameConfigEntry) {
    if (isChanged(entry)) return "待保存";
    return isOverridden(entry) ? "已覆盖" : "默认";
  }

  function valueStateClass(entry: GameConfigEntry) {
    if (isChanged(entry)) return "state-changed";
    return isOverridden(entry) ? "state-overridden" : "state-default";
  }

  function translateDescriptionLine(line: string) {
    let output = line.trim();
    if (!output) return "";
    if (output.startsWith("`")) return output.replace(/^`\s?/, "示例：");
    for (const [pattern, replacement] of DESCRIPTION_PATTERNS) {
      if (pattern.test(output)) {
        output = output.replace(pattern, replacement);
        break;
      }
    }
    for (const [pattern, replacement] of DESCRIPTION_TERMS) {
      output = output.replace(pattern, replacement);
    }
    output = output
      .replace(/\bshown\b/gi, "显示")
      .replace(/\bshow\b/gi, "显示")
      .replace(/\benable\b/gi, "启用")
      .replace(/\benabled\b/gi, "启用")
      .replace(/\bdisable\b/gi, "禁用")
      .replace(/\bdisabled\b/gi, "禁用")
      .replace(/\ballowed\b/gi, "允许")
      .replace(/\ballow\b/gi, "允许")
      .replace(/\bkicked\b/gi, "踢出")
      .replace(/\bkick\b/gi, "踢出")
      .replace(/\bspawn\b/gi, "刷新")
      .replace(/\bspawning\b/gi, "刷新")
      .replace(/\brespawn\b/gi, "重生")
      .replace(/\bdrops\b/gi, "掉落")
      .replace(/\bdrop\b/gi, "掉落")
      .replace(/\bchance\b/gi, "概率")
      .replace(/\bprobability\b/gi, "概率")
      .replace(/\bmultiplier\b/gi, "倍率")
      .replace(/\bDefault\b/g, "默认值")
      .replace(/\bEasy\b/g, "简单")
      .replace(/\bNormal\b/g, "普通")
      .replace(/\bHard\b/g, "困难");
    return output;
  }

  $effect(() => {
    if (saveId) {
      loadGameConfig();
    }
  });
</script>

<div class="space-y-4">
  <div class="config-toolbar rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-4">
    <div class="flex flex-wrap items-center justify-between gap-3">
      <div class="min-w-0">
        <h2 class="flex items-center gap-2 text-base font-semibold text-[var(--text-primary)]">
          <svg class="h-5 w-5 text-[var(--accent-light)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="M4 6h16M4 12h10M4 18h16" />
          </svg>
          游戏高级配置
        </h2>
        <p class="mt-0.5 max-w-[72ch] truncate text-xs text-[var(--text-muted)]">
          {#if config?.exists}
            {config.path}
          {:else}
            Config.txt 将在服务端生成后显示
          {/if}
        </p>
      </div>

      <div class="flex flex-wrap items-center gap-2">
        {#if changedCount > 0}
          <span class="rounded-full bg-[var(--warning-glow)] px-3 py-1 text-xs font-semibold text-[var(--warning)]">{changedCount} 项待保存</span>
        {/if}
        <button
          class="toolbar-button"
          onclick={loadGameConfig}
          disabled={loading || saving}
          title="重新读取磁盘上的 Config.txt"
        >
          重新加载
        </button>
        <button
          class="toolbar-primary"
          onclick={saveGameConfig}
          disabled={saving || loading || changedCount === 0 || readonly}
        >
          {saving ? "保存中..." : "保存更改"}
        </button>
      </div>
    </div>

    <div class="mt-3 flex flex-col gap-2 lg:flex-row lg:items-center">
      <div class="relative min-w-0 flex-1">
        <input
          type="search"
          bind:value={query}
          placeholder="搜索中文名、英文字段、分组或说明"
          class="w-full rounded-lg border border-[var(--border)] bg-[var(--bg-primary)] px-4 py-2 pl-9 text-sm text-[var(--text-primary)] placeholder:text-[var(--text-muted)] transition-colors focus:border-[var(--accent)] focus:outline-none"
        />
        <svg class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-[var(--text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.8" d="m21 21-4.3-4.3M10.5 18a7.5 7.5 0 1 1 0-15 7.5 7.5 0 0 1 0 15Z" />
        </svg>
      </div>
      <div class="segmented">
        {#each [
          { id: "all", label: "全部" },
          { id: "overridden", label: "已覆盖" },
          { id: "changed", label: "已修改" },
        ] as item}
          <button
            class:active={filter === item.id}
            onclick={() => filter = item.id as typeof filter}
          >
            {item.label}
          </button>
        {/each}
      </div>
    </div>
  </div>

  {#if loading}
    <div class="flex items-center justify-center rounded-xl border border-[var(--border)] bg-[var(--bg-card)] py-14">
      <div class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--accent)] border-t-transparent"></div>
    </div>
  {:else if !saveId}
    <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] px-4 py-12 text-center text-sm text-[var(--text-muted)]">请选择一个存档</div>
  {:else if config && !config.exists}
    <div class="rounded-xl border border-[var(--warning)]/30 bg-[var(--warning-glow)] px-5 py-6 text-sm text-[var(--text-secondary)]">
      未找到 Config.txt。通常运行一次服务端后，Unturned 会在当前存档目录生成该文件。
    </div>
  {:else if config}
    <div class="grid grid-cols-1 gap-4 xl:grid-cols-[250px_minmax(0,1fr)]">
      <aside class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] p-3 xl:sticky xl:top-0 xl:max-h-[calc(100vh-210px)] xl:overflow-y-auto">
        <div class="mb-2 flex items-center justify-between px-2 text-xs text-[var(--text-muted)]">
          <span class="font-medium">配置分组</span>
          <span>{config.sections.length} 组</span>
        </div>
        <div class="space-y-1">
          {#each config.sections as section (section.name)}
            {@const stats = sectionStats(section)}
            <button
              class="section-button"
              class:active={selectedSection === section.name}
              onclick={() => selectedSection = section.name}
            >
              <div class="flex items-center justify-between gap-3">
                <span class="truncate text-sm font-medium text-[var(--text-primary)]">{sectionLabel(section.name)}</span>
                <span class="section-count">{stats.total}</span>
              </div>
              <div class="mt-1 flex items-center justify-between gap-2 text-[11px] text-[var(--text-muted)]">
                <span>{stats.overridden} 覆盖{stats.changed > 0 ? ` · ${stats.changed} 待保存` : ""}</span>
                <span class="truncate font-mono">{section.name}</span>
              </div>
            </button>
          {/each}
        </div>
      </aside>

      <section class="min-w-0 space-y-2">
        <div class="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h3 class="text-sm font-semibold text-[var(--text-primary)]">{selectedSectionData ? sectionLabel(selectedSectionData.name) : ""}</h3>
            <p class="text-xs text-[var(--text-muted)]">{visibleEntries.length} 个配置项正在显示</p>
          </div>
          {#if config.version}
            <span class="rounded-full border border-[var(--border)] bg-[var(--bg-primary)] px-3 py-1 text-xs text-[var(--text-muted)]">Version {config.version}</span>
          {/if}
        </div>

        {#if visibleEntries.length === 0}
          <div class="rounded-xl border border-[var(--border)] bg-[var(--bg-card)] px-4 py-12 text-center text-sm text-[var(--text-muted)]">没有匹配的配置项</div>
        {:else}
          <div class="settings-list">
            {#each visibleEntries as entry (entry.id)}
              <div class="setting-row" class:changed={isChanged(entry)}>
                <div class="setting-meta">
                  <div class="flex min-w-0 flex-wrap items-center gap-2">
                    <h4 class="setting-title">{entryLabel(entry)}</h4>
                    <span class="key-tag">{entry.key}</span>
                    <span class="state-pill {valueStateClass(entry)}">{valueStateLabel(entry)}</span>
                    <span class="info-wrap relative inline-flex">
                      <button class="info-dot" type="button" aria-label={`${entry.key} 配置说明`}>!</button>
                      <span class="info-tip" role="tooltip">{descriptionText(entry)}</span>
                    </span>
                  </div>
                  <p class="setting-hint">{controlLabel(entry)}</p>
                </div>

                <div class="setting-control">
                  {#if entry.value_kind === "bool"}
                    <div class="bool-segmented" role="group" aria-label={`${entryLabel(entry)} 开关`}>
                      {#each [
                        { value: "", label: "默认" },
                        { value: "True", label: "开启" },
                        { value: "False", label: "关闭" },
                      ] as option}
                        <button
                          type="button"
                          class:active={normalizedBoolValue(entry) === option.value}
                          onclick={() => setEntryValue(entry, option.value)}
                          disabled={readonly}
                        >
                          {option.label}
                        </button>
                      {/each}
                    </div>
                  {:else if entry.value_kind === "select"}
                    <SelectCustom
                      value={entryValue(entry)}
                      options={[
                        { value: "", label: "使用游戏默认值" },
                        ...entry.options.map((option) => ({ value: option, label: optionLabel(entry, option) })),
                      ]}
                      onchange={(value) => setEntryValue(entry, String(value))}
                      disabled={readonly}
                      size="sm"
                      fullWidth
                    />
                  {:else if entry.value_kind === "raw_block"}
                    <textarea
                      value={entryValue(entry)}
                      oninput={(event) => setEntryValue(entry, (event.target as HTMLTextAreaElement).value)}
                      disabled={readonly}
                      spellcheck="false"
                      rows="4"
                      placeholder="留空表示使用默认值"
                      class="setting-input setting-textarea raw"
                    ></textarea>
                  {:else if entry.value_kind === "long_text"}
                    <textarea
                      value={entryValue(entry)}
                      oninput={(event) => setEntryValue(entry, (event.target as HTMLTextAreaElement).value)}
                      disabled={readonly}
                      rows="3"
                      placeholder="留空表示使用默认值"
                      class="setting-input setting-textarea"
                    ></textarea>
                  {:else if entry.value_kind === "percent"}
                    <div class="percent-control" class:range-hidden={!isPercentInRecommendedRange(entry)}>
                      <input
                        type="number"
                        min="0"
                        step="0.01"
                        value={entryValue(entry)}
                        oninput={(event) => setEntryValue(entry, (event.target as HTMLInputElement).value)}
                        disabled={readonly}
                        placeholder="默认"
                        class="setting-input"
                      />
                      {#if isPercentInRecommendedRange(entry)}
                        <input
                          type="range"
                          min="0"
                          max="1"
                          step="0.01"
                          value={entryValue(entry)}
                          oninput={(event) => setEntryValue(entry, (event.target as HTMLInputElement).value)}
                          disabled={readonly}
                          class="percent-range"
                          aria-label={`${entryLabel(entry)} 滑杆`}
                        />
                      {:else if isPercentOutsideRecommendedRange(entry)}
                        <span class="percent-warning">超出注释推荐 0~1，将保留现有写法</span>
                      {:else}
                        <span class="percent-note">注释推荐 0~1，留空使用游戏默认值</span>
                      {/if}
                    </div>
                  {:else if entry.value_kind === "number"}
                    <input
                      type="number"
                      step="any"
                      value={entryValue(entry)}
                      oninput={(event) => setEntryValue(entry, (event.target as HTMLInputElement).value)}
                      disabled={readonly}
                      placeholder="默认"
                      class="setting-input"
                    />
                  {:else}
                    <input
                      type={entry.value_kind === "url" ? "url" : "text"}
                      value={entryValue(entry)}
                      oninput={(event) => setEntryValue(entry, (event.target as HTMLInputElement).value)}
                      disabled={readonly}
                      placeholder="默认"
                      class="setting-input"
                    />
                  {/if}
                </div>

                <button
                  class="reset-button"
                  onclick={() => resetEntry(entry)}
                  disabled={entryValue(entry) === "" || readonly}
                  title="恢复为游戏默认值"
                  aria-label={`${entryLabel(entry)} 恢复默认`}
                >
                  默认
                </button>
              </div>
            {/each}
          </div>
        {/if}
      </section>
    </div>
  {/if}
</div>

<style>
  .config-toolbar {
    position: relative;
    z-index: 10;
  }

  .toolbar-button,
  .toolbar-primary {
    border-radius: 8px;
    padding: 8px 14px;
    font-size: 13px;
    font-weight: 600;
    transition: border-color 160ms ease, background 160ms ease, color 160ms ease, opacity 160ms ease;
  }

  .toolbar-button {
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-secondary);
  }

  .toolbar-button:hover:not(:disabled) {
    border-color: var(--accent);
    color: var(--text-primary);
  }

  .toolbar-primary {
    border: 1px solid var(--accent);
    background: var(--accent);
    color: #ffffff;
  }

  .toolbar-button:disabled,
  .toolbar-primary:disabled {
    opacity: 0.45;
  }

  .segmented,
  .bool-segmented {
    display: inline-flex;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-primary);
    padding: 3px;
  }

  .segmented button,
  .bool-segmented button {
    min-height: 30px;
    border-radius: 6px;
    padding: 5px 11px;
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 600;
    transition: background 160ms ease, color 160ms ease, box-shadow 160ms ease;
  }

  .segmented button.active,
  .bool-segmented button.active {
    background: var(--accent-subtle);
    color: var(--accent-light);
    box-shadow: var(--shadow-sm);
  }

  .section-button {
    width: 100%;
    border: 1px solid transparent;
    border-radius: 8px;
    padding: 9px 10px;
    text-align: left;
    transition: background 160ms ease, border-color 160ms ease;
  }

  .section-button:hover,
  .section-button.active {
    border-color: var(--border-accent);
    background: var(--accent-subtle);
  }

  .section-count {
    min-width: 24px;
    border-radius: 999px;
    background: var(--bg-primary);
    padding: 1px 7px;
    color: var(--text-muted);
    font-size: 11px;
    font-weight: 700;
    text-align: center;
  }

  .settings-list {
    overflow: visible;
    border: 1px solid var(--border);
    border-radius: 12px;
    background: var(--bg-card);
  }

  .setting-row {
    position: relative;
    display: grid;
    grid-template-columns: minmax(240px, 1fr) minmax(260px, 420px) auto;
    gap: 14px;
    align-items: start;
    border-bottom: 1px solid var(--border);
    padding: 13px 14px;
    overflow: visible;
    transition: background 160ms ease, border-color 160ms ease;
  }

  .setting-row:last-child {
    border-bottom: 0;
  }

  .setting-row:hover {
    background: color-mix(in srgb, var(--accent-subtle) 24%, transparent);
  }

  .setting-row.changed {
    background: color-mix(in srgb, var(--warning-glow) 44%, var(--bg-card));
  }

  .setting-meta {
    min-width: 0;
  }

  .setting-title {
    max-width: 34ch;
    overflow: hidden;
    color: var(--text-primary);
    font-size: 14px;
    font-weight: 650;
    line-height: 1.35;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .key-tag {
    max-width: 220px;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 6px;
    background: var(--bg-primary);
    padding: 1px 7px;
    color: var(--text-muted);
    font-family: "Cascadia Mono", "JetBrains Mono", Consolas, monospace;
    font-size: 11px;
    line-height: 1.5;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .state-pill {
    border-radius: 999px;
    padding: 2px 8px;
    font-size: 11px;
    font-weight: 700;
    line-height: 1.35;
  }

  .state-default {
    background: var(--bg-primary);
    color: var(--text-muted);
  }

  .state-overridden {
    background: var(--accent-subtle);
    color: var(--accent-light);
  }

  .state-changed {
    background: var(--warning-glow);
    color: var(--warning);
  }

  .setting-hint {
    margin-top: 5px;
    max-width: 70ch;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.45;
  }

  .setting-control {
    min-width: 0;
  }

  .setting-input {
    width: 100%;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--bg-primary);
    padding: 8px 11px;
    color: var(--text-primary);
    font-size: 13px;
    line-height: 1.45;
    transition: border-color 160ms ease, background 160ms ease, min-height 180ms ease;
  }

  .setting-input:focus {
    border-color: var(--accent);
    outline: none;
  }

  .setting-textarea {
    min-height: 82px;
    resize: vertical;
  }

  .setting-textarea:focus {
    min-height: 150px;
  }

  .setting-textarea.raw {
    min-height: 108px;
    font-family: "Cascadia Mono", "JetBrains Mono", Consolas, monospace;
    font-size: 12px;
  }

  .setting-textarea.raw:focus {
    min-height: 220px;
  }

  .percent-control {
    display: grid;
    grid-template-columns: 128px minmax(120px, 1fr);
    gap: 10px;
    align-items: center;
  }

  .percent-control.range-hidden {
    grid-template-columns: 128px minmax(160px, 1fr);
  }

  .percent-range {
    width: 100%;
    accent-color: var(--accent);
    opacity: 0.82;
  }

  .percent-warning,
  .percent-note {
    min-width: 0;
    border-radius: 7px;
    padding: 7px 9px;
    font-size: 12px;
    line-height: 1.35;
  }

  .percent-warning {
    border: 1px solid color-mix(in srgb, var(--warning) 42%, transparent);
    background: var(--warning-glow);
    color: var(--warning);
  }

  .percent-note {
    border: 1px solid var(--border);
    background: var(--bg-primary);
    color: var(--text-muted);
  }

  .reset-button {
    align-self: start;
    border: 1px solid transparent;
    border-radius: 7px;
    padding: 6px 9px;
    color: var(--text-muted);
    font-size: 12px;
    font-weight: 600;
    transition: background 160ms ease, border-color 160ms ease, color 160ms ease, opacity 160ms ease;
  }

  .reset-button:hover:not(:disabled) {
    border-color: var(--border);
    background: var(--bg-primary);
    color: var(--accent-light);
  }

  .reset-button:disabled {
    opacity: 0.32;
  }

  .info-dot {
    display: inline-flex;
    width: 18px;
    height: 18px;
    align-items: center;
    justify-content: center;
    border: 1px solid var(--border);
    border-radius: 999px;
    background: var(--bg-primary);
    color: var(--accent-light);
    font-size: 12px;
    font-weight: 700;
    line-height: 1;
  }

  .info-tip {
    position: absolute;
    left: 50%;
    top: 24px;
    z-index: 80;
    display: none;
    width: min(380px, calc(100vw - 48px));
    transform: translateX(-50%);
    white-space: pre-wrap;
    border: 1px solid var(--border-accent);
    border-radius: 8px;
    background: var(--bg-secondary);
    box-shadow: var(--shadow-lg);
    color: var(--text-secondary);
    font-size: 12px;
    line-height: 1.6;
    padding: 12px;
  }

  .info-wrap:hover .info-tip,
  .info-wrap:focus-within .info-tip {
    display: block;
  }

  @media (max-width: 1180px) {
    .setting-row {
      grid-template-columns: minmax(220px, 1fr) minmax(240px, 360px) auto;
    }

    .key-tag {
      max-width: 180px;
    }
  }

  @media (max-width: 900px) {
    .setting-row {
      grid-template-columns: 1fr;
      gap: 10px;
    }

    .setting-control {
      width: 100%;
    }

    .reset-button {
      justify-self: start;
    }

    .percent-control {
      grid-template-columns: 1fr;
    }

    .setting-title {
      max-width: 100%;
      white-space: normal;
    }
  }
</style>
