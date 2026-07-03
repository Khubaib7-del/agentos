use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub id: u64,
    pub text: String,
    pub why: Option<String>,
    pub locked: bool,
    pub made_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoteStatus {
    Pending,
    Delivered,
    Resolved,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewNote {
    pub id: u64,
    pub text: String,
    pub status: NoteStatus,
    pub created_at: DateTime<Utc>,
}
