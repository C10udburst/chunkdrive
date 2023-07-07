/*
   This file implements the basics of the inode system.
   Inodes are the core of the filesystem, they are the files and directories.
*/

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::global::Global;

use super::{file::File, directory::Directory, metadata::Metadata};

#[async_trait]
pub trait Inode {
    async fn metadata(&self) -> &Metadata;
    async fn delete(&mut self, global: Arc<Global>);
}

#[derive(Debug, Serialize, Deserialize)]
pub enum InodeType {
    #[serde(rename = "f")]
    File(File),
    #[serde(rename = "d")]
    Directory(Directory),
}