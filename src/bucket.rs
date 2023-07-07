use serde::Deserialize;

use crate::{sources::source::SourceType, encryption::encryption::EncryptionType};

#[derive(Deserialize, Debug)]
pub struct Bucket {
    source: SourceType,
    #[serde(default)]
    encryption: EncryptionType,
}