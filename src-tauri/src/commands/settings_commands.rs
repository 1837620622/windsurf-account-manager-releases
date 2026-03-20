use crate::models::{Settings, OperationLog, GlobalTag, SortField, SortDirection, SortConfig, Account};
use crate::repository::{DataStore, ImportResult, BackupInfo};
use std::sync::Arc;
use std::path::PathBuf;
use tauri::State;

#[tauri::command]
pub async fn get_settings(
    store: State<'_, Arc<DataStore>>,
) -> Result<Settings, String> {
    store.get_settings()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    settings: Settings,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    // 检查配置变化
    let old_settings = store.get_settings().await.map_err(|e| e.to_string())?;
    
    // 代理配置变化
    if old_settings.proxy_enabled != settings.proxy_enabled 
        || old_settings.proxy_url != settings.proxy_url {
        println!("[Settings] Proxy config changed: enabled={}, url={:?}", 
            settings.proxy_enabled, settings.proxy_url);
        crate::services::update_proxy_config(
            settings.proxy_enabled,
            settings.proxy_url.clone()
        );
    }
    
    // 轻量级API配置变化
    if old_settings.use_lightweight_api != settings.use_lightweight_api {
        println!("[Settings] Lightweight API config changed: {}", settings.use_lightweight_api);
    }
    
    store.update_settings(settings)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_groups(
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<String>, String> {
    store.get_groups()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_group(
    name: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.add_group(name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_group(
    name: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.delete_group(name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn rename_group(
    old_name: String,
    new_name: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.rename_group(old_name, new_name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_logs(
    limit: Option<usize>,
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<OperationLog>, String> {
    store.get_logs(limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn clear_logs(
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.clear_logs()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_stats(
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let config = store.config.read().await;
    let accounts = &config.accounts;
    let logs = &config.logs;
    
    // 统计成功和失败的操作
    let successful_operations = logs.iter()
        .filter(|log| matches!(log.status, crate::models::OperationStatus::Success))
        .count();
    
    let failed_operations = logs.iter()
        .filter(|log| matches!(log.status, crate::models::OperationStatus::Failed))
        .count();
    
    // 统计重置次数
    let reset_count = logs.iter()
        .filter(|log| matches!(log.operation_type, crate::models::OperationType::ResetCredits))
        .count();
    
    let successful_resets = logs.iter()
        .filter(|log| matches!(log.operation_type, crate::models::OperationType::ResetCredits) 
            && matches!(log.status, crate::models::OperationStatus::Success))
        .count();
    
    // 获取最后操作时间
    let last_operation = logs.last().map(|log| &log.timestamp);
    
    Ok(serde_json::json!({
        "total_accounts": accounts.len(),
        "active_accounts": accounts.iter().filter(|a| matches!(a.status, crate::models::AccountStatus::Active)).count(),
        "total_operations": logs.len(),
        "successful_operations": successful_operations,
        "failed_operations": failed_operations,
        "success_rate": if logs.is_empty() { 0.0 } else { successful_operations as f64 / logs.len() as f64 * 100.0 },
        "total_resets": reset_count,
        "successful_resets": successful_resets,
        "failed_resets": reset_count - successful_resets,
        "reset_success_rate": if reset_count == 0 { 0.0 } else { successful_resets as f64 / reset_count as f64 * 100.0 },
        "last_operation": last_operation,
        "groups": config.groups.len(),
        "settings": &config.settings
    }))
}

#[tauri::command]
pub async fn export_data(
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let accounts = store.get_all_accounts().await.map_err(|e| e.to_string())?;
    let groups = store.get_groups().await.map_err(|e| e.to_string())?;
    let tags = store.get_tags().await.map_err(|e| e.to_string())?;
    let settings = store.get_settings().await.map_err(|e| e.to_string())?;
    let logs = store.get_logs(Some(1000)).await.map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "accounts": accounts,
        "groups": groups,
        "tags": tags,
        "settings": settings,
        "logs": logs,
        "export_time": chrono::Utc::now().to_rfc3339()
    }))
}

// 标签管理命令
#[tauri::command]
pub async fn get_tags(
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<GlobalTag>, String> {
    store.get_tags()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_tag(
    tag: GlobalTag,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.add_tag(tag)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_tag(
    old_name: String,
    tag: GlobalTag,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.update_tag(old_name, tag)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_tag(
    name: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.delete_tag(name)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn batch_update_account_tags(
    account_ids: Vec<String>,
    add_tags: Vec<String>,
    remove_tags: Vec<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let (success_count, failed_count) = store
        .batch_update_account_tags(account_ids, add_tags, remove_tags)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success_count": success_count,
        "failed_count": failed_count
    }))
}

// ==================== 数据备份命令 ====================

/// 创建带时间戳的备份
#[tauri::command]
pub async fn create_backup(
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let backup_path = store.create_timestamped_backup()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true,
        "path": backup_path.to_string_lossy(),
        "message": "备份创建成功"
    }))
}

/// 获取备份列表
#[tauri::command]
pub async fn list_backups(
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<BackupInfo>, String> {
    store.list_backups()
        .await
        .map_err(|e| e.to_string())
}

/// 从备份恢复
#[tauri::command]
pub async fn restore_backup(
    backup_path: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&backup_path);
    store.restore_from_backup(&path)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true,
        "message": "数据恢复成功"
    }))
}

/// 导出数据到指定路径
#[tauri::command]
pub async fn export_data_to_file(
    export_path: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from(&export_path);
    store.export_data(&path)
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(serde_json::json!({
        "success": true,
        "path": export_path,
        "message": "数据导出成功"
    }))
}

/// 从文件导入数据
#[tauri::command]
pub async fn import_data_from_file(
    import_path: String,
    merge: bool,
    store: State<'_, Arc<DataStore>>,
) -> Result<ImportResult, String> {
    let path = PathBuf::from(&import_path);
    store.import_data(&path, merge)
        .await
        .map_err(|e| e.to_string())
}

/// 获取数据目录路径
#[tauri::command]
pub async fn get_data_directory(
    store: State<'_, Arc<DataStore>>,
) -> Result<String, String> {
    Ok(store.get_data_dir().to_string_lossy().to_string())
}

// ==================== 排序命令 ====================

/// 获取排序后的账户列表
#[tauri::command]
pub async fn get_sorted_accounts(
    sort_field: SortField,
    sort_direction: SortDirection,
    store: State<'_, Arc<DataStore>>,
) -> Result<Vec<Account>, String> {
    store.get_sorted_accounts(&sort_field, &sort_direction)
        .await
        .map_err(|e| e.to_string())
}

/// 更新账户排序顺序（用于拖拽排序）
#[tauri::command]
pub async fn update_accounts_order(
    account_ids: Vec<String>,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    store.update_accounts_order(account_ids)
        .await
        .map_err(|e| e.to_string())
}

/// 更新排序配置
#[tauri::command]
pub async fn update_sort_config(
    sort_config: SortConfig,
    store: State<'_, Arc<DataStore>>,
) -> Result<(), String> {
    let mut settings = store.get_settings().await.map_err(|e| e.to_string())?;
    settings.sort_config = sort_config;
    store.update_settings(settings)
        .await
        .map_err(|e| e.to_string())
}

/// 获取排序配置
#[tauri::command]
pub async fn get_sort_config(
    store: State<'_, Arc<DataStore>>,
) -> Result<SortConfig, String> {
    let settings = store.get_settings().await.map_err(|e| e.to_string())?;
    Ok(settings.sort_config)
}
