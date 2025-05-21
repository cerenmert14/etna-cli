use std::path::Path;

use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Strategy {
    pub name: String,
    pub tags: Vec<String>,
    pub dir: String,
}
