use std::sync::Arc;

use yew::prelude::*;
use yew::function_component;

use crate::services::http::service::ServerData;

#[derive(Properties)]
pub struct LayoutProps {
    #[prop_or_default]
    pub children: Children,
    pub data: Arc<ServerData>,
}

impl PartialEq for LayoutProps {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

#[function_component]
pub fn Layout(props: &LayoutProps) -> Html {
    html! {
        <html lang="en" charset="utf-8">
            <head>
                <title>{ "chunkdrive" }</title>
                <link rel="stylesheet" href="/static/style.css" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <meta charset="utf-8" />
                <script>
                    { format!("window.config = {{ readonly: {}, see_root: {}, admin: {}, path: \"{}\" }}",
                        props.data.config.readonly,
                        props.data.config.see_root,
                        props.data.config.admin,
                        props.data.config.path
                    )}
                </script>
            </head>
            <body>
                <div class="container">
                    <div class="header">
                        <h1>{ "chunkdrive" }</h1>
                    </div>
                    <div class="content">
                        { props.children.clone() }
                    </div>
                </div>
            </body>
        </html>
    }
}