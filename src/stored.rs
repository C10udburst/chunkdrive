/*
    This module implements Stored object, which serializes and deserializes objects to and from the database.
    It has no knowledge of the data types, so make sure to use the correct type when deserializing.
    It uses messagepack for serialization for backwards compatibility.
 */

use std::sync::Arc;
use serde::{Serialize, Deserialize};
use rmp_serde::{Serializer, Deserializer};
use crate::{global::{Global, Descriptor}};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Stored {
    #[serde(rename = "b")]
    bucket: String,
    #[serde(rename = "d")]
    descriptor: Descriptor,
}

impl Stored {
    pub async fn get<T: Deserialize<'static>>(&self, global: Arc<Global>) -> Result<T, String> {
        // Get bucket
        let bucket = global.get_bucket(&self.bucket).ok_or("Bucket not found")?;

        // Get data
        let data = bucket.get(&self.descriptor)
            .await
            .map_err(|e| e.to_string())?;

        // Deserialize data
        let mut deserializer = Deserializer::new(&data[..]);
        T::deserialize(&mut deserializer).map_err(|e| e.to_string())
    }

    pub async fn put<T: Serialize>(&self, global: Arc<Global>, data: T) -> Result<(), String> {
        // Serialize data
        let mut serializer = Serializer::new(Vec::new())
            .with_struct_map(); // https://github.com/3Hren/msgpack-rust/issues/318
        data.serialize(&mut serializer).map_err(|e| e.to_string())?;
        let data = serializer.into_inner();

        // Get bucket
        let bucket = global.get_bucket(&self.bucket).ok_or("Bucket not found")?;

        // Put data
        bucket.put(&self.descriptor, data)
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn create<T: Serialize>(global: Arc<Global>, data: T) -> Result<Stored, String> {
        // Serialize data
        let mut serializer = Serializer::new(Vec::new())
            .with_struct_map(); // https://github.com/3Hren/msgpack-rust/issues/318
        data.serialize(&mut serializer).map_err(|e| e.to_string())?;
        let data = serializer.into_inner();

        // Find bucket
        let bucket_name = global.next_bucket(data.len(), &Vec::new()).ok_or(format!("No bucket found for data of size {}", data.len()))?;
        let bucket = global.get_bucket(bucket_name).ok_or("Bucket not found")?;
        
        // Put data
        let descriptor = bucket.create().await?;
        bucket.put(&descriptor, data)
            .await
            .map_err(|e| e.to_string())?;

        Ok(Stored {
            bucket: bucket_name.to_owned(),
            descriptor,
        })
    }

    pub async fn delete(&self, global: Arc<Global>) -> Result<(), String> {
        // Get bucket
        let bucket = global.get_bucket(&self.bucket).ok_or("Bucket not found")?;
        
        // Delete data
        bucket.delete(&self.descriptor)
            .await
    }
}