
use std::sync::Arc;

use yew::Properties;

use crate::services::http::service::ServerData;

#[derive(Properties)]
pub struct RouteProps {
    pub path: Vec<String>,
    pub data: Arc<ServerData>,
}

impl PartialEq for RouteProps {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}