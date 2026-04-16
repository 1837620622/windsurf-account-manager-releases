use crate::repository::DataStore;
use crate::utils::errors::{AppError, AppResult};
use chrono::Utc;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::State;
use uuid::Uuid;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use winreg::{RegKey, enums::{HKEY_LOCAL_MACHINE, KEY_ALL_ACCESS}};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Debug, Serialize, Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
    expires_in: String,
    token_type: String,
    refresh_token: String,
    id_token: String,
    user_id: String,
    project_id: String,
}

/// 使用refresh_token获取新的access_token
async fn refresh_access_token(refresh_token: &str) -> AppResult<GoogleTokenResponse> {
    // 使用专门用于 googleapis 的 HTTP 客户端（支持代理）
    let client = crate::services::get_google_api_client();
    
    // Google Token API
    let url = "https://securetoken.googleapis.com/v1/token";
    let api_key = "AIzaSyDsOl-1XpT5err0Tcnx8FFod1H8gVGIycY"; // Windsurf Firebase API Key
    
    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
    ];
    
    let response = client
        .post(&format!("{}?key={}", url, api_key))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Failed to refresh token: {}", error_text);
        return Err(AppError::ApiRequest(format!("Failed to refresh token: {}", error_text)));
    }
    
    let token_response = response.json::<GoogleTokenResponse>().await
        .map_err(|e| AppError::Network(e.to_string()))?;
    
    Ok(token_response)
}

/// 触发Windsurf回调URL以完成登录（传入 Firebase ID Token，与浏览器登录一致）
async fn trigger_windsurf_callback(firebase_id_token: &str) -> AppResult<()> {
    // 生成state参数（与浏览器登录流程一致）
    let state = Uuid::new_v4().to_string();
    
    // 构建回调URL（与 Auth0 重定向格式完全一致）
    // windsurf://codeium.windsurf#access_token=<firebase_id_token>&state=<state>&token_type=Bearer
    let params = [
        ("access_token", firebase_id_token),
        ("state", &state),
        ("token_type", "Bearer"),
    ];
    
    let fragment = serde_urlencoded::to_string(&params)
        .map_err(|e| AppError::ApiRequest(format!("Failed to encode URL parameters: {}", e)))?;
    
    let callback_url = format!("windsurf://codeium.windsurf#{}", fragment);
    
    info!("Triggering Windsurf callback: {}", callback_url);
    
    // 使用系统默认程序打开URL（触发Windsurf处理）
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        // 使用 PowerShell 的 Start-Process 来正确处理包含特殊字符的 URL
        Command::new("powershell")
            .args(&["-NoProfile", "-Command", &format!("Start-Process '{}'", callback_url)])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| AppError::FileOperation(format!("Failed to open URL: {}", e)))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(&callback_url)
            .spawn()
            .map_err(|e| AppError::FileOperation(format!("Failed to open URL: {}", e)))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(&callback_url)
            .spawn()
            .map_err(|e| AppError::FileOperation(format!("Failed to open URL: {}", e)))?;
    }
    
    info!("Successfully triggered Windsurf callback");
    Ok(())
}


/// 一键切换账号命令（使用Firebase ID Token + 回调URL登录，完全复刻浏览器登录流程）
#[tauri::command]
pub async fn switch_account(
    id: String,
    data_store: State<'_, Arc<DataStore>>,
) -> Result<Value, String> {
    info!("Switching account: {}", id);
    
    let account_id = Uuid::parse_str(&id).map_err(|e| e.to_string())?;
    
    // 获取账号信息
    let account = data_store
        .get_account(account_id)
        .await
        .map_err(|e| e.to_string())?;
    
    // 检查是否有refresh_token
    if account.refresh_token.is_none() || account.refresh_token.as_ref().unwrap().is_empty() {
        return Ok(json!({
            "success": false,
            "error": "账号没有refresh_token，请先登录"
        }));
    }
    
    let refresh_token = account.refresh_token.unwrap();
    
    // Step 1: 始终刷新token以获取 Firebase ID Token（id_token）
    // 新版 Windsurf 的 registerUser 要求 firebaseIdToken，不再接受 GetOneTimeAuthToken
    info!("Refreshing token to get Firebase ID Token...");
    let token_response = match refresh_access_token(&refresh_token).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("Failed to refresh token: {:?}", e);
            return Ok(json!({
                "success": false,
                "error": format!("刷新token失败: {}", e)
            }));
        }
    };
    
    let firebase_id_token = token_response.id_token;
    let access_token = token_response.access_token;
    let expires_in = token_response.expires_in;
    
    if firebase_id_token.is_empty() {
        return Ok(json!({
            "success": false,
            "error": "获取Firebase ID Token为空，refresh_token可能已失效"
        }));
    }
    
    // Step 2: 尝试重置机器ID（可能需要管理员权限）
    info!("Attempting to reset machine ID...");
    let reset_result = reset_machine_id_internal().await;
    let machine_id_reset = match reset_result {
        Ok(_) => {
            info!("Machine ID reset successful");
            true
        },
        Err(e) => {
            warn!("Failed to reset machine ID: {:?}", e);
            warn!("重置机器ID失败，可能需要管理员权限。但切换账号仍可继续。");
            false
        }
    };
    
    // Step 3: 用 Firebase ID Token 触发 Windsurf 回调URL（与浏览器登录流程完全一致）
    info!("Triggering Windsurf callback with Firebase ID Token...");
    if let Err(e) = trigger_windsurf_callback(&firebase_id_token).await {
        error!("Failed to trigger callback: {:?}", e);
        return Ok(json!({
            "success": false,
            "error": format!("触发Windsurf登录失败: {}", e)
        }));
    }
    
    // 更新账号的token缓存信息
    let expires_at = Utc::now() + chrono::Duration::seconds(expires_in.parse::<i64>().unwrap_or(3600));
    if let Err(e) = data_store.update_account_token(
        account_id,
        access_token.clone(),
        expires_at
    ).await {
        error!("Failed to update account token: {:?}", e);
    }
    
    info!("Successfully triggered Windsurf login for account");
    
    Ok(json!({
        "success": true,
        "message": if machine_id_reset {
            "已成功触发Windsurf登录并重置机器ID"
        } else {
            "已成功触发Windsurf登录（未重置机器ID，可能需要管理员权限）"
        },
        "auth_token": firebase_id_token,
        "machine_id_reset": machine_id_reset
    }))
}

