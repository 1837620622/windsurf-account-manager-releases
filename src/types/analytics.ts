/**
 * 每日 Cascade 代码行数统计 (Field 18: cascade_lines)
 */
export interface DailyCascadeLinesCount {
  /** 日期时间戳 */
  timestamp: number;
  /** 日期字符串 (YYYY-MM-DD) */
  date: string;
  /** 接受的代码行数 (Accepted lines) */
  accepted_lines: number;
  /** 建议的代码行数 (Suggested lines) */
  suggested_lines: number;
}

/**
 * 工具使用统计条目
 */
export interface ToolUsageEntry {
  /** 工具名称 */
  tool_name: string;
  /** 使用次数 */
  count: number;
  /** 使用占比（百分比） */
  percentage: number;
}

/**
 * 模型使用统计条目
 */
export interface ModelUsageEntry {
  /** 日期时间戳 */
  timestamp: number;
  /** 日期字符串 */
  date: string;
  /** 模型名称 */
  model_name: string;
  /** 运行模式 */
  mode: string;
  /** 会话数/消息数 */
  session_count: number;
  /** Token使用量 */
  token_usage: number;
  /** 会话ID */
  session_id: string;
}

/**
 * 模型使用汇总
 */
export interface ModelUsageSummary {
  /** 模型名称 */
  model_name: string;
  /** 总使用次数 */
  total_count: number;
  /** 总Token消耗 */
  total_tokens: number;
  /** 使用占比 */
  percentage: number;
}

/**
 * 总体统计摘要
 */
export interface AnalyticsSummary {
  /** 总代码行数（接受的） */
  total_accepted_lines: number;
  /** 总代码行数（建议的） */
  total_suggested_lines: number;
  /** 平均每日代码行数（接受的） */
  avg_daily_accepted_lines: number;
  /** 峰值日期 */
  peak_date: string;
  /** 峰值代码行数 */
  peak_lines: number;
  /** 总工具使用次数 */
  total_tool_usage: number;
  /** 总会话数 */
  total_sessions: number;
  /** 总Token消耗 */
  total_tokens: number;
  /** 主要使用的模型 */
  primary_model: string;
  /** 主要使用的工具 */
  primary_tool: string;
}

// ============== 新增类型定义 ==============

/**
 * 代码贡献百分比统计
 */
export interface PercentCodeWritten {
  /** AI 编写代码占比 (%) */
  percent_code_written: number;
  /** 通过自动补全产生的字节数 */
  codeium_bytes_by_autocomplete: number;
  /** 通过命令产生的字节数 */
  codeium_bytes_by_command: number;
  /** 用户编写的字节数 */
  user_bytes: number;
  /** AI 总共产生的字节数 */
  codeium_bytes: number;
  /** 总字节数 */
  total_bytes: number;
  /** 通过 Supercomplete 产生的字节数 */
  codeium_bytes_by_supercomplete: number;
  /** 通过 Cascade 产生的字节数 */
  codeium_bytes_by_cascade: number;
}

/**
 * 代码补全统计
 */
export interface CompletionStatistics {
  /** 接受次数 */
  num_acceptances: number;
  /** 拒绝次数 */
  num_rejections: number;
  /** 接受的代码行数 */
  num_lines_accepted: number;
  /** 接受的字节数 */
  num_bytes_accepted: number;
  /** 用户数 */
  num_users: number;
  /** 活跃开发天数 */
  active_developer_days: number;
  /** 活跃开发小时数 */
  active_developer_hours: number;
  /** 接受率 (计算字段) */
  acceptance_rate: number;
}

/**
 * 按日期的补全统计
 */
export interface CompletionByDay {
  /** 日期时间戳 */
  timestamp: number;
  /** 日期字符串 */
  date: string;
  /** 补全统计 */
  statistics: CompletionStatistics;
}

/**
 * 按语言的补全统计
 */
export interface CompletionByLanguage {
  /** 语言ID */
  language_id: number;
  /** 语言名称 */
  language_name: string;
  /** 补全统计 */
  statistics: CompletionStatistics;
}

/**
 * Chat 统计
 */
export interface ChatStats {
  /** 发送的聊天数 */
  chats_sent: number;
  /** 接收的聊天数 */
  chats_received: number;
  /** 接受的聊天数 */
  chats_accepted: number;
  /** 在光标处插入的次数 */
  chats_inserted_at_cursor: number;
  /** 应用的次数 */
  chats_applied: number;
  /** 使用的代码行数 */
  chat_loc_used: number;
  /** 使用的代码块数 */
  chat_code_blocks_used: number;
  /** 函数解释次数 */
  function_explain_count: number;
  /** 文档字符串生成次数 */
  function_docstring_count: number;
  /** 函数重构次数 */
  function_refactor_count: number;
  /** 代码块解释次数 */
  code_block_explain_count: number;
  /** 代码块重构次数 */
  code_block_refactor_count: number;
  /** 问题解释次数 */
  problem_explain_count: number;
  /** 单元测试生成次数 */
  function_unit_tests_count: number;
  /** 活跃开发天数 */
  active_developer_days: number;
}

/**
 * 按日期的 Chat 统计
 */
export interface ChatStatsByDay {
  /** 日期时间戳 */
  timestamp: number;
  /** 日期字符串 */
  date: string;
  /** Chat 统计 */
  stats: ChatStats;
}

/**
 * 按模型的 Chat 统计
 */
export interface ChatStatsByModel {
  /** 模型ID */
  model_id: number;
  /** 模型名称 */
  model_name: string;
  /** Chat 统计 */
  stats: ChatStats;
}

/**
 * 自定义查询响应项
 */
export interface CustomQueryResponseItem {
  /** 键值对数据 */
  data: Record<string, string>;
}

/**
 * 自定义查询响应
 */
export interface CustomQueryResponse {
  /** 响应项列表 */
  items: CustomQueryResponseItem[];
}

// ============== 主数据结构 ==============

/**
 * 使用分析响应数据
 */
export interface AnalyticsData {
  /** 每日 Cascade 代码行数统计 (Field 18: cascade_lines) */
  daily_cascade_lines: DailyCascadeLinesCount[];
  /** 工具使用统计 (Field 19: cascade_tool_usage) */
  tool_usage: ToolUsageEntry[];
  /** 模型使用详情 (Field 20: cascade_runs) */
  model_usage_details: ModelUsageEntry[];
  /** 模型使用汇总 */
  model_usage_summary: ModelUsageSummary[];
  /** 总体统计 */
  summary: AnalyticsSummary;
  
  // ===== 新增字段 =====
  /** 代码贡献百分比 */
  percent_code_written: PercentCodeWritten;
  /** 补全统计 */
  completion_stats: CompletionStatistics;
  /** 按日期的补全统计 */
  completions_by_day: CompletionByDay[];
  /** 按语言的补全统计 */
  completions_by_language: CompletionByLanguage[];
  /** Chat 统计 */
  chat_stats: ChatStats;
  /** 按日期的 Chat 统计 */
  chats_by_day: ChatStatsByDay[];
  /** 按模型的 Chat 统计 */
  chats_by_model: ChatStatsByModel[];
  /** 自定义查询结果 */
  custom_query_results: CustomQueryResponse;
}

