import { invoke } from '@tauri-apps/api/core';
import type { Account, Settings, OperationLog, UpdateSeatsResult, BillingInfo, BatchResult, GlobalTag, SortField, SortDirection, SortConfig } from '@/types';
import type { AnalyticsData } from '@/types/analytics';

// 账号管理API
export const accountApi = {
  async addAccount(data: {
    email: string;
    password: string;
    nickname: string;
    tags: string[];
    group?: string;
  }): Promise<Account> {
    return await invoke('add_account', data);
  },

  async getAllAccounts(): Promise<Account[]> {
    return await invoke('get_all_accounts');
  },

  async getAccount(id: string): Promise<Account> {
    return await invoke('get_account', { id });
  },

  async updateAccount(account: Account): Promise<void> {
    return await invoke('update_account', { account });
  },

  async deleteAccount(id: string): Promise<void> {
    return await invoke('delete_account', { id });
  },

  async deleteAccountsBatch(ids: string[]): Promise<{ success_count: number; failed_ids: string[] }> {
    return await invoke('delete_accounts_batch', { ids });
  },

  async searchAccounts(query: string): Promise<Account[]> {
    return await invoke('search_accounts', { query });
  },

  async filterAccountsByGroup(group: string): Promise<Account[]> {
    return await invoke('filter_accounts_by_group', { group });
  },

  async filterAccountsByTags(tags: string[]): Promise<Account[]> {
    return await invoke('filter_accounts_by_tags', { tags });
  }
};

