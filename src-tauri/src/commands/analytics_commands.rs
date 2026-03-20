use crate::models::{
    AnalyticsData, DailyCascadeLinesCount, ToolUsageEntry, ModelUsageEntry, ModelUsageSummary, AnalyticsSummary,
    PercentCodeWritten, CompletionStatistics, CompletionByDay, CompletionByLanguage,
    ChatStats, ChatStatsByDay, ChatStatsByModel, CustomQueryResponse, CustomQueryResponseItem,
};
use crate::repository::DataStore;
use crate::services::{AnalyticsService, WindsurfService, proto_parser};
use serde_json::Value;
use std::sync::Arc;
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;
use chrono::{Utc, Duration, DateTime};

/// 获取账户的使用分析数据（最近30天）
#[tauri::command]
pub async fn get_account_analytics(
    id: String,
    store: State<'_, Arc<DataStore>>,
) -> Result<AnalyticsData, String> {
    let uuid = Uuid::parse_str(&id).map_err(|e| e.to_string())?;

    // 获取账号信息
    let mut account = store.get_account(uuid)
        .await
        .map_err(|e| e.to_string())?;

    // 确保有有效的Token
    super::api_commands::ensure_valid_token(&store, &mut account, uuid).await?;

    let token = account.token.clone().ok_or("No token available")?;

    // 获取或更新 Windsurf API Key，同时检查是否是团队账户
    let (windsurf_api_key, is_team) = if let Some(api_key) = &account.windsurf_api_key {
        println!("[get_account_analytics] Using cached Windsurf API Key: {}", api_key);
        // 对于缓存的 API Key，需要再次获取用户信息来判断是否是团队账户
        let windsurf_service = WindsurfService::new();
        let user_info_result = windsurf_service.get_current_user(&token)
            .await
            .map_err(|e| format!("Failed to get current user: {}", e))?;
        
        // 检查是否有 team_id (field 7 in user message)
        let user_data = user_info_result
            .get("parsed_data")
            .and_then(|data| data.get("subMesssage_1"));
        
        // 打印用户数据结构以便调试
        if let Some(user) = user_data {
            println!("[get_account_analytics] User data keys: {:?}", user.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        }
        
        let team_id = user_data
            .and_then(|user| user.get("string_7"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let has_team = !team_id.is_empty();
        println!("[get_account_analytics] Team ID: '{}', Is team account: {}", team_id, has_team);
        (api_key.clone(), has_team)
    } else {
        println!("[get_account_analytics] Fetching Windsurf API Key from GetCurrentUser API");

        // 调用 GetCurrentUser API 获取用户的 Windsurf API Key
        let windsurf_service = WindsurfService::new();
        let user_info_result = windsurf_service.get_current_user(&token)
            .await
            .map_err(|e| format!("Failed to get current user: {}", e))?;

        // 从响应中提取 API Key
        let api_key = user_info_result
            .get("parsed_data")
            .and_then(|data| data.get("subMesssage_1"))
            .and_then(|user| user.get("string_1"))
            .and_then(|v| v.as_str())
            .ok_or("Failed to extract Windsurf API Key from user info")?
            .to_string();

        // 检查是否有 team_id (field 7 in user message)
        let user_data = user_info_result
            .get("parsed_data")
            .and_then(|data| data.get("subMesssage_1"));
        
        // 打印用户数据结构以便调试
        if let Some(user) = user_data {
            println!("[get_account_analytics] User data keys: {:?}", user.as_object().map(|o| o.keys().collect::<Vec<_>>()));
        }
        
        let team_id = user_data
            .and_then(|user| user.get("string_7"))
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        let has_team = !team_id.is_empty();

        println!("[get_account_analytics] Got Windsurf API Key: {}", api_key);
        println!("[get_account_analytics] Team ID: '{}', Is team account: {}", team_id, has_team);

        // 更新账户信息，保存 API Key
        let mut updated_account = account.clone();
        updated_account.windsurf_api_key = Some(api_key.clone());
        store.update_account(updated_account)
            .await
            .map_err(|e| format!("Failed to update account with API key: {}", e))?;

        (api_key, has_team)
    };

    // 计算时间范围：最近30天
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(30);

    let start_timestamp = start_time.timestamp();
    let end_timestamp = end_time.timestamp();

    println!("[get_account_analytics] Fetching analytics for account: {}", account.email);
    println!("[get_account_analytics] Time range (30 days): {} to {}", start_time.format("%Y-%m-%d"), end_time.format("%Y-%m-%d"));
    println!("[get_account_analytics] Using Windsurf API Key: {}", windsurf_api_key);

    // 调用 GetAnalytics API
    let analytics_service = AnalyticsService::new();
    
    // 三层降级策略：
    // 1. 完整请求（包含所有查询类型）
    // 2. 非团队请求（移除 percent_code_written）
    // 3. 仅 cascade 请求（最小化请求体）
    
    let response_result = analytics_service.get_analytics(&token, start_timestamp, end_timestamp, is_team).await;
    
    let response_body = match response_result {
        Ok(body) => Some(body),
        Err(e) if is_team => {
            // 第一次降级：团队请求失败，尝试不带 percent_code_written
            println!("[get_account_analytics] Team request failed, retrying without percent_code_written: {}", e);
            match analytics_service.get_analytics(&token, start_timestamp, end_timestamp, false).await {
                Ok(body) => Some(body),
                Err(e) => {
                    // 第二次降级：尝试仅请求 cascade 数据
                    println!("[get_account_analytics] Full request failed, trying cascade-only: {}", e);
                    match analytics_service.get_analytics_cascade_only(&token, start_timestamp, end_timestamp).await {
                        Ok(body) => Some(body),
                        Err(e) => {
                            // 第三次降级：无时间戳请求
                            println!("[get_account_analytics] Cascade-only failed, trying no-timestamp: {}", e);
                            match analytics_service.get_analytics_no_timestamp(&token).await {
                                Ok(body) => Some(body),
                                Err(e) => {
                                    println!("[get_account_analytics] All requests failed, returning empty data: {}", e);
                                    None
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            // 第一次降级：尝试仅请求 cascade 数据
            println!("[get_account_analytics] Full request failed, trying cascade-only: {}", e);
            match analytics_service.get_analytics_cascade_only(&token, start_timestamp, end_timestamp).await {
                Ok(body) => Some(body),
                Err(e) => {
                    // 第二次降级：无时间戳请求（模仿官网）
                    println!("[get_account_analytics] Cascade-only failed, trying no-timestamp: {}", e);
                    match analytics_service.get_analytics_no_timestamp(&token).await {
                        Ok(body) => Some(body),
                        Err(e) => {
                            println!("[get_account_analytics] All requests failed, returning empty data: {}", e);
                            None
                        }
                    }
                }
            }
        },
    };

    // 如果 API 调用失败，返回空数据而不是错误
    let analytics_data = if let Some(body) = response_body {
        // 解析响应
        match proto_parser::parse_get_analytics_response(&body) {
            Ok(parsed_response) => {
                // 提取并转换数据
                extract_analytics_data(&parsed_response).unwrap_or_default()
            },
            Err(e) => {
                println!("[get_account_analytics] Failed to parse response, returning empty data: {}", e);
                AnalyticsData::default()
            }
        }
    } else {
        AnalyticsData::default()
    };

    Ok(analytics_data)
}

/// 从解析后的 Protobuf 数据中提取分析数据
fn extract_analytics_data(parsed: &Value) -> Result<AnalyticsData, String> {
    println!("[extract_analytics_data] Starting data extraction");
    println!("[extract_analytics_data] Parsed data keys: {:?}", parsed.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    let raw_data = parsed.get("raw_data")
        .or_else(|| parsed.get("parsed_data"))
        .ok_or("No data found in response")?;

    println!("[extract_analytics_data] Raw data keys: {:?}", raw_data.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    // GetAnalyticsResponse 包含 repeated QueryResult (field 1)
    // 实际数据结构是 subMesssage_1 而不是 repeated_1
    let query_results = raw_data.get("subMesssage_1")
        .or_else(|| raw_data.get("repeated_1"))
        .and_then(|v| v.as_array())
        .ok_or("No query results found")?;

    println!("[extract_analytics_data] Found {} query results", query_results.len());

    // 原有字段
    let mut daily_cascade_lines = Vec::new();
    let mut tool_usage = Vec::new();
    let mut model_usage_details = Vec::new();
    
    // 新增字段
    let mut percent_code_written = PercentCodeWritten::default();
    let mut completion_stats = CompletionStatistics::default();
    let mut completions_by_day = Vec::new();
    let mut completions_by_language = Vec::new();
    let mut chat_stats = ChatStats::default();
    let mut chats_by_day = Vec::new();
    let mut chats_by_model = Vec::new();
    let mut custom_query_results = CustomQueryResponse::default();

    // 遍历查询结果
    for (index, result) in query_results.iter().enumerate() {
        println!("[extract_analytics_data] Processing result {}: {:?}", index, result.as_object().map(|o| o.keys().collect::<Vec<_>>()));

        // ===== 原有字段 =====
        
        // 检查 Cascade 代码行数统计 (Field 18: cascade_lines)
        if let Some(cascade_lines_data) = result.get("subMesssage_18") {
            println!("[extract_analytics_data] Found cascade_lines data (Field 18)");
            daily_cascade_lines = extract_cascade_lines(cascade_lines_data)?;
            println!("[extract_analytics_data] Extracted {} cascade lines entries", daily_cascade_lines.len());
        }

        // 检查工具使用统计 (Field 19: cascade_tool_usage)
        if let Some(tool_data) = result.get("subMesssage_19") {
            println!("[extract_analytics_data] Found cascade_tool_usage data (Field 19)");
            tool_usage = extract_tool_usage(tool_data)?;
            println!("[extract_analytics_data] Extracted {} tool usage entries", tool_usage.len());
        }

        // 检查模型运行记录 (Field 20: cascade_runs)
        if let Some(model_data) = result.get("subMesssage_20") {
            println!("[extract_analytics_data] Found cascade_runs data (Field 20)");
            model_usage_details = extract_model_usage(model_data)?;
            println!("[extract_analytics_data] Extracted {} model usage entries", model_usage_details.len());
        }
        
        // ===== 新增字段 =====
        
        // 检查代码贡献百分比 (Field 9: percent_code_written)
        if let Some(pcw_data) = result.get("subMesssage_9") {
            println!("[extract_analytics_data] Found percent_code_written data (Field 9)");
            percent_code_written = extract_percent_code_written(pcw_data)?;
        }
        
        // 检查补全统计 (Field 1: completion_stats)
        if let Some(cs_data) = result.get("subMesssage_1") {
            println!("[extract_analytics_data] Found completion_stats data (Field 1)");
            completion_stats = extract_completion_stats(cs_data)?;
        }
        
        // 检查按日期的补全统计 (Field 2: completions_by_day)
        if let Some(cbd_data) = result.get("subMesssage_2") {
            println!("[extract_analytics_data] Found completions_by_day data (Field 2)");
            completions_by_day = extract_completions_by_day(cbd_data)?;
        }
        
        // 检查按语言的补全统计 (Field 3: completions_by_language)
        if let Some(cbl_data) = result.get("subMesssage_3") {
            println!("[extract_analytics_data] Found completions_by_language data (Field 3)");
            completions_by_language = extract_completions_by_language(cbl_data)?;
        }
        
        // 检查 Chat 统计 (Field 11: chat_stats)
        if let Some(chat_data) = result.get("subMesssage_11") {
            println!("[extract_analytics_data] Found chat_stats data (Field 11)");
            chat_stats = extract_chat_stats(chat_data)?;
        }
        
        // 检查按日期的 Chat 统计 (Field 6: chats_by_day)
        if let Some(chats_day_data) = result.get("subMesssage_6") {
            println!("[extract_analytics_data] Found chats_by_day data (Field 6)");
            chats_by_day = extract_chats_by_day(chats_day_data)?;
        }
        
        // 检查按模型的 Chat 统计 (Field 7: chats_by_model)
        if let Some(chats_model_data) = result.get("subMesssage_7") {
            println!("[extract_analytics_data] Found chats_by_model data (Field 7)");
            chats_by_model = extract_chats_by_model(chats_model_data)?;
        }
        
        // 检查自定义查询结果 (Field 16: custom_stats)
        if let Some(custom_data) = result.get("subMesssage_16") {
            println!("[extract_analytics_data] Found custom_stats data (Field 16)");
            custom_query_results = extract_custom_query_response(custom_data)?;
        }

        // ===== 新增字段调试 - 打印所有字段以便分析 =====
        // cascade_stats (QueryRequest field 20) 对应的响应字段
        if let Some(stats_data) = result.get("subMesssage_15") {
            println!("[extract_analytics_data] Found subMesssage_15 (cascade_stats?): {:?}", stats_data);
        }
        // cascade_summary (QueryRequest field 31) 对应的响应字段
        if let Some(summary_data) = result.get("subMesssage_26") {
            println!("[extract_analytics_data] Found subMesssage_26 (cascade_summary?): {:?}", summary_data);
        }
        // 打印所有未处理的字段
        if let Some(obj) = result.as_object() {
            for (key, value) in obj.iter() {
                if !["subMesssage_1", "subMesssage_2", "subMesssage_3", "subMesssage_6", "subMesssage_7", 
                     "subMesssage_9", "subMesssage_11", "subMesssage_16", "subMesssage_18", "subMesssage_19", 
                     "subMesssage_20"].contains(&key.as_str()) {
                    println!("[extract_analytics_data] UNHANDLED field {}: {:?}", key, value);
                }
            }
        }
    }

    // 计算模型使用汇总
    let model_usage_summary = calculate_model_summary(&model_usage_details);

    // 计算总体统计
    let summary = calculate_summary(&daily_cascade_lines, &tool_usage, &model_usage_details, &model_usage_summary);

    Ok(AnalyticsData {
        daily_cascade_lines,
        tool_usage,
        model_usage_details,
        model_usage_summary,
        summary,
        // 新增字段
        percent_code_written,
        completion_stats,
        completions_by_day,
        completions_by_language,
        chat_stats,
        chats_by_day,
        chats_by_model,
        custom_query_results,
    })
}

/// 提取每日 Cascade 代码行数统计 (Field 18: cascade_lines)
fn extract_cascade_lines(data: &Value) -> Result<Vec<DailyCascadeLinesCount>, String> {
    let mut lines_stats = Vec::new();

    println!("[extract_cascade_lines] Data structure: {:?}", data.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    // 实际数据结构：subMesssage_18.subMesssage_1 是一个数组，包含多天的数据
    // 每个元素的结构：
    // {
    //   "subMesssage_1": { "int_1": timestamp },
    //   "int_2": accepted_lines (接受的代码行数),
    //   "int_3": suggested_lines (建议的代码行数)
    // }
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());

    if let Some(repeated) = repeated {
        println!("[extract_cascade_lines] Found array with {} entries", repeated.len());

        for (index, entry) in repeated.iter().enumerate() {
            // 从 subMesssage_1.int_1 获取时间戳
            let timestamp = entry.get("subMesssage_1")
                .and_then(|v| v.get("int_1"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            // 从 int_2 获取建议的代码行数 (lines_suggested = 2)
            let suggested_lines = entry.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0);

            // 从 int_3 获取接受的代码行数 (lines_accepted = 3)
            let accepted_lines = entry.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0);

            println!("[extract_cascade_lines] Entry {}: timestamp={}, accepted={}, suggested={}",
                     index, timestamp, accepted_lines, suggested_lines);

            // 转换时间戳为日期字符串
            let date = if timestamp > 0 {
                DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            lines_stats.push(DailyCascadeLinesCount {
                timestamp,
                date,
                accepted_lines,
                suggested_lines,
            });
        }
    } else {
        println!("[extract_cascade_lines] No array found in subMesssage_1 or repeated_1");
    }

    println!("[extract_cascade_lines] Returning {} lines stats entries", lines_stats.len());
    Ok(lines_stats)
}

/// 提取工具使用统计
fn extract_tool_usage(data: &Value) -> Result<Vec<ToolUsageEntry>, String> {
    let mut usage = Vec::new();
    let mut total_count = 0i64;

    println!("[extract_tool_usage] Data structure: {:?}", data.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    // 实际数据结构：subMesssage_19.subMesssage_1 是一个数组
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());

    if let Some(repeated) = repeated {
        println!("[extract_tool_usage] Found {} tool entries", repeated.len());

        for entry in repeated {
            let tool_name = entry.get("string_2")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let count = entry.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0);
            total_count += count;

            println!("[extract_tool_usage] Tool: {}, Count: {}", tool_name, count);

            usage.push(ToolUsageEntry {
                tool_name,
                count,
                percentage: 0.0, // 稍后计算
            });
        }
    }

    // 计算百分比
    for entry in &mut usage {
        entry.percentage = if total_count > 0 {
            (entry.count as f64 / total_count as f64) * 100.0
        } else {
            0.0
        };
    }

    // 按使用次数降序排序
    usage.sort_by(|a, b| b.count.cmp(&a.count));

    println!("[extract_tool_usage] Returning {} tool usage entries", usage.len());
    Ok(usage)
}

/// 提取模型使用详情
fn extract_model_usage(data: &Value) -> Result<Vec<ModelUsageEntry>, String> {
    let mut usage = Vec::new();

    println!("[extract_model_usage] Data structure: {:?}", data.as_object().map(|o| o.keys().collect::<Vec<_>>()));

    // 实际数据结构：subMesssage_20.subMesssage_1 是一个数组
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());

    if let Some(repeated) = repeated {
        println!("[extract_model_usage] Found {} model entries", repeated.len());

        for entry in repeated {
            let timestamp = entry.get("subMesssage_1")
                .and_then(|v| v.get("int_1"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);

            let date = if timestamp > 0 {
                DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };

            let model_name = entry.get("string_2")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let mode = entry.get("string_3")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
                .to_string();

            let session_count = entry.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0);
            let token_usage = entry.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0);

            let session_id = entry.get("string_6")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            println!("[extract_model_usage] Model: {}, Sessions: {}, Tokens: {}", model_name, session_count, token_usage);

            usage.push(ModelUsageEntry {
                timestamp,
                date,
                model_name,
                mode,
                session_count,
                token_usage,
                session_id,
            });
        }
    }

    println!("[extract_model_usage] Returning {} model usage entries", usage.len());
    Ok(usage)
}

/// 计算模型使用汇总
fn calculate_model_summary(details: &[ModelUsageEntry]) -> Vec<ModelUsageSummary> {
    let mut summary_map: HashMap<String, (i64, i64)> = HashMap::new();

    for entry in details {
        let counter = summary_map.entry(entry.model_name.clone()).or_insert((0, 0));
        counter.0 += entry.session_count;
        counter.1 += entry.token_usage;
    }

    let total_count: i64 = summary_map.values().map(|(count, _)| count).sum();

    let mut summary: Vec<ModelUsageSummary> = summary_map
        .into_iter()
        .map(|(model_name, (total_count_val, total_tokens))| {
            let percentage = if total_count > 0 {
                (total_count_val as f64 / total_count as f64) * 100.0
            } else {
                0.0
            };

            ModelUsageSummary {
                model_name,
                total_count: total_count_val,
                total_tokens,
                percentage,
            }
        })
        .collect();

    // 按使用次数降序排序
    summary.sort_by(|a, b| b.total_count.cmp(&a.total_count));

    summary
}

/// 计算总体统计摘要
fn calculate_summary(
    cascade_lines: &[DailyCascadeLinesCount],
    tool_usage: &[ToolUsageEntry],
    model_details: &[ModelUsageEntry],
    model_summary: &[ModelUsageSummary],
) -> AnalyticsSummary {
    // 计算总代码行数
    let total_accepted_lines: i64 = cascade_lines.iter().map(|s| s.accepted_lines).sum();
    let total_suggested_lines: i64 = cascade_lines.iter().map(|s| s.suggested_lines).sum();

    let avg_daily_accepted_lines = if !cascade_lines.is_empty() {
        total_accepted_lines as f64 / cascade_lines.len() as f64
    } else {
        0.0
    };

    // 找到峰值日期（接受代码行数最多的一天）
    let (peak_date, peak_lines) = cascade_lines
        .iter()
        .max_by_key(|s| s.accepted_lines)
        .map(|s| (s.date.clone(), s.accepted_lines))
        .unwrap_or_else(|| (String::new(), 0));

    let total_tool_usage: i64 = tool_usage.iter().map(|t| t.count).sum();
    let total_sessions: i64 = model_details.iter().map(|m| m.session_count).sum();
    let total_tokens: i64 = model_details.iter().map(|m| m.token_usage).sum();

    let primary_model = model_summary
        .first()
        .map(|m| m.model_name.clone())
        .unwrap_or_else(|| String::from("N/A"));

    let primary_tool = tool_usage
        .first()
        .map(|t| t.tool_name.clone())
        .unwrap_or_else(|| String::from("N/A"));

    AnalyticsSummary {
        total_accepted_lines,
        total_suggested_lines,
        avg_daily_accepted_lines,
        peak_date,
        peak_lines,
        total_tool_usage,
        total_sessions,
        total_tokens,
        primary_model,
        primary_tool,
    }
}

// ===== 新增提取函数 =====

/// 提取代码贡献百分比 (Field 9: percent_code_written)
/// API 返回字段映射（根据实际日志）:
/// - double_1: percent_code_written (0.9997...)
/// - int_4: user_bytes (892)
/// - int_5: codeium_bytes (4383029)
/// - int_6: total_bytes (4383921)
/// - int_7: 未知 (1)
/// - int_8: codeium_bytes_by_cascade (4383028)
fn extract_percent_code_written(data: &Value) -> Result<PercentCodeWritten, String> {
    println!("[extract_percent_code_written] Data: {:?}", data);
    
    let codeium_bytes = data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0);
    let codeium_bytes_by_cascade = data.get("int_8").and_then(|v| v.as_i64()).unwrap_or(0);
    // 自动补全字节数 = AI 总字节数 - Cascade 字节数
    let codeium_bytes_by_autocomplete = codeium_bytes.saturating_sub(codeium_bytes_by_cascade);
    
    Ok(PercentCodeWritten {
        percent_code_written: data.get("double_1").and_then(|v| v.as_f64()).unwrap_or(0.0),
        codeium_bytes_by_autocomplete,
        codeium_bytes_by_command: 0, // API 未返回此字段
        user_bytes: data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
        codeium_bytes,
        total_bytes: data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
        codeium_bytes_by_supercomplete: 0, // API 未返回此字段
        codeium_bytes_by_cascade,
    })
}

/// 提取补全统计 (Field 1: completion_stats)
fn extract_completion_stats(data: &Value) -> Result<CompletionStatistics, String> {
    println!("[extract_completion_stats] Data: {:?}", data);
    
    // CompletionStatistics 嵌套在 subMesssage_1 中
    let stats_data = data.get("subMesssage_1").unwrap_or(data);
    
    let num_acceptances = stats_data.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0);
    let num_rejections = stats_data.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0);
    
    let acceptance_rate = if num_acceptances + num_rejections > 0 {
        (num_acceptances as f64 / (num_acceptances + num_rejections) as f64) * 100.0
    } else {
        0.0
    };
    
    Ok(CompletionStatistics {
        num_acceptances,
        num_rejections,
        num_lines_accepted: stats_data.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0),
        num_bytes_accepted: stats_data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
        num_users: stats_data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0),
        active_developer_days: stats_data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
        active_developer_hours: stats_data.get("int_7").and_then(|v| v.as_i64()).unwrap_or(0),
        acceptance_rate,
    })
}

/// 提取按日期的补全统计 (Field 2: completions_by_day)
fn extract_completions_by_day(data: &Value) -> Result<Vec<CompletionByDay>, String> {
    let mut results = Vec::new();
    
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());
    
    if let Some(entries) = repeated {
        for entry in entries {
            let timestamp = entry.get("subMesssage_1")
                .and_then(|v| v.get("int_1"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            
            let date = if timestamp > 0 {
                DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };
            
            // 提取 CompletionStatistics (subMesssage_2)
            let stats_data = entry.get("subMesssage_2").unwrap_or(entry);
            let num_acceptances = stats_data.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0);
            let num_rejections = stats_data.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0);
            let acceptance_rate = if num_acceptances + num_rejections > 0 {
                (num_acceptances as f64 / (num_acceptances + num_rejections) as f64) * 100.0
            } else {
                0.0
            };
            
            results.push(CompletionByDay {
                timestamp,
                date,
                statistics: CompletionStatistics {
                    num_acceptances,
                    num_rejections,
                    num_lines_accepted: stats_data.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0),
                    num_bytes_accepted: stats_data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
                    num_users: stats_data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0),
                    active_developer_days: stats_data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
                    active_developer_hours: stats_data.get("int_7").and_then(|v| v.as_i64()).unwrap_or(0),
                    acceptance_rate,
                },
            });
        }
    }
    
    println!("[extract_completions_by_day] Extracted {} entries", results.len());
    Ok(results)
}

/// 提取按语言的补全统计 (Field 3: completions_by_language)
fn extract_completions_by_language(data: &Value) -> Result<Vec<CompletionByLanguage>, String> {
    let mut results = Vec::new();
    
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());
    
    if let Some(entries) = repeated {
        for entry in entries {
            let language_id = entry.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let language_name = get_language_name(language_id);
            
            // 提取 CompletionStatistics (subMesssage_2)
            let stats_data = entry.get("subMesssage_2").unwrap_or(entry);
            let num_acceptances = stats_data.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0);
            let num_rejections = stats_data.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0);
            let acceptance_rate = if num_acceptances + num_rejections > 0 {
                (num_acceptances as f64 / (num_acceptances + num_rejections) as f64) * 100.0
            } else {
                0.0
            };
            
            results.push(CompletionByLanguage {
                language_id,
                language_name,
                statistics: CompletionStatistics {
                    num_acceptances,
                    num_rejections,
                    num_lines_accepted: stats_data.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0),
                    num_bytes_accepted: stats_data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
                    num_users: stats_data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0),
                    active_developer_days: stats_data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
                    active_developer_hours: stats_data.get("int_7").and_then(|v| v.as_i64()).unwrap_or(0),
                    acceptance_rate,
                },
            });
        }
    }
    
    // 按接受次数降序排序
    results.sort_by(|a, b| b.statistics.num_acceptances.cmp(&a.statistics.num_acceptances));
    
    println!("[extract_completions_by_language] Extracted {} entries", results.len());
    Ok(results)
}

