/**
 * 权限条目
 */
export type PermissionEntry = {
  name: string;
  cooldown: number;
};

/**
 * 权限组
 */
export type PermissionGroup = {
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

/**
 * 权限配置
 */
export type PermissionsConfig = {
  exists: boolean;
  path: string;
  default_group: string;
  groups: PermissionGroup[];
};

/**
 * 存档信息
 */
export type SaveInfo = {
  id: string;
  name: string;
  map: string;
  [key: string]: any;
};

/**
 * Workshop 项目
 */
export type WorkshopItem = {
  id: string;
  name?: string;
  [key: string]: any;
};

/**
 * 插件信息
 */
export type PluginInfo = {
  name: string;
  installed: boolean;
  [key: string]: any;
};
