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
    let api_key = "AIzaSyBPFmef6bkwMJAYP0sJZAi4k5XP1lXJXuY"; // Firebase API Key
    
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

/// 序列化Protobuf字符串（field 1, wire type 2）
fn serialize_protobuf_string(value: &str) -> Vec<u8> {
    if value.is_empty() {
        return vec![];
    }
    
    let value_bytes = value.as_bytes();
    let value_length = value_bytes.len();
    
    // Field 1, wire type 2 (length-delimited): (1 << 3) | 2 = 0x0A
    let mut result = vec![0x0A];
    
    // Encode length as varint
    let mut length = value_length;
    while length > 127 {
        result.push((length as u8 & 0x7F) | 0x80);
        length >>= 7;
    }
    result.push(length as u8 & 0x7F);
    
    // Append value bytes
    result.extend_from_slice(value_bytes);
    result
}

/// 反序列化Protobuf响应获取auth_token
fn deserialize_protobuf_response(data: &[u8]) -> Option<String> {
    if data.len() < 2 {
        return None;
    }
    
    let mut pos = 0;
    while pos < data.len() {
        // Read field tag
        let tag = data[pos];
        pos += 1;
        
        // Get wire type (low 3 bits)
        let wire_type = tag & 0x07;
        let field_number = tag >> 3;
        
        // If it's length-delimited type (wire_type = 2)
        if wire_type == 2 {
            // Read varint length
            let mut length = 0;
            let mut shift = 0;
            while pos < data.len() {
                let byte = data[pos];
                pos += 1;
                length |= ((byte & 0x7F) as usize) << shift;
                if byte & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            
            // Read string content
            if pos + length <= data.len() {
                if let Ok(value) = std::str::from_utf8(&data[pos..pos + length]) {
                    // auth_token is typically field 1
                    if field_number == 1 && !value.is_empty() {
                        return Some(value.to_string());
                    }
                }
                pos += length;
            }
        } else if wire_type == 0 {
            // Skip varint field
            while pos < data.len() {
                if data[pos] & 0x80 == 0 {
                    pos += 1;
                    break;
                }
                pos += 1;
            }
        } else {
            // Skip other types
            break;
        }
    }
    
    None
}

/// 使用access_token获取auth_token
async fn get_auth_token(access_token: &str) -> AppResult<String> {
    let client = reqwest::Client::new();
    
    // Windsurf GetOneTimeAuthToken endpoint
    let url = "https://web-backend.windsurf.com/exa.seat_management_pb.SeatManagementService/GetOneTimeAuthToken";
    
    // Serialize request as Protobuf
    let request_data = serialize_protobuf_string(access_token);
    
    let response = client
        .post(url)
        .header("Content-Type", "application/proto")
        .header("Accept", "application/proto")
        .header("User-Agent", "Windsurf/1.4.2")
        .body(request_data)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        error!("Failed to get auth token: {}", error_text);
        return Err(AppError::ApiRequest(format!("Failed to get auth token: {}", error_text)));
    }
    
    // Deserialize response
    let response_bytes = response.bytes().await
        .map_err(|e| AppError::Network(e.to_string()))?;
    
    let auth_token = deserialize_protobuf_response(&response_bytes)
        .ok_or_else(|| AppError::ApiRequest("Failed to parse auth token from response".to_string()))?;
    
    info!("Successfully obtained auth token");
    Ok(auth_token)
}

/// 触发Windsurf回调URL以完成登录
async fn trigger_windsurf_callback(auth_token: &str) -> AppResult<()> {
    // 生成state参数
    let state = Uuid::new_v4().to_string();
    
    // 构建回调URL
    // windsurf://codeium.windsurf#access_token=<auth_token>&state=<state>&token_type=Bearer
    let params = [
        ("access_token", auth_token),
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


/// 一键切换账号命令（简化版：使用回调URL登录）
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
    
    // Step 1: 检查本地token是否有效
    let (access_token, expires_in) = if let (Some(token), Some(expires_at)) = (&account.token, &account.token_expires_at) {
        // 检查token是否还有至少5分钟有效期
        let now = Utc::now();
        let buffer = chrono::Duration::minutes(5);
        if *expires_at > now + buffer {
            info!("Using cached access token, expires at: {}", expires_at);
            let remaining_seconds = (*expires_at - now).num_seconds();
            (token.clone(), remaining_seconds.to_string())
        } else {
            info!("Token expired or expiring soon, refreshing...");
            let token_response = match refresh_access_token(&refresh_token).await {
                Ok(resp) => resp,
                Err(e) => {
                    error!("Failed to refresh access token: {:?}", e);
                    return Ok(json!({
                        "success": false,
                        "error": format!("获取access_token失败: {}", e)
                    }));
                }
            };
            (token_response.access_token, token_response.expires_in)
        }
    } else {
        // 没有本地token，需要刷新
        info!("No cached token, refreshing access token...");
        let token_response = match refresh_access_token(&refresh_token).await {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to refresh access token: {:?}", e);
                return Ok(json!({
                    "success": false,
                    "error": format!("获取access_token失败: {}", e)
                }));
            }
        };
        (token_response.access_token, token_response.expires_in)
    };
    
    // Step 2: 获取auth_token
    info!("Getting auth token...");
    let auth_token = match get_auth_token(&access_token).await {
        Ok(token) => token,
        Err(e) => {
            error!("Failed to get auth token: {:?}", e);
            return Ok(json!({
                "success": false,
                "error": format!("获取auth_token失败: {}", e)
            }));
        }
    };
    
    // Step 3: 尝试重置机器ID（可能需要管理员权限）
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
    
    // Step 4: 触发Windsurf回调URL以自动登录
    info!("Triggering Windsurf callback...");
    if let Err(e) = trigger_windsurf_callback(&auth_token).await {
        error!("Failed to trigger callback: {:?}", e);
        return Ok(json!({
            "success": false,
            "error": format!("触发Windsurf登录失败: {}", e)
        }));
    }
    
    // 更新账号的token信息
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
        "auth_token": auth_token,
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
