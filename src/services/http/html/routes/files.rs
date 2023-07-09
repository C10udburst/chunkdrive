use yew::prelude::*;
use yew::function_component;

use crate::services::http::html::utils::RouteProps;

#[function_component]
pub fn FileRoute(props: &RouteProps) -> Html {
    html! {
        <div>
            <h1>{ "Files" }</h1>
            <p>{ "This is the files page." }</p>
        </div>
    }
}