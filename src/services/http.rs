use std::sync::Arc;
use serde::Deserialize;
use actix_web::{web, App, HttpServer, Responder, HttpResponse};

use crate::global::Global;
use super::service::Service;


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

fn decorate_html(html: &str) -> String {
    format!("<html><body>{}</body></html>", html)
}

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

async fn root(data: web::Data<Arc<ServerData>>) -> impl Responder {
    if data.config.see_root {
        let root = data.global.get_root();
        let mut content = "<ul>".to_string();
        for inode in root.list() {
            content.push_str(&format!("<li><a href=\"{}\">{}</a></li>", inode, inode));
        }
        content.push_str("</ul>");
        HttpResponse::Ok()
            .content_type("text/html")
            .body(decorate_html(&content))
    } else {
        HttpResponse::Unauthorized()
            .content_type("text/plain")
            .body("Unauthorized")
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
                .route(&data_clone.config.path, web::get().to(root))
        })
        .bind(format!("{}:{}", data.config.address, data.config.port))
        .map_err(|e| format!("Failed to bind to port: {}", e))?
        .run()
        .await
        .map_err(|e| format!("Failed to run server: {}", e))
    })?;

    Ok(())
}