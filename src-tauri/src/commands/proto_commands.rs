use crate::models::{OperationLog, OperationType, OperationStatus};
use crate::repository::DataStore;
use crate::services::WindsurfService;
use crate::utils::AppError;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

/// 需要Protobuf解析的API命令集合
/// 这些命令会自动解析响应中的Protobuf数据

#[tauri::command]
pub async fn get_current_user_parsed(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    
    // 获取账号信息
    let mut account = store.get_account(uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    // 确保有有效的Token（优先使用缓存）
    super::api_commands::ensure_valid_token(&store, &mut account, uuid).await?;
    
    // 使用缓存的或新刷新的Token
    let token = account.token.ok_or("No token available")?;
    
    // 用户详情始终使用完整的 GetCurrentUser API（不受轻量级 API 设置影响）
    let windsurf_service = WindsurfService::new();
    println!("[get_current_user_parsed] Using GetCurrentUser API for account: {}", id);
    
    let result: serde_json::Value = windsurf_service.get_current_user(&token)
        .await
        .map_err(|e: AppError| e.to_string())?;
    
    println!("[get_current_user_parsed] API response keys: {:?}", result.as_object().map(|o| o.keys().collect::<Vec<_>>()));
    
    // 记录日志
    let success = result.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    println!("[get_current_user_parsed] Success: {}", success);
    let log = OperationLog::new(
        OperationType::GetAccountInfo,
        if success { OperationStatus::Success } else { OperationStatus::Failed },
        format!("获取用户详细信息{}: {}", if success { "成功" } else { "失败" }, account.email),
    )
    .with_account(uuid, account.email);
    
    let _ = store.add_log(log).await;
    
    // 返回解析后的数据
    if success {
        // 提取用户友好信息
        if let Some(user_info) = result.get("user_info") {
            Ok(json!({
                "success": true,
                "data": {
                    "user": user_info.get("user").cloned().unwrap_or(Value::Null),
                    "team": user_info.get("team"),  // 包含计费周期等信息
                    "subscription": user_info.get("subscription"),
                    "plan": user_info.get("plan"),
                    "role": user_info.get("role"),
                    "roles": user_info.get("roles"),
                    "admin": user_info.get("admin"),
                    "is_root_admin": user_info.get("is_root_admin"),
                    "permissions": user_info.get("permissions"),
                    "plan_features": user_info.get("plan_features"),
                },
                "parsed_data": result.get("parsed_data"),
                "timestamp": result.get("timestamp"),
            }))
        } else {
            // 如果解析失败，返回原始响应
            Ok(result)
        }
    } else {
        Ok(result)
    }
}

/// 获取账单信息（带Protobuf解析）
/// TODO: 实现GetTeamBilling的Protobuf解析
#[tauri::command]
pub async fn get_billing_parsed(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    // 暂时调用原始API，后续可添加Protobuf解析
    super::api_commands::get_billing(id, store).await
}

/// 批量获取用户信息（带Protobuf解析）
#[tauri::command]
pub async fn batch_get_users_parsed(
    ids: Vec<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let mut results = Vec::new();
    
    for id in ids {
        match get_current_user_parsed(id.clone(), store.clone()).await {
            Ok(user_data) => {
                results.push(json!({
                    "id": id,
                    "success": true,
                    "data": user_data,
                }));
            }
            Err(e) => {
                results.push(json!({
                    "id": id,
                    "success": false,
                    "error": e.to_string(),
                }));
            }
        }
    }
    
    Ok(json!({
        "success": true,
        "results": results,
        "total": results.len(),
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}