// API操作
export const apiService = {
  async loginAccount(id: string): Promise<{ 
    success: boolean; 
    expires_at: string;
    plan_name?: string;
    used_quota?: number;
    total_quota?: number;
    subscription_expires_at?: string;
    is_disabled?: boolean;
    daily_quota_remaining_percent?: number;
    weekly_quota_remaining_percent?: number;
    daily_quota_reset_timestamp?: number;
    weekly_quota_reset_timestamp?: number;
  }> {
    return await invoke('login_account', { id });
  },

  async refreshToken(id: string): Promise<{ 
    success: boolean; 
    token?: string;
    expires_at?: string; 
    old_expires_at?: string; 
    message?: string;
    plan_name?: string;
    used_quota?: number;
    total_quota?: number;
    subscription_expires_at?: string;
    is_disabled?: boolean;
    is_team_owner?: boolean;
    daily_quota_remaining_percent?: number;
    weekly_quota_remaining_percent?: number;
    daily_quota_reset_timestamp?: number;
    weekly_quota_reset_timestamp?: number;
  }> {
    return await invoke('refresh_token', { id });
  },

  async resetCredits(id: string, seatCount?: number): Promise<any> {
    return await invoke('reset_credits', { id, seatCount });
  },

  async updateSeats(id: string, seatCount: number, retryTimes: number): Promise<UpdateSeatsResult> {
    return await invoke('update_seats', { id, seatCount, retryTimes });
  },

  async getBilling(id: string): Promise<BillingInfo> {
    return await invoke('get_billing', { id });
  },

  async batchResetCredits(ids: string[], seatCount?: number): Promise<BatchResult> {
    return await invoke('batch_reset_credits', { ids, seatCount });
  },

  async batchRefreshTokens(ids: string[]): Promise<BatchResult> {
    return await invoke('batch_refresh_tokens', { ids });
  },

  /**
   * 更新订阅计划
   * @param id 账号ID
   * @param planType 计划类型
   * @param paymentPeriod 付款周期（1=月付, 2=年付，默认1）
   * @param preview 预览模式（true=仅预览不实际执行，默认false）
   */
  async updatePlan(id: string, planType: string, paymentPeriod: number = 1, preview: boolean = false): Promise<{
    success: boolean;
    preview?: boolean;
    plan_type?: string;
    payment_period?: number;
    payment_period_name?: string;
    status_code?: number;
    applied_changes?: boolean;
    payment_failure_reason?: string | null;
    billing_update?: {
      amount_due_immediately?: number;
      price_per_seat?: number;
      num_seats?: number;
      sub_interval?: number;
      sub_interval_name?: string;
      amount_per_interval?: number;
      billing_start?: string;
      billing_end?: string;
      unused_plan_refunded?: boolean;
      has_sso_add_on?: boolean;
    };
    requires_password_reset?: boolean;
    raw_response?: string;
    timestamp: string;
  }> {
    return await invoke('update_plan', { id, planType, paymentPeriod, preview });
  },

  /**
   * 取消订阅
   * @param id 账号ID
   * @param reason 取消原因
   * @returns 包含操作结果的响应
   */
  async cancelSubscription(id: string, reason: string): Promise<{
    success: boolean;
    reason?: string;
    status_code?: number;
    raw_response?: string;
    timestamp: string;
  }> {
    return await invoke('cancel_subscription', { id, reason });
  },

  /**
   * 恢复订阅
   * @param id 账号ID
   * @returns 包含操作结果的响应
   */
  async resumeSubscription(id: string): Promise<{
    success: boolean;
    status_code?: number;
    raw_response?: string;
    timestamp: string;
  }> {
    return await invoke('resume_subscription', { id });
  },

  async getAccountInfo(id: string): Promise<any> {
    return await invoke('get_account_info', { id });
  },

  async getCurrentUser(id: string): Promise<any> {
    return await invoke('get_current_user', { id });
  },

  /**
   * 获取套餐状态（积分/配额信息）
   * 比 getCurrentUser 更轻量，专用于刷新积分状态
   * @param id 账号ID
   * @returns 包含套餐状态和积分信息的响应
   */
  async getPlanStatus(id: string): Promise<{
    success: boolean;
    plan_status?: {
      plan_name?: string;
      teams_tier?: number;
      teams_tier_name?: string;
      used_prompt_credits?: number;
      used_flow_credits?: number;
      used_flex_credits?: number;
      available_prompt_credits?: number;
      available_flow_credits?: number;
      available_flex_credits?: number;
      monthly_prompt_credits?: number;
      monthly_flow_credits?: number;
      plan_start?: number;
      plan_end?: number;
      daily_quota_remaining_percent?: number;
      weekly_quota_remaining_percent?: number;
      daily_quota_reset_timestamp?: number;
      weekly_quota_reset_timestamp?: number;
    };
    status_code?: number;
    error?: string;
    timestamp: string;
  }> {
    return await invoke('get_plan_status', { id });
  },

  /**
   * 获取试用绑卡链接
   * @param id 账号ID
   * @param priceId 可选的Stripe价格ID，默认为Pro计划
   * @returns 包含Stripe Checkout链接的响应
   */
  async getTrialPaymentLink(id: string, priceId?: string): Promise<{
    success: boolean;
    stripe_url?: string;
    price_id?: string;
    status_code?: number;
    error?: string;
    timestamp: string;
  }> {
    return await invoke('get_trial_payment_link', { id, priceId });
  },

  // === Protobuf解析API（返回解析后的结构化数据） ===
  
  /**
   * 获取当前用户信息（自动解析Protobuf）
   * 返回解析后的用户数据结构
   */
  async getCurrentUserParsed(id: string): Promise<{
    success: boolean;
    data?: {
      user: {
        id: string;
        name: string;
        email: string;
        firebase_uid: string;
        subscription_id: string;
        team: string;
        timezone: string;
      };
      subscription?: {
        id: string;
        email: string;
        stripe_subscription_id: string;
        stripe_customer_id: string;
        seats: number;
        usage: number;
        quota: number;
        used_quota: number;
      };
      plan?: {
        name: string;
        level: number;
        bandwidth_limit: number;
        cache_limit: number;
      };
      admin?: {
        id: string;
        username: string;
        role: string;
      };
      is_root_admin: boolean;
    };
    parsed_data?: any;
    timestamp: string;
    error?: string;
  }> {
    return await invoke('get_current_user_parsed', { id });
  },

  /**
   * 获取账单信息（自动解析Protobuf）
   * TODO: 实现Protobuf解析
   */
  async getBillingParsed(id: string): Promise<any> {
    return await invoke('get_billing_parsed', { id });
  },

  /**
   * 批量获取用户信息（自动解析Protobuf）
   */
  async batchGetUsersParsed(ids: string[]): Promise<{
    success: boolean;
    results: Array<{
      id: string;
      success: boolean;
      data?: any;
      error?: string;
    }>;
    total: number;
    timestamp: string;
  }> {
    return await invoke('batch_get_users_parsed', { ids });
  },

  /**
   * 一键切换账号到Windsurf
   * @param id 账号ID
   * @returns 切换结果
   */
  async switchAccount(id: string): Promise<{ 
    success: boolean; 
    message?: string;
    auth_token?: string;
    machine_id_reset?: boolean;
    error?: string;
  }> {
    return await invoke('switch_account', { id });
  },
  
  /**
   * 重置机器ID
   * @returns 重置结果
   */
  async resetMachineId(): Promise<{
    success: boolean;
    message?: string;
    error?: string;
    requires_admin?: boolean;
  }> {
    return await invoke('reset_machine_id');
  },
};