/// 提取 Chat 统计 (Field 11: chat_stats)
fn extract_chat_stats(data: &Value) -> Result<ChatStats, String> {
    println!("[extract_chat_stats] Data: {:?}", data);
    
    // ChatStats 嵌套在 subMesssage_1 中
    let stats_data = data.get("subMesssage_1").unwrap_or(data);
    
    Ok(ChatStats {
        chats_sent: stats_data.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0),
        chats_received: stats_data.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0),
        chats_accepted: stats_data.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0),
        chats_inserted_at_cursor: stats_data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
        chats_applied: stats_data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0),
        chat_loc_used: stats_data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
        chat_code_blocks_used: stats_data.get("int_7").and_then(|v| v.as_i64()).unwrap_or(0),
        function_explain_count: stats_data.get("int_8").and_then(|v| v.as_i64()).unwrap_or(0),
        function_docstring_count: stats_data.get("int_9").and_then(|v| v.as_i64()).unwrap_or(0),
        function_refactor_count: stats_data.get("int_10").and_then(|v| v.as_i64()).unwrap_or(0),
        code_block_explain_count: stats_data.get("int_11").and_then(|v| v.as_i64()).unwrap_or(0),
        code_block_refactor_count: stats_data.get("int_12").and_then(|v| v.as_i64()).unwrap_or(0),
        problem_explain_count: stats_data.get("int_13").and_then(|v| v.as_i64()).unwrap_or(0),
        function_unit_tests_count: stats_data.get("int_14").and_then(|v| v.as_i64()).unwrap_or(0),
        active_developer_days: stats_data.get("int_15").and_then(|v| v.as_i64()).unwrap_or(0),
    })
}

