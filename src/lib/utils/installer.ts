/**
 * 安装进度监听工具
 * 用于 Wizard 和 Save 页面的安装进度事件处理
 */

import { listen } from "@tauri-apps/api/event";

export interface InstallerCallbacks {
  /** 安装完成回调 */
  onDone: (payload: string) => void;
  /** 安装错误回调 */
  onError: (message: string) => void;
  /** 进度更新回调 */
  onProgress: (message: string) => void;
}

/**
 * 监听安装进度事件
 * @returns 取消监听函数
 */
export async function listenInstallerProgress(callbacks: InstallerCallbacks): Promise<() => void> {
  const { onDone, onError, onProgress } = callbacks;

  const unlisten = await listen<string>("installer-progress", (event) => {
    const msg = event.payload;
    if (msg.startsWith("DONE:")) {
      onDone(msg.slice(5));
      unlisten();
    } else if (msg.startsWith("ERROR:")) {
      onError(msg.slice(6));
      unlisten();
    } else {
      onProgress(msg);
    }
  });

  return unlisten;
}
