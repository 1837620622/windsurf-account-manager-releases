# Firebase API 数据修复说明

## 问题描述
前端显示 Firebase UID 为 "N/A"，因为后端只返回了部分 Firebase 字段。

## 问题原因

### 1. 不完整的数据结构
原始 `AccountInfo` 结构体只包含5个字段：
```rust
pub struct AccountInfo {
    pub email: String,
    pub display_name: Option<String>,
    pub email_verified: bool,
    pub created_at: Option<String>,
    pub last_login_at: Option<String>,
}
```

### 2. 缺失的关键字段
Firebase Identity Toolkit API 实际返回的完整数据包括：
- `localId` - Firebase 用户唯一标识符（最重要！）
- `passwordHash` - 密码哈希
- `passwordUpdatedAt` - 密码更新时间
- `validSince` - 有效期开始
- `disabled` - 账户状态
- `lastRefreshAt` - 最后刷新时间
- `providerUserInfo` - 认证提供商信息数组

## 解决方案

### 1. 扩展 AccountInfo 结构体
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    #[serde(rename = "localId")]
    pub local_id: String,
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "emailVerified")]
    pub email_verified: bool,
    #[serde(rename = "passwordHash")]
    pub password_hash: Option<String>,
    #[serde(rename = "passwordUpdatedAt")]
    pub password_updated_at: Option<i64>,
    #[serde(rename = "validSince")]
    pub valid_since: Option<String>,
    #[serde(rename = "disabled", default)]
    pub disabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "lastLoginAt")]
    pub last_login_at: Option<String>,
    #[serde(rename = "lastRefreshAt")]
    pub last_refresh_at: Option<String>,
    #[serde(rename = "providerUserInfo")]
    pub provider_user_info: Option<Vec<ProviderInfo>>,
}
```

### 2. 添加 ProviderInfo 结构体
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderInfo {
    #[serde(rename = "providerId")]
    pub provider_id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "federatedId")]
    pub federated_id: Option<String>,
    pub email: Option<String>,
    #[serde(rename = "rawId")]
    pub raw_id: Option<String>,
    #[serde(rename = "photoUrl")]
    pub photo_url: Option<String>,
}
```

### 3. 更新 API 响应格式
使用驼峰命名法以保持与 Firebase API 的一致性：
```json
{
  "firebase_info": {
    "localId": "0ABYadmzKIeJ0BH7hglHLExRfbL2",
    "email": "302718249@qq.com",
    "displayName": "chjaoi wang",
    "emailVerified": true,
    "passwordHash": "...",
    "passwordUpdatedAt": 1763169660723,
    "validSince": "1763376045",
    "disabled": false,
    "createdAt": "1763169660723",
    "lastLoginAt": "1763525770639",
    "lastRefreshAt": "2025-11-20T12:32:28.415381Z",
    "providerUserInfo": [...]
  }
}
```

## 前端兼容性

前端已更新为同时支持两种命名格式：
- 驼峰式：`localId`, `displayName`, `emailVerified`（Firebase 标准）
- 下划线式：`local_id`, `display_name`, `email_verified`（旧版本兼容）

## 修改的文件

1. **后端**
   - `src-tauri/src/services/auth_service.rs` - 扩展数据结构
   - `src-tauri/src/commands/api_commands.rs` - 返回完整数据

2. **前端**
   - `src/components/AccountInfoDialog.vue` - 兼容两种命名格式

## 测试验证

重新编译并运行后，Firebase 信息应该完整显示：
- ✅ Firebase UID 正确显示
- ✅ 所有时间戳正确格式化
- ✅ 认证提供商信息完整
- ✅ 账户状态准确反映