/// 提取按日期的 Chat 统计 (Field 6: chats_by_day)
fn extract_chats_by_day(data: &Value) -> Result<Vec<ChatStatsByDay>, String> {
    let mut results = Vec::new();
    
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());
    
    if let Some(entries) = repeated {
        for entry in entries {
            let timestamp = entry.get("subMesssage_1")
                .and_then(|v| v.get("int_1"))
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            
            let date = if timestamp > 0 {
                DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.format("%Y-%m-%d").to_string())
                    .unwrap_or_else(|| "Unknown".to_string())
            } else {
                "Unknown".to_string()
            };
            
            // 提取 ChatStats (subMesssage_2)
            let stats_data = entry.get("subMesssage_2").unwrap_or(entry);
            
            results.push(ChatStatsByDay {
                timestamp,
                date,
                stats: ChatStats {
                    chats_sent: stats_data.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_received: stats_data.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_accepted: stats_data.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_inserted_at_cursor: stats_data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_applied: stats_data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0),
                    chat_loc_used: stats_data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
                    chat_code_blocks_used: stats_data.get("int_7").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_explain_count: stats_data.get("int_8").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_docstring_count: stats_data.get("int_9").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_refactor_count: stats_data.get("int_10").and_then(|v| v.as_i64()).unwrap_or(0),
                    code_block_explain_count: stats_data.get("int_11").and_then(|v| v.as_i64()).unwrap_or(0),
                    code_block_refactor_count: stats_data.get("int_12").and_then(|v| v.as_i64()).unwrap_or(0),
                    problem_explain_count: stats_data.get("int_13").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_unit_tests_count: stats_data.get("int_14").and_then(|v| v.as_i64()).unwrap_or(0),
                    active_developer_days: stats_data.get("int_15").and_then(|v| v.as_i64()).unwrap_or(0),
                },
            });
        }
    }
    
    println!("[extract_chats_by_day] Extracted {} entries", results.len());
    Ok(results)
}

