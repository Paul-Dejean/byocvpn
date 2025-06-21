use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "cmd")]
pub enum DaemonCommand {
    Connect { config_path: String },
    Disconnect,
    Status,
}
