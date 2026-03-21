-- ============================================================
-- Windsurf 账号云端管理数据库表
-- 与桌面应用 Account 模型完全对齐
-- ============================================================

-- 账号主表：记录用户登录的全部账号信息
CREATE TABLE IF NOT EXISTS windsurf_accounts (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  -- 本地账号ID（桌面应用的UUID，用于关联同步）
  local_id TEXT,
  -- 基础登录信息
  email TEXT NOT NULL,
  password TEXT,
  nickname TEXT DEFAULT '',
  tags TEXT[] DEFAULT '{}',
  group_name TEXT DEFAULT '',
  -- 套餐类型（不做严格约束，Windsurf可能返回各种值如 Free/Pro/Pro_yearly/Team/Max 等）
  plan_type TEXT DEFAULT 'unknown',
  -- 账号状态
  status TEXT DEFAULT 'active',
  -- Token相关
  token_expires_at TIMESTAMPTZ,
  -- 订阅信息
  subscription_expires_at TIMESTAMPTZ,
  subscription_active BOOLEAN,
  -- 配额信息
  used_quota INTEGER DEFAULT 0,
  total_quota INTEGER DEFAULT 0,
  last_quota_update TIMESTAMPTZ,
  -- Windsurf账户信息
  windsurf_api_key TEXT,
  is_disabled BOOLEAN DEFAULT FALSE,
  is_team_owner BOOLEAN DEFAULT FALSE,
  -- 弹性计费（Usage Allowance）
  daily_quota_remaining_percent REAL,
  weekly_quota_remaining_percent REAL,
  daily_quota_reset_timestamp BIGINT,
  weekly_quota_reset_timestamp BIGINT,
  -- 登录记录
  last_login_at TIMESTAMPTZ,
  -- 附加信息
  notes TEXT DEFAULT '',
  source TEXT DEFAULT 'desktop_app',
  -- 时间戳
  created_at TIMESTAMPTZ DEFAULT NOW(),
  updated_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(email)
);

-- 登录记录表：记录每次登录/刷新操作
CREATE TABLE IF NOT EXISTS login_records (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  account_id UUID REFERENCES windsurf_accounts(id) ON DELETE CASCADE,
  email TEXT NOT NULL,
  action TEXT DEFAULT 'login',
  plan_type TEXT,
  login_at TIMESTAMPTZ DEFAULT NOW(),
  success BOOLEAN DEFAULT TRUE,
  error_message TEXT,
  device_info TEXT,
  extra JSONB DEFAULT '{}'
);

-- 操作日志表：记录所有操作
CREATE TABLE IF NOT EXISTS operation_logs (
  id UUID DEFAULT gen_random_uuid() PRIMARY KEY,
  action TEXT NOT NULL,
  target_email TEXT,
  detail JSONB DEFAULT '{}',
  created_at TIMESTAMPTZ DEFAULT NOW()
);

-- 更新时间自动触发器
CREATE OR REPLACE FUNCTION update_updated_at()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_update_windsurf_accounts ON windsurf_accounts;
CREATE TRIGGER trigger_update_windsurf_accounts
  BEFORE UPDATE ON windsurf_accounts
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at();

-- 索引：提升查询性能
CREATE INDEX IF NOT EXISTS idx_accounts_email ON windsurf_accounts(email);
CREATE INDEX IF NOT EXISTS idx_accounts_plan_type ON windsurf_accounts(plan_type);
CREATE INDEX IF NOT EXISTS idx_accounts_status ON windsurf_accounts(status);
CREATE INDEX IF NOT EXISTS idx_accounts_created_at ON windsurf_accounts(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_accounts_local_id ON windsurf_accounts(local_id);
CREATE INDEX IF NOT EXISTS idx_login_records_account_id ON login_records(account_id);
CREATE INDEX IF NOT EXISTS idx_login_records_email ON login_records(email);
CREATE INDEX IF NOT EXISTS idx_login_records_login_at ON login_records(login_at DESC);
CREATE INDEX IF NOT EXISTS idx_operation_logs_created_at ON operation_logs(created_at DESC);

-- 启用行级安全（RLS）
ALTER TABLE windsurf_accounts ENABLE ROW LEVEL SECURITY;
ALTER TABLE login_records ENABLE ROW LEVEL SECURITY;
ALTER TABLE operation_logs ENABLE ROW LEVEL SECURITY;

-- 允许通过 anon/service_role key 访问（简单模式，幂等）
DROP POLICY IF EXISTS "anon_full_access" ON windsurf_accounts;
CREATE POLICY "anon_full_access" ON windsurf_accounts FOR ALL USING (true) WITH CHECK (true);
DROP POLICY IF EXISTS "anon_full_access" ON login_records;
CREATE POLICY "anon_full_access" ON login_records FOR ALL USING (true) WITH CHECK (true);
DROP POLICY IF EXISTS "anon_full_access" ON operation_logs;
CREATE POLICY "anon_full_access" ON operation_logs FOR ALL USING (true) WITH CHECK (true);