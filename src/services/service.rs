use std::sync::Arc;
use serde::Deserialize;

use crate::global::Global;

use super::http::service::HttpService;

pub trait Service {
    fn run(&self, global: Arc<Global>);
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServiceType {
    #[serde(rename = "http")]
    Http(HttpService),
}

impl Service for ServiceType {
    fn run(&self, global: Arc<Global>) {
        match self {
            ServiceType::Http(service) => service.run(global),
        }
    }
}
