use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub name: String,
    pub length: u64,
    pub consumers: u32,
}

impl Queue {
    /// Basic constructor for creating a new Queue - kept for future API use
    #[allow(dead_code)]
    pub fn new(name: String) -> Self {
        Self {
            name,
            length: 0,
            consumers: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn has_consumers(&self) -> bool {
        self.consumers > 0
    }
}
