use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub account_id: Option<Uuid>,
    pub account_email: Option<String>,
    pub operation_type: OperationType,
    pub status: OperationStatus,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Login,
    RefreshToken,
    ResetCredits,
    UpdateSeats,
    GetBilling,
    UpdatePlan,
    GetAccountInfo,
    AddAccount,
    DeleteAccount,
    EditAccount,
    BatchOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Success,
    Failed,
    Pending,
    Processing,
}

impl OperationLog {
    pub fn new(
        operation_type: OperationType,
        status: OperationStatus,
        message: String,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            account_id: None,
            account_email: None,
            operation_type,
            status,
            message,
            details: None,
        }
    }

    pub fn with_account(mut self, account_id: Uuid, account_email: String) -> Self {
        self.account_id = Some(account_id);
        self.account_email = Some(account_email);
        self
    }

    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}
