# Windsurf Account Manager

> Windsurf 多账号管理桌面应用（CK 二次开发版）

基于 Tauri + Vue3 + Rust 构建的高性能桌面应用，支持 Windows / macOS / Linux 全平台。

## 功能特性

| 功能 | 说明 |
|------|------|
| 多账号管理 | 批量添加、分组、标签管理 |
| Token 自动刷新 | 支持邮箱密码登录和 Refresh Token 模式 |
| 弹性计费监控 | 实时显示每日/每周配额剩余百分比 |
| 无感换号 | 配额用尽自动切换到下一个可用账号，无需手动操作 |
| 套餐信息 | 查看 Free/Pro/Team/Max 等套餐详情 |
| 团队管理 | 团队成员管理、席位调整、积分重置 |
| 批量操作 | 批量导入、批量刷新、批量登录 |
| 数据导出 | 支持 JSON/CSV 格式导出账号数据 |
| 隐私模式 | 邮箱脱敏显示，保护隐私 |
| 多平台支持 | Windows x64/ARM64, macOS x64/ARM64, Linux x64/ARM64 |

## 下载安装

前往 [Releases](https://github.com/1837620622/windsurf-account-manager-releases/releases) 页面下载对应平台的安装包：

### Windows

| 文件 | 架构 | 说明 |
|------|------|------|
| `*.exe` | x64 / ARM64 | NSIS 安装程序（推荐） |
| `*.msi` | x64 / ARM64 | MSI 安装包 |

### macOS

| 文件 | 架构 | 说明 |
|------|------|------|
| `*.dmg` | Intel (x64) | 适用于 Intel Mac |
| `*.dmg` | Apple Silicon (ARM64) | 适用于 M1/M2/M3/M4 Mac（推荐） |

> macOS 首次打开可能提示「无法验证开发者」，请前往「系统偏好设置 → 安全性与隐私」点击「仍要打开」

### Linux

| 文件 | 架构 | 说明 |
|------|------|------|
| `*.deb` | x64 / ARM64 | Debian/Ubuntu |
| `*.rpm` | x64 / ARM64 | Fedora/CentOS |
| `*.AppImage` | x64 | 通用格式 |

## 使用说明

1. 下载对应平台的安装包并安装
2. 打开应用，点击「添加账号」
3. 输入 Windsurf 邮箱和密码（或 Refresh Token）
4. 应用会自动登录并获取账号信息（套餐、配额等）
5. 支持批量导入：每行一个 `邮箱 密码 备注`

## 技术栈

| 技术 | 用途 |
|------|------|
| Tauri 2.x | 跨平台桌面框架 |
| Vue 3 + TypeScript | 前端界面 |
| Rust | 后端核心逻辑 |
| Element Plus | UI 组件库 |
| Pinia | 状态管理 |

## 联系作者

| 平台 | 联系方式 |
|------|----------|
| 微信 | 1837620622（传康Kk） |
| 邮箱 | 2040168455@qq.com |
| 咸鱼/B站 | 万能程序员 |

## 免责声明

本工具仅供学习研究使用，请遵守 Windsurf 的服务条款。使用本工具产生的任何后果由使用者自行承担。
