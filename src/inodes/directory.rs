use std::{collections::HashMap, sync::Arc};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};

use crate::{stored::Stored, global::Global};
use super::{inode::{Inode, InodeType}, metadata::{Metadata, Size}};


#[derive(Debug, Serialize, Deserialize)]
pub struct Directory {
    #[serde(rename = "c")]
    #[serde(default, skip_serializing_if = "is_empty")]
    children: HashMap<String, Stored>,
    #[serde(rename = "m")]
    pub metadata: Metadata
}

fn is_empty<T>(map: &HashMap<String, T>) -> bool {
    map.is_empty()
}

#[async_trait]
impl Inode for Directory {
    async fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    async fn delete(&mut self, global: Arc<Global>) -> Result<(), String> {
        let mut errors = Vec::new();
        for (_, stored) in self.children.drain() {
            match &mut stored.get::<InodeType>(global.clone()).await {
                Ok(inode) => match inode.delete(global.clone()).await {
                    Ok(_) => (),
                    Err(e) => errors.push(e)
                },
                Err(e) => errors.push(e.clone())
            };
            match stored.delete(global.clone()).await {
                Ok(_) => (),
                Err(e) => errors.push(e)
            }
        }
        match errors.len() {
            0 => Ok(()),
            _ => Err(errors.join(", "))
        }
    }
}

impl Directory {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            metadata: Metadata::new()
        }
    }

    pub fn to_enum(self) -> InodeType {
        InodeType::Directory(self)
    }

    pub async fn add(&mut self, global: Arc<Global>, name: &String, inode: InodeType) -> Result<(), String> {
        if self.children.contains_key(name) {
            return Err(format!("File {} already exists", name));
        }

        let stored = Stored::create(global, inode).await?;
        
        self.children.insert(name.clone(), stored);
        self.metadata.modified(Size::Entries(self.children.len()));

        Ok(())
    }

    pub async fn remove(&mut self, global: Arc<Global>, name: &String) -> Result<(), String> {
        if !self.children.contains_key(name) {
            return Err(format!("File {} does not exist", name));
        }

        let stored = self.children.remove(name).unwrap();
        let mut inode: Result<InodeType, String> = stored.get(global.clone()).await;

        let res = match inode {
            Ok(ref mut inode) => inode.delete(global.clone()).await,
            Err(e) => Err(e)
        };

        stored.delete(global).await?;
        res
    }

    pub fn unlink(&mut self, name: &String) -> Result<Stored, String> {
        self.children.remove(name)
            .ok_or(format!("File {} does not exist", name))
    }

    pub fn list(&self) -> Vec<String> {
        self.children.keys().cloned().collect()
    }

    pub fn list_tuples(&self) -> Vec<(String, Stored)> {
        self.children.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    pub fn get(&self, name: &String) -> Result<&Stored, String> {
        self.children.get(name)
            .ok_or(format!("File {} does not exist", name))
    }

    pub fn put(&mut self, name: &String, stored: Stored) -> Result<(), String> {
        if self.children.contains_key(name) {
            return Err(format!("File {} already exists", name));
        }
        
        self.children.insert(name.clone(), stored);
        self.metadata.modified(Size::Entries(self.children.len()));
        
        Ok(())
    }
}