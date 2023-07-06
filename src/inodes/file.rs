use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use crate::{stored::Stored, global::Global, block::{block::BlockType, indirect_block::IndirectBlock}, sources::error::SourceError};

use super::{inode::{INode, INodeType}, metadata::Metadata};

#[derive(Serialize, Deserialize, Debug)]
pub struct File {
    pub meta: Metadata,
    data: Stored
}

#[async_trait]
impl INode for File {
    fn get_meta(&self) -> &Metadata {
        &self.meta
    }

    fn touch(&mut self) {
        self.meta.touch();
    }

    async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        let block: BlockType = self.data.get(global).await?;
        block.delete(global).await?;
        Ok(())
    }
}

impl File {
    pub fn to_enum(self) -> INodeType {
        INodeType::File(self)
    }

    pub async fn create(data: &Vec<u8>, global: &Global) -> Result<File, SourceError> {
        let block = IndirectBlock::create(global, data).await?;
        let file = File {
            meta: Metadata::new(
                &format!("{} bytes", data.len()),
                0, 0 // TODO: dont do that
            ),
            data: Stored::new(global, &block).await?
        };
        Ok(file)
    }

    pub async fn get(&self, global: &Global) -> Result<Vec<u8>, SourceError> {
        let block: BlockType = self.data.get(global).await?;
        let data = block.get(global, &block.range(global).await?).await?;
        Ok(data)
    }
}