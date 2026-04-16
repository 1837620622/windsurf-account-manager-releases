# Windsurf Account Manager (CK 二次开发版)

![首页截图](homepage.png)

## 最新动态

> **截止 2026 年 4 月 16 日，切号功能一切正常。**

### v2.4.5 更新内容

- **auth1 API 完整支持**：新注册账号调用 Windsurf API 时自动带上 `x-devin-*` 专用 headers（`account-id` / `org-id` / `auth1-token` / `session-token`），解决 GetUsers / GetPlanStatus / GetCurrentUser 对 auth1 账号返回不完整数据或 500 错误的问题
- **配额实时刷新**：refresh_token 和 refresh_token_internal 按 auth_type 分支处理，auth1 账号直接重新登录获取完整凭据（含 account_id / org_id），保证每次刷新都能拿到最新真实配额
- **Account 模型扩展**：新增 auth1 账号的 account_id / org_id / auth1_token 字段，登录时自动持久化本地
- **tier 枚举扩展**：新增 FREE=19 枚举值映射，auth1 新账号不再显示 UNKNOWN
- **新旧账号兼容**：Firebase 老账号与 auth1 新账号同时支持，登录/切号/刷新全链路打通

### v2.2.4 更新内容

- **集成 auth1 认证**：新增 Windsurf auth1 登录方式，支持新注册账号（Firebase 不可用的账号）
- **自动降级登录**：登录时先尝试 Firebase，失败后自动切换 auth1，无需手动选择
- **auth1 切号支持**：auth1 账号使用 session_token 进行切号回调，与 Firebase 账号流程统一
- **Token 刷新优化**：auth1 账号过期自动重新登录，Firebase 降级到 auth1 时自动更新认证类型
- **配额刷新修复**：添加调试日志，修复配额查询静默失败的问题
- **构建修复**：修复 cunzhi 目录缺失时的构建失败问题

### v2.2.3 更新内容

- **自动注入补丁**：启动软件后自动检测并注入无感切号补丁，无需手动开启开关
- **修复 Firebase API Key**：旧版 Codeium Firebase Key 已失效，更新为 Windsurf 当前 Key
- **修复 Referer 403**：token 刷新请求添加 Referer/Origin 头，绕过 HTTP Referrer 限制
- **适配 Windsurf v1.110.1**：使用 Firebase ID Token 替代 GetOneTimeAuthToken
- **补丁升级 v2**：内联 URLSearchParams 逻辑，不再依赖混淆变量名
- **欢迎页优化**：验证按钮改为 Enter 回车交互，布局更合理

### v2.2.1 更新内容

- **修复数据损坏**：修复账号多时 JSON 数据损坏（trailing characters）导致启动失败的问题
- **并发写入保护**：添加写入互斥锁，防止多任务并发写入文件导致数据竞态
- **智能数据恢复**：增强数据文件损坏时的自动修复能力（智能截断 + 备份恢复）
- **修复更新检测**：修复自动更新检测功能中版本号解析错误的问题
- **门户页重构**：赛博科技风格 UI + 修复 6 项前端问题
- **全平台兼容**：macOS / Windows / Linux 全平台测试通过

## 使用说明

> 启动软件后，会显示欢迎门户页面。请在输入框中输入访问口令 **`cknb`**，然后按 **Enter** 回车即可进入主界面。

基于 Tauri + Vue 3 + TypeScript 的 Windsurf 多账号管理桌面应用，支持弹性计费（Elastic Billing）模式下的配额实时监控、批量Token刷新、一键换号等功能。

> 本项目由 **传康Kk** 基于原版进行二次开发，适配 Windsurf 2026 弹性计费新模型，新增每日/每周配额可视化、重置倒计时等特性。

## 功能特性

| 模块 | 功能说明 |
|---|---|
| 账号管理 | 添加/编辑/删除账号、分组、标签、加密存储（AES-256-GCM） |
| 弹性计费监控 | 每日/每周配额剩余百分比、进度条可视化、重置倒计时 |
| Token 管理 | 自动刷新、批量刷新（最多5并发）、过期提醒 |
| 一键换号 | OAuth 回调自动登录、windsurf:// 协议触发 |
| 团队管理 | 查看/邀请/移除成员、团队积分统一管理 |
| 账单查询 | 套餐信息、使用量统计、订阅状态、积分详情 |
| 数据安全 | 系统密钥链存储加密密钥、所有数据仅本地存储 |