/// 内部重置机器ID函数
async fn reset_machine_id_internal() -> AppResult<()> {
    use std::fs;
    use rand::Rng;
    
    // 生成新的机器ID（符合VSCode格式）
    let mut rng = rand::thread_rng();
    
    // machineId: 64位hex字符串（256位）
    let machine_bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    let new_machine_id = hex::encode(&machine_bytes);
    
    // macMachineId: 32位hex字符串（MD5格式）
    let new_mac_machine_id = format!("{:032x}", rng.gen::<u128>());
    
    // sqmId: UUID格式，不带括号
    let new_sqm_id = Uuid::new_v4().to_string().to_uppercase();
    
    // devDeviceId: 标准UUID格式
    let new_device_id = Uuid::new_v4().to_string().to_lowercase();
    
    // 更新storage.json
    let mut storage_path = directories::BaseDirs::new()
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("C:/Users/Default/AppData/Roaming"));
    storage_path.push("Windsurf");
    storage_path.push("User");
    storage_path.push("globalStorage");
    storage_path.push("storage.json");
    
    if storage_path.exists() {
        let content = fs::read_to_string(&storage_path)
            .map_err(|e| AppError::FileOperation(format!("Failed to read storage.json: {}", e)))?;
        let mut storage: Value = serde_json::from_str(&content)
            .map_err(AppError::Serialization)?;
        
        storage["telemetry.machineId"] = json!(new_machine_id);
        storage["telemetry.macMachineId"] = json!(new_mac_machine_id);
        storage["telemetry.sqmId"] = json!(new_sqm_id);
        storage["telemetry.devDeviceId"] = json!(new_device_id);
        
        let updated = serde_json::to_string_pretty(&storage)
            .map_err(AppError::Serialization)?;
        fs::write(&storage_path, updated)
            .map_err(|e| AppError::FileOperation(format!("Failed to write storage.json: {}. 可能需要管理员权限", e)))?;
        
        info!("Updated storage.json with new machine IDs");
    } else {
        warn!("storage.json not found at {:?}", storage_path);
    }
    
    // Windows特定：更新注册表（程序启动时已要求管理员权限）
    #[cfg(target_os = "windows")]
    {
        // 只更新 HKEY_LOCAL_MACHINE 下的 Cryptography MachineGuid（需要管理员权限）
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        
        // 生成新的GUID（不带大括号的格式）
        let new_machine_guid = Uuid::new_v4().to_string().to_uppercase();
        
        match hklm.open_subkey_with_flags(
            "SOFTWARE\\Microsoft\\Cryptography",
            KEY_ALL_ACCESS
        ) {
            Ok(crypto_key) => {
                match crypto_key.set_value("MachineGuid", &new_machine_guid) {
                    Ok(()) => {
                        info!("Updated HKLM\\SOFTWARE\\Microsoft\\Cryptography\\MachineGuid to: {}", new_machine_guid);
                        Ok(())
                    }
                    Err(e) => {
                        let msg = format!("Failed to update MachineGuid: {}. 确保以管理员权限运行", e);
                        error!("{}", msg);
                        Err(AppError::FileOperation(msg))
                    }
                }
            }
            Err(e) => {
                let msg = format!("Failed to open HKLM\\SOFTWARE\\Microsoft\\Cryptography: {}. 需要管理员权限", e);
                error!("{}", msg);
                Err(AppError::FileOperation(msg))
            }
        }
    }
    
    // macOS特定：尝试重置系统级机器标识
    #[cfg(target_os = "macos")]
    {
        // macOS 的硬件 UUID 无法修改，但可以尝试重置一些软件级别的标识
        // 注意：某些操作可能需要 sudo 权限
        
        // 尝试删除 Windsurf 的本地缓存标识文件
        let home = std::env::var("HOME").unwrap_or_default();
        let cache_paths = vec![
            format!("{}/.config/Windsurf/machineid", home),
            format!("{}/Library/Application Support/Windsurf/.installerId", home),
        ];
        
        for cache_path in cache_paths {
            let path = PathBuf::from(&cache_path);
            if path.exists() {
                match fs::remove_file(&path) {
                    Ok(()) => info!("Removed cache file: {}", cache_path),
                    Err(e) => warn!("Failed to remove {}: {}", cache_path, e),
                }
            }
        }
        
        // 尝试重置系统级 machine-id（需要 sudo 权限）
        // /var/lib/dbus/machine-id 在 macOS 上通常不存在
        // 但某些应用可能会读取 IOPlatformUUID
        
        info!("macOS machine ID reset completed (software level only)");
        Ok(())
    }
    
    // Linux特定：尝试重置 /etc/machine-id 和 /var/lib/dbus/machine-id
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        
        // 生成新的 machine-id（32位hex字符串）
        let new_linux_machine_id = format!("{:032x}", rand::thread_rng().gen::<u128>());
        
        // 尝试更新 /etc/machine-id（需要 root 权限）
        let etc_machine_id = PathBuf::from("/etc/machine-id");
        if etc_machine_id.exists() {
            match fs::write(&etc_machine_id, format!("{}\n", new_linux_machine_id)) {
                Ok(()) => {
                    info!("Updated /etc/machine-id to: {}", new_linux_machine_id);
                }
                Err(e) => {
                    warn!("Failed to update /etc/machine-id: {}. 需要 sudo 权限", e);
                    // 尝试使用 sudo
                    let result = Command::new("sudo")
                        .args(["bash", "-c", &format!("echo '{}' > /etc/machine-id", new_linux_machine_id)])
                        .output();
                    match result {
                        Ok(output) if output.status.success() => {
                            info!("Updated /etc/machine-id via sudo");
                        }
                        _ => {
                            warn!("Could not update /etc/machine-id even with sudo");
                        }
                    }
                }
            }
        }
        
        // 尝试更新 /var/lib/dbus/machine-id（通常是 /etc/machine-id 的符号链接）
        let dbus_machine_id = PathBuf::from("/var/lib/dbus/machine-id");
        if dbus_machine_id.exists() && !dbus_machine_id.is_symlink() {
            match fs::write(&dbus_machine_id, format!("{}\n", new_linux_machine_id)) {
                Ok(()) => {
                    info!("Updated /var/lib/dbus/machine-id");
                }
                Err(e) => {
                    warn!("Failed to update /var/lib/dbus/machine-id: {}", e);
                }
            }
        }
        
        // 尝试删除 Windsurf 的本地缓存标识文件
        let home = std::env::var("HOME").unwrap_or_default();
        let cache_paths = vec![
            format!("{}/.config/Windsurf/machineid", home),
            format!("{}/.local/share/Windsurf/.installerId", home),
        ];
        
        for cache_path in cache_paths {
            let path = PathBuf::from(&cache_path);
            if path.exists() {
                match fs::remove_file(&path) {
                    Ok(()) => info!("Removed cache file: {}", cache_path),
                    Err(e) => warn!("Failed to remove {}: {}", cache_path, e),
                }
            }
        }
        
        info!("Linux machine ID reset completed");
        Ok(())
    }
}

