use crate::models::{Account, OperationLog, OperationType, OperationStatus};
use crate::repository::DataStore;
use crate::services::{AuthService, WindsurfService};
use serde_json::json;
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

#[tauri::command]
pub async fn add_account(
    email: String,
    password: String,
    nickname: String,
    tags: Vec<String>,
    group: Option<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<Account, String> {
    let mut account = store.add_account(email.clone(), password, nickname)
        .await
        .map_err(|e| e.to_string())?;
    
    // 设置标签和分组
    account.tags = tags;
    account.group = group;
    
    store.update_account(account.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // 记录日志
    let log = OperationLog::new(
        OperationType::AddAccount,
        OperationStatus::Success,
        format!("添加账号: {}", email),
    )
    .with_account(account.id, email);
    
    let _ = store.add_log(log).await;
    
    Ok(account)
}

/// 通过 refresh_token 添加账号
/// 使用 refresh_token 获取 access_token，然后获取用户信息并创建账号
#[tauri::command]
pub async fn add_account_by_refresh_token(
    refresh_token: String,
    nickname: Option<String>,
    tags: Vec<String>,
    group: Option<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let auth_service = AuthService::new();
    
    // Step 1: 使用 refresh_token 获取 access_token
    let (token, new_refresh_token, expires_at) = auth_service.refresh_token(&refresh_token)
        .await
        .map_err(|e| format!("刷新Token失败: {}", e))?;
    
    // Step 2: 使用 token 获取用户信息
    let account_info = auth_service.get_account_info(&token)
        .await
        .map_err(|e| format!("获取用户信息失败: {}", e))?;
    
    let email = account_info.email.clone();
    
    // 检查账号是否已存在
    let existing_accounts = store.get_all_accounts()
        .await
        .map_err(|e| e.to_string())?;
    
    if existing_accounts.iter().any(|acc| acc.email.to_lowercase() == email.to_lowercase()) {
        return Err(format!("账号 {} 已存在", email));
    }
    
    // Step 3: 创建账号（使用空密码，因为我们有 refresh_token）
    let final_nickname = nickname.unwrap_or_else(|| email.split('@').next().unwrap_or(&email).to_string());
    
    let mut account = store.add_account(email.clone(), String::new(), final_nickname)
        .await
        .map_err(|e| e.to_string())?;
    
    // 设置标签和分组
    account.tags = tags;
    account.group = group;
    account.token = Some(token.clone());
    account.token_expires_at = Some(expires_at);
    account.refresh_token = Some(new_refresh_token);
    account.status = crate::models::account::AccountStatus::Active;
    account.last_login_at = Some(chrono::Utc::now());
    
    // 获取账号详细信息（套餐、积分等）
    let windsurf_service = WindsurfService::new();
    if let Ok(user_info_result) = windsurf_service.get_current_user(&token).await {
        if let Some(user_info) = user_info_result.get("user_info") {
            // 提取用户基本信息（包含api_key）
            if let Some(user) = user_info.get("user") {
                if let Some(api_key) = user.get("api_key").and_then(|v| v.as_str()) {
                    account.windsurf_api_key = Some(api_key.to_string());
                }
                // 提取账户禁用状态
                let disable_codeium = user.get("disable_codeium")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                account.is_disabled = Some(disable_codeium);
            }

            // 提取套餐信息
            if let Some(plan) = user_info.get("plan") {
                if let Some(plan_name) = plan.get("plan_name").and_then(|v| v.as_str()) {
                    account.plan_name = Some(plan_name.to_string());
                }
            }

            // 提取配额信息
            if let Some(subscription) = user_info.get("subscription") {
                if let Some(used) = subscription.get("used_quota").and_then(|v| v.as_i64()) {
                    account.used_quota = Some(used as i32);
                }
                if let Some(total) = subscription.get("quota").and_then(|v| v.as_i64()) {
                    account.total_quota = Some(total as i32);
                }
                // 提取订阅到期时间
                if let Some(expires_at) = subscription.get("expires_at").and_then(|v| v.as_i64()) {
                    account.subscription_expires_at = chrono::DateTime::from_timestamp(expires_at, 0);
                }
            }

            account.last_quota_update = Some(chrono::Utc::now());
        }
    }
    
    store.update_account(account.clone())
        .await
        .map_err(|e| e.to_string())?;
    
    // 记录日志
    let log = OperationLog::new(
        OperationType::AddAccount,
        OperationStatus::Success,
        format!("通过RefreshToken添加账号: {}", email),
    )
    .with_account(account.id, email.clone());
    
    let _ = store.add_log(log).await;
    
    Ok(json!({
        "success": true,
        "account": account,
        "email": email,
        "plan_name": account.plan_name,
        "used_quota": account.used_quota,
        "total_quota": account.total_quota
    }))
}

#[tauri::command]
pub async fn get_all_accounts(
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<Account>, String> {
    store.get_all_accounts()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_account(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<Account, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    store.get_account(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_account(
    account: serde_json::Value,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    // 解析账号JSON，处理可能的密码更新
    let account_id = account.get("id")
        .and_then(|v| v.as_str())
        .ok_or("Invalid account ID")?;

    let id = Uuid::parse_str(account_id).map_err(|e| e.to_string())?;

    // 获取现有账号
    let mut existing_account = store.get_account(id)
        .await
        .map_err(|e| e.to_string())?;

    // 更新基本信息
    if let Some(nickname) = account.get("nickname").and_then(|v| v.as_str()) {
        existing_account.nickname = nickname.to_string();
    }

    if let Some(group) = account.get("group").and_then(|v| v.as_str()) {
        existing_account.group = if group.is_empty() { None } else { Some(group.to_string()) };
    }

    if let Some(tags) = account.get("tags").and_then(|v| v.as_array()) {
        existing_account.tags = tags.iter()
            .filter_map(|t| t.as_str().map(|s| s.to_string()))
            .collect();
    }

    // 更新配额和套餐信息（从API获取的数据）
    if let Some(plan_name) = account.get("plan_name").and_then(|v| v.as_str()) {
        existing_account.plan_name = Some(plan_name.to_string());
    }

    if let Some(used_quota) = account.get("used_quota").and_then(|v| v.as_i64()) {
        existing_account.used_quota = Some(used_quota as i32);
    }

    if let Some(total_quota) = account.get("total_quota").and_then(|v| v.as_i64()) {
        existing_account.total_quota = Some(total_quota as i32);
    }

    if let Some(last_quota_update) = account.get("last_quota_update").and_then(|v| v.as_str()) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(last_quota_update) {
            existing_account.last_quota_update = Some(dt.with_timezone(&chrono::Utc));
        }
    }

    // 更新订阅到期时间
    if let Some(subscription_expires_at) = account.get("subscription_expires_at").and_then(|v| v.as_str()) {
        if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(subscription_expires_at) {
            existing_account.subscription_expires_at = Some(dt.with_timezone(&chrono::Utc));
        }
    }

    // 更新账户禁用状态
    if let Some(is_disabled) = account.get("is_disabled") {
        if is_disabled.is_null() {
            existing_account.is_disabled = None;
        } else if let Some(disabled) = is_disabled.as_bool() {
            existing_account.is_disabled = Some(disabled);
        }
    }

    // 更新 Windsurf API Key
    if let Some(windsurf_api_key) = account.get("windsurf_api_key").and_then(|v| v.as_str()) {
        existing_account.windsurf_api_key = Some(windsurf_api_key.to_string());
    }

    // 更新 Token（如果有）
    if let Some(token) = account.get("token").and_then(|v| v.as_str()) {
        if !token.is_empty() {
            existing_account.token = Some(token.to_string());
        }
    }

    // 更新账户状态
    if let Some(status) = account.get("status").and_then(|v| v.as_str()) {
        existing_account.status = match status {
            "active" => crate::models::account::AccountStatus::Active,
            "inactive" => crate::models::account::AccountStatus::Inactive,
            "error" => crate::models::account::AccountStatus::Error("API错误".to_string()),
            _ => crate::models::account::AccountStatus::Error(status.to_string()),
        };
    }

    // 先更新基本信息
    store.update_account(existing_account.clone())
        .await
        .map_err(|e| e.to_string())?;

    // 如果有新密码，单独更新
    if let Some(new_password) = account.get("password").and_then(|v| v.as_str()) {
        if !new_password.is_empty() {
            // 调用专门的密码更新方法
            store.update_account_password(id, new_password.to_string())
                .await
                .map_err(|e| e.to_string())?;
        }
    }

    // 记录日志
    let log = OperationLog::new(
        OperationType::EditAccount,
        OperationStatus::Success,
        format!("更新账号: {}", existing_account.email),
    )
    .with_account(existing_account.id, existing_account.email.clone());

    let _ = store.add_log(log).await;

    Ok(())
}

#[tauri::command]
pub async fn delete_account(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    
    // 获取账号信息用于日志
    let account = store.get_account(uuid).await.ok();
    
    store.delete_account(uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    // 记录日志
    if let Some(acc) = account {
        let log = OperationLog::new(
            OperationType::DeleteAccount,
            OperationStatus::Success,
            format!("删除账号: {}", acc.email),
        );
        let _ = store.add_log(log).await;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn delete_accounts_batch(
    ids: Vec<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let mut success_count = 0;
    let mut failed_ids = Vec::new();
    
    for id_str in ids {
        if let Ok(uuid) = Uuid::parse_str(&id_str) {
            if store.delete_account(uuid).await.is_ok() {
                success_count += 1;
            } else {
                failed_ids.push(id_str);
            }
        } else {
            failed_ids.push(id_str);
        }
    }
    
    // 记录批量操作日志
    let log = OperationLog::new(
        OperationType::BatchOperation,
        if failed_ids.is_empty() { OperationStatus::Success } else { OperationStatus::Failed },
        format!("批量删除账号: 成功{}个，失败{}个", success_count, failed_ids.len()),
    );
    let _ = store.add_log(log).await;
    
    Ok(json!({
        "success_count": success_count,
        "failed_ids": failed_ids
    }))
}

#[tauri::command]
pub async fn search_accounts(
    query: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<Account>, String> {
    let all_accounts = store.get_all_accounts()
        .await
        .map_err(|e| e.to_string())?;
    
    let query_lower = query.to_lowercase();
    let filtered: Vec<Account> = all_accounts
        .into_iter()
        .filter(|acc| {
            acc.email.to_lowercase().contains(&query_lower) ||
            acc.nickname.to_lowercase().contains(&query_lower) ||
            acc.tags.iter().any(|tag| tag.to_lowercase().contains(&query_lower))
        })
        .collect();
    
    Ok(filtered)
}

#[tauri::command]
pub async fn filter_accounts_by_group(
    group: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<Account>, String> {
    let all_accounts = store.get_all_accounts()
        .await
        .map_err(|e| e.to_string())?;
    
    let filtered: Vec<Account> = all_accounts
        .into_iter()
        .filter(|acc| acc.group.as_ref() == Some(&group))
        .collect();
    
    Ok(filtered)
}

#[tauri::command]
pub async fn filter_accounts_by_tags(
    tags: Vec<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<Account>, String> {
    let all_accounts = store.get_all_accounts()
        .await
        .map_err(|e| e.to_string())?;
    
    let filtered: Vec<Account> = all_accounts
        .into_iter()
        .filter(|acc| {
            tags.iter().any(|tag| acc.tags.contains(tag))
        })
        .collect();
    
    Ok(filtered)
}
