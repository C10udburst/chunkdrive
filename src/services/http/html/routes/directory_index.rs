use yew::prelude::*;
use yew::function_component;
use std::sync::Arc;

use crate::inodes::directory::Directory;
use crate::services::http::service::ServerData;

#[derive(Properties)]
pub struct DirectoryIndexProps {
    pub path: Vec<String>,
    pub data: Arc<ServerData>,
    pub dir: Directory
}

impl PartialEq for DirectoryIndexProps {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}


#[function_component]
pub fn DirectoryIndex(props: &DirectoryIndexProps) -> Html {
    html! {
        <div>
            <h1>{ "Directory" }</h1>
            <ul>
                if props.path.len() > 1 {
                    <li>
                        <a href={ format!("/files/{}", props.path[..props.path.len()-1].join("/")) }>{ ".." }</a>
                    </li>
                }
                { props.dir.list_tuples().iter().map(|(name, inode)| {
                    html! {
                        <li>
                            <a href={ format!("/files/{}/{}${}", props.path.join("/"), inode.as_url(), name.replace("$", "%24")) }>{ name }</a>
                        </li>
                    }
                }).collect::<Html>()}
            </ul>
        </div>
    }
}