//! Network Lightcone records shared by the capture worker and dashboard.
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Report {
    pub role: String,
    pub pack_sha256: String,
    pub trained_paths: usize,
    pub deny_paths: usize,
    pub samples: u64,
    pub matched: u64,
    pub denied: u64,
    pub unrepresented: u64,
    pub truncated: u64,
    pub missing_link_metadata: u64,
    pub inbound: u64,
    pub outbound: u64,
    pub kernel_drops: u64,
    pub interface_drops: u64,
    pub last: Option<Receipt>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Receipt {
    pub sequence: u64,
    pub direction: String,
    pub kind: String,
    pub path: Vec<String>,
    pub admitted: bool,
    pub denied: bool,
    pub query_sha256: String,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "event", content = "data", rename_all = "snake_case")]
pub enum Message {
    Ready(Report),
    Progress(Report),
    Complete(Report),
    Error(String),
}
