use yew::prelude::*;
use yew::function_component;
use std::sync::Arc;

use crate::services::http::service::ServerData;
use crate::stored::Stored;

#[derive(Properties)]
pub struct DirectoryEntryProps {
    pub path: Vec<String>,
    pub data: Arc<ServerData>,
    pub name: String,
    pub inode: Stored
}

impl PartialEq for DirectoryEntryProps {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path &&
        self.name == other.name &&
        self.inode == other.inode
    }
}

#[function_component]
pub fn DirectoryEntry(props: &DirectoryEntryProps) -> Html {
    let url = format!("/files/{}/{}${}", props.path.join("/"), props.inode.as_url(), props.name.replace("$", "%24"));

    html! {
        <li class="entry inode">
            <a href={ url.clone() }>{ &props.name }</a>
            if !props.data.config.readonly {
                <div class="edit">
                    <button class="hamburger">{"☰"}</button>
                    <nav class="menu">
                        <ul>
                            <li class="delete-option destructive">
                                <form action={ url.clone() } method="POST" class="delete">
                                    <input type="hidden" name="request" value="delete" />
                                    <input type="submit" value="Delete" />
                                </form>
                            </li>
                            <li class="cut-option">
                                <form action={ url } method="POST" class="cut">
                                    <input type="hidden" name="request" value="cut" />
                                    <input type="submit" value="Cut" />
                                </form>
                            </li>
                        </ul>
                    </nav>
                </div>
            }
        </li>
    }
}