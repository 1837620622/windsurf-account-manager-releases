import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { Account, AccountFilter, PaginationConfig, AccountStatusType, SortConfig, SortField, SortDirection } from '@/types';
import { accountApi, apiService, settingsApi } from '@/api';
import dayjs from 'dayjs';

export const useAccountsStore = defineStore('accounts', () => {
  const accounts = ref<Account[]>([]);
  const selectedAccounts = ref<Set<string>>(new Set());
  const currentFilter = ref<AccountFilter>({});
  const loading = ref(false);
  const error = ref<string | null>(null);
  
  // 批量更新队列（用于优化大量账号更新时的性能）
  const pendingUpdates = ref<Map<string, Account>>(new Map());
  let batchUpdateTimer: ReturnType<typeof setTimeout> | null = null;

  // 分页状态
  const pagination = ref<PaginationConfig>({
    currentPage: 1,
    pageSize: 20,
    pageSizes: [10, 20, 50, 100]
  });

  // 排序配置
  const sortConfig = ref<SortConfig>({
    field: 'created_at',
    direction: 'asc'
  });

  // 辅助函数：计算剩余额度
  function getRemainingQuota(account: Account): number {
    if (!account.total_quota || account.used_quota === undefined) return 0;
    return Math.max(0, account.total_quota - account.used_quota);
  }

  // 辅助函数：计算剩余天数
  function getDaysUntilExpiry(account: Account): number | null {
    if (!account.subscription_expires_at) return null;
    const now = dayjs();
    const expiry = dayjs(account.subscription_expires_at);
    return expiry.diff(now, 'day');
  }

  // 辅助函数：判断是否为付费计划
  function isPaidPlan(account: Account): boolean {
    const planName = account.plan_name?.toLowerCase();
    return !!planName && planName !== 'free';
  }

  // 辅助函数：获取账户状态类型
  function getAccountStatusType(account: Account): AccountStatusType {
    // 错误状态（密码错误等）
    if (account.status === 'error' || (typeof account.status === 'object' && 'error' in account.status)) {
      return 'error';
    }
    // 未激活（付费计划且订阅未激活）- 优先级高于已禁用
    if (isPaidPlan(account) && account.subscription_active === false) {
      return 'inactive';
    }
    // 已禁用（被 Windsurf 禁用）
    if (account.is_disabled === true) {
      return 'disabled';
    }
    // 离线（Token 失效）
    if (!account.token_expires_at || dayjs(account.token_expires_at).isBefore(dayjs())) {
      return 'offline';
    }
    // 正常
    return 'normal';
  }

  // 获取所有唯一标签
  const allTags = computed(() => {
    const tagSet = new Set<string>();
    accounts.value.forEach(acc => {
      acc.tags.forEach(tag => tagSet.add(tag));
    });
    return Array.from(tagSet).sort();
  });

  // 获取所有唯一套餐名称
  const allPlanNames = computed(() => {
    const planSet = new Set<string>();
    accounts.value.forEach(acc => {
      if (acc.plan_name) planSet.add(acc.plan_name);
    });
    return Array.from(planSet).sort();
  });

  // 获取所有唯一域名
  const allDomains = computed(() => {
    const domainSet = new Set<string>();
    accounts.value.forEach(acc => {
      const domain = acc.email.split('@')[1];
      if (domain) domainSet.add(domain);
    });
    return Array.from(domainSet).sort();
  });

  // Computed
  const filteredAccounts = computed(() => {
    let result = [...accounts.value];
    
    // 按分组筛选
    if (currentFilter.value.group) {
      result = result.filter(acc => acc.group === currentFilter.value.group);
    }
    
    // 按标签筛选
    if (currentFilter.value.tags && currentFilter.value.tags.length > 0) {
      result = result.filter(acc => 
        currentFilter.value.tags!.some(tag => acc.tags.includes(tag))
      );
    }
    
    // 搜索
    if (currentFilter.value.search) {
      const query = currentFilter.value.search.toLowerCase();
      result = result.filter(acc => 
        acc.email.toLowerCase().includes(query) ||
        acc.nickname.toLowerCase().includes(query) ||
        acc.tags.some(tag => tag.toLowerCase().includes(query))
      );
    }

    // 高级筛选：剩余额度范围（用户输入的是显示值，需要*100转换为实际值）
    if (currentFilter.value.remainingQuotaMin !== undefined) {
      const minValue = currentFilter.value.remainingQuotaMin * 100;
      result = result.filter(acc => getRemainingQuota(acc) >= minValue);
    }
    if (currentFilter.value.remainingQuotaMax !== undefined) {
      const maxValue = currentFilter.value.remainingQuotaMax * 100;
      result = result.filter(acc => getRemainingQuota(acc) <= maxValue);
    }

    // 高级筛选：总额度范围（用户输入的是显示值，需要*100转换为实际值）
    if (currentFilter.value.totalQuotaMin !== undefined) {
      const minValue = currentFilter.value.totalQuotaMin * 100;
      result = result.filter(acc => (acc.total_quota || 0) >= minValue);
    }
    if (currentFilter.value.totalQuotaMax !== undefined) {
      const maxValue = currentFilter.value.totalQuotaMax * 100;
      result = result.filter(acc => (acc.total_quota || 0) <= maxValue);
    }

    // 高级筛选：剩余天数范围
    if (currentFilter.value.expiryDaysMin !== undefined) {
      result = result.filter(acc => {
        const days = getDaysUntilExpiry(acc);
        return days !== null && days >= currentFilter.value.expiryDaysMin!;
      });
    }
    if (currentFilter.value.expiryDaysMax !== undefined) {
      result = result.filter(acc => {
        const days = getDaysUntilExpiry(acc);
        return days !== null && days <= currentFilter.value.expiryDaysMax!;
      });
    }

    // 高级筛选：套餐名称
    if (currentFilter.value.planNames && currentFilter.value.planNames.length > 0) {
      result = result.filter(acc => 
        acc.plan_name && currentFilter.value.planNames!.includes(acc.plan_name)
      );
    }

    // 高级筛选：域名
    if (currentFilter.value.domains && currentFilter.value.domains.length > 0) {
      result = result.filter(acc => {
        const domain = acc.email.split('@')[1];
        return domain && currentFilter.value.domains!.includes(domain);
      });
    }

    // 高级筛选：状态
    if (currentFilter.value.statuses && currentFilter.value.statuses.length > 0) {
      result = result.filter(acc => 
        currentFilter.value.statuses!.includes(getAccountStatusType(acc))
      );
    }
    
    return result;
  });

  // 分页后的账号列表
  const paginatedAccounts = computed(() => {
    const start = (pagination.value.currentPage - 1) * pagination.value.pageSize;
    const end = start + pagination.value.pageSize;
    return filteredAccounts.value.slice(start, end);
  });

  // 总页数
  const totalPages = computed(() => {
    return Math.ceil(filteredAccounts.value.length / pagination.value.pageSize);
  });

  // 总记录数
  const totalCount = computed(() => filteredAccounts.value.length);

  const selectedAccountsList = computed(() => {
    return accounts.value.filter(acc => selectedAccounts.value.has(acc.id));
  });

  const activeAccountsCount = computed(() => {
    return accounts.value.filter(acc => acc.status === 'active').length;
  });

  // Actions
  async function loadAccounts() {
    loading.value = true;
    error.value = null;
    try {
      accounts.value = await accountApi.getAllAccounts();
    } catch (e) {
      error.value = (e as Error).message;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function addAccount(data: {
    email: string;
    password: string;
    nickname: string;
    tags: string[];
    group?: string;
  }) {
    loading.value = true;
    error.value = null;
    try {
      const account = await accountApi.addAccount(data);
      accounts.value.push(account);
      return account;
    } catch (e) {
      error.value = (e as Error).message;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function updateAccount(account: Account) {
    // 单个账号更新不触发全局loading，避免页面闪烁
    error.value = null;
    try {
      await accountApi.updateAccount(account);
      const index = accounts.value.findIndex(a => a.id === account.id);
      if (index !== -1) {
        // 使用splice确保触发响应式更新
        accounts.value.splice(index, 1, account);
      }
    } catch (e) {
      error.value = (e as Error).message;
      throw e;
    }
  }

  /**
   * 将账号加入批量更新队列（不立即触发UI更新）
   * 用于大量账号刷新时的性能优化
   */
  function queueAccountUpdate(account: Account) {
    pendingUpdates.value.set(account.id, account);
    
    // 使用防抖，300ms内的更新合并为一次
    if (batchUpdateTimer) {
      clearTimeout(batchUpdateTimer);
    }
    batchUpdateTimer = setTimeout(() => {
      flushPendingUpdates();
    }, 300);
  }

  /**
   * 立即应用所有待更新的账号（一次性更新UI）
   */
  async function flushPendingUpdates() {
    if (pendingUpdates.value.size === 0) return;
    
    const updates = Array.from(pendingUpdates.value.values());
    console.log(`[批量更新] 一次性更新 ${updates.length} 个账号到UI`);
    
    // 清空队列
    pendingUpdates.value.clear();
    if (batchUpdateTimer) {
      clearTimeout(batchUpdateTimer);
      batchUpdateTimer = null;
    }
    
    // 构建ID到更新数据的映射
    const updateMap = new Map(updates.map(acc => [acc.id, acc]));
    
    // 一次性更新所有账号（只触发一次响应式更新）
    accounts.value = accounts.value.map(acc => {
      const updated = updateMap.get(acc.id);
      return updated || acc;
    });
    
    // 批量保存到后端（使用 Promise.all 但不等待）
    // 这里先更新UI，后台异步保存
    Promise.all(updates.map(acc => accountApi.updateAccount(acc).catch(e => {
      console.error(`[批量更新] 保存账号 ${acc.email} 失败:`, e);
    })));
  }

  async function deleteAccount(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await accountApi.deleteAccount(id);
      accounts.value = accounts.value.filter(a => a.id !== id);
      selectedAccounts.value.delete(id);
    } catch (e) {
      error.value = (e as Error).message;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  async function deleteSelectedAccounts() {
    if (selectedAccounts.value.size === 0) return;
    
    loading.value = true;
    error.value = null;
    try {
      const ids = Array.from(selectedAccounts.value);
      const result = await accountApi.deleteAccountsBatch(ids);
      
      // 移除成功删除的账号
      accounts.value = accounts.value.filter(a => !ids.includes(a.id) || result.failed_ids.includes(a.id));
      
      // 清空选中状态
      selectedAccounts.value.clear();
      
      return result;
    } catch (e) {
      error.value = (e as Error).message;
      throw e;
    } finally {
      loading.value = false;
    }
  }

  function toggleAccountSelection(id: string) {
    if (selectedAccounts.value.has(id)) {
      selectedAccounts.value.delete(id);
    } else {
      selectedAccounts.value.add(id);
    }
  }

  function selectAll() {
    filteredAccounts.value.forEach(acc => {
      selectedAccounts.value.add(acc.id);
    });
  }

  function clearSelection() {
    selectedAccounts.value.clear();
  }

  function setFilter(filter: AccountFilter) {
    currentFilter.value = filter;
    // 重置到第一页
    pagination.value.currentPage = 1;
  }

  function clearFilter() {
    currentFilter.value = {};
    // 重置到第一页
    pagination.value.currentPage = 1;
  }

  // 分页操作
  function setCurrentPage(page: number) {
    pagination.value.currentPage = page;
  }

  function setPageSize(size: number) {
    pagination.value.pageSize = size;
    pagination.value.currentPage = 1; // 重置到第一页
  }

  // 自动刷新Token功能
  const autoRefreshTimerId = ref<number | null>(null);
  const refreshingAccounts = ref<Set<string>>(new Set()); // 跟踪正在刷新的账号

  /**
   * 检查Token是否已过期或即将过期（5分钟内）
   */
  function isTokenExpiredOrExpiring(account: Account): boolean {
    if (!account.token_expires_at) return true;
    
    const expiresAt = dayjs(account.token_expires_at);
    const now = dayjs();
    const fiveMinutesLater = now.add(5, 'minute');
    
    // Token已过期或将5分钟内过期
    return expiresAt.isBefore(fiveMinutesLater);
  }

  /**
   * 获取需要刷新Token的账号列表
   */
  function getAccountsNeedingRefresh(): Account[] {
    return accounts.value.filter(account => {
      // 跳过状态为 inactive 或 error 的账号
      if (account.status === 'inactive' || account.status === 'error') {
        return false;
      }
      
      // 跳过正在刷新的账号
      if (refreshingAccounts.value.has(account.id)) {
        return false;
      }
      
      // 检查Token是否需要刷新
      return isTokenExpiredOrExpiring(account);
    });
  }

  /**
   * 刷新单个账号的Token
   * @param useBatchUpdate 是否使用批量更新（大量刷新时设为true提升性能）
   */
  async function refreshAccountToken(account: Account, useBatchUpdate: boolean = false): Promise<{ success: boolean; error?: string }> {
    // 标记为正在刷新
    refreshingAccounts.value.add(account.id);
    
    try {
      const result = await apiService.refreshToken(account.id);
      
      if (result.success) {
        // 更新账号信息
        const updatedAccount = { ...account, status: 'active' as const };
        
        // 更新新的 token
        if (result.token) {
          updatedAccount.token = result.token;
        }
        if (result.expires_at) {
          updatedAccount.token_expires_at = result.expires_at;
        }
        if (result.plan_name) {
          updatedAccount.plan_name = result.plan_name;
        }
        if (result.used_quota !== undefined) {
          updatedAccount.used_quota = result.used_quota;
        }
        if (result.total_quota !== undefined) {
          updatedAccount.total_quota = result.total_quota;
        }
        if (result.subscription_expires_at) {
          updatedAccount.subscription_expires_at = result.subscription_expires_at;
        }
        // 更新账户禁用状态
        if (result.is_disabled !== undefined) {
          updatedAccount.is_disabled = result.is_disabled;
        }
        // 更新团队所有者状态
        if (result.is_team_owner !== undefined) {
          updatedAccount.is_team_owner = result.is_team_owner;
        }
        // ====== 弹性计费（Usage Allowance）字段 ======
        if (result.daily_quota_remaining_percent !== undefined) {
          updatedAccount.dailyQuotaRemainingPercent = result.daily_quota_remaining_percent;
        }
        if (result.weekly_quota_remaining_percent !== undefined) {
          updatedAccount.weeklyQuotaRemainingPercent = result.weekly_quota_remaining_percent;
        }
        if (result.daily_quota_reset_timestamp !== undefined) {
          updatedAccount.dailyQuotaResetTimestamp = result.daily_quota_reset_timestamp;
        }
        if (result.weekly_quota_reset_timestamp !== undefined) {
          updatedAccount.weeklyQuotaResetTimestamp = result.weekly_quota_reset_timestamp;
        }
        updatedAccount.last_quota_update = dayjs().toISOString();
        
        // 根据模式选择更新方式
        if (useBatchUpdate) {
          // 批量模式：加入队列，稍后一次性更新UI
          queueAccountUpdate(updatedAccount);
        } else {
          // 单个模式：立即更新
          await updateAccount(updatedAccount);
        }
        
        console.log(`[自动刷新] ${account.email} Token刷新成功`);
        return { success: true };
      } else {
        // 刷新失败，更新账号状态为error
        const updatedAccount = { ...account, status: 'error' as const };
        if (useBatchUpdate) {
          queueAccountUpdate(updatedAccount);
        } else {
          await updateAccount(updatedAccount);
        }
        
        console.error(`[自动刷新] ${account.email} Token刷新失败`);
        return { success: false, error: 'Token刷新失败' };
      }
    } catch (error) {
      // 刷新失败，更新账号状态为error
      const updatedAccount = { ...account, status: 'error' as const };
      if (useBatchUpdate) {
        queueAccountUpdate(updatedAccount);
      } else {
        await updateAccount(updatedAccount);
      }
      
      console.error(`[自动刷新] ${account.email} Token刷新异常:`, error);
      return { success: false, error: String(error) };
    } finally {
      // 移除正在刷新标记
      refreshingAccounts.value.delete(account.id);
    }
  }

  /**
   * 批量刷新Token（使用优化的批量 API，后端只保存一次）
   */
  async function batchRefreshTokens(accountsToRefresh?: Account[], _concurrentLimit: number = 3): Promise<{
    total: number;
    success: number;
    failed: number;
    results: Array<{ id: string; email: string; success: boolean; error?: string }>;
  }> {
    const targetAccounts = accountsToRefresh || getAccountsNeedingRefresh();
    
    if (targetAccounts.length === 0) {
      return { total: 0, success: 0, failed: 0, results: [] };
    }
    
    console.log(`[自动刷新] 开始批量刷新 ${targetAccounts.length} 个账号的Token（使用优化API）`);
    
    // 标记所有账号为正在刷新
    targetAccounts.forEach(a => refreshingAccounts.value.add(a.id));
    
    try {
      // 使用优化的批量刷新 API（后端只保存一次）
      const ids = targetAccounts.map(a => a.id);
      const apiResult = await apiService.batchRefreshTokens(ids);
      
      const results: Array<{ id: string; email: string; success: boolean; error?: string }> = [];
      
      // 处理结果，直接用返回的数据更新本地 store
      if (apiResult.results) {
        for (const item of apiResult.results) {
          const idx = accounts.value.findIndex(a => a.id === item.id);
          if (idx === -1) continue;
          
          const account = targetAccounts.find(a => a.id === item.id);
          if (!account) continue;
          
          if (item.success && item.data) {
            // 使用后端返回的完整数据更新本地 store
            // 使用 splice 替换整个对象以确保触发 Vue 响应式更新
            const updatedAcc = { ...accounts.value[idx] };
            if (item.data.plan_name) updatedAcc.plan_name = item.data.plan_name;
            if (item.data.used_quota !== undefined) updatedAcc.used_quota = item.data.used_quota;
            if (item.data.total_quota !== undefined) updatedAcc.total_quota = item.data.total_quota;
            if (item.data.expires_at) updatedAcc.token_expires_at = item.data.expires_at;
            if (item.data.windsurf_api_key) updatedAcc.windsurf_api_key = item.data.windsurf_api_key;
            if (item.data.is_disabled !== undefined) updatedAcc.is_disabled = item.data.is_disabled;
            if (item.data.is_team_owner !== undefined) updatedAcc.is_team_owner = item.data.is_team_owner;
            if (item.data.subscription_active !== undefined) updatedAcc.subscription_active = item.data.subscription_active;
            if (item.data.subscription_expires_at) updatedAcc.subscription_expires_at = dayjs.unix(item.data.subscription_expires_at).toISOString();
            if (item.data.last_quota_update) updatedAcc.last_quota_update = item.data.last_quota_update;
            // ====== 弹性计费（Usage Allowance）字段 ======
            if (item.data.daily_quota_remaining_percent !== undefined) updatedAcc.dailyQuotaRemainingPercent = item.data.daily_quota_remaining_percent;
            if (item.data.weekly_quota_remaining_percent !== undefined) updatedAcc.weeklyQuotaRemainingPercent = item.data.weekly_quota_remaining_percent;
            if (item.data.daily_quota_reset_timestamp !== undefined) updatedAcc.dailyQuotaResetTimestamp = item.data.daily_quota_reset_timestamp;
            if (item.data.weekly_quota_reset_timestamp !== undefined) updatedAcc.weeklyQuotaResetTimestamp = item.data.weekly_quota_reset_timestamp;
            updatedAcc.status = 'active';
            accounts.value.splice(idx, 1, updatedAcc);
            
            results.push({ id: account.id, email: account.email, success: true });
          } else {
            // 刷新失败，使用 splice 确保响应式更新
            const failedAcc = { ...accounts.value[idx], status: 'error' as const };
            accounts.value.splice(idx, 1, failedAcc);
            results.push({ id: account.id, email: account.email, success: false, error: item.error });
          }
        }
      }
      
      const successCount = results.filter(r => r.success).length;
      const failedCount = results.filter(r => !r.success).length;
      
      console.log(`[自动刷新] 批量刷新完成: 成功 ${successCount}/${targetAccounts.length}, 失败 ${failedCount}`);
      
      return {
        total: targetAccounts.length,
        success: successCount,
        failed: failedCount,
        results
      };
    } finally {
      // 移除正在刷新标记
      targetAccounts.forEach(a => refreshingAccounts.value.delete(a.id));
    }
  }

  /**
   * 检查并自动刷新过期Token（供外部调用）
   */
  async function checkAndRefreshExpiredTokens(settingsStore?: any): Promise<void> {
    // 检查是否开启自动刷新
    if (settingsStore && !settingsStore.settings.auto_refresh_token) {
      return;
    }
    
    const accountsToRefresh = getAccountsNeedingRefresh();
    
    if (accountsToRefresh.length === 0) {
      return;
    }
    
    console.log(`[自动刷新] 检测到 ${accountsToRefresh.length} 个账号需要刷新Token`);
    
    // 获取并发限制
    // 如果开启了全量并发刷新，则不限制并发数（使用账号数量作为并发数）
    const unlimitedConcurrent = settingsStore?.settings.unlimitedConcurrentRefresh;
    const concurrentLimit = unlimitedConcurrent 
      ? accountsToRefresh.length 
      : (settingsStore?.settings.concurrent_limit || 3);
    
    if (unlimitedConcurrent) {
      console.log(`[自动刷新] 全量并发模式，同时刷新 ${accountsToRefresh.length} 个账号`);
    }
    
    await batchRefreshTokens(accountsToRefresh, concurrentLimit);
  }

  /**
   * 启动定时轮询（每10分钟检查一次）
   */
  function startAutoRefreshTimer(settingsStore?: any) {
    // 先清除旧的定时器
    stopAutoRefreshTimer();
    
    // 检查是否开启自动刷新
    if (settingsStore && !settingsStore.settings.auto_refresh_token) {
      console.log('[自动刷新] 自动刷新Token功能已关闭');
      return;
    }
    
    console.log('[自动刷新] 启动定时轮询，间陔10分钟');
    
    // 立即执行一次检查
    checkAndRefreshExpiredTokens(settingsStore);
    
    // 设置定时器，每10分钟执行一次
    autoRefreshTimerId.value = window.setInterval(() => {
      checkAndRefreshExpiredTokens(settingsStore);
    }, 10 * 60 * 1000); // 10分钟
  }

  /**
   * 停止定时轮询
   */
  function stopAutoRefreshTimer() {
    if (autoRefreshTimerId.value !== null) {
      clearInterval(autoRefreshTimerId.value);
      autoRefreshTimerId.value = null;
      console.log('[自动刷新] 定时轮询已停止');
    }
  }

  // ==================== 排序功能 ====================

  /**
   * 加载排序配置
   */
  async function loadSortConfig() {
    try {
      const config = await settingsApi.getSortConfig();
      sortConfig.value = config;
    } catch (e) {
      console.error('加载排序配置失败:', e);
    }
  }

  /**
   * 更新排序配置并重新排序
   */
  async function setSortConfig(field: SortField, direction: SortDirection) {
    sortConfig.value = { field, direction };
    try {
      await settingsApi.updateSortConfig(sortConfig.value);
      await applySorting();
    } catch (e) {
      console.error('更新排序配置失败:', e);
    }
  }

  /**
   * 应用当前排序配置
   */
  async function applySorting() {
    try {
      const sortedAccounts = await settingsApi.getSortedAccounts(
        sortConfig.value.field,
        sortConfig.value.direction
      );
      accounts.value = sortedAccounts;
    } catch (e) {
      console.error('应用排序失败:', e);
    }
  }

  /**
   * 更新账户顺序（用于拖拽排序）
   */
  async function updateAccountsOrder(accountIds: string[]) {
    try {
      await settingsApi.updateAccountsOrder(accountIds);
      // 更新本地顺序
      const newAccounts: Account[] = [];
      for (const id of accountIds) {
        const account = accounts.value.find(a => a.id === id);
        if (account) {
          newAccounts.push(account);
        }
      }
      // 添加不在列表中的账户（如果有的话）
      for (const account of accounts.value) {
        if (!accountIds.includes(account.id)) {
          newAccounts.push(account);
        }
      }
      accounts.value = newAccounts;
    } catch (e) {
      console.error('更新账户顺序失败:', e);
      throw e;
    }
  }

  return {
    // State
    accounts,
    selectedAccounts,
    currentFilter,
    loading,
    error,
    pagination,
    sortConfig,
    
    // Computed
    filteredAccounts,
    paginatedAccounts,
    selectedAccountsList,
    activeAccountsCount,
    totalPages,
    totalCount,
    allTags,
    allPlanNames,
    allDomains,
    
    // Actions
    loadAccounts,
    addAccount,
    updateAccount,
    deleteAccount,
    deleteSelectedAccounts,
    toggleAccountSelection,
    selectAll,
    clearSelection,
    setFilter,
    clearFilter,
    setCurrentPage,
    setPageSize,
    
    // 辅助函数
    getRemainingQuota,
    getDaysUntilExpiry,
    
    // 自动刷新Token
    isTokenExpiredOrExpiring,
    getAccountsNeedingRefresh,
    refreshAccountToken,
    batchRefreshTokens,
    checkAndRefreshExpiredTokens,
    startAutoRefreshTimer,
    stopAutoRefreshTimer,
    
    // 批量更新优化
    flushPendingUpdates,
    
    // 排序功能
    loadSortConfig,
    setSortConfig,
    applySorting,
    updateAccountsOrder,
  };
});
