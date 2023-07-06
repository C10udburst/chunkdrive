use std::collections::HashMap;

use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{stored::Stored, global::Global, block::{block::BlockType}, sources::error::SourceError};

use super::{inode::{INode, INodeType}, metadata::Metadata};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Folder {
    pub meta: Metadata,
    children: HashMap<String, Stored>
}

#[async_trait]
impl INode for Folder {
    fn get_meta(&self) -> &Metadata {
        &self.meta
    }

    fn touch(&mut self) {
        self.meta.touch();
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        for (_, stored) in self.children.iter() {
            let block: BlockType = stored.get(global).await?;
            block.delete(global).await?;
            stored.delete(global).await?;
        }
        Ok(())
    }
}

impl Folder {
    pub fn to_enum(self) -> INodeType {
        INodeType::Folder(self)
    }

    pub fn create() -> Result<Folder, SourceError> {
        let folder = Folder {
            meta: Metadata::new(
                &"0 entries".to_string(),
                0, 0 // TODO: dont do that
            ),
            children: HashMap::new()
        };
        Ok(folder)
    }

    pub fn list(&self) -> Vec<String> {
        self.children.keys().map(|k| k.clone()).collect()
    }

    pub async fn get(&self, global: &Global, name: &str) -> Result<INodeType, SourceError> {
        let stored = self.children.get(name).ok_or(SourceError::new("File not found".to_string()))?;
        let inode: INodeType = stored.get(global).await?;
        Ok(inode)
    }

    pub async fn update(&mut self, global: &Global, name: &str, inode: &INodeType) -> Result<(), SourceError> {
        let stored = self.children.get_mut(name).ok_or(SourceError::new("File not found".to_string()))?;
        stored.put(global, inode).await?;
        Ok(())
    }

    pub async fn add(&mut self, global: &Global, name: &str, inode: INodeType) -> Result<(), SourceError> {
        let stored = Stored::new(global, &inode).await?;
        self.children.insert(name.to_string(), stored);
        self.meta.update(&format!("{} entries", self.children.len()));
        Ok(())
    }

    pub async fn remove(&mut self, global: &Global, name: &str) -> Result<(), SourceError> {
        let stored = self.children.remove(name).ok_or(SourceError::new("File not found".to_string()))?;
        let block: INodeType = stored.get(global).await?;
        block.delete(global).await?;
        stored.delete(global).await?;
        self.meta.update(&format!("{} entries", self.children.len()));
        Ok(())
    }
}