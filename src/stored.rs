
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::{global::Global, sources::error::SourceError, block::{block::BlockType, indirect_block::IndirectBlock}};
use rmp_serde::{Serializer, Deserializer};

#[derive(Debug, Serialize, Deserialize)]
pub struct Stored {
    block: Box<BlockType>,
}

impl Stored {
    pub async fn new<T>(global: &Global, data: &T) -> Result<Self, SourceError>
    where T: Serialize
    {
        let mut buf = Vec::new();
        data.serialize(&mut Serializer::new(&mut buf)).map_err(|e| SourceError::new(format!("Failed to serialize data: {}", e)))?;
        let block = IndirectBlock::create(global, &buf).await?;
        Ok(Stored {
            block: Box::new(block.to_enum())
        })
    }

    pub async fn get<T>(&self, global: &Global) -> Result<T, SourceError> 
    where T: DeserializeOwned
    {
        let data = self.block.get(global, self.block.range(global).await?).await?;
        let mut de = Deserializer::new(&data[..]);
        let deserialized: T = Deserialize::deserialize(&mut de).map_err(|e| SourceError::new(format!("Failed to deserialize data: {}", e)))?;
        Ok(deserialized)
    }

    pub async fn put<T>(&mut self, global: &Global, data: &T) -> Result<(), SourceError>
    where T: Serialize
    {
        let mut buf = Vec::new();
        data.serialize(&mut Serializer::new(&mut buf)).map_err(|e| SourceError::new(format!("Failed to serialize data: {}", e)))?;
        self.block.put(global, self.block.range(global).await?, buf).await?;
        Ok(())
    }

    pub async fn delete(&self, global: &Global) -> Result<(), SourceError> {
        self.block.delete(global).await?;
        Ok(())
    }
}