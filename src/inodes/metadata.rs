use serde::{Serialize, Deserialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Permissions {
    pub user_read: bool,
    pub user_write: bool,
    pub group_read: bool,
    pub group_write: bool,
    pub other_read: bool,
    pub other_write: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Metadata {
    pub creation_time: u64,
    pub modification_time: u64,
    pub human_size: String,
    pub owner: u64,
    pub group: u64,
    pub permissions: Permissions,
}

impl Permissions {
    pub fn new() -> Self {
        Self {
            user_read: true,
            user_write: true,
            group_read: true,
            group_write: true,
            other_read: true,
            other_write: true,
        }
    }
}

impl Metadata {
    pub fn new(size: &String, owner: u64, group: u64) -> Self {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        Self {
            creation_time: now,
            modification_time: now,
            human_size: size.clone(),
            owner,
            group,
            permissions: Permissions::new(),
        }
    }

    pub fn can_read(&self, user: u64, group: u64) -> bool {
        if user == self.owner {
            return self.permissions.user_read;
        }
        if group == self.group {
            return self.permissions.group_read;
        }
        self.permissions.other_read
    }

    pub fn can_write(&self, user: u64, group: u64) -> bool {
        if user == self.owner {
            return self.permissions.user_write;
        }
        if group == self.group {
            return self.permissions.group_write;
        }
        self.permissions.other_write
    }

    pub fn update(&mut self, size: &String) {
        self.touch();
        self.human_size = size.clone();
    }

    pub fn touch(&mut self) {
        let now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        self.modification_time = now;
    }

}