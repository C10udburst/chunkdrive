use yew::prelude::*;
use yew::function_component;
use std::sync::Arc;

use crate::inodes::directory::Directory;
use crate::services::http::html::components::directory_entry::DirectoryEntry;
use crate::services::http::html::components::layout::Layout;
use crate::services::http::service::ServerData;

#[derive(Properties)]
pub struct DirectoryIndexProps {
    pub path: Vec<String>,
    pub data: Arc<ServerData>,
    pub dir: Directory,
    pub cut_inode: Option<String>
}

impl PartialEq for DirectoryIndexProps {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

#[function_component]
pub fn DirectoryIndex(props: &DirectoryIndexProps) -> Html {    
    // for each part of the path where <a>$<b>$<c> strip $<c> if it exists
    let path = props.path.iter().map(|part| {
        let parts = part.split("$").collect::<Vec<&str>>();
        if parts.len() <= 2 {
            return part.clone();
        }
        format!("{}${}", parts[0], parts[1])
    }).collect::<Vec<String>>();
    
    html! {
        <Layout data={props.data.clone()}>
            <ul class="index">
                if path.len() > 1 {
                    <li class="entry back">
                        <a href={ format!("/files/{}", path[..path.len()-1].join("/")) }>{ ".." }</a>
                    </li>
                } else if path.len() == 1 && props.data.config.see_root {
                    <li class="entry back">
                        <a href={"/files/"} >{ ".." }</a>
                    </li>
                }
                { props.dir.list_tuples().iter().map(|(name, inode)| {
                    html! {
                        <DirectoryEntry name={name.clone()} inode={inode.clone()} data={props.data.clone()} path={path.clone()} />
                    }
                }).collect::<Html>()}
                if !props.data.config.readonly {
                    <div class="create-entries">
                        <li class="entry create create-file">
                            <span>{"Upload file"}</span>
                            <button class="create-btn">{"↑"}</button>
                            <form action={ format!("/files/{}/", path.join("/")) } method="POST" enctype="multipart/form-data" class="create-form file-upload">
                                <input type="file" name="file" />
                                <input type="submit" value="Upload file" />
                            </form>
                        </li>
                        <li class="entry create create-directory">
                            <span>{"Create directory"}</span>
                            <button class="create-btn">{"+"}</button>
                            <form action={ format!("/files/{}/", path.join("/")) } method="POST" enctype="multipart/form-data" class="create-form directory-create">
                                <input type="text" name="directory_name" placeholder="Directory name" />
                                <input type="submit" value="Create directory" />
                            </form>
                        </li>
                        if props.cut_inode.is_some() {
                            <li class="entry create paste">
                                <span>{"Paste"}</span>
                                <button class="create-btn">{"V"}</button>
                                <form action={ format!("/files/{}/", path.join("/")) } method="POST" enctype="multipart/form-data" class="create-form paste-inode">
                                    <input type="text" name="paste_name" placeholder="Paste as" />
                                    <input type="submit" value="Paste" />
                                </form>
                            </li>
                        }
                    </div>
                }
            </ul>
        </Layout>
    }
}