/// 提取按模型的 Chat 统计 (Field 7: chats_by_model)
fn extract_chats_by_model(data: &Value) -> Result<Vec<ChatStatsByModel>, String> {
    let mut results = Vec::new();
    
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());
    
    if let Some(entries) = repeated {
        for entry in entries {
            let model_id = entry.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
            let model_name = get_model_name(model_id);
            
            // 提取 ChatStats (subMesssage_2)
            let stats_data = entry.get("subMesssage_2").unwrap_or(entry);
            
            results.push(ChatStatsByModel {
                model_id,
                model_name,
                stats: ChatStats {
                    chats_sent: stats_data.get("int_1").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_received: stats_data.get("int_2").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_accepted: stats_data.get("int_3").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_inserted_at_cursor: stats_data.get("int_4").and_then(|v| v.as_i64()).unwrap_or(0),
                    chats_applied: stats_data.get("int_5").and_then(|v| v.as_i64()).unwrap_or(0),
                    chat_loc_used: stats_data.get("int_6").and_then(|v| v.as_i64()).unwrap_or(0),
                    chat_code_blocks_used: stats_data.get("int_7").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_explain_count: stats_data.get("int_8").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_docstring_count: stats_data.get("int_9").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_refactor_count: stats_data.get("int_10").and_then(|v| v.as_i64()).unwrap_or(0),
                    code_block_explain_count: stats_data.get("int_11").and_then(|v| v.as_i64()).unwrap_or(0),
                    code_block_refactor_count: stats_data.get("int_12").and_then(|v| v.as_i64()).unwrap_or(0),
                    problem_explain_count: stats_data.get("int_13").and_then(|v| v.as_i64()).unwrap_or(0),
                    function_unit_tests_count: stats_data.get("int_14").and_then(|v| v.as_i64()).unwrap_or(0),
                    active_developer_days: stats_data.get("int_15").and_then(|v| v.as_i64()).unwrap_or(0),
                },
            });
        }
    }
    
    // 按发送数降序排序
    results.sort_by(|a, b| b.stats.chats_sent.cmp(&a.stats.chats_sent));
    
    println!("[extract_chats_by_model] Extracted {} entries", results.len());
    Ok(results)
}

