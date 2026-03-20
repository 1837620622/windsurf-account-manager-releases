use crate::models::{Account, AppConfig, OperationLog};
use crate::utils::{AppError, AppResult};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use tauri::{Manager, Emitter};
use chrono::Local;
use serde::Serialize;

/// Token 刷新事件负载
#[derive(Clone, Serialize)]
pub struct TokenRefreshedPayload {
    pub account_id: String,
    pub token: String,
    pub token_expires_at: String,
}

pub struct DataStore {
    pub config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
    pub logs: Arc<RwLock<Vec<OperationLog>>>,
    logs_path: PathBuf,
    app_handle: tauri::AppHandle,
}

impl DataStore {
    pub fn new(app_handle: &tauri::AppHandle) -> AppResult<Self> {
        let app_data_dir = app_handle.path().app_data_dir()
            .map_err(|e| AppError::Config(format!("Failed to get app data dir: {}", e)))?;
        
        // 确保目录存在
        fs::create_dir_all(&app_data_dir)?;
        
        let config_path = app_data_dir.join("accounts.json");
        let mut config = Self::load_config(&config_path)?;
        
        let logs_path = app_data_dir.join("logs.json");
        let mut logs = Self::load_logs(&logs_path)?;
        
        // 迁移旧的日志数据
        if !config.logs.is_empty() && logs.is_empty() {
            logs = config.logs.clone();
            config.logs.clear();
            
            // 保存迁移后的数据
            let logs_data = serde_json::to_string_pretty(&logs)?;
            fs::write(&logs_path, logs_data)?;
            
            let config_data = serde_json::to_string_pretty(&config)?;
            fs::write(&config_path, config_data)?;
        }
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            config_path,
            logs: Arc::new(RwLock::new(logs)),
            logs_path,
            app_handle: app_handle.clone(),
        })
    }

    fn load_config(path: &PathBuf) -> AppResult<AppConfig> {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(data) => {
                    match serde_json::from_str(&data) {
                        Ok(config) => Ok(config),
                        Err(e) => {
                            // JSON 解析失败，尝试从备份恢复
                            println!("[DataStore] Config file corrupted: {}, trying backup...", e);
                            Self::recover_from_backup(path)
                        }
                    }
                }
                Err(e) => {
                    // 文件读取失败，尝试从备份恢复
                    println!("[DataStore] Failed to read config: {}, trying backup...", e);
                    Self::recover_from_backup(path)
                }
            }
        } else {
            Ok(AppConfig::default())
        }
    }
    
    /// 从备份文件恢复配置
    fn recover_from_backup(path: &PathBuf) -> AppResult<AppConfig> {
        let backup_path = path.with_extension("json.backup");
        
        if backup_path.exists() {
            println!("[DataStore] Found backup file, attempting recovery...");
            let data = fs::read_to_string(&backup_path)?;
            let config: AppConfig = serde_json::from_str(&data)?;
            
            // 恢复成功后，将备份复制回主文件
            fs::copy(&backup_path, path)?;
            println!("[DataStore] Successfully recovered from backup!");
            
            Ok(config)
        } else {
            println!("[DataStore] No backup found, using default config");
            Ok(AppConfig::default())
        }
    }
    
    fn load_logs(path: &PathBuf) -> AppResult<Vec<OperationLog>> {
        if path.exists() {
            let data = fs::read_to_string(path)?;
            let logs: Vec<OperationLog> = serde_json::from_str(&data)?;
            Ok(logs)
        } else {
            Ok(Vec::new())
        }
    }

    pub async fn save(&self) -> AppResult<()> {
        let config = self.config.read().await;
        let data = serde_json::to_string_pretty(&*config)?;
        let path = self.config_path.clone();
        drop(config); // 提前释放读锁
        
        // 使用 spawn_blocking 将同步文件写入移到阻塞线程池，避免阻塞 tokio 运行时
        tokio::task::spawn_blocking(move || {
            Self::atomic_write(&path, &data)
        }).await
            .map_err(|e| AppError::Config(format!("Task join error: {}", e)))?
            .map_err(AppError::from)?;
        
        Ok(())
    }
    
    /// 原子写入：先写临时文件，创建备份，再重命名
    fn atomic_write(path: &PathBuf, data: &str) -> std::io::Result<()> {
        use std::time::{SystemTime, UNIX_EPOCH};
        
        // 使用时间戳+进程ID生成唯一临时文件名，避免并发冲突
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let pid = std::process::id();
        
        let file_stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("data");
        let parent = path.parent().unwrap_or(path);
        
        let temp_path = parent.join(format!("{}.tmp.{}.{}", file_stem, pid, timestamp));
        let backup_path = path.with_extension("json.backup");
        
        // 1. 先写入临时文件
        fs::write(&temp_path, data)?;
        
        // 2. 验证临时文件可以正常解析
        let verify_data = fs::read_to_string(&temp_path)?;
        if serde_json::from_str::<serde_json::Value>(&verify_data).is_err() {
            let _ = fs::remove_file(&temp_path);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Written data failed JSON validation"
            ));
        }
        
        // 3. 如果原文件存在，创建备份
        if path.exists() {
            // 复制到备份文件（覆盖旧备份）
            if let Err(e) = fs::copy(path, &backup_path) {
                let _ = fs::remove_file(&temp_path);
                return Err(e);
            }
        }
        
        // 4. 原子重命名临时文件为目标文件
        if let Err(e) = fs::rename(&temp_path, path) {
            // 重命名失败时清理临时文件
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }
        
        Ok(())
    }
    
    pub async fn save_logs(&self) -> AppResult<()> {
        let logs = self.logs.read().await;
        let data = serde_json::to_string_pretty(&*logs)?;
        let path = self.logs_path.clone();
        drop(logs); // 提前释放读锁
        
        // 使用 spawn_blocking 将同步文件写入移到阻塞线程池，避免阻塞 tokio 运行时
        tokio::task::spawn_blocking(move || {
            fs::write(&path, data)
        }).await
            .map_err(|e| AppError::Config(format!("Task join error: {}", e)))?
            .map_err(AppError::from)?;
        
        Ok(())
    }

    // 账号管理方法
    pub async fn add_account(&self, email: String, password: String, nickname: String) -> AppResult<Account> {
        let mut config = self.config.write().await;
        
        // 检查邮箱是否已存在
        if config.accounts.iter().any(|a| a.email == email) {
            return Err(AppError::Config(format!("Account with email {} already exists", email)));
        }
        
        // 直接保存密码，不加密，初始化标签为空
        let account = Account::new(email, password, nickname, Vec::new());
        
        config.accounts.push(account.clone());
        drop(config); // 释放写锁
        
        self.save().await?;
        Ok(account)
    }

    pub async fn get_account(&self, id: Uuid) -> AppResult<Account> {
        let config = self.config.read().await;
        config.accounts
            .iter()
            .find(|a| a.id == id)
            .cloned()
            .ok_or_else(|| AppError::AccountNotFound(id.to_string()))
    }

    pub async fn get_all_accounts(&self) -> AppResult<Vec<Account>> {
        let config = self.config.read().await;
        Ok(config.accounts.clone())
    }

    pub async fn update_account(&self, account: Account) -> AppResult<()> {
        self.update_account_internal(account, true).await
    }
    
    /// 更新账号信息，不立即保存（用于批量操作）
    pub async fn update_account_no_save(&self, account: Account) -> AppResult<()> {
        self.update_account_internal(account, false).await
    }
    
    /// 内部方法：更新账号信息
    async fn update_account_internal(&self, account: Account, save_immediately: bool) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        if let Some(existing) = config.accounts.iter_mut().find(|a| a.id == account.id) {
            // 保存原有的密码（永远不通过这个方法更新密码）
            let original_password = existing.password.clone();
            
            // Token直接保存，不加密
            
            // 更新账号信息
            *existing = account;
            
            // 恢复原有密码（密码更新应该通过专门的update_account_password方法）
            existing.password = original_password;
        } else {
            return Err(AppError::AccountNotFound(account.id.to_string()));
        }
        
        drop(config);
        
        if save_immediately {
            self.save().await?;
        }
        Ok(())
    }

    pub async fn delete_account(&self, id: Uuid) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        let initial_len = config.accounts.len();
        config.accounts.retain(|a| a.id != id);
        
        if config.accounts.len() == initial_len {
            return Err(AppError::AccountNotFound(id.to_string()));
        }
        
        drop(config);
        
        // 同时删除相关日志
        let mut logs = self.logs.write().await;
        logs.retain(|log| log.account_id != Some(id));
        drop(logs);
        
        self.save().await?;
        self.save_logs().await?;
        Ok(())
    }

    pub async fn update_account_password(&self, id: Uuid, new_password: String) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        if let Some(account) = config.accounts.iter_mut().find(|a| a.id == id) {
            // 直接保存新密码，不加密
            account.password = new_password;
        } else {
            return Err(AppError::AccountNotFound(id.to_string()));
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }
    
    pub async fn update_account_token(&self, id: Uuid, token: String, expires_at: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        if let Some(account) = config.accounts.iter_mut().find(|a| a.id == id) {
            // 直接保存Token，不加密
            account.token = Some(token);
            account.token_expires_at = Some(expires_at);
            account.last_login_at = Some(chrono::Utc::now());
            account.status = crate::models::AccountStatus::Active;
        } else {
            return Err(AppError::AccountNotFound(id.to_string()));
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }
    
    /// 更新账号 token，默认立即保存
    pub async fn update_account_tokens(&self, id: Uuid, token: String, refresh_token: String, expires_at: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
        self.update_account_tokens_internal(id, token, refresh_token, expires_at, true).await
    }
    
    /// 更新账号 token，不立即保存（用于批量操作）
    pub async fn update_account_tokens_no_save(&self, id: Uuid, token: String, refresh_token: String, expires_at: chrono::DateTime<chrono::Utc>) -> AppResult<()> {
        self.update_account_tokens_internal(id, token, refresh_token, expires_at, false).await
    }
    
    /// 内部方法：更新账号 token
    async fn update_account_tokens_internal(&self, id: Uuid, token: String, refresh_token: String, expires_at: chrono::DateTime<chrono::Utc>, save_immediately: bool) -> AppResult<()> {
        // 保存 token 副本用于事件发送
        let token_for_event = token.clone();
        
        let mut config = self.config.write().await;
        
        if let Some(account) = config.accounts.iter_mut().find(|a| a.id == id) {
            // 保存两种token
            account.token = Some(token);
            account.refresh_token = Some(refresh_token);
            account.token_expires_at = Some(expires_at);
            account.last_login_at = Some(chrono::Utc::now());
            account.status = crate::models::AccountStatus::Active;
        } else {
            return Err(AppError::AccountNotFound(id.to_string()));
        }
        
        drop(config);
        
        if save_immediately {
            self.save().await?;
        }
        
        // 发送事件通知前端 token 已刷新
        let payload = TokenRefreshedPayload {
            account_id: id.to_string(),
            token: token_for_event,
            token_expires_at: expires_at.to_rfc3339(),
        };
        if let Err(e) = self.app_handle.emit("token-refreshed", payload) {
            println!("[DataStore] Failed to emit token-refreshed event: {}", e);
        }
        
        Ok(())
    }
    
    /// 手动触发保存（用于批量操作结束后）
    pub async fn flush(&self) -> AppResult<()> {
        self.save().await
    }

    pub async fn get_decrypted_password(&self, id: Uuid) -> AppResult<String> {
        let config = self.config.read().await;
        
        let account = config.accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| AppError::AccountNotFound(id.to_string()))?;
        
        // 直接返回密码，因为已经是明文
        Ok(account.password.clone())
    }

    pub async fn get_decrypted_token(&self, id: Uuid) -> AppResult<Option<String>> {
        let config = self.config.read().await;
        
        let account = config.accounts
            .iter()
            .find(|a| a.id == id)
            .ok_or_else(|| AppError::AccountNotFound(id.to_string()))?;
        
        // 直接返回Token，因为已经是明文
        Ok(account.token.clone())
    }

    // 分组管理
    pub async fn add_group(&self, name: String) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        if !config.groups.contains(&name) {
            config.groups.push(name);
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }

    pub async fn delete_group(&self, name: String) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        config.groups.retain(|g| g != &name);
        
        // 移除账号中的分组引用
        for account in &mut config.accounts {
            if account.group == Some(name.clone()) {
                account.group = None;
            }
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }
    
    pub async fn rename_group(&self, old_name: String, new_name: String) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        // 检查新名称是否已存在
        if config.groups.contains(&new_name) {
            return Err(AppError::Config(format!("Group '{}' already exists", new_name)));
        }
        
        // 查找并重命名分组
        if let Some(index) = config.groups.iter().position(|g| g == &old_name) {
            config.groups[index] = new_name.clone();
            
            // 更新账号中的分组引用
            for account in &mut config.accounts {
                if account.group == Some(old_name.clone()) {
                    account.group = Some(new_name.clone());
                }
            }
        } else {
            return Err(AppError::Config(format!("Group '{}' not found", old_name)));
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }

    pub async fn get_groups(&self) -> AppResult<Vec<String>> {
        let config = self.config.read().await;
        Ok(config.groups.clone())
    }

    // 标签管理
    pub async fn get_tags(&self) -> AppResult<Vec<crate::models::GlobalTag>> {
        let config = self.config.read().await;
        Ok(config.tags.clone())
    }

    pub async fn add_tag(&self, tag: crate::models::GlobalTag) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        // 检查标签是否已存在
        if config.tags.iter().any(|t| t.name == tag.name) {
            return Err(AppError::Config(format!("Tag '{}' already exists", tag.name)));
        }
        
        config.tags.push(tag);
        drop(config);
        self.save().await?;
        Ok(())
    }

    pub async fn update_tag(&self, old_name: String, tag: crate::models::GlobalTag) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        // 如果名称改变，检查新名称是否已存在
        if old_name != tag.name && config.tags.iter().any(|t| t.name == tag.name) {
            return Err(AppError::Config(format!("Tag '{}' already exists", tag.name)));
        }
        
        // 查找并更新标签
        if let Some(index) = config.tags.iter().position(|t| t.name == old_name) {
            // 如果名称改变，更新账号中的标签引用和颜色
            if old_name != tag.name {
                for account in &mut config.accounts {
                    // 更新标签名称
                    if let Some(tag_index) = account.tags.iter().position(|t| t == &old_name) {
                        account.tags[tag_index] = tag.name.clone();
                    }
                    // 更新标签颜色
                    if let Some(color_index) = account.tag_colors.iter().position(|tc| tc.name == old_name) {
                        account.tag_colors[color_index].name = tag.name.clone();
                        account.tag_colors[color_index].color = tag.color.clone();
                    }
                }
            } else {
                // 只更新颜色，同步更新所有账号的默认颜色（如果账号没有自定义颜色）
                // 这里我们不强制覆盖账号的自定义颜色
            }
            config.tags[index] = tag;
        } else {
            return Err(AppError::Config(format!("Tag '{}' not found", old_name)));
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }

    pub async fn delete_tag(&self, name: String) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        config.tags.retain(|t| t.name != name);
        
        // 移除账号中的标签引用
        for account in &mut config.accounts {
            account.tags.retain(|t| t != &name);
            account.tag_colors.retain(|tc| tc.name != name);
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }

    pub async fn batch_update_account_tags(
        &self,
        account_ids: Vec<String>,
        add_tags: Vec<String>,
        remove_tags: Vec<String>,
    ) -> AppResult<(usize, usize)> {
        let mut config = self.config.write().await;
        let mut success_count = 0;
        let mut failed_count = 0;
        
        // 先克隆全局标签，避免借用冲突
        let global_tags = config.tags.clone();
        
        for id in account_ids {
            if let Ok(uuid) = uuid::Uuid::parse_str(&id) {
                if let Some(account) = config.accounts.iter_mut().find(|a| a.id == uuid) {
                    // 添加标签
                    for tag_name in &add_tags {
                        if !account.tags.contains(tag_name) {
                            account.tags.push(tag_name.clone());
                            // 如果全局标签有默认颜色，添加到账号
                            if let Some(global_tag) = global_tags.iter().find(|t| &t.name == tag_name) {
                                if !account.tag_colors.iter().any(|tc| &tc.name == tag_name) {
                                    account.tag_colors.push(crate::models::TagWithColor {
                                        name: tag_name.clone(),
                                        color: global_tag.color.clone(),
                                    });
                                }
                            }
                        }
                    }
                    // 移除标签
                    for tag_name in &remove_tags {
                        account.tags.retain(|t| t != tag_name);
                        account.tag_colors.retain(|tc| &tc.name != tag_name);
                    }
                    success_count += 1;
                } else {
                    failed_count += 1;
                }
            } else {
                failed_count += 1;
            }
        }
        
        drop(config);
        self.save().await?;
        Ok((success_count, failed_count))
    }

    // 日志管理
    pub async fn add_log(&self, log: OperationLog) -> AppResult<()> {
        let mut logs = self.logs.write().await;
        
        logs.push(log);
        
        // 限制日志数量，保留最新的1000条
        if logs.len() > 1000 {
            let start = logs.len() - 1000;
            logs.drain(0..start);
        }
        
        drop(logs);
        self.save_logs().await?;
        Ok(())
    }

    pub async fn get_logs(&self, limit: Option<usize>) -> AppResult<Vec<OperationLog>> {
        let logs = self.logs.read().await;
        let logs_vec = logs.clone();
        
        if let Some(limit) = limit {
            let start = logs_vec.len().saturating_sub(limit);
            Ok(logs_vec[start..].to_vec())
        } else {
            Ok(logs_vec)
        }
    }

    pub async fn clear_logs(&self) -> AppResult<()> {
        let mut logs = self.logs.write().await;
        logs.clear();
        drop(logs);
        self.save_logs().await?;
        Ok(())
    }

    // 设置管理
    pub async fn get_settings(&self) -> AppResult<crate::models::Settings> {
        let config = self.config.read().await;
        Ok(config.settings.clone())
    }

    pub async fn update_settings(&self, settings: crate::models::Settings) -> AppResult<()> {
        let mut config = self.config.write().await;
        config.settings = settings;
        drop(config);
        self.save().await?;
        Ok(())
    }
    
    // ==================== 数据安全功能 ====================
    
    /// 创建带时间戳的备份
    pub async fn create_timestamped_backup(&self) -> AppResult<PathBuf> {
        let config = self.config.read().await;
        let data = serde_json::to_string_pretty(&*config)?;
        drop(config);
        
        let timestamp = Local::now().format("%Y%m%d_%H%M%S");
        let backup_dir = self.config_path.parent()
            .ok_or_else(|| AppError::Config("Invalid config path".to_string()))?
            .join("backups");
        
        // 确保备份目录存在
        fs::create_dir_all(&backup_dir)?;
        
        let backup_path = backup_dir.join(format!("accounts_{}.json", timestamp));
        fs::write(&backup_path, data)?;
        
        // 清理旧备份，只保留最近10个
        Self::cleanup_old_backups(&backup_dir, 10)?;
        
        Ok(backup_path)
    }
    
    /// 清理旧备份文件，只保留最近 N 个
    fn cleanup_old_backups(backup_dir: &PathBuf, keep_count: usize) -> std::io::Result<()> {
        let mut backup_files: Vec<_> = fs::read_dir(backup_dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| name.starts_with("accounts_") && name.ends_with(".json"))
                    .unwrap_or(false)
            })
            .collect();
        
        // 按修改时间排序（最新的在前）
        backup_files.sort_by(|a, b| {
            let time_a = fs::metadata(a).and_then(|m| m.modified()).ok();
            let time_b = fs::metadata(b).and_then(|m| m.modified()).ok();
            time_b.cmp(&time_a)
        });
        
        // 删除超出数量的旧备份
        for old_backup in backup_files.iter().skip(keep_count) {
            let _ = fs::remove_file(old_backup);
        }
        
        Ok(())
    }
    
    /// 导出数据到指定路径
    pub async fn export_data(&self, export_path: &PathBuf) -> AppResult<()> {
        let config = self.config.read().await;
        let export_data = serde_json::json!({
            "version": "1.0",
            "exported_at": Local::now().to_rfc3339(),
            "accounts": config.accounts,
            "groups": config.groups,
            "settings": config.settings
        });
        
        let data = serde_json::to_string_pretty(&export_data)?;
        fs::write(export_path, data)?;
        
        Ok(())
    }
    
    /// 从指定路径导入数据
    pub async fn import_data(&self, import_path: &PathBuf, merge: bool) -> AppResult<ImportResult> {
        let data = fs::read_to_string(import_path)?;
        let import_data: serde_json::Value = serde_json::from_str(&data)?;
        
        // 先创建当前数据的备份
        self.create_timestamped_backup().await?;
        
        let mut config = self.config.write().await;
        let mut result = ImportResult::default();
        
        // 导入账号
        if let Some(accounts) = import_data.get("accounts") {
            let imported_accounts: Vec<Account> = serde_json::from_value(accounts.clone())?;
            
            if merge {
                // 合并模式：只添加不存在的账号
                for account in imported_accounts {
                    if !config.accounts.iter().any(|a| a.email == account.email) {
                        config.accounts.push(account);
                        result.accounts_added += 1;
                    } else {
                        result.accounts_skipped += 1;
                    }
                }
            } else {
                // 替换模式：完全替换
                result.accounts_added = imported_accounts.len();
                config.accounts = imported_accounts;
            }
        }
        
        // 导入分组
        if let Some(groups) = import_data.get("groups") {
            let imported_groups: Vec<String> = serde_json::from_value(groups.clone())?;
            for group in imported_groups {
                if !config.groups.contains(&group) {
                    config.groups.push(group);
                    result.groups_added += 1;
                }
            }
        }
        
        drop(config);
        self.save().await?;
        
        Ok(result)
    }
    
    /// 获取备份列表
    pub async fn list_backups(&self) -> AppResult<Vec<BackupInfo>> {
        let backup_dir = self.config_path.parent()
            .ok_or_else(|| AppError::Config("Invalid config path".to_string()))?
            .join("backups");
        
        if !backup_dir.exists() {
            return Ok(Vec::new());
        }
        
        let mut backups: Vec<BackupInfo> = fs::read_dir(&backup_dir)?
            .filter_map(|entry| entry.ok())
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_name()?.to_str()?.to_string();
                if name.starts_with("accounts_") && name.ends_with(".json") {
                    let metadata = fs::metadata(&path).ok()?;
                    Some(BackupInfo {
                        name,
                        path: path.to_string_lossy().to_string(),
                        size: metadata.len(),
                        created_at: metadata.modified().ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| d.as_secs() as i64),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        // 按创建时间降序排列
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        
        Ok(backups)
    }
    
    /// 从备份恢复
    pub async fn restore_from_backup(&self, backup_path: &PathBuf) -> AppResult<()> {
        let data = fs::read_to_string(backup_path)?;
        
        // 验证备份数据
        let config: AppConfig = serde_json::from_str(&data)?;
        
        // 先备份当前数据
        self.create_timestamped_backup().await?;
        
        // 恢复数据
        let mut current_config = self.config.write().await;
        *current_config = config;
        drop(current_config);
        
        self.save().await?;
        
        Ok(())
    }
    
    /// 获取数据目录路径
    pub fn get_data_dir(&self) -> PathBuf {
        self.config_path.parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
    }
    
    /// 更新账户排序顺序（用于拖拽排序）
    pub async fn update_accounts_order(&self, account_ids: Vec<String>) -> AppResult<()> {
        let mut config = self.config.write().await;
        
        // 为每个账户设置新的 sort_order
        for (index, id_str) in account_ids.iter().enumerate() {
            if let Ok(uuid) = Uuid::parse_str(id_str) {
                if let Some(account) = config.accounts.iter_mut().find(|a| a.id == uuid) {
                    account.sort_order = index as i32;
                }
            }
        }
        
        drop(config);
        self.save().await?;
        Ok(())
    }
    
    /// 获取排序后的账户列表
    pub async fn get_sorted_accounts(&self, sort_field: &crate::models::SortField, sort_direction: &crate::models::SortDirection) -> AppResult<Vec<Account>> {
        use crate::models::{SortField, SortDirection};
        
        let config = self.config.read().await;
        let mut accounts = config.accounts.clone();
        
        // 根据排序字段排序
        match sort_field {
            SortField::Email => {
                accounts.sort_by(|a, b| a.email.to_lowercase().cmp(&b.email.to_lowercase()));
            }
            SortField::CreatedAt => {
                accounts.sort_by_key(|a| a.created_at);
            }
            SortField::UsedQuota => {
                accounts.sort_by_key(|a| a.used_quota.unwrap_or(0));
            }
            SortField::RemainingQuota => {
                accounts.sort_by_key(|a| {
                    let total = a.total_quota.unwrap_or(0);
                    let used = a.used_quota.unwrap_or(0);
                    total - used
                });
            }
            SortField::TokenExpiresAt => {
                accounts.sort_by(|a, b| {
                    match (&a.token_expires_at, &b.token_expires_at) {
                        (Some(a_exp), Some(b_exp)) => a_exp.cmp(b_exp),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
            SortField::SubscriptionExpiresAt => {
                accounts.sort_by(|a, b| {
                    match (&a.subscription_expires_at, &b.subscription_expires_at) {
                        (Some(a_exp), Some(b_exp)) => a_exp.cmp(b_exp),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (None, None) => std::cmp::Ordering::Equal,
                    }
                });
            }
            SortField::PlanName => {
                // 定义套餐优先级: Enterprise > Teams > Pro > Trial > Free > None
                let plan_priority = |plan: &Option<String>| -> i32 {
                    match plan.as_ref().map(|s| s.to_lowercase()).as_deref() {
                        Some("enterprise") => 5,
                        Some("teams") => 4,
                        Some("pro") => 3,
                        Some("trial") => 2,
                        Some("free") => 1,
                        _ => 0,
                    }
                };
                accounts.sort_by(|a, b| plan_priority(&b.plan_name).cmp(&plan_priority(&a.plan_name)));
            }
        }
        
        // 根据排序方向反转
        if *sort_direction == SortDirection::Desc && *sort_field != SortField::PlanName {
            accounts.reverse();
        } else if *sort_direction == SortDirection::Asc && *sort_field == SortField::PlanName {
            accounts.reverse();
        }
        
        Ok(accounts)
    }
}

/// 导入结果
#[derive(Debug, Default, serde::Serialize)]
pub struct ImportResult {
    pub accounts_added: usize,
    pub accounts_skipped: usize,
    pub groups_added: usize,
}

/// 备份信息
#[derive(Debug, serde::Serialize)]
pub struct BackupInfo {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub created_at: Option<i64>,
}
