use crate::utils::errors::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct WindsurfCurrentInfo {
    pub email: Option<String>,
    pub name: Option<String>,
    pub api_key: Option<String>,
    pub plan_name: Option<String>,
    pub team_id: Option<String>,
    pub version: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct WindsurfAuthStatus {
    name: Option<String>,
    #[serde(rename = "apiKey")]
    api_key: Option<String>,
    email: Option<String>,
    #[serde(rename = "teamId")]
    team_id: Option<String>,
    #[serde(rename = "planName")]
    plan_name: Option<String>,
}

/// 获取当前Windsurf账号信息
#[tauri::command]
pub fn get_current_windsurf_info() -> Result<WindsurfCurrentInfo, AppError> {
    // 获取state.vscdb路径
    let appdata = std::env::var("APPDATA")
        .map_err(|e| AppError::Config(format!("Failed to get APPDATA: {}", e)))?;
    let db_path = PathBuf::from(appdata)
        .join("Windsurf")
        .join("User")
        .join("globalStorage")
        .join("state.vscdb");
    
    if !db_path.exists() {
        return Ok(WindsurfCurrentInfo {
            email: None,
            name: None,
            api_key: None,
            plan_name: None,
            team_id: None,
            version: None,
            is_active: false,
        });
    }
    
    // 连接数据库
    let connection = rusqlite::Connection::open(&db_path)
        .map_err(|e| AppError::Database(format!("Failed to open state.vscdb: {}", e)))?;
    
    // 读取账号信息
    let auth_status = connection
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["windsurfAuthStatus"],
            |row| -> Result<String, rusqlite::Error> {
                row.get(0)
            },
        )
        .ok();
    
    // 读取版本信息
    let version = connection
        .query_row(
            "SELECT value FROM ItemTable WHERE key = ?",
            ["windsurfChangelog/lastVersion"],
            |row| -> Result<String, rusqlite::Error> {
                row.get(0)
            },
        )
        .ok();
    
    // 解析账号信息
    let mut info = WindsurfCurrentInfo {
        email: None,
        name: None,
        api_key: None,
        plan_name: None,
        team_id: None,
        version,
        is_active: false,
    };
    
    if let Some(auth_json) = auth_status {
        if let Ok(auth_data) = serde_json::from_str::<WindsurfAuthStatus>(&auth_json) {
            info.email = auth_data.email;
            info.name = auth_data.name;
            info.api_key = auth_data.api_key;
            info.plan_name = auth_data.plan_name;
            info.team_id = auth_data.team_id;
            info.is_active = true;
        }
    }
    
    Ok(info)
}