/// 重置机器ID命令（供前端调用）
#[tauri::command]
pub async fn reset_machine_id() -> Result<Value, String> {
    match reset_machine_id_internal().await {
        Ok(()) => Ok(json!({
            "success": true,
            "message": "机器ID重置成功"
        })),
        Err(e) => Ok(json!({
            "success": false,
            "message": format!("机器ID重置失败: {}", e)
        }))
    }
}

#[cfg(target_os = "windows")]
pub fn is_elevated() -> bool {
    use std::ptr;
    use winapi::um::securitybaseapi::GetTokenInformation;
    use winapi::um::winnt::{TokenElevation, HANDLE, TOKEN_ELEVATION, TOKEN_QUERY};
    use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
    use winapi::um::handleapi::CloseHandle;
    
    unsafe {
        let mut token_handle: HANDLE = ptr::null_mut();
        
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY,
            &mut token_handle
        ) == 0 {
            return false;
        }
        
        let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
        let mut size = 0u32;
        
        let result = GetTokenInformation(
            token_handle,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size
        );
        
        CloseHandle(token_handle);
        
        result != 0 && elevation.TokenIsElevated != 0
    }
}

/// 检查应用程序是否以管理员/root权限运行
#[tauri::command]
pub async fn check_admin_privileges() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        Ok(is_elevated())
    }
    
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Unix系统：检查 euid 是否为 0 (root)
        Ok(is_root())
    }
}

/// 检查是否以 root 权限运行 (Unix)
#[cfg(any(target_os = "macos", target_os = "linux"))]
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}