## 弹性计费适配（v2.0 新增）

Windsurf 于 2026 年从积分制切换为弹性计费模型（Usage Allowance），本项目已完成全链路适配：

- **后端**：Protobuf 解析器新增 `daily_quota_remaining_percent`、`weekly_quota_remaining_percent`、`daily_quota_reset_timestamp`、`weekly_quota_reset_timestamp` 字段提取
- **数据模型**：Account 结构体扩展弹性计费字段
- **前端卡片**：AccountCard 新增每日/每周配额进度条和重置倒计时
- **详情弹窗**：AccountInfoDialog 新增弹性计费配额可视化面板

## 技术栈

| 层级 | 技术 |
|---|---|
| 前端框架 | Vue 3 + TypeScript |
| UI 组件库 | Element Plus |
| 状态管理 | Pinia |
| 构建工具 | Vite |
| 后端框架 | Tauri 2.x (Rust) |
| 加密方案 | AES-256-GCM + 系统密钥链 |
| 网络请求 | Reqwest + Tokio |
| API协议 | Protobuf (gRPC-Web) + Firebase Auth |

## 安装和运行

### 前提条件

- Node.js 16+
- Rust 1.70+
- macOS / Windows / Linux

### Mac 部署步骤

```bash
git clone https://github.com/1837620622/windsurf-account-manager.git
cd windsurf-account-manager
npm install
npm run tauri dev
```

构建发布版：

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`（`.dmg` 安装包）。

### Windows 部署步骤

```bash
git clone https://github.com/1837620622/windsurf-account-manager.git
cd windsurf-account-manager
npm install
npm run tauri dev
```

构建发布版：

```bash
npm run tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`（`.msi` 或 `.exe` 安装包）。

### Linux 部署步骤

```bash
git clone https://github.com/1837620622/windsurf-account-manager.git
cd windsurf-account-manager
npm install
npm run tauri dev
```

构建产物位于 `src-tauri/target/release/bundle/`（`.deb` / `.AppImage`）。

## 项目结构

```
windsurf-account-manager/
├── src/                      # Vue 前端源码
│   ├── components/           # 组件（AccountCard、AccountInfoDialog 等）
│   ├── store/modules/        # Pinia 状态管理
│   ├── api/                  # Tauri 命令调用封装
│   ├── types/                # TypeScript 类型定义
│   └── utils/                # 工具函数
├── src-tauri/                # Rust 后端源码
│   ├── src/models/           # 数据模型（Account 等）
│   ├── src/services/         # 业务逻辑（proto_parser、windsurf_service）
│   ├── src/commands/         # Tauri 命令（api_commands 等）
│   └── Cargo.toml            # Rust 依赖
├── package.json
└── vite.config.ts
```

## 数据存储

| 系统 | 路径 |
|---|---|
| macOS | `~/Library/Application Support/com.chao.windsurf-account-manager/accounts.json` |
| Windows | `%APPDATA%\com.chao.windsurf-account-manager\accounts.json` |
| Linux | `~/.config/com.chao.windsurf-account-manager/accounts.json` |

所有敏感信息（密码、Token）均使用 AES-256-GCM 加密，密钥存储在系统密钥链中。

## 常见问题

**Token 刷新失败**：检查账号密码是否正确、网络是否通畅、Token 是否已过期（尝试重新登录）。

**配额显示为空**：需先刷新 Token 以触发 GetPlanStatus API 调用获取最新配额数据。

**如何备份数据**：将上述路径的 `accounts.json` 文件复制到安全位置即可。

## 二次开发信息

- **二次开发者**：传康Kk
- **微信**：1837620622
- **邮箱**：2040168455@qq.com
- **咸鱼/B站**：万能程序员
- **开源地址**：https://github.com/1837620622/windsurf-account-manager

## 许可证

MIT License

## 免责声明

本工具仅供学习和个人使用，请遵守 Windsurf 服务条款。作者不对因使用本工具产生的任何问题负责。
