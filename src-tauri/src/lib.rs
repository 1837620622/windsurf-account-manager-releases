mod models;
mod repository;
mod services;
mod commands;
mod utils;

use repository::DataStore;
use commands::{AutoResetStore, ResetRecordStore};
use std::sync::Arc;
use tauri::Manager;

/// 在 Windows Release 模式下弹出错误对话框（因为没有控制台窗口）
fn show_fatal_error(msg: &str) {
    #[cfg(target_os = "windows")]
    {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        let wide_msg: Vec<u16> = OsStr::new(msg).encode_wide().chain(std::iter::once(0)).collect();
        let wide_title: Vec<u16> = OsStr::new("启动失败 - Windsurf Account Manager").encode_wide().chain(std::iter::once(0)).collect();
        unsafe {
            winapi::um::winuser::MessageBoxW(
                std::ptr::null_mut(),
                wide_msg.as_ptr(),
                wide_title.as_ptr(),
                winapi::um::winuser::MB_ICONERROR,
            );
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        eprintln!("[FATAL] {}", msg);
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // 初始化数据存储（失败时弹出错误提示而非静默崩溃）
            let store = match DataStore::new(app.handle()) {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    let msg = format!("数据存储初始化失败，请检查磁盘权限或重新安装。\n\n详细错误: {}", e);
                    show_fatal_error(&msg);
                    return Err(e.into());
                }
            };
            
            // 将数据存储注入到应用状态
            app.manage(store.clone());
            
            // 初始化自动重置配置存储
            let auto_reset_store = match AutoResetStore::new(app.handle()) {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    let msg = format!("自动重置配置初始化失败。\n\n详细错误: {}", e);
                    show_fatal_error(&msg);
                    return Err(e.into());
                }
            };
            app.manage(auto_reset_store);
            
            // 初始化重置记录存储
            let reset_record_store = match ResetRecordStore::new(app.handle()) {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    let msg = format!("重置记录存储初始化失败。\n\n详细错误: {}", e);
                    show_fatal_error(&msg);
                    return Err(e.into());
                }
            };
            app.manage(reset_record_store);
            
            // 初始化代理配置
            let store_for_proxy = store.clone();
            tauri::async_runtime::spawn(async move {
                if let Ok(settings) = store_for_proxy.get_settings().await {
                    if settings.proxy_enabled || settings.proxy_url.is_some() {
                        println!("[Init] Loading proxy config: enabled={}, url={:?}", 
                            settings.proxy_enabled, settings.proxy_url);
                        services::update_proxy_config(
                            settings.proxy_enabled,
                            settings.proxy_url
                        );
                    }
                }
            });
            
            // 获取版本号并设置窗口标题
            let version = app.package_info().version.to_string();
            if let Some(window) = app.get_webview_window("main") {
                let title = format!("windsurf-account-manager-simple v{}", version);
                window.set_title(&title).ok();
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // 账号管理命令
            commands::add_account,
            commands::add_account_by_refresh_token,
            commands::get_all_accounts,
            commands::get_account,
            commands::update_account,
            commands::delete_account,
            commands::delete_accounts_batch,
            commands::search_accounts,
            commands::filter_accounts_by_group,
            commands::filter_accounts_by_tags,
            
            // API操作命令
            commands::login_account,
            commands::refresh_token,
            commands::get_plan_status,
            commands::reset_credits,
            commands::update_seats,
            commands::get_billing,
            commands::update_plan,
            commands::cancel_subscription,
            commands::resume_subscription,
            commands::get_account_info,
            commands::get_current_user,
            commands::batch_reset_credits,
            commands::batch_refresh_tokens,
            commands::get_team_credit_entries,
            commands::get_trial_payment_link,
            commands::get_team_config,
            commands::update_team_config,
            commands::get_cascade_model_configs,
            commands::get_command_model_configs,
            commands::get_team_organizational_controls,
            commands::upsert_team_organizational_controls,
            commands::get_available_mcp_plugins,
            commands::delete_windsurf_user,
            
            // 支付相关命令
            commands::generate_virtual_card,
            commands::open_payment_window,
            commands::inject_card_info,
            commands::validate_card_number,
            commands::auto_fill_payment_form,
            commands::get_trial_payment_link_enhanced,
            commands::open_external_link,
            commands::open_external_link_incognito,
            commands::inject_auto_submit_script,
            commands::close_payment_window,
            commands::get_success_bins,
            commands::add_success_bin,
            commands::clear_success_bins,
            commands::get_random_success_bin,
            commands::reset_test_mode_progress,
            commands::get_test_mode_progress,
            
            // Protobuf解析API命令（返回解析后的数据）
            commands::get_current_user_parsed,
            commands::get_billing_parsed,
            commands::batch_get_users_parsed,

            // Analytics 分析命令
            commands::get_account_analytics,

            // 设置管理命令
            commands::get_settings,
            commands::update_settings,
            commands::get_groups,
            commands::add_group,
            commands::delete_group,
            commands::rename_group,
            commands::get_tags,
            commands::add_tag,
            commands::update_tag,
            commands::delete_tag,
            commands::batch_update_account_tags,
            commands::get_logs,
            commands::clear_logs,
            commands::get_stats,
            commands::export_data,
            
            // 切号相关命令
            commands::switch_account,
            commands::reset_machine_id,
            commands::check_admin_privileges,
            
            // Windsurf信息命令
            commands::get_current_windsurf_info,
            
            // 应用信息命令
            commands::get_app_version,
            commands::get_app_title,
            commands::reset_http_client,
            
            // 无感换号补丁命令
            commands::get_windsurf_path,
            commands::apply_seamless_patch,
            commands::restore_seamless_patch,
            commands::check_patch_status,
            commands::validate_windsurf_path,
            
            // 数据备份命令
            commands::create_backup,
            commands::list_backups,
            commands::restore_backup,
            commands::export_data_to_file,
            commands::import_data_from_file,
            commands::get_data_directory,
            
            // 排序命令
            commands::get_sorted_accounts,
            commands::update_accounts_order,
            commands::update_sort_config,
            commands::get_sort_config,
            
            // 团队管理命令
            commands::get_team_members,
            commands::invite_team_members,
            commands::remove_team_member,
            commands::revoke_invitation,
            commands::get_pending_invitations,
            commands::get_my_pending_invitation,
            commands::accept_invitation,
            commands::reject_invitation,
            commands::request_team_access,
            commands::approve_team_join_request,
            // 自动充值管理
            commands::get_credit_top_up_settings,
            commands::update_credit_top_up_settings,
            // 成员权限管理
            commands::update_codeium_access,
            commands::add_user_role,
            commands::remove_user_role,
            commands::transfer_subscription,
            
            // 自动重置命令
            commands::get_auto_reset_configs,
            commands::add_auto_reset_config,
            commands::update_auto_reset_config,
            commands::delete_auto_reset_config,
            commands::check_and_auto_reset,
            commands::force_reset_config,
            commands::get_reset_records,
            commands::get_reset_stats,
            commands::clear_reset_records,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
