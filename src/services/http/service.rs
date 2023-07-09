use std::sync::Arc;
use serde::Deserialize;
use actix_web::{web, App, HttpServer, Responder, HttpResponse, route};

use yew::{ServerRenderer, props};

use crate::{global::Global, services::{service::Service, http::html::utils::RouteProps}};
use super::html::routes::files::FileRoute;


#[derive(Debug, Deserialize, Clone)]
pub struct HttpService {
    port: u16,
    #[serde(default = "default_address")]
    address: String,

    #[serde(default = "default_path")]
    path: String,

    #[serde(default = "fn_false")]
    readonly: bool,

    #[serde(default = "fn_true")]
    see_root: bool,

    #[serde(default = "fn_true")]
    admin: bool
}

#[derive(Debug)]
pub struct ServerData {
    global: Arc<Global>,
    config: HttpService,
}

fn default_address() -> String { "127.0.0.1".to_string() }
fn default_path() -> String { "/".to_string() }
const fn fn_false() -> bool { false }
const fn fn_true() -> bool { true }

impl Service for HttpService {
    fn run(&self, global: Arc<Global>) {
        let data = Arc::new(ServerData { global, config: self.clone() });
        std::thread::spawn(move || {
            match run_blocking(data) {
                Ok(_) => {},
                Err(e) => println!("Failed to run HTTP service: {}", e),
            }
        });
    }
}

fn run_blocking(data: Arc<ServerData>) -> Result<(), String> {
    println!("Starting HTTP service on {}:{}", data.config.address, data.config.port);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let data_clone = data.clone();
    rt.block_on(async {
        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(data_clone.clone()))
                .service(redirect)
                .service(get)
                .service(post)
        })
        .bind(format!("{}:{}", data.config.address, data.config.port))
        .map_err(|e| format!("Failed to bind to port: {}", e))?
        .run()
        .await
        .map_err(|e| format!("Failed to run server: {}", e))
    })?;

    Ok(())
}

/* #region Routes */

#[route("/", method = "GET")]
async fn redirect(data: web::Data<Arc<ServerData>>) -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", format!("{}files/", data.config.path)))
        .finish()
}

/*async fn render_html(data: Arc<ServerData>, path: String) -> HttpResponse {
    let path = path.split('/').map(|s| s.to_string()).collect::<Vec<String>>();

    let renderer = ServerRenderer::<FileRoute>::with_props(|| {
        RouteProps {
            data: data,
            path: path,
        }
    });
    let html = renderer.render().await;

    HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}*/

#[route("/files/{path:.*}", method = "GET")]
async fn get(data: web::Data<Arc<ServerData>>, path: web::Path<String>) -> impl Responder {
    if !data.config.see_root && path.is_empty() {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    // check if is a file

    HttpResponse::Ok()
        .body("")
}

#[route("/files/{path:.*}", method = "POST")]
async fn post(data: web::Data<Arc<ServerData>>, path: web::Path<String>) -> impl Responder {
    if data.config.readonly {
        return HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized");
    }

    
    HttpResponse::Ok()
        .body("")
}

/* #endregion */