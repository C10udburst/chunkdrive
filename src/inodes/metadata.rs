use serde::{Deserialize, Serialize};
use std::{time::{SystemTime, UNIX_EPOCH}, cmp::Ordering};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub enum Size {
    #[serde(rename = "e")]
    Entries(usize),
    #[serde(rename = "b")]
    Bytes(usize),
    Empty,
}

impl Default for Size {
    fn default() -> Self {
        Self::Empty
    }
}

impl PartialOrd for Size {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // entries are always less than bytes, empty is always less than anything
        match (self, other) {
            (Size::Entries(a), Size::Entries(b)) => a.partial_cmp(b),
            (Size::Bytes(a), Size::Bytes(b)) => a.partial_cmp(b),
            (Size::Entries(_), Size::Bytes(_)) => Some(Ordering::Less),
            (Size::Bytes(_), Size::Entries(_)) => Some(Ordering::Greater),
            (Size::Empty, _) => Some(Ordering::Less),
            (_, Size::Empty) => Some(Ordering::Greater),
        }
    } 
}

impl Size {
    pub fn human(&self) -> String {
        match self {
            Size::Entries(entries) => format!("{} entries", entries),
            Size::Bytes(bytes) => {
                let mut bytes = *bytes;
                let mut i = 0;
                let units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
                while bytes > 1024 {
                    bytes /= 1024;
                    i += 1;
                }
                format!("{} {}", bytes, units[i])
            },
            Size::Empty => String::from("Empty"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Metadata {
    #[serde(rename = "c")]
    pub created: u64,
    #[serde(rename = "m")]
    pub modified: u64,

    #[serde(rename = "s")]
    #[serde(default, skip_serializing_if = "is_default")]
    pub size: Size,
}

const fn is_default(size: &Size) -> bool {
    matches!(size, Size::Empty)
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            created: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            modified: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            size: Size::Empty,
        }
    }

    pub fn touch(&mut self) {
        self.modified = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    pub fn modified(&mut self, size: Size) {
        self.touch();
        self.size = size;
    }

    pub fn human_created(&self) -> String {
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.created);
        let datetime: chrono::DateTime<chrono::Utc> = time.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    pub fn human_modified(&self) -> String {
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(self.modified);
        let datetime: chrono::DateTime<chrono::Utc> = time.into();
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}