// 设置管理API
export const settingsApi = {
  async getSettings(): Promise<Settings> {
    return await invoke('get_settings');
  },

  async updateSettings(settings: Settings): Promise<void> {
    return await invoke('update_settings', { settings });
  },

  async getGroups(): Promise<string[]> {
    return await invoke('get_groups');
  },

  async addGroup(name: string): Promise<void> {
    return await invoke('add_group', { name });
  },

  async deleteGroup(name: string): Promise<void> {
    return await invoke('delete_group', { name });
  },

  async renameGroup(oldName: string, newName: string): Promise<void> {
    return await invoke('rename_group', { oldName, newName });
  },

  async getLogs(limit?: number): Promise<OperationLog[]> {
    return await invoke('get_logs', { limit });
  },

  async clearLogs(): Promise<void> {
    return await invoke('clear_logs');
  },

  async exportData(): Promise<any> {
    return await invoke('export_data');
  },

  async getStats(): Promise<any> {
    return await invoke('get_stats');
  },

  async getCurrentWindsurfInfo(): Promise<{
    email?: string;
    name?: string;
    api_key?: string;
    plan_name?: string;
    team_id?: string;
    version?: string;
    is_active: boolean;
  }> {
    return await invoke('get_current_windsurf_info');
  },

  // 标签管理
  async getTags(): Promise<GlobalTag[]> {
    return await invoke('get_tags');
  },

  async addTag(tag: GlobalTag): Promise<void> {
    return await invoke('add_tag', { tag });
  },

  async updateTag(oldName: string, tag: GlobalTag): Promise<void> {
    return await invoke('update_tag', { oldName, tag });
  },

  async deleteTag(name: string): Promise<void> {
    return await invoke('delete_tag', { name });
  },

  async batchUpdateAccountTags(accountIds: string[], addTags: string[], removeTags: string[]): Promise<{
    success_count: number;
    failed_count: number;
  }> {
    return await invoke('batch_update_account_tags', { accountIds, addTags, removeTags });
  },

  // Team Settings API
  async getTeamConfig(id: string): Promise<{
    success: boolean;
    data?: {
      team_id?: string;
      allow_auto_run_commands?: boolean;
      allow_mcp_servers?: boolean;
      allow_app_deployments?: boolean;
      allow_sandbox_app_deployments?: boolean;
      allow_teams_app_deployments?: boolean;
      allow_github_reviews?: boolean;
      allow_github_description_edits?: boolean;
      allow_conversation_sharing?: boolean;
      allow_individual_level_analytics?: boolean;
      allow_attribution?: boolean;
      allow_vibe_and_replace?: boolean;
      allow_browser_experimental_features?: boolean;
      disable_deepwiki?: boolean;
      allowed_mcp_servers?: string[];
    };
    error?: string;
  }> {
    return await invoke('get_team_config', { id });
  },

  async updateTeamConfig(id: string, config: {
    allow_auto_run_commands?: boolean;
    allow_mcp_servers?: boolean;
    allow_app_deployments?: boolean;
    allow_sandbox_app_deployments?: boolean;
    allow_teams_app_deployments?: boolean;
    allow_github_reviews?: boolean;
    allow_github_description_edits?: boolean;
    allow_conversation_sharing?: boolean;
    allow_individual_level_analytics?: boolean;
    allow_attribution?: boolean;
    allow_vibe_and_replace?: boolean;
    allow_browser_experimental_features?: boolean;
    disable_deepwiki?: boolean;
    allowed_mcp_servers?: string;
  }): Promise<{ success: boolean; error?: string }> {
    return await invoke('update_team_config', { id, config });
  },

  // 排序管理
  async getSortedAccounts(sortField: SortField, sortDirection: SortDirection): Promise<Account[]> {
    return await invoke('get_sorted_accounts', { sortField, sortDirection });
  },

  async updateAccountsOrder(accountIds: string[]): Promise<void> {
    return await invoke('update_accounts_order', { accountIds });
  },

  async updateSortConfig(sortConfig: SortConfig): Promise<void> {
    return await invoke('update_sort_config', { sortConfig });
  },

  async getSortConfig(): Promise<SortConfig> {
    return await invoke('get_sort_config');
  }
};

// Analytics API
export const analyticsApi = {
  /**
   * 获取账户的使用分析数据（最近30天）
   * @param id 账户ID
   * @returns 分析数据
   */
  async getAccountAnalytics(id: string): Promise<AnalyticsData> {
    return await invoke('get_account_analytics', { id });
  }
};

// 系统维护 API
export const systemApi = {
  /**
   * 重置HTTP客户端（用于从网络故障中恢复）
   * 当遇到连续的API请求失败时，可以调用此方法重置HTTP连接池
   */
  async resetHttpClient(): Promise<{ success: boolean; message: string }> {
    return await invoke('reset_http_client');
  }
};