/// 提取自定义查询响应 (Field 16: custom_stats)
fn extract_custom_query_response(data: &Value) -> Result<CustomQueryResponse, String> {
    let mut items = Vec::new();
    
    let repeated = data.get("subMesssage_1")
        .or_else(|| data.get("repeated_1"))
        .and_then(|v| v.as_array());
    
    if let Some(entries) = repeated {
        for entry in entries {
            // 每个 QueryResponseItem 包含 map<string, string> item (field 1)
            let mut data_map = HashMap::new();
            
            if let Some(map_data) = entry.get("map_1").and_then(|v| v.as_object()) {
                for (key, value) in map_data {
                    if let Some(val_str) = value.as_str() {
                        data_map.insert(key.clone(), val_str.to_string());
                    }
                }
            }
            
            items.push(CustomQueryResponseItem { data: data_map });
        }
    }
    
    println!("[extract_custom_query_response] Extracted {} items", items.len());
    Ok(CustomQueryResponse { items })
}

/// 根据语言ID获取语言名称
fn get_language_name(language_id: i32) -> String {
    match language_id {
        0 => "Unspecified".to_string(),
        1 => "C".to_string(),
        2 => "Clojure".to_string(),
        3 => "CoffeeScript".to_string(),
        4 => "C++".to_string(),
        5 => "C#".to_string(),
        6 => "CSS".to_string(),
        7 => "Cuda".to_string(),
        8 => "Dockerfile".to_string(),
        9 => "Elixir".to_string(),
        10 => "Go".to_string(),
        11 => "Groovy".to_string(),
        12 => "Handlebars".to_string(),
        13 => "Haskell".to_string(),
        14 => "HCL".to_string(),
        15 => "HTML".to_string(),
        16 => "INI".to_string(),
        17 => "Java".to_string(),
        18 => "JavaScript".to_string(),
        19 => "JSON".to_string(),
        20 => "Julia".to_string(),
        21 => "Kotlin".to_string(),
        22 => "Lua".to_string(),
        23 => "Makefile".to_string(),
        24 => "Markdown".to_string(),
        25 => "Objective-C".to_string(),
        26 => "Perl".to_string(),
        27 => "PHP".to_string(),
        28 => "PowerShell".to_string(),
        29 => "Proto".to_string(),
        30 => "Python".to_string(),
        31 => "R".to_string(),
        32 => "Ruby".to_string(),
        33 => "Rust".to_string(),
        34 => "Sass".to_string(),
        35 => "Scala".to_string(),
        36 => "Shell".to_string(),
        37 => "SQL".to_string(),
        38 => "Swift".to_string(),
        39 => "TypeScript".to_string(),
        40 => "TSX".to_string(),
        41 => "VB".to_string(),
        42 => "Vue".to_string(),
        43 => "XML".to_string(),
        44 => "YAML".to_string(),
        45 => "JSX".to_string(),
        _ => format!("Language_{}", language_id),
    }
}

/// 根据模型ID获取模型名称
fn get_model_name(model_id: i32) -> String {
    match model_id {
        0 => "Unspecified".to_string(),
        1 => "GPT-3.5".to_string(),
        2 => "GPT-4".to_string(),
        3 => "Claude".to_string(),
        4 => "Claude 2".to_string(),
        5 => "Claude Instant".to_string(),
        _ => format!("Model_{}", model_id),
    }
}
