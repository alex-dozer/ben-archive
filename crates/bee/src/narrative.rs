use crate::cli::{Mode, OsType};

pub fn alloc_id() -> String {
    "id".to_string()
}

pub fn greet(_id: &str, _pid: u32, _mode: &Mode, _os: &OsType) {}

pub fn retire(_id: &str, _cert: &str) {}