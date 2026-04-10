// ============================================================
// 门控状态管理
// 控制应用入口验证和数据采集模式
// ============================================================

import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

// 运行模式枚举
export type AppMode = 'normal' | 'admin';

export const useGateStore = defineStore('gate', () => {
  // 是否已通过门控验证
  const isAuthenticated = ref(false);
  // 当前运行模式：normal=普通用户（启用数据采集），admin=管理员（禁用数据采集）
  const mode = ref<AppMode>('normal');

  // 是否启用数据采集（仅普通模式下启用）
  const isCollectionEnabled = computed(() => mode.value === 'normal');

  // 是否为管理员模式
  const isAdmin = computed(() => mode.value === 'admin');

  /**
   * 验证入口口令
   * @param code 用户输入的口令
   * @returns 是否验证通过
   */
  function authenticate(code: string): boolean {
    const trimmed = code.trim().toLowerCase();
    if (trimmed === 'cknb') {
      // 普通模式 - 启用数据采集
      mode.value = 'normal';
      isAuthenticated.value = true;
      return true;
    } else if (trimmed === 'ck666') {
      // 管理员模式 - 禁用数据采集
      mode.value = 'admin';
      isAuthenticated.value = true;
      return true;
    }
    return false;
  }

  // 重置门控状态（退出登录）
  function reset() {
    isAuthenticated.value = false;
    mode.value = 'normal';
  }

  return {
    isAuthenticated,
    mode,
    isCollectionEnabled,
    isAdmin,
    authenticate,
    reset,
  };
});
