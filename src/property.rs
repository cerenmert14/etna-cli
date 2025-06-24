use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Hash)]
pub struct Property {
    pub name: String,
}
