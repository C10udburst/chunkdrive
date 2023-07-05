use serde::{Serialize, Deserialize};
use crate::{stored::Stored, global::Global, block::block::BlockType};

use super::{inode::INode, metadata::Metadata};

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub meta: Metadata,
    data: Stored
}

impl INode for File {
    fn get_meta(&self) -> &Metadata {
        &self.meta
    }

    fn touch(&mut self) {
        self.meta.touch();
    }
}

impl File {
    
}