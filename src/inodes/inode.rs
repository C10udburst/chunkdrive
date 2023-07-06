use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{global::Global, sources::error::SourceError};

use super::{metadata::Metadata, file::File, folder::Folder};

#[async_trait]
pub trait INode {
    fn get_meta(&self) -> &Metadata;
    fn touch(&mut self);
    async fn delete(&self, global: &Global) -> Result<(), SourceError>;
}

#[derive(Debug, Serialize, Deserialize)]
pub enum INodeType {
    File(File),
    Folder(Folder)
}

impl INodeType {
    pub fn get_meta(&self) -> &Metadata {
        match self {
            INodeType::File(file) => file.get_meta(),
            INodeType::Folder(folder) => folder.get_meta()
        }
    }

    pub fn touch(&mut self) {
        match self {
            INodeType::File(file) => file.touch(),
            INodeType::Folder(folder) => folder.touch()
        }
    }

    pub async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        match self {
            INodeType::File(file) => file.delete(global).await,
            INodeType::Folder(folder) => folder.delete(global).await
        }
    }
}