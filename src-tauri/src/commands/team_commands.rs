//! 团队管理相关命令
//! 
//! 包含团队成员管理、邀请、移除等功能

use crate::commands::api_commands::{ensure_valid_token, ensure_valid_token_with_force, is_401_error};
use crate::repository::DataStore;
use crate::services::WindsurfService;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;

/// 邀请用户信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteUser {
    pub name: String,
    pub email: String,
}

/// 获取团队成员列表
#[tauri::command]
pub async fn get_team_members(
    id: String,
    group_id: Option<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    // 最多重试一次（401 时强制刷新 token）
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.get_team_members(&token, group_id.as_deref())
            .await
            .map_err(|e| e.to_string())?;
        
        // 检查 401 错误，如果是第一次则重试
        if is_401_error(&result) && retry == 0 {
            println!("[get_team_members] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 邀请成员加入团队
#[tauri::command]
pub async fn invite_team_members(
    id: String,
    users: Vec<InviteUser>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    // 转换为 (name, email) 元组
    let user_tuples: Vec<(String, String)> = users
        .into_iter()
        .map(|u| (u.name, u.email))
        .collect();
    
    // 最多重试一次（401 时强制刷新 token）
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.grant_preapproval(&token, user_tuples.clone())
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[invite_team_members] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 从团队中移除成员
#[tauri::command]
pub async fn remove_team_member(
    id: String,
    member_api_key: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.remove_user_from_team(&token, &member_api_key)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[remove_team_member] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 撤销预审批邀请
#[tauri::command]
pub async fn revoke_invitation(
    id: String,
    approval_id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.revoke_preapproval(&token, &approval_id)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[revoke_invitation] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 获取所有待处理的预审批邀请（管理员）
#[tauri::command]
pub async fn get_pending_invitations(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.get_preapprovals(&token)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[get_pending_invitations] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 获取当前用户的待处理邀请（普通用户）
#[tauri::command]
pub async fn get_my_pending_invitation(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.get_preapproval_for_user(&token)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[get_my_pending_invitation] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 接受团队邀请
/// 如果 approval_id 为空，自动获取最新的待处理邀请
#[tauri::command]
pub async fn accept_invitation(
    id: String,
    approval_id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    
    let mut account = store.get_account(uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    ensure_valid_token(&store, &mut account, uuid).await?;
    
    let token = account.token.ok_or("No token available")?;
    
    let windsurf_service = WindsurfService::new();
    
    // 如果没有提供 approval_id，先获取最新的待处理邀请
    let actual_approval_id = if approval_id.is_empty() {
        let preapproval_result = windsurf_service.get_preapproval_for_user(&token)
            .await
            .map_err(|e| e.to_string())?;
        
        // 从响应中提取 approval_id
        // 结构: data.subMesssage_1.string_1 = approval_id
        if let Some(data) = preapproval_result.get("data") {
            if let Some(sub) = data.get("subMesssage_1") {
                if let Some(aid) = sub.get("string_1").and_then(|v| v.as_str()) {
                    aid.to_string()
                } else {
                    return Ok(serde_json::json!({
                        "success": false,
                        "error": "没有找到待处理的邀请"
                    }));
                }
            } else {
                return Ok(serde_json::json!({
                    "success": false,
                    "error": "没有找到待处理的邀请"
                }));
            }
        } else {
            return Ok(serde_json::json!({
                "success": false,
                "error": "没有找到待处理的邀请"
            }));
        }
    } else {
        approval_id
    };
    
    let result = windsurf_service.accept_preapproval(&token, &actual_approval_id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}

/// 拒绝团队邀请
#[tauri::command]
pub async fn reject_invitation(
    id: String,
    approval_id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.reject_preapproval(&token, &approval_id)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[reject_invitation] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 申请加入团队（通过邀请链接）
#[tauri::command]
pub async fn request_team_access(
    id: String,
    invite_id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    
    let account = store.get_account(uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    // 需要用户的 api_key（UUID格式）
    let api_key = account.windsurf_api_key
        .ok_or("需要先登录获取 API Key")?;
    
    let windsurf_service = WindsurfService::new();
    let result = windsurf_service.request_team_access(&api_key, &invite_id)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(result)
}

/// 审批用户加入团队申请（管理员）
/// action: "approve" 或 "reject"
#[tauri::command]
pub async fn approve_team_join_request(
    id: String,
    user_api_key: String,
    action: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    // 2 = APPROVED, 3 = REJECTED
    let status: u8 = match action.to_lowercase().as_str() {
        "approve" => 2,
        "reject" => 3,
        _ => return Err("无效的操作，请使用 'approve' 或 'reject'".to_string()),
    };
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.update_user_team_status(&token, &user_api_key, status)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[approve_team_join_request] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

// ==================== 自动充值管理命令 ====================

/// 获取自动充值设置
#[tauri::command]
pub async fn get_credit_top_up_settings(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.get_credit_top_up_settings(&token)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[get_credit_top_up_settings] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 更新自动充值设置
#[tauri::command]
pub async fn update_credit_top_up_settings(
    id: String,
    enabled: bool,
    monthly_top_up_amount: i32,
    top_up_increment: i32,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.update_credit_top_up_settings(
            &token,
            enabled,
            monthly_top_up_amount,
            top_up_increment
        )
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[update_credit_top_up_settings] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

// ==================== 成员权限管理命令 ====================

/// 更新成员的 Windsurf 访问权限（禁用/启用）
#[tauri::command]
pub async fn update_codeium_access(
    id: String,
    member_api_key: String,
    disable_access: bool,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.update_codeium_access(&token, &member_api_key, disable_access)
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[update_codeium_access] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 添加用户角色
#[tauri::command]
pub async fn add_user_role(
    id: String,
    member_api_key: String,
    role: String,
    group_id: Option<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.add_user_role(&token, &member_api_key, &role, group_id.as_deref())
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[add_user_role] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 移除用户角色
#[tauri::command]
pub async fn remove_user_role(
    id: String,
    member_api_key: String,
    role: String,
    group_id: Option<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    let windsurf_service = WindsurfService::new();
    
    for retry in 0..2 {
        let mut account = store.get_account(uuid)
            .await
            .map_err(|e| e.to_string())?;
        
        ensure_valid_token_with_force(&store, &mut account, uuid, retry > 0).await?;
        
        let token = account.token.ok_or("No token available")?;
        
        let result = windsurf_service.remove_user_role(&token, &member_api_key, &role, group_id.as_deref())
            .await
            .map_err(|e| e.to_string())?;
        
        if is_401_error(&result) && retry == 0 {
            println!("[remove_user_role] 收到 401 错误，强制刷新 token 并重试...");
            continue;
        }
        
        return Ok(result);
    }
    
    Err("重试次数已用尽".to_string())
}

/// 转让订阅
/// 执行步骤：1.禁用自己的访问 2.邀请目标用户 3.目标用户自动接受邀请 4.授予管理员权限 5.移除自己
#[tauri::command]
pub async fn transfer_subscription(
    id: String,
    target_email: String,
    target_name: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    
    let mut account = store.get_account(uuid)
        .await
        .map_err(|e| e.to_string())?;
    
    ensure_valid_token(&store, &mut account, uuid).await?;
    
    let token = account.token.ok_or("No token available")?;
    let windsurf_service = WindsurfService::new();
    
    // Step 1: 获取当前用户信息
    let current_user = windsurf_service.get_current_user(&token)
        .await
        .map_err(|e| format!("获取当前用户信息失败: {}", e))?;
    
    let current_api_key = current_user.get("user_info")
        .and_then(|u| u.get("user"))
        .and_then(|u| u.get("api_key"))
        .and_then(|k| k.as_str())
        .ok_or("无法获取当前用户 API Key")?
        .to_string();
    
    // Step 2: 禁用自己的访问权限
    let _ = windsurf_service.update_codeium_access(&token, &current_api_key, true)
        .await
        .map_err(|e| format!("禁用访问权限失败: {}", e))?;
    
    // Step 3: 邀请目标用户
    let users = vec![(target_name.clone(), target_email.clone())];
    let _ = windsurf_service.grant_preapproval(&token, users)
        .await
        .map_err(|e| format!("邀请用户失败: {}", e))?;
    
    // Step 4: 查找目标用户是否在账号管理器中，如果在则自动接受邀请
    let all_accounts = store.get_all_accounts()
        .await
        .map_err(|e| e.to_string())?;
    
    let target_account = all_accounts.iter().find(|a| {
        a.email.to_lowercase() == target_email.to_lowercase()
    });
    
    if let Some(mut target_acc) = target_account.cloned() {
        // 确保目标账户的 token 有效
        let target_uuid = target_acc.id;
        if let Ok(_) = ensure_valid_token(&store, &mut target_acc, target_uuid).await {
            if let Some(target_token) = &target_acc.token {
                // 获取目标用户的待处理邀请
                if let Ok(preapproval_result) = windsurf_service.get_preapproval_for_user(target_token).await {
                    // 解析 approval_id，路径: data.subMesssage_1.string_1
                    if let Some(approval_id) = preapproval_result.get("data")
                        .and_then(|d| d.get("subMesssage_1"))
                        .and_then(|s| s.get("string_1"))
                        .and_then(|id| id.as_str()) 
                    {
                        // 自动接受邀请
                        let _ = windsurf_service.accept_preapproval(target_token, approval_id).await;
                    }
                }
            }
        }
    }
    
    // Step 5: 等待并重试获取团队成员，检查目标用户是否已加入
    let mut target_api_key: Option<String> = None;
    
    // 重试最多3次，每次等待1秒
    for retry in 0..3 {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        
        let members = windsurf_service.get_team_members(&token, None)
            .await
            .map_err(|e| format!("获取团队成员失败: {}", e))?;
        
        // 从原始 protobuf 数据中提取成员信息
        // 数据结构: { "data": { "subMesssage_1": [{ "string_1": api_key, "string_3": email }, ...] } }
        if let Some(data) = members.get("data") {
            // subMesssage_1 是用户数组
            if let Some(users_arr) = data.get("subMesssage_1").and_then(|v| v.as_array()) {
                for user in users_arr {
                    if let Some(email) = user.get("string_3").and_then(|e| e.as_str()) {
                        if email.to_lowercase() == target_email.to_lowercase() {
                            if let Some(api_key) = user.get("string_1").and_then(|k| k.as_str()) {
                                target_api_key = Some(api_key.to_string());
                                println!("[TransferSubscription] Found target user: {}, api_key: {}", email, api_key);
                                break;
                            }
                        }
                    }
                }
            }
        }
        
        if target_api_key.is_some() {
            break;
        }
        
        if retry < 2 {
            println!("[TransferSubscription] Retry {}: Target user not found yet, will retry...", retry + 1);
            continue;
        }
    }
    
    if let Some(api_key) = target_api_key {
        // Step 6: 授予管理员权限
        let _ = windsurf_service.add_user_role(&token, &api_key, "root.admin", None)
            .await
            .map_err(|e| format!("授予管理员权限失败: {}", e))?;
        
        // Step 7: 移除自己
        let _ = windsurf_service.remove_user_from_team(&token, &current_api_key)
            .await
            .map_err(|e| format!("移除自己失败: {}", e))?;
        
        Ok(serde_json::json!({
            "success": true,
            "message": format!("订阅已成功转让给 {}", target_email)
        }))
    } else {
        // 目标用户未能自动加入，返回提示
        Ok(serde_json::json!({
            "success": false,
            "error": "邀请已发送，但目标用户需要手动接受邀请后才能完成转让"
        }))
    }
}
