use tauri::command;
use std::path::PathBuf;

/// 获取 extension.js 相对路径（跨平台）
fn get_extension_js_relative_path() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        // macOS: Windsurf.app/Contents/Resources/app/extensions/windsurf/dist/extension.js
        PathBuf::from("Contents")
            .join("Resources")
            .join("app")
            .join("extensions")
            .join("windsurf")
            .join("dist")
            .join("extension.js")
    }
    #[cfg(not(target_os = "macos"))]
    {
        // Windows/Linux: resources/app/extensions/windsurf/dist/extension.js
        PathBuf::from("resources")
            .join("app")
            .join("extensions")
            .join("windsurf")
            .join("dist")
            .join("extension.js")
    }
}

/// 验证Windsurf路径是否有效
#[command]
pub async fn validate_windsurf_path(path: String) -> Result<bool, String> {
    let windsurf_path = PathBuf::from(&path);
    
    // 检查extension.js文件是否存在（跨平台）
    let extension_file = windsurf_path.join(get_extension_js_relative_path());
    
    Ok(extension_file.exists())
}
