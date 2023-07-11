use yew::prelude::*;
use yew::function_component;
use std::sync::Arc;

use crate::services::http::html::components::layout::Layout;
use crate::services::http::service::ServerData;


#[derive(Properties)]
pub struct ErrorPageProps {
    pub data: Arc<ServerData>,
    pub error: String
}

impl PartialEq for ErrorPageProps {
    fn eq(&self, other: &Self) -> bool {
        self.error == other.error
    }
}

#[function_component(ErrorPage)]
pub fn error_page(props: &ErrorPageProps) -> Html {
    html! {
        <Layout data={props.data.clone()}>
            <article class="error banner">
                <h1>{"Error"}</h1>
                <p>{props.error.clone()}</p>
            </article>
        </Layout>
    }
}