/**
 * Vitest 测试环境设置
 */

import { expect, afterEach } from 'vitest';
import { cleanup } from '@testing-library/svelte';
import * as matchers from '@testing-library/jest-dom/matchers';

type TauriTestGlobal = typeof globalThis & {
  __TAURI_INTERNALS__: {
    invoke: () => Promise<Record<string, never>>;
    convertFileSrc: (path: string) => string;
  };
};

// 扩展 expect 匹配器
expect.extend(matchers);

// 每个测试后清理 DOM
afterEach(() => {
  cleanup();
});

// Mock Tauri API
const tauriGlobal = globalThis as TauriTestGlobal;

tauriGlobal.__TAURI_INTERNALS__ = {
  invoke: async () => ({}),
  convertFileSrc: (path: string) => path,
};

// Mock window API
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => true,
  }),
});
