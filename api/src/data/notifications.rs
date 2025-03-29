use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    id: u64,
    content: String,
    channel: String,     // "push" or "email"
    recipient: String,
    scheduled_time: String, // For simplicity, using String (ideally use a DateTime type)
    timezone: String,
    priority: String,
    status: String,      // "pending", "sent", "failed", "cancelled"
    retry_count: u32,
}
