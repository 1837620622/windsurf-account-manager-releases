use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 单次重置记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResetRecord {
    pub id: String,
    pub config_id: String,
    pub account_id: String,
    pub account_email: String,
    pub account_nickname: Option<String>,
    pub master_email: String,
    pub used_quota_before: i32,
    pub total_quota: i32,
    pub usage_percent: i32,
    pub auto_joined: bool,
    pub reset_at: DateTime<Utc>,
}

impl ResetRecord {
    pub fn new(
        config_id: String,
        account_id: String,
        account_email: String,
        account_nickname: Option<String>,
        master_email: String,
        used_quota_before: i32,
        total_quota: i32,
        auto_joined: bool,
    ) -> Self {
        let usage_percent = if total_quota > 0 {
            (used_quota_before as f64 / total_quota as f64 * 100.0) as i32
        } else {
            0
        };
        
        Self {
            id: Uuid::new_v4().to_string(),
            config_id,
            account_id,
            account_email,
            account_nickname,
            master_email,
            used_quota_before,
            total_quota,
            usage_percent,
            auto_joined,
            reset_at: Utc::now(),
        }
    }
}

/// 账号统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountResetStats {
    pub account_id: String,
    pub account_email: String,
    pub account_nickname: Option<String>,
    pub reset_count: i32,
    pub total_used_quota: i64,
    pub last_reset_at: Option<DateTime<Utc>>,
}

impl AccountResetStats {
    pub fn new(account_id: String, account_email: String, account_nickname: Option<String>) -> Self {
        Self {
            account_id,
            account_email,
            account_nickname,
            reset_count: 0,
            total_used_quota: 0,
            last_reset_at: None,
        }
    }
    
    pub fn add_reset(&mut self, used_quota: i32, reset_at: DateTime<Utc>) {
        self.reset_count += 1;
        self.total_used_quota += used_quota as i64;
        self.last_reset_at = Some(reset_at);
    }
}
