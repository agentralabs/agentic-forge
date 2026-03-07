//! Client capabilities.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(default)]
    pub roots: Option<RootsCapability>,
    #[serde(default)]
    pub sampling: Option<SamplingCapability>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RootsCapability {
    #[serde(rename = "listChanged", default)]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SamplingCapability {}
