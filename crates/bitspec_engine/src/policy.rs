use std::sync::Arc;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PolicyKey {
    pub tenant: Arc<str>,
    pub event: Arc<str>,
    pub version: u32,
}

impl PolicyKey {
    pub fn new(tenant: Arc<str>, event: Arc<str>, version: u32) -> Self {
        Self {
            tenant,
            event,
            version,
        }
    }
}
