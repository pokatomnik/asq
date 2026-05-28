use serde::{Deserialize, Serialize};

use crate::entities::role::Role;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Message {
    role: Role,
    contents: String,
}

impl Message {
    pub fn new(role: Role, contents: &str) -> Self {
        Self {
            role,
            contents: contents.to_string(),
        }
    }

    pub fn contents(&self) -> &str {
        &self.contents
    }

    pub fn role(&self) -> &Role {
        &self.role
    }
}
